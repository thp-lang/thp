//! Portable PHPT discovery and execution for THP.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use regex::bytes::Regex;
use thp_compiler::ProjectRequest;
use thp_config::{AutoloadConfig, ProjectConfig, RuntimeConfig};
use thp_embed::{Engine, PreparedProject, RequestStats, Response, Status as ExecutionStatus};
use thp_modules::{AutoloadMapping, FilesystemSourceProvider};
use thp_runtime::RequestInput;
use thp_vm::{ExecutionContext, Limits};

const KNOWN_SECTIONS: &[&str] = &[
    "TEST",
    "FILE",
    "FILE_EXTERNAL",
    "EXPECT",
    "EXPECTF",
    "EXPECTREGEX",
    "SKIPIF",
    "CLEAN",
    "CONFIG",
    "CREDITS",
    "DESCRIPTION",
    "INI",
    "EXTENSIONS",
    "ARGS",
    "ENV",
    "STDIN",
    "POST",
    "POST_RAW",
    "GZIP_POST",
    "DEFLATE_POST",
    "PUT",
    "GET",
    "COOKIE",
    "CGI",
    "EXPECTHEADERS",
    "CAPTURE_STDIO",
    "PHPDBG",
];
const UNSUPPORTED_SECTIONS: &[&str] = &[
    "INI",
    "EXTENSIONS",
    "ARGS",
    "ENV",
    "POST",
    "POST_RAW",
    "GZIP_POST",
    "DEFLATE_POST",
    "PUT",
    "GET",
    "COOKIE",
    "CGI",
    "EXPECTHEADERS",
    "CAPTURE_STDIO",
    "PHPDBG",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Pass,
    Fail,
    Skip,
    Bork,
}

impl Status {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
            Self::Bork => "BORK",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RunnerOptions {
    pub max_instructions: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestResult {
    pub path: PathBuf,
    pub name: String,
    pub status: Status,
    pub details: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Summary {
    pub tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub borked: usize,
    pub results: Vec<TestResult>,
}

impl Summary {
    pub const fn is_success(&self) -> bool {
        self.failed == 0 && self.borked == 0
    }

    fn push(&mut self, result: TestResult) {
        self.tests += 1;
        match result.status {
            Status::Pass => self.passed += 1,
            Status::Fail => self.failed += 1,
            Status::Skip => self.skipped += 1,
            Status::Bork => self.borked += 1,
        }
        self.results.push(result);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerError {
    message: String,
}

impl RunnerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RunnerError {}

#[derive(Clone, Debug, Default)]
pub struct Runner {
    options: RunnerOptions,
}

impl Runner {
    pub const fn new(options: RunnerOptions) -> Self {
        Self { options }
    }

    /// Discovers and runs PHPT files in deterministic path order.
    ///
    /// # Errors
    ///
    /// Returns an error when an input cannot be inspected, discovery fails, or
    /// no PHPT files are found. Malformed individual fixtures are BORK results.
    pub fn run_paths<I, P>(&self, paths: I) -> Result<Summary, RunnerError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let tests = discover(paths)?;
        if tests.is_empty() {
            return Err(RunnerError::new("no PHPT tests discovered"));
        }
        let mut summary = Summary::default();
        for path in tests {
            summary.push(self.run_test(path));
        }
        Ok(summary)
    }

    fn run_test(&self, path: PathBuf) -> TestResult {
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                return result(
                    &path,
                    "",
                    Status::Bork,
                    format!("cannot read test: {error}"),
                );
            }
        };
        let fixture = match Fixture::parse(&path, &bytes) {
            Ok(fixture) => fixture,
            Err(error) => return result(&path, "", Status::Bork, error),
        };
        self.run_fixture(path, &fixture)
    }

    fn run_fixture(&self, path: PathBuf, fixture: &Fixture) -> TestResult {
        let name = fixture.name.clone();
        let program = match fixture.program(&path) {
            Ok(program) => program,
            Err(error) => return result(&path, &name, Status::Bork, error),
        };
        let (runtime, autoload) = match fixture.runtime_config(&path) {
            Ok(ConfigOutcome::Runtime { runtime, autoload }) => (runtime, autoload),
            Ok(ConfigOutcome::Extensions) => {
                return result(
                    &path,
                    &name,
                    Status::Skip,
                    "extension configuration is not supported",
                );
            }
            Err(error) => return result(&path, &name, Status::Bork, error),
        };
        if let Some(section) = UNSUPPORTED_SECTIONS
            .iter()
            .find(|section| fixture.sections.contains_key(**section))
        {
            return result(
                &path,
                &name,
                Status::Skip,
                format!("unsupported --{section}-- section"),
            );
        }
        let limits = Limits {
            max_instructions: self.options.max_instructions,
            max_execution: runtime
                .max_execution
                .finite()
                .map(|duration| Duration::from_secs(duration.seconds())),
            max_heap_bytes: runtime
                .memory_limit
                .finite()
                .map(|size| usize::try_from(size.bytes()).unwrap_or(usize::MAX)),
            max_input_bytes: runtime
                .post_max_size
                .finite()
                .map(thp_config::ByteSize::bytes),
            max_input_time: runtime
                .max_input
                .finite()
                .map(|duration| Duration::from_secs(duration.seconds())),
            max_stack_depth: runtime
                .max_stack_depth
                .map(|limit| usize::try_from(limit).unwrap_or(usize::MAX)),
            max_open_handles: runtime
                .max_open_handles
                .map(|limit| usize::try_from(limit).unwrap_or(usize::MAX)),
        };
        let base = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

        if let Some(result) = run_skipif(&path, &name, fixture, limits, &base) {
            return result;
        }
        let project_response = if autoload.is_empty() {
            None
        } else if !program.external {
            return result(
                &path,
                &name,
                Status::Bork,
                "autoload fixtures must use --FILE_EXTERNAL--",
            );
        } else {
            let mappings = match autoload
                .into_iter()
                .map(|(prefix, directories)| AutoloadMapping::new(prefix, directories))
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(mappings) => mappings,
                Err(error) => return result(&path, &name, Status::Bork, error.to_string()),
            };
            let provider = FilesystemSourceProvider::new(&base, mappings, &program.path);
            let engine = Engine::new(limits);
            let request = ProjectRequest::new(&base, &program.path);
            match engine.prepare_project(&request, &provider) {
                Ok(prepared) => Some(execute_prepared_fixture(
                    &engine, &prepared, fixture, limits, &base,
                )),
                Err(response) => Some(response),
            }
        };
        run_main(
            path,
            name,
            fixture,
            &program.source,
            limits,
            &base,
            project_response,
        )
    }
}

fn run_skipif(
    path: &Path,
    name: &str,
    fixture: &Fixture,
    limits: Limits,
    base: &Path,
) -> Option<TestResult> {
    let source = fixture.sections.get("SKIPIF")?;
    let source = match utf8_section(source, "SKIPIF") {
        Ok(source) => source,
        Err(error) => return Some(result(path, name, Status::Bork, error)),
    };
    let response = execute(path, source, limits, base, &[]);
    if response.status != ExecutionStatus::Success {
        return Some(result(
            path,
            name,
            Status::Bork,
            format!(
                "SKIPIF execution failed:\n{}",
                display_bytes(&combined_output(response))
            ),
        ));
    }
    let output = normalize(response.output);
    if output.is_empty() {
        None
    } else if let Some(reason) = skip_reason(&output) {
        Some(result(path, name, Status::Skip, reason))
    } else {
        Some(result(
            path,
            name,
            Status::Bork,
            format!("invalid SKIPIF output: {}", display_bytes(&output)),
        ))
    }
}

fn run_main(
    path: PathBuf,
    name: String,
    fixture: &Fixture,
    source: &str,
    limits: Limits,
    base: &Path,
    response: Option<Response>,
) -> TestResult {
    let body = fixture.sections.get("STDIN").map_or(&[][..], Vec::as_slice);
    let response = response.unwrap_or_else(|| execute(&path, source, limits, base, body));
    let host_failure = response.status == ExecutionStatus::HostError;
    let actual = normalize(combined_output(response));
    let expectation = fixture.expectation();
    let comparison = if host_failure {
        Err(ComparisonError::Bork(
            "host failed while executing the main program".to_owned(),
        ))
    } else {
        compare(expectation.kind, &normalize(expectation.bytes), &actual)
    };

    let mut status = match &comparison {
        Ok(()) => Status::Pass,
        Err(ComparisonError::Mismatch) => Status::Fail,
        Err(ComparisonError::Bork(_)) => Status::Bork,
    };
    let mut details = match comparison {
        Ok(()) => None,
        Err(ComparisonError::Mismatch) => Some(mismatch_details(
            expectation.kind,
            &normalize(expectation.bytes),
            &actual,
        )),
        Err(ComparisonError::Bork(message)) => Some(message),
    };

    if let Some(cleanup) = cleanup_failure(&path, fixture, limits, base) {
        if status != Status::Fail {
            status = Status::Bork;
        }
        match &mut details {
            Some(details) => {
                let _ = write!(details, "\n{cleanup}");
            }
            None => details = Some(cleanup),
        }
    }

    TestResult {
        path,
        name,
        status,
        details,
    }
}

fn cleanup_failure(path: &Path, fixture: &Fixture, limits: Limits, base: &Path) -> Option<String> {
    let clean = fixture.sections.get("CLEAN")?;
    let source = match utf8_section(clean, "CLEAN") {
        Ok(source) => source,
        Err(error) => return Some(error),
    };
    let response = execute(path, source, limits, base, &[]);
    let execution_status = response.status;
    let output = normalize(combined_output(response));
    if execution_status != ExecutionStatus::Success {
        Some(format!(
            "CLEAN execution failed: {}",
            display_bytes(&output)
        ))
    } else if !output.is_empty() {
        Some(format!("CLEAN produced output: {}", display_bytes(&output)))
    } else {
        None
    }
}

fn result(path: &Path, name: &str, status: Status, details: impl Into<String>) -> TestResult {
    TestResult {
        path: path.to_path_buf(),
        name: name.to_owned(),
        status,
        details: Some(details.into()),
    }
}

fn discover<I, P>(paths: I) -> Result<Vec<PathBuf>, RunnerError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut tests = BTreeSet::new();
    for input in paths {
        let input = input.as_ref();
        let metadata = fs::metadata(input).map_err(|error| {
            RunnerError::new(format!("cannot inspect `{}`: {error}", input.display()))
        })?;
        if metadata.is_dir() {
            discover_directory(input, &mut tests)?;
        } else if metadata.is_file()
            && input.extension().and_then(|extension| extension.to_str()) == Some("phpt")
        {
            tests.insert(fs::canonicalize(input).map_err(|error| {
                RunnerError::new(format!("cannot resolve `{}`: {error}", input.display()))
            })?);
        } else {
            return Err(RunnerError::new(format!(
                "`{}` is not a PHPT file or directory",
                input.display()
            )));
        }
    }
    Ok(tests.into_iter().collect())
}

fn discover_directory(directory: &Path, tests: &mut BTreeSet<PathBuf>) -> Result<(), RunnerError> {
    let entries = fs::read_dir(directory).map_err(|error| {
        RunnerError::new(format!(
            "cannot read directory `{}`: {error}",
            directory.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            RunnerError::new(format!(
                "cannot read an entry in `{}`: {error}",
                directory.display()
            ))
        })?;
        if is_hidden(&entry.file_name()) {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| {
            RunnerError::new(format!(
                "cannot inspect `{}`: {error}",
                entry.path().display()
            ))
        })?;
        if file_type.is_dir() {
            discover_directory(&entry.path(), tests)?;
        } else if file_type.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("phpt")
        {
            tests.insert(fs::canonicalize(entry.path()).map_err(|error| {
                RunnerError::new(format!("cannot resolve discovered test: {error}"))
            })?);
        } else if file_type.is_symlink()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("phpt")
        {
            let target = fs::metadata(entry.path()).map_err(|error| {
                RunnerError::new(format!(
                    "cannot inspect symlink `{}`: {error}",
                    entry.path().display()
                ))
            })?;
            if target.is_file() {
                tests.insert(fs::canonicalize(entry.path()).map_err(|error| {
                    RunnerError::new(format!("cannot resolve discovered test symlink: {error}"))
                })?);
            }
        }
    }
    Ok(())
}

fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.as_encoded_bytes().first() == Some(&b'.')
}

#[derive(Debug)]
struct Fixture {
    name: String,
    sections: BTreeMap<String, Vec<u8>>,
}

impl Fixture {
    fn parse(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        let mut sections = BTreeMap::<String, Vec<u8>>::new();
        let mut current: Option<String> = None;
        let mut offset = 0;
        while offset < bytes.len() {
            let end = bytes[offset..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |length| offset + length + 1);
            let mut line = &bytes[offset..end];
            line = line.strip_suffix(b"\n").unwrap_or(line);
            line = line.strip_suffix(b"\r").unwrap_or(line);
            if let Some(section) = parse_header(line) {
                if !KNOWN_SECTIONS.contains(&section.as_str()) {
                    return Err(format!("unknown --{section}-- section"));
                }
                if sections.contains_key(&section) {
                    return Err(format!("duplicate --{section}-- section"));
                }
                sections.insert(section.clone(), Vec::new());
                current = Some(section);
            } else if let Some(section) = &current {
                sections
                    .get_mut(section)
                    .expect("current section exists")
                    .extend_from_slice(&bytes[offset..end]);
            } else if !line.is_empty() {
                return Err("test must start with a section header".to_owned());
            }
            offset = end;
        }
        if current.is_none() {
            return Err("test contains no sections".to_owned());
        }
        for required in ["TEST"] {
            if !sections.contains_key(required) {
                return Err(format!("missing --{required}-- section"));
            }
        }
        if usize::from(sections.contains_key("FILE"))
            + usize::from(sections.contains_key("FILE_EXTERNAL"))
            != 1
        {
            return Err(
                "test requires exactly one --FILE-- or --FILE_EXTERNAL-- section".to_owned(),
            );
        }
        if ["EXPECT", "EXPECTF", "EXPECTREGEX"]
            .iter()
            .filter(|section| sections.contains_key(**section))
            .count()
            != 1
        {
            return Err(
                "test requires exactly one --EXPECT--, --EXPECTF--, or --EXPECTREGEX-- section"
                    .to_owned(),
            );
        }
        let name = std::str::from_utf8(sections.get("TEST").expect("validated TEST"))
            .map_err(|_| "--TEST-- must be valid UTF-8".to_owned())?
            .trim()
            .to_owned();
        if name.is_empty() {
            return Err(format!("{} has an empty --TEST-- section", path.display()));
        }
        for section in ["FILE", "FILE_EXTERNAL", "SKIPIF", "CLEAN", "CONFIG"] {
            if let Some(bytes) = sections.get(section) {
                utf8_section(bytes, section)?;
            }
        }
        Ok(Self { name, sections })
    }

    fn runtime_config(&self, path: &Path) -> Result<ConfigOutcome, String> {
        let Some(config) = self.sections.get("CONFIG") else {
            return Ok(ConfigOutcome::Runtime {
                runtime: RuntimeConfig::default(),
                autoload: AutoloadConfig::default(),
            });
        };
        let source = utf8_section(config, "CONFIG")?;
        let parsed = ProjectConfig::parse(path, source).map_err(|error| error.to_string())?;
        if parsed.target_names().next().is_some() {
            return Err("--CONFIG-- cannot declare targets".to_owned());
        }
        let resolved = parsed.resolve(None).map_err(|error| error.to_string())?;
        if !resolved.extensions().is_empty() {
            return Ok(ConfigOutcome::Extensions);
        }
        Ok(ConfigOutcome::Runtime {
            runtime: resolved.runtime,
            autoload: parsed.autoload().clone(),
        })
    }

    fn program(&self, path: &Path) -> Result<FixtureProgram, String> {
        if let Some(source) = self.sections.get("FILE") {
            return utf8_section(source, "FILE").map(|source| FixtureProgram {
                source: source.to_owned(),
                path: path.to_path_buf(),
                external: false,
            });
        }
        let external = self
            .sections
            .get("FILE_EXTERNAL")
            .expect("program section validated");
        let relative = utf8_section(external, "FILE_EXTERNAL")?.trim();
        if relative.is_empty() {
            return Err("--FILE_EXTERNAL-- path is empty".to_owned());
        }
        let relative = Path::new(relative);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err("--FILE_EXTERNAL-- path must stay within the fixture directory".to_owned());
        }
        let directory = path.parent().unwrap_or_else(|| Path::new(""));
        let root = fs::canonicalize(directory)
            .map_err(|error| format!("cannot resolve fixture directory: {error}"))?;
        let external = fs::canonicalize(directory.join(relative))
            .map_err(|error| format!("cannot resolve --FILE_EXTERNAL-- path: {error}"))?;
        if !external.starts_with(&root) || !external.is_file() {
            return Err(
                "--FILE_EXTERNAL-- path must name a file within the fixture directory".to_owned(),
            );
        }
        let bytes = fs::read(&external)
            .map_err(|error| format!("cannot read --FILE_EXTERNAL-- file: {error}"))?;
        let source = String::from_utf8(bytes)
            .map_err(|_| "--FILE_EXTERNAL-- source must be valid UTF-8".to_owned())?;
        Ok(FixtureProgram {
            source,
            path: external,
            external: true,
        })
    }

    fn expectation(&self) -> Expectation<'_> {
        for (section, kind) in [
            ("EXPECT", ExpectationKind::Exact),
            ("EXPECTF", ExpectationKind::Formatted),
            ("EXPECTREGEX", ExpectationKind::Regex),
        ] {
            if let Some(bytes) = self.sections.get(section) {
                return Expectation { kind, bytes };
            }
        }
        unreachable!("expectation validated")
    }
}

enum ConfigOutcome {
    Runtime {
        runtime: RuntimeConfig,
        autoload: AutoloadConfig,
    },
    Extensions,
}

struct FixtureProgram {
    source: String,
    path: PathBuf,
    external: bool,
}

#[derive(Clone, Copy, Debug)]
enum ExpectationKind {
    Exact,
    Formatted,
    Regex,
}

struct Expectation<'fixture> {
    kind: ExpectationKind,
    bytes: &'fixture [u8],
}

fn parse_header(line: &[u8]) -> Option<String> {
    let name = line.strip_prefix(b"--")?.strip_suffix(b"--")?;
    if name.is_empty()
        || !name
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || *byte == b'_')
    {
        return None;
    }
    Some(String::from_utf8(name.to_vec()).expect("ASCII section name"))
}

fn utf8_section<'section>(bytes: &'section [u8], name: &str) -> Result<&'section str, String> {
    std::str::from_utf8(bytes).map_err(|_| format!("--{name}-- must be valid UTF-8"))
}

fn execute_prepared_fixture(
    engine: &Engine,
    prepared: &PreparedProject,
    fixture: &Fixture,
    limits: Limits,
    base: &Path,
) -> Response {
    let body = fixture.sections.get("STDIN").map_or(&[][..], Vec::as_slice);
    let request_input = RequestInput::from_bytes(body.to_vec(), None, None)
        .expect("an unlimited in-memory request input cannot fail construction");
    engine.execute_prepared_with_context(
        prepared,
        &ExecutionContext {
            limits,
            filesystem_base: base.to_path_buf(),
            request_input,
        },
    )
}

#[allow(
    clippy::default_trait_access,
    reason = "the metrics concrete type is intentionally hidden behind thp-embed"
)]
fn execute(path: &Path, source: &str, limits: Limits, base: &Path, body: &[u8]) -> Response {
    let request_input = match RequestInput::from_bytes(
        body.to_vec(),
        limits.max_input_bytes,
        limits.max_input_time,
    ) {
        Ok(input) => input,
        Err(error) => {
            return Response {
                status: ExecutionStatus::RuntimeError,
                output: Vec::new(),
                error: error.to_string().into_bytes(),
                metrics: Default::default(),
                request: RequestStats::default(),
            };
        }
    };
    Engine::default().execute_with_context(
        path,
        source,
        &ExecutionContext {
            limits,
            filesystem_base: base.to_path_buf(),
            request_input,
        },
    )
}

fn combined_output(mut response: Response) -> Vec<u8> {
    response.output.append(&mut response.error);
    response.output
}

fn normalize(bytes: impl AsRef<[u8]>) -> Vec<u8> {
    let bytes = bytes.as_ref();
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"\r\n") {
            normalized.push(b'\n');
            index += 2;
        } else {
            normalized.push(bytes[index]);
            index += 1;
        }
    }
    let start = normalized
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(normalized.len());
    let end = normalized
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    normalized[start..end].to_vec()
}

fn skip_reason(output: &[u8]) -> Option<String> {
    if output.len() < 4 || !output[..4].eq_ignore_ascii_case(b"skip") {
        return None;
    }
    if output
        .get(4)
        .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b':')
    {
        return None;
    }
    let reason = output[4..]
        .iter()
        .copied()
        .skip_while(|byte| byte.is_ascii_whitespace() || *byte == b':')
        .collect::<Vec<_>>();
    if reason.is_empty() {
        Some("skip requested".to_owned())
    } else {
        Some(String::from_utf8_lossy(&reason).into_owned())
    }
}

enum ComparisonError {
    Mismatch,
    Bork(String),
}

fn compare(kind: ExpectationKind, expected: &[u8], actual: &[u8]) -> Result<(), ComparisonError> {
    match kind {
        ExpectationKind::Exact => {
            if expected == actual {
                Ok(())
            } else {
                Err(ComparisonError::Mismatch)
            }
        }
        ExpectationKind::Formatted => {
            let pattern = expectf_pattern(expected).map_err(ComparisonError::Bork)?;
            regex_matches(&pattern, actual)
        }
        ExpectationKind::Regex => {
            let pattern = std::str::from_utf8(expected).map_err(|_| {
                ComparisonError::Bork("--EXPECTREGEX-- must be valid UTF-8".to_owned())
            })?;
            regex_matches(pattern, actual)
        }
    }
}

fn regex_matches(pattern: &str, actual: &[u8]) -> Result<(), ComparisonError> {
    let anchored = format!(r"(?s:\A(?:{pattern})\z)");
    let regex = Regex::new(&anchored)
        .map_err(|error| ComparisonError::Bork(format!("invalid expectation regex: {error}")))?;
    if regex.is_match(actual) {
        Ok(())
    } else {
        Err(ComparisonError::Mismatch)
    }
}

fn expectf_pattern(expected: &[u8]) -> Result<String, String> {
    let mut pattern = String::new();
    let mut literal = Vec::new();
    let mut index = 0;
    while index < expected.len() {
        if expected[index] != b'%' || index + 1 >= expected.len() {
            literal.push(expected[index]);
            index += 1;
            continue;
        }
        if expected[index + 1] == b'r' {
            append_literal_pattern(&mut pattern, &literal);
            literal.clear();
            let raw_start = index + 2;
            let Some(relative_end) = expected[raw_start..]
                .windows(2)
                .position(|window| window == b"%r")
            else {
                return Err("unterminated %r block in --EXPECTF--".to_owned());
            };
            let raw_end = raw_start + relative_end;
            let raw = std::str::from_utf8(&expected[raw_start..raw_end])
                .map_err(|_| "%r block must be valid UTF-8".to_owned())?;
            pattern.push_str(&replace_placeholders(raw));
            index = raw_end + 2;
            continue;
        }
        let replacement = match expected[index + 1] {
            b'e' => Some(if std::path::MAIN_SEPARATOR == '\\' {
                r"\\"
            } else {
                "/"
            }),
            b's' => Some(r"[^\r\n]+"),
            b'S' => Some(r"[^\r\n]*"),
            b'a' => Some(r"(?-u:.)+?"),
            b'A' => Some(r"(?-u:.)*?"),
            b'w' => Some(r"\s*"),
            b'i' => Some(r"[+-]?\d+"),
            b'd' => Some(r"\d+"),
            b'x' => Some(r"[0-9a-fA-F]+"),
            b'f' => Some(r"[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:[Ee][+-]?\d+)?"),
            b'c' => Some(r"(?-u:.)"),
            b'0' => Some(r"\x00"),
            _ => None,
        };
        if let Some(replacement) = replacement {
            append_literal_pattern(&mut pattern, &literal);
            literal.clear();
            pattern.push_str(replacement);
            index += 2;
        } else {
            literal.push(expected[index]);
            index += 1;
        }
    }
    append_literal_pattern(&mut pattern, &literal);
    Ok(pattern)
}

fn replace_placeholders(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut replaced = String::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 1 < bytes.len() {
            let replacement = match bytes[index + 1] {
                b'e' => Some(if std::path::MAIN_SEPARATOR == '\\' {
                    r"\\"
                } else {
                    "/"
                }),
                b's' => Some(r"[^\r\n]+"),
                b'S' => Some(r"[^\r\n]*"),
                b'a' => Some(r"(?-u:.)+?"),
                b'A' => Some(r"(?-u:.)*?"),
                b'w' => Some(r"\s*"),
                b'i' => Some(r"[+-]?\d+"),
                b'd' => Some(r"\d+"),
                b'x' => Some(r"[0-9a-fA-F]+"),
                b'f' => Some(r"[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:[Ee][+-]?\d+)?"),
                b'c' => Some(r"(?-u:.)"),
                b'0' => Some(r"\x00"),
                _ => None,
            };
            if let Some(replacement) = replacement {
                replaced.push_str(replacement);
                index += 2;
                continue;
            }
        }
        let character = raw[index..]
            .chars()
            .next()
            .expect("index is within the string");
        replaced.push(character);
        index += character.len_utf8();
    }
    replaced
}

fn append_literal_pattern(pattern: &mut String, literal: &[u8]) {
    if literal.is_empty() {
        return;
    }
    pattern.push_str("(?-u:");
    for byte in literal {
        let _ = write!(pattern, r"\x{byte:02X}");
    }
    pattern.push(')');
}

fn mismatch_details(kind: ExpectationKind, expected: &[u8], actual: &[u8]) -> String {
    let label = match kind {
        ExpectationKind::Exact => "exact",
        ExpectationKind::Formatted => "formatted",
        ExpectationKind::Regex => "regex",
    };
    let first = expected
        .iter()
        .zip(actual)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or(expected.len().min(actual.len()));
    format!(
        "{label} output mismatch at byte {first}\nexpected: {}\nactual:   {}",
        display_bytes(expected),
        display_bytes(actual)
    )
}

fn display_bytes(bytes: &[u8]) -> String {
    let mut display = String::from("\"");
    for byte in bytes {
        match byte {
            b'\n' => display.push_str("\\n"),
            b'\r' => display.push_str("\\r"),
            b'\t' => display.push_str("\\t"),
            b'\\' => display.push_str("\\\\"),
            b'"' => display.push_str("\\\""),
            0x20..=0x7e => display.push(char::from(*byte)),
            _ => {
                let _ = write!(display, "\\x{byte:02x}");
            }
        }
    }
    display.push('"');
    display
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_crlf_and_trims_ascii_whitespace() {
        assert_eq!(normalize(b" \r\nhello\r\n\t"), b"hello");
    }

    #[test]
    fn exact_expectations_support_binary_bytes() {
        assert!(compare(ExpectationKind::Exact, b"a\0\xff", b"a\0\xff").is_ok());
        assert!(matches!(
            compare(ExpectationKind::Exact, b"a", b"b"),
            Err(ComparisonError::Mismatch)
        ));
    }

    #[test]
    fn formatted_placeholders_match_portable_values() {
        let expected = b"%e%s%S%a%A%w%i%d%x%f%c%0%%";
        let pattern = expectf_pattern(expected).expect("pattern");
        assert!(regex_matches(&pattern, b"/xanything \n+123ff1.5Z\0%%").is_ok());
        assert!(regex_matches(&expectf_pattern(b"\xff").expect("binary"), b"\xff").is_ok());
        assert!(matches!(
            regex_matches(&expectf_pattern(b"%f").expect("float"), b"1."),
            Err(ComparisonError::Mismatch)
        ));
    }

    #[test]
    fn formatted_raw_regex_is_dot_all_and_anchored() {
        let pattern = expectf_pattern(b"before%r.+%rafter").expect("pattern");
        assert!(regex_matches(&pattern, b"beforea\nafter").is_ok());
        assert!(matches!(
            regex_matches(&pattern, b"xbeforea\nafter"),
            Err(ComparisonError::Mismatch)
        ));
    }

    #[test]
    fn invalid_and_unterminated_regexes_bork() {
        assert!(expectf_pattern(b"%r[%r").is_ok());
        let pattern = expectf_pattern(b"%r[%r").expect("raw pattern");
        assert!(matches!(
            regex_matches(&pattern, b""),
            Err(ComparisonError::Bork(_))
        ));
        assert!(expectf_pattern(b"%rnever").is_err());
    }

    #[test]
    fn parser_rejects_duplicate_and_missing_sections() {
        let duplicate = b"--TEST--\na\n--FILE--\n<?thp\n--FILE--\n<?thp\n--EXPECT--\n";
        assert!(
            Fixture::parse(Path::new("a.phpt"), duplicate)
                .unwrap_err()
                .contains("duplicate")
        );
        let missing = b"--TEST--\na\n--FILE--\n<?thp\n";
        assert!(
            Fixture::parse(Path::new("a.phpt"), missing)
                .unwrap_err()
                .contains("exactly one --EXPECT--")
        );
    }

    #[test]
    fn parser_validates_program_utf8_but_allows_binary_expectation() {
        let invalid = b"--TEST--\na\n--FILE--\n\xff\n--EXPECT--\n";
        assert!(
            Fixture::parse(Path::new("a.phpt"), invalid)
                .unwrap_err()
                .contains("--FILE-- must be valid UTF-8")
        );

        let binary = b"--TEST--\na\n--FILE--\n<?thp\n--EXPECT--\n\xff";
        let fixture = Fixture::parse(Path::new("a.phpt"), binary).expect("binary expectation");
        assert_eq!(fixture.expectation().bytes, b"\xff");
    }

    #[test]
    fn discovery_recurses_ignores_hidden_and_deduplicates() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir(root.path().join("nested")).expect("nested");
        fs::write(root.path().join("nested/a.phpt"), "").expect("test");
        fs::write(root.path().join("nested/.hidden.phpt"), "").expect("hidden");
        let tests = discover([root.path(), &root.path().join("nested/a.phpt")]).expect("discover");
        assert_eq!(tests.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn discovery_follows_file_symlinks_but_not_directory_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("tempdir");
        let outside = tempfile::tempdir().expect("outside");
        fs::write(root.path().join("test.phpt"), "").expect("test");
        fs::write(outside.path().join("outside.phpt"), "").expect("outside test");
        symlink(
            root.path().join("test.phpt"),
            root.path().join("alias.phpt"),
        )
        .expect("file symlink");
        symlink(outside.path(), root.path().join("linked-directory")).expect("directory symlink");

        let tests = discover([root.path()]).expect("discover");
        assert_eq!(
            tests,
            [root.path().join("test.phpt").canonicalize().expect("path")]
        );
    }
}
