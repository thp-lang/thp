use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

use crate::{ByteSize, Diagnostic, Duration, Limit};

pub(crate) const PROJECT_FILE_NAME: &str = "thp.toml";
pub(crate) const LOCAL_FILE_NAME: &str = "thp.local.toml";

/// A validated target identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetName(String);

/// A validated extension identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtensionName(String);

/// Ordered source directories for each statically mapped namespace prefix.
pub type AutoloadConfig = BTreeMap<String, Vec<PathBuf>>;

macro_rules! identifier {
    ($name:ident, $kind:literal) => {
        impl $name {
            /// Creates a validated identifier.
            ///
            /// # Errors
            ///
            /// Returns a message when the value is reserved or does not match
            /// the lowercase ASCII identifier grammar.
            pub fn new(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                validate_identifier(&value, $kind)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

identifier!(TargetName, "target");
identifier!(ExtensionName, "extension");

fn validate_identifier(value: &str, kind: &str) -> Result<(), String> {
    let mut bytes = value.bytes();
    if value == "default" {
        return Err(format!(
            "`default` is reserved and cannot be used as a {kind} name"
        ));
    }
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        || !bytes
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte))
    {
        return Err(format!(
            "{kind} names must begin with a lowercase ASCII letter and contain only lowercase letters, digits, `_`, or `-`"
        ));
    }
    Ok(())
}

/// Fully resolved core runtime limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub memory_limit: Limit<ByteSize>,
    pub post_max_size: Limit<ByteSize>,
    pub max_stack_depth: Option<u64>,
    pub max_open_handles: Option<u64>,
    pub max_input: Limit<Duration>,
    pub max_execution: Limit<Duration>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: Limit::Finite(ByteSize::from_bytes(128 * 1024 * 1024)),
            post_max_size: Limit::Finite(ByteSize::from_bytes(8 * 1024 * 1024)),
            max_stack_depth: Some(512),
            max_open_handles: Some(256),
            max_input: Limit::Finite(Duration::from_seconds(60)),
            max_execution: Limit::Finite(Duration::from_seconds(30)),
        }
    }
}

/// Canonical TOML for one extension, decoded only when requested.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionConfig {
    raw_toml: Box<str>,
}

impl ExtensionConfig {
    pub(crate) fn new(raw_toml: impl Into<Box<str>>) -> Self {
        Self {
            raw_toml: raw_toml.into(),
        }
    }

    /// Returns canonical TOML without decoding it.
    pub fn raw_toml(&self) -> &str {
        &self.raw_toml
    }

    /// Decodes the extension-owned configuration on demand.
    ///
    /// # Errors
    ///
    /// Returns a TOML decoding error when the extension's schema does not
    /// match its stored table.
    pub fn deserialize<T>(&self) -> Result<T, toml::de::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        toml::from_str(&self.raw_toml)
    }
}

/// One fully resolved common or target profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProfile {
    pub runtime: RuntimeConfig,
    pub(crate) extensions: BTreeMap<ExtensionName, ExtensionConfig>,
}

impl ResolvedProfile {
    pub fn extensions(&self) -> &BTreeMap<ExtensionName, ExtensionConfig> {
        &self.extensions
    }

    pub fn extension(&self, name: &str) -> Option<&ExtensionConfig> {
        self.extensions.get(name)
    }
}

/// Every profile resolved from a project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProject {
    pub common: ResolvedProfile,
    pub targets: BTreeMap<TargetName, ResolvedProfile>,
}

impl ResolvedProject {
    /// Selects a target, falling back to the common profile when undeclared.
    pub fn select(&self, target: Option<&str>) -> &ResolvedProfile {
        target
            .and_then(|target| self.targets.get(target))
            .unwrap_or(&self.common)
    }
}

/// Validated and merged project source configuration.
#[derive(Clone, Debug)]
pub struct ProjectConfig {
    root: PathBuf,
    autoload: AutoloadConfig,
    common: ProfileLayer,
    targets: BTreeMap<TargetName, ProfileLayer>,
}

impl ProjectConfig {
    /// Parses one project configuration document without reading the filesystem.
    ///
    /// The supplied `path` is retained for diagnostics and as the project root
    /// label. Unlike [`Self::load`], no local override is loaded.
    ///
    /// # Errors
    ///
    /// Returns a structured diagnostic for TOML, schema, limit, or identifier
    /// errors.
    pub fn parse(path: impl AsRef<Path>, source: &str) -> Result<Self, Diagnostic> {
        let path = path.as_ref();
        let layer = parse_layer(path, source)?;
        Ok(Self {
            root: path.parent().unwrap_or_else(|| Path::new("")).to_path_buf(),
            autoload: layer.autoload,
            common: layer.common,
            targets: layer.targets,
        })
    }

    /// Loads required `thp.toml` and optional `thp.local.toml` from `root`.
    ///
    /// # Errors
    ///
    /// Returns a structured diagnostic for I/O, UTF-8, TOML, schema, limit, or
    /// identifier errors in either source.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, Diagnostic> {
        let root = root.as_ref();
        let project_path = root.join(PROJECT_FILE_NAME);
        let project = load_layer(&project_path, true)?;
        let local_path = root.join(LOCAL_FILE_NAME);
        let local = load_layer(&local_path, false)?;

        let mut autoload = project.autoload;
        autoload.extend(local.autoload);
        let mut common = project.common;
        common.merge(local.common);
        let mut targets = project.targets;
        for (name, profile) in local.targets {
            targets.entry(name).or_default().merge(profile);
        }

        Ok(Self {
            root: root.to_path_buf(),
            autoload,
            common,
            targets,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn autoload(&self) -> &AutoloadConfig {
        &self.autoload
    }

    pub fn target_names(&self) -> impl Iterator<Item = &TargetName> {
        self.targets.keys()
    }

    /// Resolves a selected target, or the common profile for `None` or an
    /// undeclared target.
    ///
    /// # Errors
    ///
    /// Returns a structured diagnostic if canonical extension TOML cannot be
    /// encoded.
    pub fn resolve(&self, target: Option<&str>) -> Result<ResolvedProfile, Diagnostic> {
        let mut profile = self.common.clone();
        if let Some(target) = target.and_then(|target| self.targets.get(target)) {
            profile.merge(target.clone());
        }
        resolve_profile(&profile, &self.root.join(PROJECT_FILE_NAME))
    }

    /// Resolves the common profile and every declared target.
    ///
    /// # Errors
    ///
    /// Returns a structured diagnostic if canonical extension TOML cannot be
    /// encoded.
    pub fn resolve_all(&self) -> Result<ResolvedProject, Diagnostic> {
        let common = self.resolve(None)?;
        let targets = self
            .targets
            .keys()
            .map(|target| {
                self.resolve(Some(target.as_str()))
                    .map(|profile| (target.clone(), profile))
            })
            .collect::<Result<_, _>>()?;
        Ok(ResolvedProject { common, targets })
    }
}

#[derive(Clone, Debug, Default)]
struct ProfileLayer {
    memory_limit: Option<Limit<ByteSize>>,
    post_max_size: Option<Limit<ByteSize>>,
    max_stack_depth: Option<CountLimit>,
    max_open_handles: Option<CountLimit>,
    max_input: Option<Limit<Duration>>,
    max_execution: Option<Limit<Duration>>,
    extensions: BTreeMap<ExtensionName, toml::Value>,
}

#[derive(Clone, Copy, Debug)]
enum CountLimit {
    Unlimited,
    Finite(u64),
}

impl CountLimit {
    const fn finite(self) -> Option<u64> {
        match self {
            Self::Unlimited => None,
            Self::Finite(value) => Some(value),
        }
    }
}

impl ProfileLayer {
    fn merge(&mut self, later: Self) {
        if later.memory_limit.is_some() {
            self.memory_limit = later.memory_limit;
        }
        if later.post_max_size.is_some() {
            self.post_max_size = later.post_max_size;
        }
        if later.max_stack_depth.is_some() {
            self.max_stack_depth = later.max_stack_depth;
        }
        if later.max_open_handles.is_some() {
            self.max_open_handles = later.max_open_handles;
        }
        if later.max_input.is_some() {
            self.max_input = later.max_input;
        }
        if later.max_execution.is_some() {
            self.max_execution = later.max_execution;
        }
        for (name, later_value) in later.extensions {
            match self.extensions.get_mut(&name) {
                Some(earlier) => merge_toml(earlier, later_value),
                None => {
                    self.extensions.insert(name, later_value);
                }
            }
        }
    }
}

fn merge_toml(earlier: &mut toml::Value, later: toml::Value) {
    match (earlier, later) {
        (toml::Value::Table(earlier), toml::Value::Table(later)) => {
            for (key, value) in later {
                match earlier.get_mut(&key) {
                    Some(earlier) => merge_toml(earlier, value),
                    None => {
                        earlier.insert(key, value);
                    }
                }
            }
        }
        (earlier, later) => *earlier = later,
    }
}

fn resolve_profile(layer: &ProfileLayer, path: &Path) -> Result<ResolvedProfile, Diagnostic> {
    let defaults = RuntimeConfig::default();
    let runtime = RuntimeConfig {
        memory_limit: layer.memory_limit.unwrap_or(defaults.memory_limit),
        post_max_size: layer.post_max_size.unwrap_or(defaults.post_max_size),
        max_stack_depth: layer
            .max_stack_depth
            .map_or(defaults.max_stack_depth, CountLimit::finite),
        max_open_handles: layer
            .max_open_handles
            .map_or(defaults.max_open_handles, CountLimit::finite),
        max_input: layer.max_input.unwrap_or(defaults.max_input),
        max_execution: layer.max_execution.unwrap_or(defaults.max_execution),
    };
    let extensions = layer
        .extensions
        .iter()
        .map(|(name, value)| {
            toml::to_string(value)
                .map(|raw| (name.clone(), ExtensionConfig::new(raw)))
                .map_err(|error| {
                    Diagnostic::new(
                        path,
                        None,
                        None,
                        Some(format!("extensions.{name}")),
                        format!("could not encode extension configuration: {error}"),
                    )
                })
        })
        .collect::<Result<_, _>>()?;
    Ok(ResolvedProfile {
        runtime,
        extensions,
    })
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDocument {
    #[serde(default)]
    autoload: BTreeMap<String, RawAutoloadDirectories>,
    memory: Option<RawMemory>,
    request: Option<RawRequest>,
    time: Option<RawTime>,
    #[serde(default)]
    extensions: BTreeMap<String, toml::Value>,
    #[serde(default)]
    targets: BTreeMap<String, RawProfile>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawAutoloadDirectories {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    memory: Option<RawMemory>,
    request: Option<RawRequest>,
    time: Option<RawTime>,
    #[serde(default)]
    extensions: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMemory {
    limit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    post_max_size: Option<String>,
    max_stack_depth: Option<u64>,
    max_open_handles: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTime {
    max_input: Option<String>,
    max_execution: Option<String>,
}

#[derive(Debug, Default)]
struct DocumentLayer {
    autoload: AutoloadConfig,
    common: ProfileLayer,
    targets: BTreeMap<TargetName, ProfileLayer>,
}

fn load_layer(path: &Path, required: bool) -> Result<DocumentLayer, Diagnostic> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if !required && error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DocumentLayer::default());
        }
        Err(error) => {
            return Err(Diagnostic::new(
                path,
                None,
                None,
                None,
                if error.kind() == std::io::ErrorKind::NotFound {
                    format!("required configuration file `{PROJECT_FILE_NAME}` is missing")
                } else {
                    format!("could not read configuration: {error}")
                },
            ));
        }
    };
    let source = std::str::from_utf8(&bytes).map_err(|error| {
        let valid_prefix = std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or_default();
        Diagnostic::new(
            path,
            Some(valid_prefix),
            Some(error.valid_up_to()..error.valid_up_to().saturating_add(1)),
            None,
            "configuration must be valid UTF-8",
        )
    })?;
    parse_layer(path, source)
}

fn parse_layer(path: &Path, source: &str) -> Result<DocumentLayer, Diagnostic> {
    let raw: RawDocument = toml::from_str(source).map_err(|error: toml::de::Error| {
        let span = error.span();
        let message = error.to_string();
        let field = message
            .split_once("unknown field `")
            .and_then(|(_, tail)| tail.split_once('`'))
            .map(|(field, _)| field.to_owned());
        Diagnostic::new(
            path,
            Some(source),
            span,
            field,
            format!("invalid configuration: {message}"),
        )
    })?;

    let autoload = raw
        .autoload
        .into_iter()
        .map(|(prefix, directories)| {
            validate_namespace_prefix(&prefix).map_err(|message| {
                Diagnostic::at_field(path, source, format!("autoload.{prefix}"), message)
            })?;
            let directories = match directories {
                RawAutoloadDirectories::One(directory) => vec![directory],
                RawAutoloadDirectories::Many(directories) => directories,
            };
            if directories.is_empty() || directories.iter().any(String::is_empty) {
                return Err(Diagnostic::at_field(
                    path,
                    source,
                    format!("autoload.{prefix}"),
                    "an autoload mapping requires at least one non-empty directory",
                ));
            }
            Ok((prefix, directories.into_iter().map(PathBuf::from).collect()))
        })
        .collect::<Result<AutoloadConfig, Diagnostic>>()?;
    let common = convert_profile(
        RawProfile {
            memory: raw.memory,
            request: raw.request,
            time: raw.time,
            extensions: raw.extensions,
        },
        path,
        source,
        "",
    )?;
    let targets = raw
        .targets
        .into_iter()
        .map(|(name, profile)| {
            let target = TargetName::new(name.clone()).map_err(|message| {
                Diagnostic::at_field(path, source, format!("targets.{name}"), message)
            })?;
            let profile = convert_profile(profile, path, source, &format!("targets.{name}."))?;
            Ok((target, profile))
        })
        .collect::<Result<_, Diagnostic>>()?;
    Ok(DocumentLayer {
        autoload,
        common,
        targets,
    })
}

fn validate_namespace_prefix(prefix: &str) -> Result<(), &'static str> {
    if prefix.is_empty() {
        return Ok(());
    }
    let Some(without_separator) = prefix.strip_suffix('\\') else {
        return Err("a non-empty autoload namespace prefix must end with `\\`");
    };
    if without_separator.is_empty()
        || without_separator.split('\\').any(|segment| {
            let mut bytes = segment.bytes();
            !bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
    {
        return Err("autoload namespace prefixes must contain valid case-sensitive name segments");
    }
    Ok(())
}

fn convert_profile(
    raw: RawProfile,
    path: &Path,
    source: &str,
    prefix: &str,
) -> Result<ProfileLayer, Diagnostic> {
    let parse_size = |value: Option<String>, field: &str| {
        value
            .map(|value| {
                value.parse::<Limit<ByteSize>>().map_err(|error| {
                    Diagnostic::at_field(
                        path,
                        source,
                        format!("{prefix}{field}"),
                        error.to_string(),
                    )
                })
            })
            .transpose()
    };
    let parse_time = |value: Option<String>, field: &str| {
        value
            .map(|value| {
                value.parse::<Limit<Duration>>().map_err(|error| {
                    Diagnostic::at_field(
                        path,
                        source,
                        format!("{prefix}{field}"),
                        error.to_string(),
                    )
                })
            })
            .transpose()
    };
    let memory_limit = parse_size(raw.memory.and_then(|memory| memory.limit), "memory.limit")?;
    let post_max_size = parse_size(
        raw.request
            .as_ref()
            .and_then(|request| request.post_max_size.clone()),
        "request.post_max_size",
    )?;
    let max_stack_depth = raw
        .request
        .as_ref()
        .and_then(|request| request.max_stack_depth)
        .map(|value| {
            if value == 0 {
                CountLimit::Unlimited
            } else {
                CountLimit::Finite(value)
            }
        });
    let max_open_handles = raw
        .request
        .and_then(|request| request.max_open_handles)
        .map(|value| {
            if value == 0 {
                CountLimit::Unlimited
            } else {
                CountLimit::Finite(value)
            }
        });
    let max_input = parse_time(
        raw.time.as_ref().and_then(|time| time.max_input.clone()),
        "time.max_input",
    )?;
    let max_execution = parse_time(
        raw.time.and_then(|time| time.max_execution),
        "time.max_execution",
    )?;
    let extensions = raw
        .extensions
        .into_iter()
        .map(|(name, value)| {
            let name = ExtensionName::new(name.clone()).map_err(|message| {
                Diagnostic::at_field(path, source, format!("{prefix}extensions.{name}"), message)
            })?;
            if !value.is_table() {
                return Err(Diagnostic::at_field(
                    path,
                    source,
                    format!("{prefix}extensions.{name}"),
                    "extension configuration must be a TOML table",
                ));
            }
            Ok((name, value))
        })
        .collect::<Result<_, _>>()?;

    Ok(ProfileLayer {
        memory_limit,
        post_max_size,
        max_stack_depth,
        max_open_handles,
        max_input,
        max_execution,
        extensions,
    })
}
