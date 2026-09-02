//! Deterministic, statically linked THP module discovery and indexing.

#![allow(clippy::result_large_err, clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use thp_diagnostics::{Diagnostic, SourceFile, SourceId, Span};
use thp_syntax::{
    Block, ClassDecl, Expr, ExprKind, ForClause, ForClauseKind, FunctionDecl, InterfaceDecl,
    NameRef, Program, ScopeTarget, Stmt, StmtKind, TraitAdaptation, TraitDecl, TraitUse,
    TypeSyntax, TypeSyntaxKind, UseKind,
};

pub const INTERFACE_FORMAT_VERSION: u16 = 1;
pub const OBJECT_FORMAT_VERSION: u16 = 1;
pub const MANIFEST_FORMAT_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId(String);

impl ModuleId {
    /// Creates a validated, non-synthetic logical module ID.
    ///
    /// # Errors
    ///
    /// Returns [`ModuleError::InvalidModuleId`] for malformed segments.
    pub fn new(value: impl Into<String>) -> Result<Self, ModuleError> {
        let value = value.into();
        if value.is_empty() || value.starts_with('\\') || value.ends_with('\\') {
            return Err(ModuleError::InvalidModuleId(value));
        }
        if value.split('\\').any(|segment| !valid_segment(segment)) {
            return Err(ModuleError::InvalidModuleId(value));
        }
        Ok(Self(value))
    }

    pub fn synthetic_entry(project_relative: &Path) -> Self {
        let normalized = project_relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(value) => value.to_str(),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/");
        Self(format!("@entry/{normalized}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn namespace(&self) -> &str {
        self.0
            .rsplit_once('\\')
            .map_or("", |(namespace, _)| namespace)
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoloadMapping {
    pub prefix: String,
    pub directories: Vec<PathBuf>,
}

impl AutoloadMapping {
    /// Creates one validated prefix-to-directories mapping.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed prefixes or empty directory lists.
    pub fn new(prefix: impl Into<String>, directories: Vec<PathBuf>) -> Result<Self, ModuleError> {
        let prefix = prefix.into();
        validate_prefix(&prefix)?;
        if directories.is_empty() || directories.iter().any(|path| path.as_os_str().is_empty()) {
            return Err(ModuleError::InvalidAutoload {
                prefix,
                message: "at least one non-empty directory is required".to_owned(),
            });
        }
        Ok(Self {
            prefix,
            directories,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModulePath {
    pub id: ModuleId,
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub expected_namespace: String,
    pub is_entry: bool,
}

#[derive(Debug)]
pub enum ModuleError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    InvalidUtf8 {
        path: PathBuf,
    },
    InvalidModuleId(String),
    InvalidAutoload {
        prefix: String,
        message: String,
    },
    AmbiguousModule {
        id: ModuleId,
        first: PathBuf,
        second: PathBuf,
    },
    AmbiguousFile {
        path: PathBuf,
        first: ModuleId,
        second: ModuleId,
    },
}

impl fmt::Display for ModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::InvalidUtf8 { path } => {
                write!(formatter, "{}: THP source must be UTF-8", path.display())
            }
            Self::InvalidModuleId(id) => write!(formatter, "invalid module ID `{id}`"),
            Self::InvalidAutoload { prefix, message } => {
                write!(formatter, "invalid autoload prefix `{prefix}`: {message}")
            }
            Self::AmbiguousModule { id, first, second } => write!(
                formatter,
                "logical module `{id}` is provided by both {} and {}",
                first.display(),
                second.display()
            ),
            Self::AmbiguousFile {
                path,
                first,
                second,
            } => write!(
                formatter,
                "{} maps to both `{first}` and `{second}`",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ModuleError {}

pub trait ModuleSourceProvider {
    /// Enumerates all modules in deterministic logical order.
    ///
    /// # Errors
    ///
    /// Returns provider-specific discovery failures.
    fn enumerate(&self) -> Result<Vec<ModulePath>, ModuleError>;

    /// Loads and copies one enumerated UTF-8 source.
    ///
    /// # Errors
    ///
    /// Returns provider-specific loading or decoding failures.
    fn load(&self, module: &ModulePath) -> Result<SourceFile, ModuleError>;
}

#[derive(Clone, Debug)]
pub struct FilesystemSourceProvider {
    root: PathBuf,
    mappings: Vec<AutoloadMapping>,
    entry: PathBuf,
}

impl FilesystemSourceProvider {
    pub fn new(
        root: impl Into<PathBuf>,
        mappings: Vec<AutoloadMapping>,
        entry: impl Into<PathBuf>,
    ) -> Self {
        Self {
            root: root.into(),
            mappings,
            entry: entry.into(),
        }
    }

    fn discover(&self) -> Result<Vec<ModulePath>, ModuleError> {
        let mut by_id = BTreeMap::<ModuleId, ModulePath>::new();
        let mut by_file = BTreeMap::<PathBuf, ModuleId>::new();
        for mapping in &self.mappings {
            for directory in &mapping.directories {
                let directory = if directory.is_absolute() {
                    directory.clone()
                } else {
                    self.root.join(directory)
                };
                let canonical_directory =
                    fs::canonicalize(&directory).map_err(|source| ModuleError::Io {
                        path: directory.clone(),
                        source,
                    })?;
                let mut files = Vec::new();
                collect_thp_files(&canonical_directory, &mut files)?;
                files.sort();
                for path in files {
                    let canonical_path =
                        fs::canonicalize(&path).map_err(|source| ModuleError::Io {
                            path: path.clone(),
                            source,
                        })?;
                    let relative = path.strip_prefix(&canonical_directory).map_err(|_| {
                        ModuleError::InvalidAutoload {
                            prefix: mapping.prefix.clone(),
                            message: format!("{} is outside the mapped directory", path.display()),
                        }
                    })?;
                    let id = module_id(&mapping.prefix, relative)?;
                    let expected_namespace = id.namespace().to_owned();
                    if let Some(first) = by_file.get(&canonical_path) {
                        if first != &id {
                            return Err(ModuleError::AmbiguousFile {
                                path: canonical_path,
                                first: first.clone(),
                                second: id,
                            });
                        }
                        continue;
                    }
                    let module = ModulePath {
                        id: id.clone(),
                        path: path.clone(),
                        canonical_path: canonical_path.clone(),
                        expected_namespace,
                        is_entry: same_file(&canonical_path, &self.entry),
                    };
                    if let Some(first) = by_id.get(&id) {
                        if first.canonical_path != canonical_path {
                            return Err(ModuleError::AmbiguousModule {
                                id,
                                first: first.path.clone(),
                                second: path,
                            });
                        }
                    } else {
                        by_file.insert(canonical_path, id.clone());
                        by_id.insert(id, module);
                    }
                }
            }
        }

        let entry_path = if self.entry.is_absolute() {
            self.entry.clone()
        } else {
            self.root.join(&self.entry)
        };
        let canonical_entry = fs::canonicalize(&entry_path).map_err(|source| ModuleError::Io {
            path: entry_path.clone(),
            source,
        })?;
        if by_file.contains_key(&canonical_entry) {
            for module in by_id.values_mut() {
                module.is_entry = module.canonical_path == canonical_entry;
            }
        } else {
            let relative = entry_path.strip_prefix(&self.root).unwrap_or(&entry_path);
            let id = ModuleId::synthetic_entry(relative);
            by_id.insert(
                id.clone(),
                ModulePath {
                    id,
                    path: entry_path,
                    canonical_path: canonical_entry,
                    expected_namespace: String::new(),
                    is_entry: true,
                },
            );
        }
        Ok(by_id.into_values().collect())
    }
}

impl ModuleSourceProvider for FilesystemSourceProvider {
    fn enumerate(&self) -> Result<Vec<ModulePath>, ModuleError> {
        self.discover()
    }

    fn load(&self, module: &ModulePath) -> Result<SourceFile, ModuleError> {
        let bytes = fs::read(&module.path).map_err(|source| ModuleError::Io {
            path: module.path.clone(),
            source,
        })?;
        let text = String::from_utf8(bytes).map_err(|_| ModuleError::InvalidUtf8 {
            path: module.path.clone(),
        })?;
        Ok(SourceFile::new(&module.path, text))
    }
}

#[derive(Clone, Debug, Default)]
pub struct InMemorySourceProvider {
    modules: BTreeMap<ModuleId, (PathBuf, String, String, bool)>,
}

impl InMemorySourceProvider {
    pub fn insert(
        &mut self,
        id: ModuleId,
        path: impl Into<PathBuf>,
        expected_namespace: impl Into<String>,
        source: impl Into<String>,
        is_entry: bool,
    ) {
        self.modules.insert(
            id,
            (
                path.into(),
                expected_namespace.into(),
                source.into(),
                is_entry,
            ),
        );
    }
}

impl ModuleSourceProvider for InMemorySourceProvider {
    fn enumerate(&self) -> Result<Vec<ModulePath>, ModuleError> {
        Ok(self
            .modules
            .iter()
            .map(|(id, (path, _, _, is_entry))| ModulePath {
                id: id.clone(),
                path: path.clone(),
                canonical_path: path.clone(),
                expected_namespace: self.modules[id].1.clone(),
                is_entry: *is_entry,
            })
            .collect())
    }

    fn load(&self, module: &ModulePath) -> Result<SourceFile, ModuleError> {
        let (_, _, source, _) = self
            .modules
            .get(&module.id)
            .expect("enumerated in-memory module exists");
        Ok(SourceFile::new(&module.path, source.clone()))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeclarationKind {
    Function,
    Class,
    Interface,
    Trait,
}

impl DeclarationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Trait => "trait",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Export {
    pub name: String,
    pub kind: DeclarationKind,
    pub module: ModuleId,
    pub source: SourceId,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleInterface {
    pub module: ModuleId,
    pub namespace: String,
    pub exports: Vec<Export>,
    pub interface_hash: String,
}

#[derive(Clone, Debug, Default)]
pub struct ExportIndex {
    exports: BTreeMap<(DeclarationKind, String), Export>,
}

impl ExportIndex {
    pub fn get(&self, kind: DeclarationKind, name: &str) -> Option<&Export> {
        self.exports.get(&(kind, name.to_owned()))
    }

    pub fn find_type(&self, name: &str) -> Option<&Export> {
        [
            DeclarationKind::Class,
            DeclarationKind::Interface,
            DeclarationKind::Trait,
        ]
        .into_iter()
        .find_map(|kind| self.get(kind, name))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Export> {
        self.exports.values()
    }

    /// Adds one canonical export.
    ///
    /// # Errors
    ///
    /// Returns both the previous and duplicate export when the key exists.
    pub fn insert(&mut self, export: Export) -> Result<(), (Export, Export)> {
        let key = (export.kind, export.name.clone());
        if let Some(previous) = self.exports.get(&key) {
            return Err((previous.clone(), export));
        }
        self.exports.insert(key, export);
        Ok(())
    }
}

/// Extracts the body-independent exports for one parsed source unit.
///
/// # Errors
///
/// Returns a namespace-mismatch diagnostic for mapped library modules.
pub fn extract_interface(
    module: &ModulePath,
    source: SourceId,
    program: &Program,
) -> Result<ModuleInterface, Diagnostic> {
    let namespace = program
        .namespace
        .as_ref()
        .map_or_else(String::new, |namespace| namespace.name.segments.join("\\"));
    if !module.id.as_str().starts_with("@entry/") && namespace != module.expected_namespace {
        let span = program
            .namespace
            .as_ref()
            .map_or(program.span, |namespace| namespace.span);
        return Err(Diagnostic::error(
            "modules",
            "M0001",
            span,
            format!(
                "module `{}` must declare namespace `{}`",
                module.id,
                if module.expected_namespace.is_empty() {
                    "<global>"
                } else {
                    &module.expected_namespace
                }
            ),
        ));
    }
    let mut exports = Vec::new();
    let mut canonical = Vec::new();
    for statement in &program.statements {
        let Some((short, span, kind, signature)) = declaration_signature(statement) else {
            continue;
        };
        let name = qualify_declaration(&namespace, short);
        canonical.push(format!("{}:{name}:{signature}", kind.as_str()));
        exports.push(Export {
            name,
            kind,
            module: module.id.clone(),
            source,
            span,
        });
    }
    canonical.sort();
    let mut hash = blake3::Hasher::new();
    hash.update(b"THP module interface");
    hash.update(&INTERFACE_FORMAT_VERSION.to_le_bytes());
    hash.update(module.id.as_str().as_bytes());
    hash.update(&[0]);
    hash.update(namespace.as_bytes());
    hash.update(&[0]);
    for item in canonical {
        hash.update(item.as_bytes());
        hash.update(&[0]);
    }
    exports.sort_by(|left, right| {
        (left.name.as_str(), left.kind).cmp(&(right.name.as_str(), right.kind))
    });
    Ok(ModuleInterface {
        module: module.id.clone(),
        namespace,
        exports,
        interface_hash: hash.finalize().to_hex().to_string(),
    })
}

/// Builds one authoritative project-wide export index.
///
/// # Errors
///
/// Returns every pair of colliding canonical exports.
pub fn build_export_index(
    interfaces: &[ModuleInterface],
) -> Result<ExportIndex, Vec<(Export, Export)>> {
    let mut index = ExportIndex::default();
    let mut duplicates = Vec::new();
    for export in interfaces.iter().flat_map(|interface| &interface.exports) {
        if let Err(duplicate) = index.insert(export.clone()) {
            duplicates.push(duplicate);
        }
    }
    if duplicates.is_empty() {
        Ok(index)
    } else {
        Err(duplicates)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DependencyKind {
    Import,
    Signature,
    Body,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    pub from: ModuleId,
    pub to: ModuleId,
    pub kind: DependencyKind,
    pub symbol: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleGraph {
    pub modules: Vec<ModuleId>,
    pub edges: Vec<DependencyEdge>,
    pub declaration_groups: Vec<Vec<ModuleId>>,
}

impl ModuleGraph {
    /// Builds deterministic import, signature, and body dependency edges.
    ///
    /// # Errors
    ///
    /// Returns imports whose targets are absent from the export index.
    pub fn build(
        units: &[(ModulePath, Program)],
        index: &ExportIndex,
    ) -> Result<Self, Vec<(ModuleId, String, Span)>> {
        let mut modules = units
            .iter()
            .map(|(module, _)| module.id.clone())
            .collect::<Vec<_>>();
        modules.sort();
        let mut edges = BTreeSet::new();
        let mut unknown = Vec::new();
        for (module, program) in units {
            for import in &program.imports {
                let target = import.target.segments.join("\\");
                let export = match import.kind {
                    UseKind::Function => index.get(DeclarationKind::Function, &target),
                    UseKind::Type => index.find_type(&target),
                };
                if let Some(export) = export {
                    if export.module != module.id {
                        edges.insert((
                            module.id.clone(),
                            export.module.clone(),
                            DependencyKind::Import,
                            target,
                        ));
                    }
                } else {
                    unknown.push((module.id.clone(), target, import.target.span));
                }
            }
            let mut resolved = program.clone();
            let _ = resolve_program(&mut resolved, index);
            let mut signature_dependencies = Vec::new();
            collect_signature_dependencies(&resolved, &mut signature_dependencies);
            for (is_function, symbol) in signature_dependencies {
                if let Some(export) = dependency_export(index, is_function, &symbol)
                    && export.module != module.id
                {
                    edges.insert((
                        module.id.clone(),
                        export.module.clone(),
                        DependencyKind::Signature,
                        symbol,
                    ));
                }
            }
            let mut body_dependencies = Vec::new();
            for statement in &resolved.statements {
                collect_body_statement(statement, &mut body_dependencies);
            }
            for (is_function, symbol) in body_dependencies {
                if let Some(export) = dependency_export(index, is_function, &symbol)
                    && export.module != module.id
                {
                    edges.insert((
                        module.id.clone(),
                        export.module.clone(),
                        DependencyKind::Body,
                        symbol,
                    ));
                }
            }
        }
        if !unknown.is_empty() {
            return Err(unknown);
        }
        let edges = edges
            .into_iter()
            .map(|(from, to, kind, symbol)| DependencyEdge {
                from,
                to,
                kind,
                symbol,
            })
            .collect::<Vec<_>>();
        let declaration_groups = strongly_connected_components(&modules, &edges);
        Ok(Self {
            modules,
            edges,
            declaration_groups,
        })
    }
}

fn dependency_export<'a>(
    index: &'a ExportIndex,
    is_function: bool,
    symbol: &str,
) -> Option<&'a Export> {
    if is_function {
        index.get(DeclarationKind::Function, symbol)
    } else {
        index.find_type(symbol)
    }
}

fn collect_signature_dependencies(program: &Program, output: &mut Vec<(bool, String)>) {
    for statement in &program.statements {
        match &statement.kind {
            StmtKind::Function(function) => collect_function_signature(function, output),
            StmtKind::Class(class) => {
                if let Some(parent) = &class.parent {
                    output.push((false, parent.name.clone()));
                }
                output.extend(
                    class
                        .interfaces
                        .iter()
                        .map(|name| (false, name.name.clone())),
                );
                for trait_use in &class.trait_uses {
                    output.extend(
                        trait_use
                            .traits
                            .iter()
                            .map(|name| (false, name.name.clone())),
                    );
                }
                for property in &class.properties {
                    collect_type_dependencies(&property.ty, output);
                }
                for method in &class.methods {
                    collect_function_signature(&method.function, output);
                }
            }
            StmtKind::Interface(interface) => {
                if let Some(parent) = &interface.parent {
                    output.push((false, parent.name.clone()));
                }
                for method in &interface.methods {
                    collect_function_signature(&method.function, output);
                }
            }
            StmtKind::Trait(trait_decl) => {
                for trait_use in &trait_decl.trait_uses {
                    output.extend(
                        trait_use
                            .traits
                            .iter()
                            .map(|name| (false, name.name.clone())),
                    );
                }
                for property in &trait_decl.properties {
                    collect_type_dependencies(&property.ty, output);
                }
                for method in &trait_decl.methods {
                    collect_function_signature(&method.function, output);
                }
            }
            _ => {}
        }
    }
}

fn collect_function_signature(function: &FunctionDecl, output: &mut Vec<(bool, String)>) {
    for parameter in &function.parameters {
        collect_type_dependencies(&parameter.ty, output);
    }
    collect_type_dependencies(&function.return_type, output);
}

fn collect_type_dependencies(ty: &TypeSyntax, output: &mut Vec<(bool, String)>) {
    match &ty.kind {
        TypeSyntaxKind::Named { name, arguments } => {
            if !is_builtin_type(name) {
                output.push((false, name.clone()));
            }
            for argument in arguments {
                collect_type_dependencies(argument, output);
            }
        }
        TypeSyntaxKind::Nullable(inner) => collect_type_dependencies(inner, output),
        TypeSyntaxKind::Union(members) => {
            for member in members {
                collect_type_dependencies(member, output);
            }
        }
    }
}

fn collect_body_statement(statement: &Stmt, output: &mut Vec<(bool, String)>) {
    match &statement.kind {
        StmtKind::Function(function) => collect_body_block(&function.body, output),
        StmtKind::Class(class) => {
            for property in &class.properties {
                if let Some(value) = &property.initializer {
                    collect_body_expr(value, output);
                }
            }
            for method in &class.methods {
                collect_body_block(&method.function.body, output);
            }
        }
        StmtKind::Interface(_) | StmtKind::Break | StmtKind::Continue => {}
        StmtKind::Trait(trait_decl) => {
            for property in &trait_decl.properties {
                if let Some(value) = &property.initializer {
                    collect_body_expr(value, output);
                }
            }
            for method in &trait_decl.methods {
                collect_body_block(&method.function.body, output);
            }
        }
        StmtKind::Assign {
            annotation, value, ..
        } => {
            if let Some(annotation) = annotation {
                collect_type_dependencies(annotation, output);
            }
            collect_body_expr(value, output);
        }
        StmtKind::Echo(value) | StmtKind::Throw(value) | StmtKind::Expression(value) => {
            collect_body_expr(value, output);
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                collect_body_expr(value, output);
            }
        }
        StmtKind::If {
            branches,
            otherwise,
        } => {
            for (condition, block) in branches {
                collect_body_expr(condition, output);
                collect_body_block(block, output);
            }
            if let Some(block) = otherwise {
                collect_body_block(block, output);
            }
        }
        StmtKind::While { condition, body } => {
            collect_body_expr(condition, output);
            collect_body_block(body, output);
        }
        StmtKind::For {
            initializers,
            conditions,
            updates,
            body,
        } => {
            for clause in initializers.iter().chain(conditions).chain(updates) {
                match &clause.kind {
                    ForClauseKind::Assign {
                        annotation, value, ..
                    } => {
                        if let Some(annotation) = annotation {
                            collect_type_dependencies(annotation, output);
                        }
                        collect_body_expr(value, output);
                    }
                    ForClauseKind::SetProperty { object, value, .. } => {
                        collect_body_expr(object, output);
                        collect_body_expr(value, output);
                    }
                    ForClauseKind::SetIndex { indices, value, .. } => {
                        for index in indices {
                            collect_body_expr(index, output);
                        }
                        collect_body_expr(value, output);
                    }
                    ForClauseKind::Expression(value) => collect_body_expr(value, output),
                }
            }
            collect_body_block(body, output);
        }
        StmtKind::Foreach { source, body, .. } => {
            collect_body_expr(source, output);
            collect_body_block(body, output);
        }
        StmtKind::SetProperty { object, value, .. } => {
            collect_body_expr(object, output);
            collect_body_expr(value, output);
        }
        StmtKind::SetIndex { indices, value, .. } => {
            for index in indices {
                collect_body_expr(index, output);
            }
            collect_body_expr(value, output);
        }
        StmtKind::Try {
            body,
            catches,
            finally,
        } => {
            collect_body_block(body, output);
            for catch in catches {
                output.push((false, catch.class_name.clone()));
                collect_body_block(&catch.body, output);
            }
            if let Some(block) = finally {
                collect_body_block(block, output);
            }
        }
        StmtKind::Using { value, body, .. } => {
            collect_body_expr(value, output);
            collect_body_block(body, output);
        }
        StmtKind::Block(body) => collect_body_block(body, output),
    }
}

fn collect_body_block(block: &Block, output: &mut Vec<(bool, String)>) {
    for statement in block {
        collect_body_statement(statement, output);
    }
}

fn collect_body_expr(expression: &Expr, output: &mut Vec<(bool, String)>) {
    match &expression.kind {
        ExprKind::Call { callee, arguments } => {
            if let ExprKind::Name(name) = &callee.kind {
                output.push((true, name.clone()));
            } else {
                collect_body_expr(callee, output);
            }
            for argument in arguments {
                collect_body_expr(&argument.value, output);
            }
        }
        ExprKind::New {
            class_name,
            arguments,
            ..
        } => {
            output.push((false, class_name.clone()));
            for argument in arguments {
                collect_body_expr(&argument.value, output);
            }
        }
        ExprKind::StaticCall {
            target, arguments, ..
        } => {
            if let ScopeTarget::Named(name) = target {
                output.push((false, name.clone()));
            }
            for argument in arguments {
                collect_body_expr(&argument.value, output);
            }
        }
        ExprKind::ClassConstant { class_name, .. } => {
            output.push((false, class_name.clone()));
        }
        ExprKind::InstanceOf {
            value, class_name, ..
        } => {
            output.push((false, class_name.clone()));
            collect_body_expr(value, output);
        }
        ExprKind::Unary { operand, .. } => collect_body_expr(operand, output),
        ExprKind::Binary { left, right, .. } => {
            collect_body_expr(left, output);
            collect_body_expr(right, output);
        }
        ExprKind::Vector(values) => {
            for value in values {
                collect_body_expr(value, output);
            }
        }
        ExprKind::Map(entries) => {
            for entry in entries {
                collect_body_expr(&entry.key, output);
                collect_body_expr(&entry.value, output);
            }
        }
        ExprKind::Index { collection, index } => {
            collect_body_expr(collection, output);
            collect_body_expr(index, output);
        }
        ExprKind::Property { object, .. } => collect_body_expr(object, output),
        ExprKind::MethodCall {
            object, arguments, ..
        } => {
            collect_body_expr(object, output);
            for argument in arguments {
                collect_body_expr(&argument.value, output);
            }
        }
        ExprKind::Match { subject, arms } => {
            collect_body_expr(subject, output);
            for arm in arms {
                for condition in &arm.conditions {
                    collect_body_expr(condition, output);
                }
                collect_body_expr(&arm.value, output);
            }
        }
        ExprKind::Integer(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::String(_)
        | ExprKind::Variable(_)
        | ExprKind::Name(_) => {}
    }
}

impl fmt::Display for ModuleGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for group in &self.declaration_groups {
            if group.len() > 1 {
                writeln!(formatter, "cycle [{}]", join_ids(group))?;
            } else {
                writeln!(formatter, "module {}", group[0])?;
            }
        }
        for edge in &self.edges {
            writeln!(
                formatter,
                "{} -> {} [{:?}: {}]",
                edge.from, edge.to, edge.kind, edge.symbol
            )?;
        }
        Ok(())
    }
}

/// Resolves every source-level reference to a canonical project name.
pub fn resolve_program(program: &mut Program, index: &ExportIndex) -> Vec<Diagnostic> {
    let namespace = program
        .namespace
        .as_ref()
        .map_or_else(String::new, |value| value.name.segments.join("\\"));
    let mut type_aliases = BTreeMap::new();
    let mut function_aliases = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for import in &program.imports {
        let target = import.target.segments.join("\\");
        let aliases = match import.kind {
            UseKind::Type => &mut type_aliases,
            UseKind::Function => &mut function_aliases,
        };
        if let Some(previous) = aliases.insert(import.alias.clone(), target.clone()) {
            diagnostics.push(
                Diagnostic::error(
                    "resolution",
                    "M0002",
                    import.alias_span,
                    format!("duplicate import alias `{}`", import.alias),
                )
                .with_note(format!("the alias was already bound to `{previous}`")),
            );
        }
        let found = match import.kind {
            UseKind::Type => index.find_type(&target).is_some(),
            UseKind::Function => index.get(DeclarationKind::Function, &target).is_some(),
        };
        if !found {
            diagnostics.push(Diagnostic::error(
                "resolution",
                "M0003",
                import.target.span,
                format!(
                    "unknown imported {} `{target}`",
                    match import.kind {
                        UseKind::Type => "type",
                        UseKind::Function => "function",
                    }
                ),
            ));
        }
    }
    for statement in &mut program.statements {
        resolve_statement(
            statement,
            &namespace,
            &type_aliases,
            &function_aliases,
            index,
        );
    }
    diagnostics
}

fn resolve_statement(
    statement: &mut Stmt,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    match &mut statement.kind {
        StmtKind::Function(declaration) => {
            declaration.name = qualify_declaration(namespace, &declaration.name);
            resolve_function(
                declaration,
                namespace,
                type_aliases,
                function_aliases,
                index,
            );
        }
        StmtKind::Class(declaration) => {
            declaration.name = qualify_declaration(namespace, &declaration.name);
            resolve_class(
                declaration,
                namespace,
                type_aliases,
                function_aliases,
                index,
            );
        }
        StmtKind::Interface(declaration) => {
            declaration.name = qualify_declaration(namespace, &declaration.name);
            resolve_interface(
                declaration,
                namespace,
                type_aliases,
                function_aliases,
                index,
            );
        }
        StmtKind::Trait(declaration) => {
            declaration.name = qualify_declaration(namespace, &declaration.name);
            resolve_trait(
                declaration,
                namespace,
                type_aliases,
                function_aliases,
                index,
            );
        }
        StmtKind::Assign {
            annotation, value, ..
        } => {
            if let Some(annotation) = annotation {
                resolve_type(annotation, namespace, type_aliases);
            }
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::Echo(value) | StmtKind::Throw(value) | StmtKind::Expression(value) => {
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                resolve_expr(value, namespace, type_aliases, function_aliases, index);
            }
        }
        StmtKind::If {
            branches,
            otherwise,
        } => {
            for (condition, body) in branches {
                resolve_expr(condition, namespace, type_aliases, function_aliases, index);
                resolve_block(body, namespace, type_aliases, function_aliases, index);
            }
            if let Some(body) = otherwise {
                resolve_block(body, namespace, type_aliases, function_aliases, index);
            }
        }
        StmtKind::While { condition, body } => {
            resolve_expr(condition, namespace, type_aliases, function_aliases, index);
            resolve_block(body, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::For {
            initializers,
            conditions,
            updates,
            body,
        } => {
            for clause in initializers.iter_mut().chain(conditions).chain(updates) {
                resolve_for_clause(clause, namespace, type_aliases, function_aliases, index);
            }
            resolve_block(body, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::Foreach { source, body, .. } => {
            resolve_expr(source, namespace, type_aliases, function_aliases, index);
            resolve_block(body, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::SetProperty { object, value, .. } => {
            resolve_expr(object, namespace, type_aliases, function_aliases, index);
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::SetIndex { indices, value, .. } => {
            for index_expr in indices {
                resolve_expr(index_expr, namespace, type_aliases, function_aliases, index);
            }
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::Try {
            body,
            catches,
            finally,
        } => {
            resolve_block(body, namespace, type_aliases, function_aliases, index);
            for catch in catches {
                catch.class_name = resolve_type_name(&catch.class_name, namespace, type_aliases);
                resolve_block(
                    &mut catch.body,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
            if let Some(body) = finally {
                resolve_block(body, namespace, type_aliases, function_aliases, index);
            }
        }
        StmtKind::Using { value, body, .. } => {
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
            resolve_block(body, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::Block(body) => {
            resolve_block(body, namespace, type_aliases, function_aliases, index);
        }
        StmtKind::Break | StmtKind::Continue => {}
    }
}

fn resolve_block(
    block: &mut Block,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    for statement in block {
        resolve_statement(statement, namespace, type_aliases, function_aliases, index);
    }
}

fn resolve_function(
    function: &mut FunctionDecl,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    for parameter in &mut function.parameters {
        resolve_type(&mut parameter.ty, namespace, type_aliases);
        if let Some(default) = &mut parameter.default {
            resolve_expr(default, namespace, type_aliases, function_aliases, index);
        }
    }
    resolve_type(&mut function.return_type, namespace, type_aliases);
    resolve_block(
        &mut function.body,
        namespace,
        type_aliases,
        function_aliases,
        index,
    );
}

fn resolve_class(
    class: &mut ClassDecl,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    if let Some(parent) = &mut class.parent {
        resolve_name_ref(parent, namespace, type_aliases);
    }
    for interface in &mut class.interfaces {
        resolve_name_ref(interface, namespace, type_aliases);
    }
    resolve_trait_uses(&mut class.trait_uses, namespace, type_aliases);
    for property in &mut class.properties {
        resolve_type(&mut property.ty, namespace, type_aliases);
        if let Some(initializer) = &mut property.initializer {
            resolve_expr(
                initializer,
                namespace,
                type_aliases,
                function_aliases,
                index,
            );
        }
    }
    for method in &mut class.methods {
        resolve_function(
            &mut method.function,
            namespace,
            type_aliases,
            function_aliases,
            index,
        );
    }
}

fn resolve_interface(
    interface: &mut InterfaceDecl,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    if let Some(parent) = &mut interface.parent {
        resolve_name_ref(parent, namespace, type_aliases);
    }
    for method in &mut interface.methods {
        resolve_function(
            &mut method.function,
            namespace,
            type_aliases,
            function_aliases,
            index,
        );
    }
}

fn resolve_trait(
    trait_decl: &mut TraitDecl,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    resolve_trait_uses(&mut trait_decl.trait_uses, namespace, type_aliases);
    for property in &mut trait_decl.properties {
        resolve_type(&mut property.ty, namespace, type_aliases);
    }
    for method in &mut trait_decl.methods {
        resolve_function(
            &mut method.function,
            namespace,
            type_aliases,
            function_aliases,
            index,
        );
    }
}

fn resolve_trait_uses(
    uses: &mut [TraitUse],
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
) {
    for trait_use in uses {
        for name in &mut trait_use.traits {
            resolve_name_ref(name, namespace, type_aliases);
        }
        for adaptation in &mut trait_use.adaptations {
            match adaptation {
                TraitAdaptation::InsteadOf {
                    trait_name,
                    excluded,
                    ..
                } => {
                    resolve_name_ref(trait_name, namespace, type_aliases);
                    for name in excluded {
                        resolve_name_ref(name, namespace, type_aliases);
                    }
                }
                TraitAdaptation::Alias { trait_name, .. } => {
                    resolve_name_ref(trait_name, namespace, type_aliases);
                }
            }
        }
    }
}

fn resolve_for_clause(
    clause: &mut ForClause,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    match &mut clause.kind {
        ForClauseKind::Assign {
            annotation, value, ..
        } => {
            if let Some(annotation) = annotation {
                resolve_type(annotation, namespace, type_aliases);
            }
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        ForClauseKind::SetProperty { object, value, .. } => {
            resolve_expr(object, namespace, type_aliases, function_aliases, index);
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        ForClauseKind::SetIndex { indices, value, .. } => {
            for index_expr in indices {
                resolve_expr(index_expr, namespace, type_aliases, function_aliases, index);
            }
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
        ForClauseKind::Expression(value) => {
            resolve_expr(value, namespace, type_aliases, function_aliases, index);
        }
    }
}

fn resolve_expr(
    expression: &mut Expr,
    namespace: &str,
    type_aliases: &BTreeMap<String, String>,
    function_aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) {
    match &mut expression.kind {
        ExprKind::Call { callee, arguments } => {
            if let ExprKind::Name(name) = &mut callee.kind {
                *name = resolve_function_name(name, namespace, function_aliases, index);
            } else {
                resolve_expr(callee, namespace, type_aliases, function_aliases, index);
            }
            for argument in arguments {
                resolve_expr(
                    &mut argument.value,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
        }
        ExprKind::New {
            class_name,
            arguments,
            ..
        } => {
            *class_name = resolve_type_name(class_name, namespace, type_aliases);
            for argument in arguments {
                resolve_expr(
                    &mut argument.value,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
        }
        ExprKind::StaticCall {
            target, arguments, ..
        } => {
            if let ScopeTarget::Named(name) = target {
                *name = resolve_type_name(name, namespace, type_aliases);
            }
            for argument in arguments {
                resolve_expr(
                    &mut argument.value,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
        }
        ExprKind::ClassConstant { class_name, .. } | ExprKind::InstanceOf { class_name, .. } => {
            *class_name = resolve_type_name(class_name, namespace, type_aliases);
            if let ExprKind::InstanceOf { value, .. } = &mut expression.kind {
                resolve_expr(value, namespace, type_aliases, function_aliases, index);
            }
        }
        ExprKind::Unary { operand, .. } => {
            resolve_expr(operand, namespace, type_aliases, function_aliases, index);
        }
        ExprKind::Binary { left, right, .. } => {
            resolve_expr(left, namespace, type_aliases, function_aliases, index);
            resolve_expr(right, namespace, type_aliases, function_aliases, index);
        }
        ExprKind::Vector(values) => {
            for value in values {
                resolve_expr(value, namespace, type_aliases, function_aliases, index);
            }
        }
        ExprKind::Map(entries) => {
            for entry in entries {
                resolve_expr(
                    &mut entry.key,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
                resolve_expr(
                    &mut entry.value,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
        }
        ExprKind::Index {
            collection,
            index: key,
        } => {
            resolve_expr(collection, namespace, type_aliases, function_aliases, index);
            resolve_expr(key, namespace, type_aliases, function_aliases, index);
        }
        ExprKind::Property { object, .. } => {
            resolve_expr(object, namespace, type_aliases, function_aliases, index);
        }
        ExprKind::MethodCall {
            object, arguments, ..
        } => {
            resolve_expr(object, namespace, type_aliases, function_aliases, index);
            for argument in arguments {
                resolve_expr(
                    &mut argument.value,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
        }
        ExprKind::Match { subject, arms } => {
            resolve_expr(subject, namespace, type_aliases, function_aliases, index);
            for arm in arms {
                for condition in &mut arm.conditions {
                    resolve_expr(condition, namespace, type_aliases, function_aliases, index);
                }
                resolve_expr(
                    &mut arm.value,
                    namespace,
                    type_aliases,
                    function_aliases,
                    index,
                );
            }
        }
        ExprKind::Integer(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::String(_)
        | ExprKind::Variable(_)
        | ExprKind::Name(_) => {}
    }
}

fn resolve_type(ty: &mut TypeSyntax, namespace: &str, aliases: &BTreeMap<String, String>) {
    match &mut ty.kind {
        TypeSyntaxKind::Named { name, arguments } => {
            *name = resolve_type_name(name, namespace, aliases);
            for argument in arguments {
                resolve_type(argument, namespace, aliases);
            }
        }
        TypeSyntaxKind::Nullable(inner) => resolve_type(inner, namespace, aliases),
        TypeSyntaxKind::Union(members) => {
            for member in members {
                resolve_type(member, namespace, aliases);
            }
        }
    }
}

fn resolve_name_ref(reference: &mut NameRef, namespace: &str, aliases: &BTreeMap<String, String>) {
    reference.name = resolve_type_name(&reference.name, namespace, aliases);
}

fn resolve_type_name(name: &str, namespace: &str, aliases: &BTreeMap<String, String>) -> String {
    if name.starts_with('\\') {
        return name.trim_start_matches('\\').to_owned();
    }
    if is_builtin_type(name) {
        return name.to_owned();
    }
    resolve_qualified(name, namespace, aliases)
}

fn resolve_function_name(
    name: &str,
    namespace: &str,
    aliases: &BTreeMap<String, String>,
    index: &ExportIndex,
) -> String {
    if name.starts_with('\\') {
        return name.trim_start_matches('\\').to_owned();
    }
    let resolved = resolve_qualified(name, namespace, aliases);
    if name.contains('\\')
        || aliases.contains_key(name)
        || index.get(DeclarationKind::Function, &resolved).is_some()
    {
        resolved
    } else {
        name.to_owned()
    }
}

fn resolve_qualified(name: &str, namespace: &str, aliases: &BTreeMap<String, String>) -> String {
    let (first, rest) = name.split_once('\\').unwrap_or((name, ""));
    if let Some(target) = aliases.get(first) {
        return if rest.is_empty() {
            target.clone()
        } else {
            format!("{target}\\{rest}")
        };
    }
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}\\{name}")
    }
}

fn declaration_signature(statement: &Stmt) -> Option<(&str, Span, DeclarationKind, String)> {
    match &statement.kind {
        StmtKind::Function(value) => Some((
            &value.name,
            value.name_span,
            DeclarationKind::Function,
            function_signature(value),
        )),
        StmtKind::Class(value) => Some((
            &value.name,
            value.name_span,
            DeclarationKind::Class,
            class_signature(value),
        )),
        StmtKind::Interface(value) => Some((
            &value.name,
            value.name_span,
            DeclarationKind::Interface,
            interface_signature(value),
        )),
        StmtKind::Trait(value) => Some((
            &value.name,
            value.name_span,
            DeclarationKind::Trait,
            trait_signature(value),
        )),
        _ => None,
    }
}

fn function_signature(function: &FunctionDecl) -> String {
    let parameters = function
        .parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}:{}:{}:{}",
                type_signature(&parameter.ty),
                parameter.name,
                parameter.variadic,
                parameter
                    .default
                    .as_ref()
                    .map_or_else(|| "-".to_owned(), expression_signature)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("({parameters}):{}", type_signature(&function.return_type))
}

fn class_signature(class: &ClassDecl) -> String {
    format!(
        "abstract={};final={};parent={:?};interfaces={:?};traits={:?};properties={};methods={}",
        class.abstract_class,
        class.final_class,
        class.parent.as_ref().map(|value| &value.name),
        class
            .interfaces
            .iter()
            .map(|value| &value.name)
            .collect::<Vec<_>>(),
        class
            .trait_uses
            .iter()
            .flat_map(|value| value.traits.iter().map(|name| &name.name))
            .collect::<Vec<_>>(),
        class
            .properties
            .iter()
            .map(|value| {
                format!(
                    "{}:{}:{:?}:{}",
                    value.name,
                    type_signature(&value.ty),
                    value.visibility,
                    value
                        .initializer
                        .as_ref()
                        .map_or_else(|| "-".to_owned(), expression_signature)
                )
            })
            .collect::<Vec<_>>()
            .join(","),
        class
            .methods
            .iter()
            .map(method_signature)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn interface_signature(interface: &InterfaceDecl) -> String {
    format!(
        "parent={:?};methods={}",
        interface.parent.as_ref().map(|value| &value.name),
        interface
            .methods
            .iter()
            .map(method_signature)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn trait_signature(trait_decl: &TraitDecl) -> String {
    format!(
        "traits={:?};properties={};methods={}",
        trait_decl
            .trait_uses
            .iter()
            .flat_map(|value| value.traits.iter().map(|name| &name.name))
            .collect::<Vec<_>>(),
        trait_decl
            .properties
            .iter()
            .map(|value| {
                format!(
                    "{}:{}:{:?}",
                    value.name,
                    type_signature(&value.ty),
                    value.visibility
                )
            })
            .collect::<Vec<_>>()
            .join(","),
        trait_decl
            .methods
            .iter()
            .map(method_signature)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn method_signature(method: &thp_syntax::MethodDecl) -> String {
    format!(
        "{}:{}:{:?}:static={}:abstract={}:final={}",
        method.function.name,
        function_signature(&method.function),
        method.visibility,
        method.static_method,
        method.abstract_method,
        method.final_method
    )
}

fn expression_signature(expression: &Expr) -> String {
    match &expression.kind {
        ExprKind::Integer(value) => format!("i:{value}"),
        ExprKind::Float(value) => format!("f:{:016x}", value.to_bits()),
        ExprKind::Bool(value) => format!("b:{value}"),
        ExprKind::Null => "null".to_owned(),
        ExprKind::String(value) => format!("s:{}", blake3::hash(value).to_hex()),
        ExprKind::Vector(values) => format!(
            "[{}]",
            values
                .iter()
                .map(expression_signature)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExprKind::Map(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|entry| format!(
                    "{}=>{}",
                    expression_signature(&entry.key),
                    expression_signature(&entry.value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExprKind::Unary { op, operand } => {
            format!("{op:?}:{}", expression_signature(operand))
        }
        _ => "<non-constant>".to_owned(),
    }
}

fn type_signature(ty: &TypeSyntax) -> String {
    match &ty.kind {
        TypeSyntaxKind::Named { name, arguments } if arguments.is_empty() => name.clone(),
        TypeSyntaxKind::Named { name, arguments } => format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(type_signature)
                .collect::<Vec<_>>()
                .join(",")
        ),
        TypeSyntaxKind::Nullable(inner) => format!("?{}", type_signature(inner)),
        TypeSyntaxKind::Union(members) => members
            .iter()
            .map(type_signature)
            .collect::<Vec<_>>()
            .join("|"),
    }
}

fn strongly_connected_components(
    modules: &[ModuleId],
    edges: &[DependencyEdge],
) -> Vec<Vec<ModuleId>> {
    struct Tarjan<'a> {
        next: usize,
        stack: Vec<ModuleId>,
        on_stack: BTreeSet<ModuleId>,
        indices: BTreeMap<ModuleId, usize>,
        low: BTreeMap<ModuleId, usize>,
        adjacency: &'a BTreeMap<ModuleId, Vec<ModuleId>>,
        groups: Vec<Vec<ModuleId>>,
    }
    fn visit(node: &ModuleId, state: &mut Tarjan<'_>) {
        let index = state.next;
        state.next += 1;
        state.indices.insert(node.clone(), index);
        state.low.insert(node.clone(), index);
        state.stack.push(node.clone());
        state.on_stack.insert(node.clone());
        for target in state.adjacency.get(node).into_iter().flatten() {
            if !state.indices.contains_key(target) {
                visit(target, state);
                let target_low = state.low[target];
                state
                    .low
                    .entry(node.clone())
                    .and_modify(|low| *low = (*low).min(target_low));
            } else if state.on_stack.contains(target) {
                let target_index = state.indices[target];
                state
                    .low
                    .entry(node.clone())
                    .and_modify(|low| *low = (*low).min(target_index));
            }
        }
        if state.low[node] == state.indices[node] {
            let mut group = Vec::new();
            loop {
                let member = state.stack.pop().expect("SCC root has a stack member");
                state.on_stack.remove(&member);
                group.push(member.clone());
                if &member == node {
                    break;
                }
            }
            group.sort();
            state.groups.push(group);
        }
    }
    let mut adjacency = modules
        .iter()
        .cloned()
        .map(|module| (module, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }
    for targets in adjacency.values_mut() {
        targets.sort();
        targets.dedup();
    }
    let mut state = Tarjan {
        next: 0,
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        indices: BTreeMap::new(),
        low: BTreeMap::new(),
        adjacency: &adjacency,
        groups: Vec::new(),
    };
    for module in modules {
        if !state.indices.contains_key(module) {
            visit(module, &mut state);
        }
    }
    state.groups.sort_by(|left, right| left[0].cmp(&right[0]));
    state.groups
}

fn collect_thp_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), ModuleError> {
    collect_thp_files_inner(directory, output, &mut BTreeSet::new())
}

fn collect_thp_files_inner(
    directory: &Path,
    output: &mut Vec<PathBuf>,
    visited: &mut BTreeSet<PathBuf>,
) -> Result<(), ModuleError> {
    let canonical = fs::canonicalize(directory).map_err(|source| ModuleError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    if !visited.insert(canonical) {
        return Ok(());
    }
    let entries = fs::read_dir(directory).map_err(|source| ModuleError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ModuleError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let metadata = fs::metadata(&path).map_err(|source| ModuleError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.is_dir() {
            collect_thp_files_inner(&path, output, visited)?;
        } else if metadata.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("thp")
        {
            output.push(path);
        }
    }
    Ok(())
}

fn module_id(prefix: &str, relative: &Path) -> Result<ModuleId, ModuleError> {
    let without_extension = relative.with_extension("");
    let relative = without_extension
        .components()
        .map(|component| match component {
            Component::Normal(segment) => segment
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| ModuleError::InvalidModuleId(relative.display().to_string())),
            _ => Err(ModuleError::InvalidModuleId(relative.display().to_string())),
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\\");
    ModuleId::new(format!("{prefix}{relative}"))
}

fn validate_prefix(prefix: &str) -> Result<(), ModuleError> {
    if prefix.is_empty() {
        return Ok(());
    }
    let Some(prefix_without_separator) = prefix.strip_suffix('\\') else {
        return Err(ModuleError::InvalidAutoload {
            prefix: prefix.to_owned(),
            message: "non-empty prefixes must end with `\\`".to_owned(),
        });
    };
    if prefix_without_separator
        .split('\\')
        .any(|segment| !valid_segment(segment))
    {
        return Err(ModuleError::InvalidAutoload {
            prefix: prefix.to_owned(),
            message: "prefix contains an invalid namespace segment".to_owned(),
        });
    }
    Ok(())
}

fn valid_segment(segment: &str) -> bool {
    let mut bytes = segment.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn same_file(canonical: &Path, entry: &Path) -> bool {
    fs::canonicalize(entry).is_ok_and(|candidate| candidate == canonical)
}

fn qualify_declaration(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}\\{name}")
    }
}

fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "int"
            | "float"
            | "bool"
            | "string"
            | "void"
            | "never"
            | "mixed"
            | "null"
            | "Vec"
            | "Map"
            | "Stream"
            | "Exception"
            | "Resource"
    )
}

fn join_ids(ids: &[ModuleId]) -> String {
    ids.iter()
        .map(ModuleId::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use thp_diagnostics::{SourceFile, SourceId};
    use thp_syntax::{StmtKind, parse};

    use super::{
        AutoloadMapping, DeclarationKind, FilesystemSourceProvider, ModuleGraph, ModuleId,
        ModuleSourceProvider, build_export_index, extract_interface, resolve_program,
    };

    #[test]
    fn discovers_modules_in_deterministic_logical_order() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("src/Service")).unwrap();
        fs::write(
            root.path().join("src/Service/Zed.thp"),
            "<?thp\nnamespace App\\Service;\nclass Zed {}",
        )
        .unwrap();
        fs::write(
            root.path().join("src/Service/Client.thp"),
            "<?thp\nnamespace App\\Service;\nclass Client {}",
        )
        .unwrap();
        let entry = root.path().join("main.thp");
        fs::write(&entry, "<?thp\n").unwrap();
        let provider = FilesystemSourceProvider::new(
            root.path(),
            vec![AutoloadMapping::new("App\\", vec!["src".into()]).unwrap()],
            &entry,
        );
        let ids = provider
            .enumerate()
            .unwrap()
            .into_iter()
            .map(|module| module.id.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            [
                "@entry/main.thp",
                "App\\Service\\Client",
                "App\\Service\\Zed"
            ]
        );
    }

    #[test]
    fn extracts_body_independent_interfaces_and_resolves_aliases() {
        let module = super::ModulePath {
            id: ModuleId::new("Vendor\\Client").unwrap(),
            path: "Client.thp".into(),
            canonical_path: "Client.thp".into(),
            expected_namespace: "Vendor".to_owned(),
            is_entry: false,
        };
        let first = parse(&SourceFile::new(
            "Client.thp",
            "<?thp\nnamespace Vendor;\nfunction make(): int { return 1; }\nclass Client {}",
        ))
        .program;
        let second = parse(&SourceFile::new(
            "Client.thp",
            "<?thp\nnamespace Vendor;\nfunction make(): int { return 2; }\nclass Client {}",
        ))
        .program;
        let one = extract_interface(&module, SourceId(0), &first).unwrap();
        let two = extract_interface(&module, SourceId(0), &second).unwrap();
        assert_eq!(one.interface_hash, two.interface_hash);

        let index = build_export_index(&[one]).unwrap();
        assert!(
            index
                .get(DeclarationKind::Function, "Vendor\\make")
                .is_some()
        );
        let source = SourceFile::new(
            "main.thp",
            "<?thp\nuse Vendor\\Client as C;\nuse function Vendor\\make;\nfunction run(C $c): int { return make(); }",
        );
        let mut program = parse(&source).program;
        assert!(resolve_program(&mut program, &index).is_empty());
        let StmtKind::Function(function) = &program.statements[0].kind else {
            panic!("expected function");
        };
        let thp_syntax::TypeSyntaxKind::Named { name, .. } = &function.parameters[0].ty.kind else {
            panic!("expected named type");
        };
        assert_eq!(name, "Vendor\\Client");
    }

    #[test]
    fn graph_marks_mutual_imports_as_one_declaration_group() {
        let a_source = SourceFile::new("A.thp", "<?thp\nnamespace App;\nuse App\\B;\nclass A {}");
        let b_source = SourceFile::new("B.thp", "<?thp\nnamespace App;\nuse App\\A;\nclass B {}");
        let a = super::ModulePath {
            id: ModuleId::new("App\\A").unwrap(),
            path: "A.thp".into(),
            canonical_path: "A.thp".into(),
            expected_namespace: "App".to_owned(),
            is_entry: true,
        };
        let b = super::ModulePath {
            id: ModuleId::new("App\\B").unwrap(),
            path: "B.thp".into(),
            canonical_path: "B.thp".into(),
            expected_namespace: "App".to_owned(),
            is_entry: false,
        };
        let a_program = parse(&a_source).program;
        let b_program = parse(&b_source).program;
        let interfaces = [
            extract_interface(&a, SourceId(0), &a_program).unwrap(),
            extract_interface(&b, SourceId(1), &b_program).unwrap(),
        ];
        let index = build_export_index(&interfaces).unwrap();
        let graph =
            ModuleGraph::build(&[(b.clone(), b_program), (a.clone(), a_program)], &index).unwrap();
        assert_eq!(graph.declaration_groups, vec![vec![a.id, b.id]]);
    }
}
