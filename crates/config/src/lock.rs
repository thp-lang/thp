use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::diagnostic::SourceLocation;
use crate::model::{LOCAL_FILE_NAME, PROJECT_FILE_NAME};
use crate::{
    ByteSize, Diagnostic, Duration, ExtensionConfig, ExtensionName, Limit, ProjectConfig,
    ResolvedProfile, ResolvedProject, RuntimeConfig, TargetName,
};

pub const LOCK_FILE_NAME: &str = "thp.lock";
pub const LOCK_VERSION: u32 = 1;
const HEADER: &str = "THP-LOCK";

/// The result of generating a lock file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockBuild {
    pub path: PathBuf,
    pub fingerprint: String,
    /// False when the existing lock already had identical bytes.
    pub changed: bool,
}

/// A freshness-checked, owned lock file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockFile {
    fingerprint: String,
    project: ResolvedProject,
}

impl LockFile {
    /// Loads `thp.lock` and verifies it against the exact presence and contents
    /// of `thp.toml` and `thp.local.toml`.
    ///
    /// # Errors
    ///
    /// Returns a structured error when the lock or a source cannot be read,
    /// the lock is malformed or unsupported, or its fingerprint is stale.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, LockError> {
        let root = root.as_ref();
        let path = root.join(LOCK_FILE_NAME);
        let bytes = fs::read(&path).map_err(|error| {
            LockError::io(
                &path,
                if error.kind() == io::ErrorKind::NotFound {
                    LockErrorKind::Missing
                } else {
                    LockErrorKind::Io
                },
                if error.kind() == io::ErrorKind::NotFound {
                    "lock file is missing".to_owned()
                } else {
                    format!("could not read lock file: {error}")
                },
            )
        })?;
        let parsed = parse_lock_at(&bytes, &path)?;
        let current = source_fingerprint(root)?;
        if parsed.fingerprint != current {
            return Err(LockError::new(
                LockErrorKind::Stale,
                &path,
                None,
                Some("fingerprint".to_owned()),
                "lock file is stale because project configuration changed",
            ));
        }
        Ok(parsed.into_owned())
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn project(&self) -> &ResolvedProject {
        &self.project
    }

    /// Selects a target, falling back to common configuration when undeclared.
    pub fn select(&self, target: Option<&str>) -> &ResolvedProfile {
        self.project.select(target)
    }
}

/// One extension payload borrowed directly from lock bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedExtension<'a> {
    pub name: &'a str,
    pub raw_toml: &'a str,
}

/// One profile parsed without decoding its extension TOML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedProfile<'a> {
    pub target: Option<&'a str>,
    pub runtime: RuntimeConfig,
    pub extensions: Vec<ParsedExtension<'a>>,
}

/// A lock representation borrowing extension payloads from the caller's bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedLock<'a> {
    pub fingerprint: &'a str,
    pub common: ParsedProfile<'a>,
    pub targets: Vec<ParsedProfile<'a>>,
}

impl ParsedLock<'_> {
    fn into_owned(self) -> LockFile {
        fn profile(value: ParsedProfile<'_>) -> ResolvedProfile {
            let extensions = value
                .extensions
                .into_iter()
                .map(|extension| {
                    (
                        ExtensionName::new(extension.name)
                            .expect("lock parser validated extension identifier"),
                        ExtensionConfig::new(extension.raw_toml.to_owned()),
                    )
                })
                .collect();
            ResolvedProfile {
                runtime: value.runtime,
                extensions,
            }
        }

        let common = profile(self.common);
        let targets = self
            .targets
            .into_iter()
            .map(|value| {
                let target =
                    TargetName::new(value.target.expect("target profiles have a target name"))
                        .expect("lock parser validated target identifier");
                (target, profile(value))
            })
            .collect();
        LockFile {
            fingerprint: self.fingerprint.to_owned(),
            project: ResolvedProject { common, targets },
        }
    }
}

/// Categories callers can handle without parsing an error message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockErrorKind {
    Missing,
    Io,
    InvalidUtf8,
    UnsupportedVersion,
    Corrupt,
    Stale,
    Source,
}

/// A structured lock loading or generation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockError {
    pub kind: LockErrorKind,
    pub path: PathBuf,
    pub location: Option<SourceLocation>,
    pub field: Option<String>,
    pub message: String,
}

impl LockError {
    fn new(
        kind: LockErrorKind,
        path: impl Into<PathBuf>,
        span: Option<Range<usize>>,
        field: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let location = span.map(|span| SourceLocation {
            line: 0,
            column: 0,
            span: Some(span),
        });
        Self {
            kind,
            path,
            location,
            field,
            message: message.into(),
        }
    }

    fn at_offset(
        kind: LockErrorKind,
        path: &Path,
        source: &str,
        offset: usize,
        field: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        let offset = offset.min(source.len());
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix
            .rsplit_once('\n')
            .map_or(prefix, |(_, tail)| tail)
            .chars()
            .count()
            + 1;
        Self {
            kind,
            path: path.to_path_buf(),
            location: Some(SourceLocation {
                span: Some(offset..offset),
                line,
                column,
            }),
            field,
            message: message.into(),
        }
    }

    fn io(path: &Path, kind: LockErrorKind, message: String) -> Self {
        Self::new(kind, path, None, None, message)
    }

    fn source(error: Diagnostic) -> Self {
        Self {
            kind: LockErrorKind::Source,
            path: error.path,
            location: error.location,
            field: error.field,
            message: error.message,
        }
    }
}

impl fmt::Display for LockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.path.display())?;
        if let Some(location) = &self.location
            && location.line != 0
        {
            write!(formatter, ":{}:{}", location.line, location.column)?;
        }
        if let Some(field) = &self.field {
            write!(formatter, " ({field})")?;
        }
        write!(formatter, ": {}", self.message)
    }
}

impl std::error::Error for LockError {}

/// Parses lock bytes without checking source freshness and without parsing any
/// extension payload as TOML.
///
/// # Errors
///
/// Returns a structured corrupt, UTF-8, or unsupported-version error when the
/// byte slice does not follow the lock grammar.
pub fn parse_lock(bytes: &[u8]) -> Result<ParsedLock<'_>, LockError> {
    parse_lock_at(bytes, Path::new("<lock bytes>"))
}

/// Generates `thp.lock` atomically. Identical output is not rewritten.
///
/// # Errors
///
/// Returns a structured source or I/O error when configuration cannot be
/// loaded, resolved, encoded, or atomically persisted.
pub fn build_lock(root: impl AsRef<Path>) -> Result<LockBuild, LockError> {
    let root = root.as_ref();
    let before = source_fingerprint(root)?;
    let config = ProjectConfig::load(root).map_err(LockError::source)?;
    let project = config.resolve_all().map_err(LockError::source)?;
    let after = source_fingerprint(root)?;
    if before != after {
        return Err(LockError::new(
            LockErrorKind::Source,
            root.join(PROJECT_FILE_NAME),
            None,
            None,
            "configuration changed while the lock file was being built; retry",
        ));
    }
    let bytes = encode_lock(&after, &project);
    let path = root.join(LOCK_FILE_NAME);
    if fs::read(&path).is_ok_and(|existing| existing == bytes) {
        set_owner_only(&path)?;
        return Ok(LockBuild {
            path,
            fingerprint: after,
            changed: false,
        });
    }

    let mut temporary = tempfile::Builder::new()
        .prefix(".thp.lock.")
        .tempfile_in(root)
        .map_err(|error| {
            LockError::io(
                &path,
                LockErrorKind::Io,
                format!("could not create temporary lock file: {error}"),
            )
        })?;
    set_owner_only(temporary.path())?;
    temporary.write_all(&bytes).map_err(|error| {
        LockError::io(
            &path,
            LockErrorKind::Io,
            format!("could not write temporary lock file: {error}"),
        )
    })?;
    temporary.as_file_mut().sync_all().map_err(|error| {
        LockError::io(
            &path,
            LockErrorKind::Io,
            format!("could not sync temporary lock file: {error}"),
        )
    })?;
    temporary.persist(&path).map_err(|error| {
        LockError::io(
            &path,
            LockErrorKind::Io,
            format!("could not replace lock file atomically: {}", error.error),
        )
    })?;
    set_owner_only(&path)?;
    Ok(LockBuild {
        path,
        fingerprint: after,
        changed: true,
    })
}

fn set_owner_only(path: &Path) -> Result<(), LockError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| {
            LockError::io(
                path,
                LockErrorKind::Io,
                format!("could not set owner-only lock permissions: {error}"),
            )
        })?;
    }
    Ok(())
}

fn source_fingerprint(root: &Path) -> Result<String, LockError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"THP-CONFIG-SOURCES\0\x01");
    for (name, required) in [(PROJECT_FILE_NAME, true), (LOCAL_FILE_NAME, false)] {
        hasher.update(&(name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        match fs::read(root.join(name)) {
            Ok(bytes) => {
                hasher.update(&[1]);
                hasher.update(&(bytes.len() as u64).to_le_bytes());
                hasher.update(&bytes);
            }
            Err(error) if !required && error.kind() == io::ErrorKind::NotFound => {
                hasher.update(&[0]);
            }
            Err(error) => {
                let message = if error.kind() == io::ErrorKind::NotFound {
                    format!("required configuration file `{PROJECT_FILE_NAME}` is missing")
                } else {
                    format!("could not fingerprint configuration source: {error}")
                };
                return Err(LockError::io(
                    &root.join(name),
                    LockErrorKind::Source,
                    message,
                ));
            }
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn encode_lock(fingerprint: &str, project: &ResolvedProject) -> Vec<u8> {
    let mut output = Vec::new();
    writeln!(output, "{HEADER} {LOCK_VERSION}").expect("writing to Vec cannot fail");
    writeln!(output, "fingerprint {fingerprint}").expect("writing to Vec cannot fail");
    encode_profile(&mut output, None, &project.common);
    for (target, profile) in &project.targets {
        encode_profile(&mut output, Some(target.as_str()), profile);
    }
    output.extend_from_slice(b"end-lock\n");
    output
}

fn encode_profile(output: &mut Vec<u8>, target: Option<&str>, profile: &ResolvedProfile) {
    match target {
        Some(target) => writeln!(output, "profile target {target}"),
        None => writeln!(output, "profile common"),
    }
    .expect("writing to Vec cannot fail");
    writeln!(output, "memory.limit {}", profile.runtime.memory_limit)
        .expect("writing to Vec cannot fail");
    writeln!(
        output,
        "request.post_max_size {}",
        profile.runtime.post_max_size
    )
    .expect("writing to Vec cannot fail");
    writeln!(
        output,
        "request.max_stack_depth {}",
        profile.runtime.max_stack_depth.unwrap_or(0)
    )
    .expect("writing to Vec cannot fail");
    writeln!(
        output,
        "request.max_open_handles {}",
        profile.runtime.max_open_handles.unwrap_or(0)
    )
    .expect("writing to Vec cannot fail");
    writeln!(output, "time.max_input {}", profile.runtime.max_input)
        .expect("writing to Vec cannot fail");
    writeln!(
        output,
        "time.max_execution {}",
        profile.runtime.max_execution
    )
    .expect("writing to Vec cannot fail");
    for (name, extension) in &profile.extensions {
        let payload = extension.raw_toml().as_bytes();
        writeln!(output, "extension {name} {}", payload.len()).expect("writing to Vec cannot fail");
        output.extend_from_slice(payload);
        output.push(b'\n');
    }
    output.extend_from_slice(b"end-profile\n");
}

fn parse_lock_at<'a>(bytes: &'a [u8], path: &Path) -> Result<ParsedLock<'a>, LockError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        let valid_prefix = std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or_default();
        LockError::at_offset(
            LockErrorKind::InvalidUtf8,
            path,
            valid_prefix,
            error.valid_up_to(),
            None,
            "lock file must be valid UTF-8",
        )
    })?;
    let mut cursor = Cursor {
        source,
        position: 0,
        path,
    };
    let header = cursor.line("header")?;
    let version = header
        .strip_prefix(&format!("{HEADER} "))
        .ok_or_else(|| cursor.error_at(0, None, "invalid lock header"))?;
    let version = parse_canonical_u64(version)
        .ok_or_else(|| cursor.error_at(0, None, "invalid lock version"))?;
    if version != u64::from(LOCK_VERSION) {
        return Err(LockError::at_offset(
            LockErrorKind::UnsupportedVersion,
            path,
            source,
            0,
            Some("version".to_owned()),
            format!("unsupported lock version {version}"),
        ));
    }

    let fingerprint_line = cursor.line("fingerprint")?;
    let fingerprint = fingerprint_line
        .strip_prefix("fingerprint ")
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| cursor.error(None, "invalid source fingerprint"))?;

    let common_header = cursor.line("common profile")?;
    if common_header != "profile common" {
        return Err(cursor.error(None, "missing common profile"));
    }
    let common = parse_profile(&mut cursor, None)?;
    let mut targets = Vec::new();
    let mut previous_target: Option<&str> = None;
    loop {
        let offset = cursor.position;
        let line = cursor.line("profile or end-lock")?;
        if line == "end-lock" {
            if cursor.position != source.len() {
                return Err(cursor.error(None, "malformed trailing data after end-lock"));
            }
            break;
        }
        let target = line
            .strip_prefix("profile target ")
            .ok_or_else(|| cursor.error_at(offset, None, "unknown record kind"))?;
        TargetName::new(target)
            .map_err(|message| cursor.error_at(offset, Some("target".to_owned()), message))?;
        if previous_target.is_some_and(|previous| previous >= target) {
            return Err(cursor.error_at(
                offset,
                Some("target".to_owned()),
                "target profiles must be unique and lexicographically ordered",
            ));
        }
        previous_target = Some(target);
        targets.push(parse_profile(&mut cursor, Some(target))?);
    }

    Ok(ParsedLock {
        fingerprint,
        common,
        targets,
    })
}

fn parse_profile<'a>(
    cursor: &mut Cursor<'a, '_>,
    target: Option<&'a str>,
) -> Result<ParsedProfile<'a>, LockError> {
    let memory_limit = parse_size_record(cursor, "memory.limit")?;
    let post_max_size = parse_size_record(cursor, "request.post_max_size")?;
    let max_stack_depth = parse_count_record(cursor, "request.max_stack_depth")?;
    let max_open_handles = parse_count_record(cursor, "request.max_open_handles")?;
    let max_input = parse_time_record(cursor, "time.max_input")?;
    let max_execution = parse_time_record(cursor, "time.max_execution")?;
    let mut extensions = Vec::new();
    let mut previous_extension: Option<&str> = None;
    loop {
        let offset = cursor.position;
        let line = cursor.line("extension or end-profile")?;
        if line == "end-profile" {
            break;
        }
        let fields = line.split(' ').collect::<Vec<_>>();
        if fields.len() != 3 || fields[0] != "extension" {
            return Err(cursor.error_at(offset, None, "unknown record kind"));
        }
        let name = fields[1];
        ExtensionName::new(name).map_err(|message| {
            cursor.error_at(offset, Some(format!("extensions.{name}")), message)
        })?;
        if previous_extension.is_some_and(|previous| previous >= name) {
            return Err(cursor.error_at(
                offset,
                Some(format!("extensions.{name}")),
                "extensions must be unique and lexicographically ordered",
            ));
        }
        let length = parse_canonical_u64(fields[2])
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| {
                cursor.error_at(
                    offset,
                    Some(format!("extensions.{name}")),
                    "invalid extension byte length",
                )
            })?;
        let raw_toml = cursor.payload(length, &format!("extensions.{name}"))?;
        previous_extension = Some(name);
        extensions.push(ParsedExtension { name, raw_toml });
    }
    Ok(ParsedProfile {
        target,
        runtime: RuntimeConfig {
            memory_limit,
            post_max_size,
            max_stack_depth,
            max_open_handles,
            max_input,
            max_execution,
        },
        extensions,
    })
}

fn parse_size_record(
    cursor: &mut Cursor<'_, '_>,
    name: &str,
) -> Result<Limit<ByteSize>, LockError> {
    let value = record_value(cursor, name)?;
    parse_lock_limit(value, ByteSize::from_bytes)
        .ok_or_else(|| cursor.error(Some(name.to_owned()), "invalid canonical byte limit"))
}

fn parse_time_record(
    cursor: &mut Cursor<'_, '_>,
    name: &str,
) -> Result<Limit<Duration>, LockError> {
    let value = record_value(cursor, name)?;
    parse_lock_limit(value, Duration::from_seconds)
        .ok_or_else(|| cursor.error(Some(name.to_owned()), "invalid canonical duration limit"))
}

fn parse_count_record(cursor: &mut Cursor<'_, '_>, name: &str) -> Result<Option<u64>, LockError> {
    let value = record_value(cursor, name)?;
    parse_canonical_u64(value)
        .map(|value| (value != 0).then_some(value))
        .ok_or_else(|| cursor.error(Some(name.to_owned()), "invalid canonical count limit"))
}

fn record_value<'a>(cursor: &mut Cursor<'a, '_>, name: &str) -> Result<&'a str, LockError> {
    let offset = cursor.position;
    let line = cursor.line(name)?;
    line.strip_prefix(name)
        .and_then(|value| value.strip_prefix(' '))
        .filter(|value| !value.contains(' '))
        .ok_or_else(|| {
            cursor.error_at(
                offset,
                Some(name.to_owned()),
                format!("expected `{name}` record"),
            )
        })
}

fn parse_lock_limit<T>(value: &str, constructor: impl FnOnce(u64) -> T) -> Option<Limit<T>> {
    if value == "unlimited" {
        Some(Limit::Unlimited)
    } else {
        parse_canonical_u64(value)
            .filter(|value| *value != 0)
            .map(|value| Limit::Finite(constructor(value)))
    }
}

fn parse_canonical_u64(value: &str) -> Option<u64> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return None;
    }
    value.parse().ok()
}

struct Cursor<'a, 'p> {
    source: &'a str,
    position: usize,
    path: &'p Path,
}

impl<'a> Cursor<'a, '_> {
    fn line(&mut self, context: &str) -> Result<&'a str, LockError> {
        let start = self.position;
        let tail = self.source.get(start..).unwrap_or_default();
        let newline = tail.find('\n').ok_or_else(|| {
            self.error_at(
                start,
                Some(context.to_owned()),
                "record is not terminated by LF",
            )
        })?;
        self.position = start + newline + 1;
        Ok(&self.source[start..start + newline])
    }

    fn payload(&mut self, length: usize, context: &str) -> Result<&'a str, LockError> {
        let start = self.position;
        let end = start.checked_add(length).ok_or_else(|| {
            self.error_at(
                start,
                Some(context.to_owned()),
                "extension byte length overflows",
            )
        })?;
        let payload = self.source.get(start..end).ok_or_else(|| {
            self.error_at(
                start,
                Some(context.to_owned()),
                "extension payload is shorter than its declared byte length",
            )
        })?;
        if self.source.as_bytes().get(end) != Some(&b'\n') {
            return Err(self.error_at(
                end.min(self.source.len()),
                Some(context.to_owned()),
                "extension payload is not followed by its required LF terminator",
            ));
        }
        self.position = end + 1;
        Ok(payload)
    }

    fn error(&self, field: Option<String>, message: impl Into<String>) -> LockError {
        self.error_at(self.position, field, message)
    }

    fn error_at(
        &self,
        offset: usize,
        field: Option<String>,
        message: impl Into<String>,
    ) -> LockError {
        LockError::at_offset(
            LockErrorKind::Corrupt,
            self.path,
            self.source,
            offset,
            field,
            message,
        )
    }
}
