//! Measured orchestration for THP compiler phases.

#![allow(clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};

use thp_bytecode::{Program as BytecodeProgram, lower as lower_bytecode, verify};
use thp_config::ProjectConfig;
use thp_diagnostics::{Diagnostic, SourceFile, SourceId, SourceMap, Span};
use thp_hir::{Module as HirModule, lower as lower_hir};
use thp_metrics::{Metrics, Stage};
use thp_mir::{Module as MirModule, lower as lower_mir};
use thp_modules::{
    AutoloadMapping, FilesystemSourceProvider, ModuleGraph, ModuleId, ModuleInterface, ModulePath,
    ModuleSourceProvider, build_export_index, extract_interface, resolve_program,
};
use thp_opcache::{
    ArtifactKind, CACHE_FORMAT_VERSION, CacheKey, CacheStatus, FrozenManifest, Store,
};
use thp_syntax::{Program as AstProgram, Token, lex, parse_tokens};

#[derive(Debug)]
pub struct Compilation {
    pub source: SourceFile,
    pub tokens: Vec<Token>,
    pub ast: AstProgram,
    pub hir: Option<HirModule>,
    pub mir: Option<MirModule>,
    pub bytecode: Option<BytecodeProgram>,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: Metrics,
}

#[derive(Debug)]
pub struct ExecutableCompilation {
    pub source: SourceFile,
    pub bytecode: Option<BytecodeProgram>,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: Metrics,
    pub cache_status: CacheStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitCacheReport {
    pub interface_reused: bool,
    pub object_reused: bool,
    pub rebuilt: bool,
}

#[derive(Debug)]
pub struct ProjectUnit {
    pub module: ModulePath,
    pub source_id: SourceId,
    pub source: SourceFile,
    pub tokens: Vec<Token>,
    pub ast: AstProgram,
    pub interface: Option<ModuleInterface>,
    pub body_hash: String,
    pub cache: UnitCacheReport,
}

#[derive(Clone, Debug)]
pub struct ProjectRequest {
    pub project_root: PathBuf,
    pub entry: PathBuf,
    pub target: Option<String>,
}

impl ProjectRequest {
    pub fn new(project_root: impl Into<PathBuf>, entry: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            entry: entry.into(),
            target: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProjectDiagnostic {
    pub source: SourceId,
    pub diagnostic: Diagnostic,
}

#[derive(Debug)]
pub struct ProjectCompilation {
    pub sources: SourceMap,
    pub units: Vec<ProjectUnit>,
    pub interfaces: Vec<ModuleInterface>,
    pub graph: Option<ModuleGraph>,
    pub hir: Option<HirModule>,
    pub mir: Option<MirModule>,
    pub bytecode: Option<BytecodeProgram>,
    pub diagnostics: Vec<ProjectDiagnostic>,
    pub metrics: Metrics,
}

impl ProjectCompilation {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty() && self.bytecode.is_some()
    }

    pub fn rendered_diagnostics(&self) -> String {
        self.diagnostics
            .iter()
            .map(|item| {
                item.diagnostic
                    .render_with_sources(&self.sources, item.source)
            })
            .collect()
    }
}

/// A verified linked program retained independently of its source inputs.
#[derive(Clone, Debug)]
pub struct PreparedProject {
    pub bytecode: BytecodeProgram,
    pub sources: SourceMap,
    pub entry_source: SourceId,
}

impl PreparedProject {
    pub fn from_compilation(compilation: &ProjectCompilation) -> Option<Self> {
        let entry_source = compilation
            .units
            .iter()
            .find(|unit| unit.module.is_entry)?
            .source_id;
        Some(Self {
            bytecode: compilation.bytecode.clone()?,
            sources: compilation.sources.clone(),
            entry_source,
        })
    }
}

impl ExecutableCompilation {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty() && self.bytecode.is_some()
    }

    pub fn rendered_diagnostics(&self) -> String {
        self.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.render(&self.source))
            .collect::<String>()
    }
}

impl Compilation {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty() && self.bytecode.is_some()
    }

    pub fn rendered_diagnostics(&self) -> String {
        self.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.render(&self.source))
            .collect::<String>()
    }
}

#[derive(Debug)]
pub struct LoadError {
    pub path: PathBuf,
    pub message: String,
    pub metrics: Metrics,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.message)
    }
}

impl std::error::Error for LoadError {}

/// Loads and compiles a UTF-8 THP source file.
///
/// # Errors
///
/// Returns an I/O or UTF-8 load error. Language errors are returned inside
/// `Compilation` so callers can inspect every structured diagnostic.
pub fn compile_path(path: impl AsRef<Path>) -> Result<Compilation, LoadError> {
    let path = path.as_ref();
    let mut metrics = Metrics::default();
    let loaded = metrics.measure(Stage::SourceLoading, || fs::read(path));
    let bytes = match loaded {
        Ok(bytes) => bytes,
        Err(error) => {
            return Err(LoadError {
                path: path.to_path_buf(),
                message: error.to_string(),
                metrics,
            });
        }
    };
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return Err(LoadError {
                path: path.to_path_buf(),
                message: format!("THP source must be UTF-8: {error}"),
                metrics,
            });
        }
    };
    let source = SourceFile::new(path, text);
    Ok(compile_source_with_metrics(source, metrics))
}

/// Loads `thp.toml` from exactly the requested root and compiles the statically
/// discovered module set as one linked program.
///
/// # Errors
///
/// Returns project configuration, discovery, or source loading failures.
/// Language errors remain available in [`ProjectCompilation::diagnostics`].
pub fn compile_project(request: &ProjectRequest) -> Result<ProjectCompilation, LoadError> {
    let configuration = ProjectConfig::load(&request.project_root).map_err(|error| LoadError {
        path: error.path.clone(),
        message: error.to_string(),
        metrics: Metrics::default(),
    })?;
    let mappings = configuration
        .autoload()
        .iter()
        .map(|(prefix, directories)| AutoloadMapping::new(prefix, directories.clone()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| LoadError {
            path: request.project_root.join("thp.toml"),
            message: error.to_string(),
            metrics: Metrics::default(),
        })?;
    let provider = FilesystemSourceProvider::new(&request.project_root, mappings, &request.entry);
    compile_project_with_provider(request, &provider)
}

/// Compiles a project using a host-provided synchronous source provider.
///
/// # Errors
///
/// Returns provider enumeration or loading failures.
pub fn compile_project_with_provider(
    request: &ProjectRequest,
    provider: &dyn ModuleSourceProvider,
) -> Result<ProjectCompilation, LoadError> {
    let mut metrics = Metrics::default();
    let modules = metrics
        .measure(Stage::ModuleDiscovery, || provider.enumerate())
        .map_err(|error| LoadError {
            path: request.project_root.clone(),
            message: error.to_string(),
            metrics: Metrics::default(),
        })?;
    let mut sources = SourceMap::default();
    let mut units = Vec::new();
    let mut diagnostics = Vec::new();

    for module in modules {
        let source = metrics
            .measure(Stage::SourceLoading, || provider.load(&module))
            .map_err(|error| LoadError {
                path: module.path.clone(),
                message: error.to_string(),
                metrics: Metrics::default(),
            })?;
        let source_id = sources.add(source.clone());
        let lexed = metrics.measure(Stage::Lexing, || lex(&source));
        let parsed = metrics.measure(Stage::Parsing, || {
            parse_tokens(&source, lexed.tokens, lexed.diagnostics)
        });
        diagnostics.extend(
            parsed
                .diagnostics
                .into_iter()
                .map(|diagnostic| ProjectDiagnostic {
                    source: source_id,
                    diagnostic,
                }),
        );
        let mut body_hasher = blake3::Hasher::new();
        body_hasher.update(b"THP module body");
        body_hasher.update(source.text().as_bytes());
        body_hasher.update(std::env::consts::ARCH.as_bytes());
        body_hasher.update(std::env::consts::OS.as_bytes());
        let body_hash = body_hasher.finalize().to_hex().to_string();
        units.push(ProjectUnit {
            module,
            source_id,
            source,
            tokens: parsed.tokens,
            ast: parsed.program,
            interface: None,
            body_hash,
            cache: UnitCacheReport {
                interface_reused: false,
                object_reused: false,
                rebuilt: true,
            },
        });
    }

    for unit in &mut units {
        if !unit.module.is_entry {
            for statement in &unit.ast.statements {
                if !matches!(
                    statement.kind,
                    thp_syntax::StmtKind::Function(_)
                        | thp_syntax::StmtKind::Class(_)
                        | thp_syntax::StmtKind::Interface(_)
                        | thp_syntax::StmtKind::Trait(_)
                ) {
                    diagnostics.push(ProjectDiagnostic {
                        source: unit.source_id,
                        diagnostic: Diagnostic::error(
                            "modules",
                            "M0004",
                            statement.span,
                            "an imported module cannot contain executable top-level statements",
                        )
                        .with_note(
                            "move initialization into an explicit function called by the entry module",
                        ),
                    });
                }
            }
        }
        match metrics.measure(Stage::InterfaceExtraction, || {
            extract_interface(&unit.module, unit.source_id, &unit.ast)
        }) {
            Ok(interface) => unit.interface = Some(interface),
            Err(diagnostic) => diagnostics.push(ProjectDiagnostic {
                source: unit.source_id,
                diagnostic,
            }),
        }
    }
    let interfaces = units
        .iter()
        .filter_map(|unit| unit.interface.clone())
        .collect::<Vec<_>>();
    let index = match build_export_index(&interfaces) {
        Ok(index) => Some(index),
        Err(duplicates) => {
            for (first, second) in duplicates {
                diagnostics.push(ProjectDiagnostic {
                    source: second.source,
                    diagnostic: Diagnostic::error(
                        "modules",
                        "M0005",
                        second.span,
                        format!(
                            "duplicate exported {} `{}`",
                            second.kind.as_str(),
                            second.name
                        ),
                    )
                    .with_source_label(
                        first.source,
                        first.span,
                        "first exported here",
                    ),
                });
            }
            None
        }
    };

    let mut graph = None;
    if let Some(index) = &index {
        let mut graph_reported_unknown_imports = false;
        let graph_units = units
            .iter()
            .map(|unit| (unit.module.clone(), unit.ast.clone()))
            .collect::<Vec<_>>();
        match ModuleGraph::build(&graph_units, index) {
            Ok(resolved) => graph = Some(resolved),
            Err(unknown) => {
                graph_reported_unknown_imports = true;
                for (module, name, span) in unknown {
                    if let Some(unit) = units.iter().find(|unit| unit.module.id == module) {
                        diagnostics.push(ProjectDiagnostic {
                            source: unit.source_id,
                            diagnostic: Diagnostic::error(
                                "modules",
                                "M0003",
                                span,
                                format!("unknown imported symbol `{name}`"),
                            ),
                        });
                    }
                }
            }
        }
        for unit in &mut units {
            diagnostics.extend(
                resolve_program(&mut unit.ast, index)
                    .into_iter()
                    .filter(|diagnostic| {
                        !graph_reported_unknown_imports || diagnostic.code != "M0003"
                    })
                    .map(|diagnostic| ProjectDiagnostic {
                        source: unit.source_id,
                        diagnostic,
                    }),
            );
        }
    }

    if !diagnostics.is_empty() {
        return Ok(ProjectCompilation {
            sources,
            units,
            interfaces,
            graph,
            hir: None,
            mir: None,
            bytecode: None,
            diagnostics,
            metrics,
        });
    }

    let Some(entry) = units.iter().find(|unit| unit.module.is_entry) else {
        return Err(LoadError {
            path: request.entry.clone(),
            message: "module provider did not enumerate an entry".to_owned(),
            metrics,
        });
    };
    let mut statements = Vec::new();
    for unit in &units {
        statements.extend(unit.ast.statements.clone());
    }
    let linked_ast = AstProgram {
        namespace: None,
        imports: Vec::new(),
        statements,
        span: entry.ast.span,
    };
    let lowered_hir = metrics.measure(Stage::Hir, || lower_hir(&linked_ast));
    diagnostics.extend(
        lowered_hir
            .diagnostics
            .into_iter()
            .map(|diagnostic| ProjectDiagnostic {
                source: entry.source_id,
                diagnostic,
            }),
    );
    let hir = lowered_hir.module;
    if !diagnostics.is_empty() {
        return Ok(ProjectCompilation {
            sources,
            units,
            interfaces,
            graph,
            hir: Some(hir),
            mir: None,
            bytecode: None,
            diagnostics,
            metrics,
        });
    }
    let mir = metrics.measure(Stage::Mir, || lower_mir(&hir));
    let bytecode = metrics.measure(Stage::Linking, || lower_bytecode(&mir));
    if let Err(error) = metrics.measure(Stage::Verification, || verify(&bytecode)) {
        diagnostics.push(ProjectDiagnostic {
            source: entry.source_id,
            diagnostic: Diagnostic::error(
                "bytecode_verification",
                "B0001",
                Span::empty(0),
                error.to_string(),
            ),
        });
    }
    Ok(ProjectCompilation {
        sources,
        units,
        interfaces,
        graph,
        hir: Some(hir),
        mir: Some(mir),
        bytecode: diagnostics.is_empty().then_some(bytecode),
        diagnostics,
        metrics,
    })
}

/// Compiles every discovered unit and atomically publishes a frozen manifest
/// after its interfaces, objects, and linked program are durable.
///
/// # Errors
///
/// Returns project compilation or cache filesystem failures.
pub fn cache_warm_project(
    request: &ProjectRequest,
    store: &Store,
) -> Result<(ProjectCompilation, Option<FrozenManifest>), LoadError> {
    let mut compilation = compile_project(request)?;
    if !compilation.is_success() {
        return Ok((compilation, None));
    }
    let configuration =
        fs::read(request.project_root.join("thp.toml")).map_err(|error| LoadError {
            path: request.project_root.join("thp.toml"),
            message: error.to_string(),
            metrics: Metrics::default(),
        })?;
    let mut interface_hashes = compilation
        .interfaces
        .iter()
        .map(|interface| interface.interface_hash.clone())
        .collect::<Vec<_>>();
    interface_hashes.sort();
    for interface in &compilation.interfaces {
        let key =
            CacheKey::from_digest(interface.interface_hash.clone()).map_err(|error| LoadError {
                path: store.directory().to_path_buf(),
                message: error.to_string(),
                metrics: Metrics::default(),
            })?;
        let payload = format!(
            "THPI {}\nmodule={}\nnamespace={}\nexports={:?}\n",
            thp_modules::INTERFACE_FORMAT_VERSION,
            interface.module,
            interface.namespace,
            interface
                .exports
                .iter()
                .map(|export| (export.kind.as_str(), export.name.as_str()))
                .collect::<Vec<_>>()
        );
        let reused = compilation
            .metrics
            .measure(Stage::CacheLookup, || {
                store.lookup_artifact(ArtifactKind::Interface, &key)
            })
            .ok()
            .flatten()
            .is_some();
        if !reused {
            compilation
                .metrics
                .measure(Stage::IncrementalRebuild, || {
                    store.store_artifact(ArtifactKind::Interface, &key, payload.as_bytes())
                })
                .map_err(|error| LoadError {
                    path: store.directory().to_path_buf(),
                    message: format!("cannot write module interface cache: {error}"),
                    metrics: Metrics::default(),
                })?;
        }
        if let Some(unit) = compilation
            .units
            .iter_mut()
            .find(|unit| unit.module.id == interface.module)
        {
            unit.cache.interface_reused = reused;
        }
    }
    let mut object_hashes = Vec::new();
    for unit in &mut compilation.units {
        let mut consumed_interfaces = compilation
            .graph
            .as_ref()
            .into_iter()
            .flat_map(|graph| &graph.edges)
            .filter(|edge| edge.from == unit.module.id)
            .filter_map(|edge| {
                compilation
                    .interfaces
                    .iter()
                    .find(|interface| interface.module == edge.to)
                    .map(|interface| interface.interface_hash.clone())
            })
            .collect::<Vec<_>>();
        if let Some(own) = compilation
            .interfaces
            .iter()
            .find(|interface| interface.module == unit.module.id)
        {
            consumed_interfaces.push(own.interface_hash.clone());
        }
        consumed_interfaces.sort();
        consumed_interfaces.dedup();
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"THP module object");
        hasher.update(&thp_modules::OBJECT_FORMAT_VERSION.to_le_bytes());
        hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
        hasher.update(&thp_bytecode::BYTECODE_SCHEMA_VERSION.to_le_bytes());
        hasher.update(b"THP prelude v1");
        hasher.update(blake3::hash(&configuration).as_bytes());
        hasher.update(unit.module.id.as_str().as_bytes());
        hasher.update(unit.body_hash.as_bytes());
        for interface_hash in &consumed_interfaces {
            hasher.update(interface_hash.as_bytes());
        }
        hasher.update(std::env::consts::OS.as_bytes());
        hasher.update(std::env::consts::ARCH.as_bytes());
        let hash = hasher.finalize().to_hex().to_string();
        let key = CacheKey::from_digest(hash.clone()).map_err(|error| LoadError {
            path: store.directory().to_path_buf(),
            message: error.to_string(),
            metrics: Metrics::default(),
        })?;
        let payload = format!(
            "THPO {}\nmodule={}\nbody={}\ninterfaces={}\n",
            thp_modules::OBJECT_FORMAT_VERSION,
            unit.module.id,
            unit.body_hash,
            consumed_interfaces.join(",")
        );
        let reused = compilation
            .metrics
            .measure(Stage::CacheLookup, || {
                store.lookup_artifact(ArtifactKind::Object, &key)
            })
            .ok()
            .flatten()
            .is_some();
        if !reused {
            compilation
                .metrics
                .measure(Stage::IncrementalRebuild, || {
                    store.store_artifact(ArtifactKind::Object, &key, payload.as_bytes())
                })
                .map_err(|error| LoadError {
                    path: store.directory().to_path_buf(),
                    message: format!("cannot write module object cache: {error}"),
                    metrics: Metrics::default(),
                })?;
        }
        unit.cache.object_reused = reused;
        unit.cache.rebuilt = !reused;
        object_hashes.push(hash);
    }
    object_hashes.sort();
    let entry_id = compilation
        .units
        .iter()
        .find(|unit| unit.module.is_entry)
        .ok_or_else(|| LoadError {
            path: request.entry.clone(),
            message: "project compilation has no entry".to_owned(),
            metrics: Metrics::default(),
        })?
        .module
        .id
        .to_string();
    let linker_inputs = format!("{entry_id}\n{}", object_hashes.join("\n"));
    let program_key = CacheKey::calculate(
        linker_inputs.as_bytes(),
        env!("CARGO_PKG_VERSION"),
        &configuration,
    );
    let program = compilation.bytecode.as_ref().ok_or_else(|| LoadError {
        path: request.entry.clone(),
        message: "successful project compilation has no bytecode".to_owned(),
        metrics: Metrics::default(),
    })?;
    compilation
        .metrics
        .measure(Stage::Cache, || store.store(&program_key, program))
        .map_err(|error| LoadError {
            path: store.directory().to_path_buf(),
            message: format!("cannot write linked program cache: {error}"),
            metrics: Metrics::default(),
        })?;
    let project_fingerprint = project_fingerprint(request, &configuration);
    let manifest = FrozenManifest {
        manifest_version: CACHE_FORMAT_VERSION,
        compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
        project_fingerprint: project_fingerprint.clone(),
        entry_id,
        program_key,
        interface_hashes,
        object_hashes,
    };
    let manifest_key = manifest_key(&project_fingerprint);
    compilation
        .metrics
        .measure(Stage::Cache, || {
            store.store_manifest(&manifest_key, &manifest)
        })
        .map_err(|error| LoadError {
            path: store.directory().to_path_buf(),
            message: format!("cannot publish frozen manifest: {error}"),
            metrics: Metrics::default(),
        })?;
    Ok((compilation, Some(manifest)))
}

/// Loads one verified linked program using only project configuration and
/// frozen cache artifacts; source directories are not enumerated.
///
/// # Errors
///
/// Returns missing, stale, corrupt, or incompatible artifact errors.
pub fn load_frozen_project(
    request: &ProjectRequest,
    store: &Store,
) -> Result<PreparedProject, LoadError> {
    let configuration_path = request.project_root.join("thp.toml");
    let configuration = fs::read(&configuration_path).map_err(|error| LoadError {
        path: configuration_path.clone(),
        message: error.to_string(),
        metrics: Metrics::default(),
    })?;
    let expected_fingerprint = project_fingerprint(request, &configuration);
    let manifest = store
        .load_manifest(&manifest_key(&expected_fingerprint))
        .map_err(|error| LoadError {
            path: store.directory().to_path_buf(),
            message: format!("cannot load frozen manifest: {error}"),
            metrics: Metrics::default(),
        })?;
    if manifest.compiler_version != env!("CARGO_PKG_VERSION")
        || manifest.project_fingerprint != expected_fingerprint
    {
        return Err(LoadError {
            path: store.directory().to_path_buf(),
            message: "frozen manifest is stale or belongs to another compiler/project".to_owned(),
            metrics: Metrics::default(),
        });
    }
    let lookup = store
        .lookup(&manifest.program_key)
        .map_err(|error| LoadError {
            path: store.directory().to_path_buf(),
            message: format!("cannot load frozen linked program: {error}"),
            metrics: Metrics::default(),
        })?;
    let bytecode = lookup.program.ok_or_else(|| LoadError {
        path: store.directory().to_path_buf(),
        message: match lookup.status {
            CacheStatus::Corrupt => "frozen linked program is corrupt",
            _ => "frozen linked program is missing",
        }
        .to_owned(),
        metrics: Metrics::default(),
    })?;
    let entry_path = if request.entry.is_absolute() {
        request.entry.clone()
    } else {
        request.project_root.join(&request.entry)
    };
    let mut sources = SourceMap::default();
    let entry_source = sources.add(SourceFile::new(entry_path, ""));
    Ok(PreparedProject {
        bytecode,
        sources,
        entry_source,
    })
}

fn project_fingerprint(request: &ProjectRequest, configuration: &[u8]) -> String {
    let entry = request
        .entry
        .strip_prefix(&request.project_root)
        .unwrap_or(&request.entry)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"THP frozen project");
    hasher.update(&thp_modules::MANIFEST_FORMAT_VERSION.to_le_bytes());
    hasher.update(configuration);
    hasher.update(entry.as_bytes());
    if let Some(target) = &request.target {
        hasher.update(target.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn manifest_key(project_fingerprint: &str) -> CacheKey {
    let digest = blake3::hash(format!("THP manifest key\n{project_fingerprint}").as_bytes())
        .to_hex()
        .to_string();
    CacheKey::from_digest(digest).expect("BLAKE3 is a valid cache key")
}

/// Loads executable bytecode through a content-addressed persistent cache.
///
/// Cache keys cover source, compiler/bytecode versions, host target, and the
/// caller-provided effective configuration bytes. Invalid cache artifacts are
/// compiled again and atomically replaced.
///
/// # Errors
///
/// Returns source or cache filesystem errors. Language errors are returned in
/// the executable compilation.
pub fn compile_path_cached(
    path: impl AsRef<Path>,
    store: &Store,
    configuration: &[u8],
) -> Result<ExecutableCompilation, LoadError> {
    let path = path.as_ref();
    let mut metrics = Metrics::default();
    let bytes = metrics
        .measure(Stage::SourceLoading, || fs::read(path))
        .map_err(|error| LoadError {
            path: path.to_path_buf(),
            message: error.to_string(),
            metrics: Metrics::default(),
        })?;
    let text = String::from_utf8(bytes.clone()).map_err(|error| LoadError {
        path: path.to_path_buf(),
        message: format!("THP source must be UTF-8: {error}"),
        metrics: Metrics::default(),
    })?;
    let source = SourceFile::new(path, text);
    let key = CacheKey::calculate(&bytes, env!("CARGO_PKG_VERSION"), configuration);
    let lookup = metrics
        .measure(Stage::Cache, || store.lookup(&key))
        .map_err(|error| LoadError {
            path: store.directory().to_path_buf(),
            message: format!("cannot read OPcache: {error}"),
            metrics: Metrics::default(),
        })?;
    if let Some(program) = lookup.program {
        if let Some(measurement) = metrics.last_mut() {
            measurement.set_output(
                program.instruction_count(),
                thp_bytecode::encode(&program).len(),
            );
        }
        return Ok(ExecutableCompilation {
            source,
            bytecode: Some(program),
            diagnostics: Vec::new(),
            metrics,
            cache_status: CacheStatus::Hit,
        });
    }

    let status = lookup.status;
    let mut compilation = compile_source_with_metrics(source, metrics);
    if let Some(program) = &compilation.bytecode {
        compilation
            .metrics
            .measure(Stage::Cache, || store.store(&key, program))
            .map_err(|error| LoadError {
                path: store.directory().to_path_buf(),
                message: format!("cannot write OPcache: {error}"),
                metrics: Metrics::default(),
            })?;
    }
    Ok(ExecutableCompilation {
        source: compilation.source,
        bytecode: compilation.bytecode,
        diagnostics: compilation.diagnostics,
        metrics: compilation.metrics,
        cache_status: status,
    })
}

pub fn compile_text(path: impl Into<PathBuf>, source: impl Into<String>) -> Compilation {
    let source = SourceFile::new(path, source.into());
    compile_source_with_metrics(source, Metrics::default())
}

pub fn compile_source(source: SourceFile) -> Compilation {
    compile_source_with_metrics(source, Metrics::default())
}

fn compile_source_with_metrics(source: SourceFile, mut metrics: Metrics) -> Compilation {
    let lexed = metrics.measure(Stage::Lexing, || lex(&source));
    if let Some(measurement) = metrics.last_mut() {
        measurement.set_output(
            lexed.tokens.len(),
            lexed.tokens.len() * std::mem::size_of::<Token>(),
        );
    }
    let parsed = metrics.measure(Stage::Parsing, || {
        parse_tokens(&source, lexed.tokens, lexed.diagnostics)
    });
    if let Some(measurement) = metrics.last_mut() {
        measurement.set_output(parsed.program.statements.len(), source.len());
    }
    let mut diagnostics = parsed.diagnostics;
    let tokens = parsed.tokens;
    let mut ast = parsed.program;
    if !diagnostics.is_empty() {
        return Compilation {
            source,
            tokens,
            ast,
            hir: None,
            mir: None,
            bytecode: None,
            diagnostics,
            metrics,
        };
    }

    if ast.namespace.is_some() || !ast.imports.is_empty() {
        let module = ModulePath {
            id: ModuleId::synthetic_entry(source.path()),
            path: source.path().to_path_buf(),
            canonical_path: source.path().to_path_buf(),
            expected_namespace: String::new(),
            is_entry: true,
        };
        if let Ok(interface) = extract_interface(&module, SourceId(0), &ast) {
            match build_export_index(&[interface]) {
                Ok(index) => diagnostics.extend(resolve_program(&mut ast, &index)),
                Err(duplicates) => {
                    diagnostics.extend(duplicates.into_iter().map(|(_, duplicate)| {
                        Diagnostic::error(
                            "modules",
                            "M0005",
                            duplicate.span,
                            format!(
                                "duplicate exported {} `{}`",
                                duplicate.kind.as_str(),
                                duplicate.name
                            ),
                        )
                    }));
                }
            }
        }
    }
    if !diagnostics.is_empty() {
        return Compilation {
            source,
            tokens,
            ast,
            hir: None,
            mir: None,
            bytecode: None,
            diagnostics,
            metrics,
        };
    }

    let lowered_hir = metrics.measure(Stage::Hir, || lower_hir(&ast));
    if let Some(measurement) = metrics.last_mut() {
        measurement.set_output(
            lowered_hir.module.expression_count(),
            std::mem::size_of_val(&lowered_hir.module),
        );
    }
    diagnostics.extend(lowered_hir.diagnostics);
    let hir = lowered_hir.module;
    if !diagnostics.is_empty() {
        return Compilation {
            source,
            tokens,
            ast,
            hir: Some(hir),
            mir: None,
            bytecode: None,
            diagnostics,
            metrics,
        };
    }

    let mir = metrics.measure(Stage::Mir, || lower_mir(&hir));
    if let Some(measurement) = metrics.last_mut() {
        measurement.set_output(mir.instruction_count(), std::mem::size_of_val(&mir));
    }
    let bytecode = metrics.measure(Stage::Bytecode, || lower_bytecode(&mir));
    if let Some(measurement) = metrics.last_mut() {
        measurement.set_output(
            bytecode.instruction_count(),
            thp_bytecode::encode(&bytecode).len(),
        );
    }
    let verification = metrics.measure(Stage::Verification, || verify(&bytecode));
    if let Err(error) = verification {
        diagnostics.push(
            Diagnostic::error(
                "bytecode_verification",
                "B0001",
                Span::empty(0),
                error.to_string(),
            )
            .with_note("this is a compiler defect; verified HIR must produce valid bytecode"),
        );
    }

    Compilation {
        source,
        tokens,
        ast,
        hir: Some(hir),
        mir: Some(mir),
        bytecode: diagnostics.is_empty().then_some(bytecode),
        diagnostics,
        metrics,
    }
}

#[cfg(test)]
mod tests {
    use thp_opcache::{CacheStatus, Store};

    use super::{
        ProjectRequest, cache_warm_project, compile_path_cached, compile_project, compile_text,
        load_frozen_project,
    };

    #[test]
    fn compiles_vertical_core_through_all_phases() {
        let compilation = compile_text(
            "test.thp",
            "<?thp\nfunction square(int $x): int { return $x * $x; }\necho square(4);",
        );
        assert!(
            compilation.is_success(),
            "{}",
            compilation.rendered_diagnostics()
        );
        assert!(compilation.hir.is_some());
        assert!(compilation.mir.is_some());
        assert!(compilation.bytecode.is_some());
        assert_eq!(compilation.metrics.measurements().len(), 6);
    }

    #[test]
    fn stops_before_hir_on_parse_errors() {
        let compilation = compile_text("test.thp", "<?thp\n$x = ;");
        assert!(!compilation.is_success());
        assert!(compilation.hir.is_none());
        assert!(compilation.bytecode.is_none());
    }

    #[test]
    fn accepts_echoing_output_scalars() {
        let compilation = compile_text(
            "test.thp",
            "<?thp\necho \"value\";\necho 42;\necho 1.5;\necho true;\necho false;",
        );
        assert!(
            compilation.is_success(),
            "{}",
            compilation.rendered_diagnostics()
        );
        assert!(compilation.bytecode.is_some());
    }

    #[test]
    fn rejects_null_output() {
        let compilation = compile_text(
            "test.thp",
            "<?thp\necho null;\n$value = \"missing: \" . null;",
        );
        assert!(!compilation.is_success());
        assert!(compilation.bytecode.is_none());
        assert_eq!(
            compilation
                .diagnostics
                .iter()
                .filter(|diagnostic| matches!(diagnostic.code, "T0002" | "T0502"))
                .count(),
            2
        );
    }

    #[test]
    fn cached_compilation_skips_frontend_phases_on_a_hit() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("program.thp");
        std::fs::write(&source, "<?thp\necho \"cached\";").unwrap();
        let store = Store::new(directory.path().join("cache"));

        let first = compile_path_cached(&source, &store, b"test").unwrap();
        assert_eq!(first.cache_status, CacheStatus::Miss);
        let second = compile_path_cached(&source, &store, b"test").unwrap();
        assert_eq!(second.cache_status, CacheStatus::Hit);
        assert_eq!(second.metrics.measurements().len(), 2);
        assert!(second.is_success());
    }

    #[test]
    fn compiles_a_namespaced_cross_file_project() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("src/Math")).unwrap();
        std::fs::write(
            directory.path().join("thp.toml"),
            "[autoload]\n\"App\\\\\" = \"src/\"\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("src/Math/functions.thp"),
            "<?thp\nnamespace App\\Math;\nfunction square(int $x): int { return $x * $x; }\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("main.thp"),
            "<?thp\nuse function App\\Math\\square;\necho square(4) . \"\";\n",
        )
        .unwrap();
        let compilation = compile_project(&ProjectRequest::new(
            directory.path(),
            directory.path().join("main.thp"),
        ))
        .unwrap();
        assert!(
            compilation.is_success(),
            "{}",
            compilation.rendered_diagnostics()
        );
        assert_eq!(compilation.interfaces.len(), 2);
        let kinds = compilation
            .graph
            .as_ref()
            .unwrap()
            .edges
            .iter()
            .map(|edge| edge.kind)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(kinds.contains(&thp_modules::DependencyKind::Import));
        assert!(kinds.contains(&thp_modules::DependencyKind::Body));
    }

    #[test]
    fn warms_and_loads_a_frozen_project_without_sources() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("src")).unwrap();
        std::fs::write(
            directory.path().join("thp.toml"),
            "[autoload]\n\"App\\\\\" = \"src/\"\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("src/Message.thp"),
            "<?thp\nnamespace App;\nfunction message(): string { return \"frozen\"; }\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("main.thp"),
            "<?thp\nuse function App\\message;\necho message();\n",
        )
        .unwrap();
        let request = ProjectRequest::new(directory.path(), directory.path().join("main.thp"));
        let store = Store::new(directory.path().join("cache"));
        let (compilation, manifest) = cache_warm_project(&request, &store).unwrap();
        assert!(compilation.is_success());
        assert!(manifest.is_some());
        std::fs::remove_file(directory.path().join("main.thp")).unwrap();
        std::fs::remove_dir_all(directory.path().join("src")).unwrap();
        let prepared = load_frozen_project(&request, &store).unwrap();
        assert!(prepared.bytecode.instruction_count() > 0);
    }

    #[test]
    fn cache_reports_body_and_interface_invalidation_per_unit() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("src")).unwrap();
        std::fs::write(
            directory.path().join("thp.toml"),
            "[autoload]\n\"App\\\\\" = \"src/\"\n",
        )
        .unwrap();
        let library = directory.path().join("src/Math.thp");
        std::fs::write(
            &library,
            "<?thp\nnamespace App;\nfunction calculate(int $value): int { return $value * 2; }\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("main.thp"),
            "<?thp\nuse function App\\calculate;\necho calculate(2) . \"\";\n",
        )
        .unwrap();
        let request = ProjectRequest::new(directory.path(), directory.path().join("main.thp"));
        let store = Store::new(directory.path().join("cache"));
        let (first, _) = cache_warm_project(&request, &store).unwrap();
        assert!(first.units.iter().all(|unit| unit.cache.rebuilt));

        std::fs::write(
            &library,
            "<?thp\nnamespace App;\nfunction calculate(int $value): int { return $value + 2; }\n",
        )
        .unwrap();
        let (body_edit, _) = cache_warm_project(&request, &store).unwrap();
        assert_eq!(
            body_edit
                .units
                .iter()
                .filter(|unit| unit.cache.rebuilt)
                .count(),
            1
        );
        assert!(
            body_edit
                .units
                .iter()
                .all(|unit| unit.cache.interface_reused)
        );

        std::fs::write(
            &library,
            "<?thp\nnamespace App;\nfunction calculate(int $number): int { return $number + 2; }\n",
        )
        .unwrap();
        let (interface_edit, _) = cache_warm_project(&request, &store).unwrap();
        assert_eq!(
            interface_edit
                .units
                .iter()
                .filter(|unit| unit.cache.rebuilt)
                .count(),
            2
        );
    }
}
