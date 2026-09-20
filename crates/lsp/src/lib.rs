//! Detached documentation indexing and Language Server Protocol support for THP.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use lsp_server::{Connection, Message, Request as ServerRequest, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic as LspDiagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, Documentation, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, Location, MarkupContent, MarkupKind, OneOf,
    OptionalVersionedTextDocumentIdentifier, ParameterInformation, ParameterLabel, Position,
    PrepareRenameResponse, PublishDiagnosticsParams, Range, RenameOptions, SemanticToken,
    SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
    ServerCapabilities, SignatureHelp, SignatureHelpOptions, SignatureHelpParams,
    SignatureInformation, SymbolInformation, SymbolKind, TextDocumentEdit,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Uri, WorkspaceEdit,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidChangeWorkspaceFolders,
        DidCloseTextDocument, DidOpenTextDocument, Notification as LspNotification,
        PublishDiagnostics,
    },
    request::{
        Completion, DocumentSymbolRequest, Formatting, GotoDefinition, HoverRequest,
        PrepareRenameRequest, References, Rename, Request as LspRequest, SemanticTokensFullRequest,
        SignatureHelpRequest, WorkspaceSymbolRequest,
    },
};
use thp_compiler::{Compilation, ProjectRequest, compile_project_with_provider, compile_text};
use thp_config::ProjectConfig;
use thp_diagnostics::{Severity, SourceFile, Span};
use thp_hir::Module as HirModule;
use thp_modules::{
    AutoloadMapping, FilesystemSourceProvider, InMemorySourceProvider, ModuleId,
    ModuleSourceProvider, resolve_type_syntax_in_namespace,
};
use thp_syntax::{
    ClassDecl, DocblockSpan, ExprKind, FunctionDecl, InterfaceDecl, MethodDecl, Program,
    PropertyDecl, Stmt, StmtKind, Token, TokenKind, TraitDecl, TypeSyntax, TypeSyntaxKind, UseKind,
    lex, lex_with_docblocks, parse_tokens, parse_type_prefix,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(lsp_types::PositionEncodingKind::UTF16),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions::default()),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_owned(), ",".to_owned()]),
            ..SignatureHelpOptions::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: lsp_types::WorkDoneProgressOptions::default(),
        })),
        document_symbol_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: semantic_legend(),
                range: None,
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..SemanticTokensOptions::default()
            },
        )),
        document_formatting_provider: Some(OneOf::Left(true)),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..ServerCapabilities::default()
    }
}

fn semantic_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::TYPE,
            SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::METHOD,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::COMMENT,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::STATIC,
            SemanticTokenModifier::DEPRECATED,
        ],
    }
}

/// Runs one synchronous LSP connection until the client shuts it down.
///
/// # Errors
///
/// Returns a protocol or transport error when the connection cannot continue.
pub fn run(connection: &Connection) -> Result<(), String> {
    let initialize = serde_json::json!({
        "capabilities": capabilities(),
        "serverInfo": { "name": "thp-lsp", "version": VERSION },
    });
    let (id, params) = connection
        .initialize_start()
        .map_err(|error| error.to_string())?;
    let params: InitializeParams = decode(params)?;
    connection
        .initialize_finish(id, initialize)
        .map_err(|error| error.to_string())?;
    let mut server = Server::from_initialize(&params);
    server.rescan(connection)?;
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection
                    .handle_shutdown(&request)
                    .map_err(|error| error.to_string())?
                {
                    break;
                }
                server.handle_request(connection, request)?;
            }
            Message::Notification(notification) => {
                if notification.method == "exit" {
                    return Err("exit received without shutdown".to_owned());
                }
                if let Err(error) = server.handle_notification(connection, notification) {
                    eprintln!("ignored malformed LSP notification: {error}");
                }
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

#[derive(Default)]
struct Server {
    documents: HashMap<Uri, DocumentState>,
    open: HashMap<PathBuf, (Uri, i32, String)>,
    workspaces: BTreeMap<PathBuf, Uri>,
    published: BTreeMap<String, Uri>,
}

impl Server {
    #[allow(deprecated)]
    fn from_initialize(params: &InitializeParams) -> Self {
        let mut server = Self::default();
        if let Some(folders) = &params.workspace_folders {
            for folder in folders {
                if let Ok(path) = uri_to_path(&folder.uri) {
                    server
                        .workspaces
                        .insert(normalize_path(&path), folder.uri.clone());
                }
            }
        } else if let Some(uri) = &params.root_uri
            && let Ok(path) = uri_to_path(uri)
        {
            server.workspaces.insert(normalize_path(&path), uri.clone());
        }
        server
    }

    fn handle_notification(
        &mut self,
        connection: &Connection,
        notification: lsp_server::Notification,
    ) -> Result<(), String> {
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams = decode(notification.params)?;
                let item = params.text_document;
                let path = uri_to_path(&item.uri)?;
                self.open.insert(
                    normalize_path(&path),
                    (item.uri.clone(), item.version, item.text),
                );
                self.rescan(connection)
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = decode(notification.params)?;
                let Some(change) = params.content_changes.into_iter().last() else {
                    return Ok(());
                };
                let uri = params.text_document.uri;
                let path = uri_to_path(&uri)?;
                self.open.insert(
                    normalize_path(&path),
                    (uri, params.text_document.version, change.text),
                );
                self.rescan(connection)
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams = decode(notification.params)?;
                let path = uri_to_path(&params.text_document.uri)?;
                self.open.remove(&normalize_path(&path));
                self.rescan(connection)
            }
            DidChangeWatchedFiles::METHOD => {
                let _: DidChangeWatchedFilesParams = decode(notification.params)?;
                self.rescan(connection)
            }
            DidChangeWorkspaceFolders::METHOD => {
                let params: DidChangeWorkspaceFoldersParams = decode(notification.params)?;
                for folder in params.event.removed {
                    if let Ok(path) = uri_to_path(&folder.uri) {
                        self.workspaces.remove(&normalize_path(&path));
                    }
                }
                for folder in params.event.added {
                    if let Ok(path) = uri_to_path(&folder.uri) {
                        self.workspaces.insert(normalize_path(&path), folder.uri);
                    }
                }
                self.rescan(connection)
            }
            _ => Ok(()),
        }
    }

    fn rescan(&mut self, connection: &Connection) -> Result<(), String> {
        let old = self.published.clone();
        self.documents.clear();
        self.published.clear();

        let roots = self.workspaces.keys().cloned().collect::<Vec<_>>();
        for root in roots {
            self.scan_workspace(connection, &root)?;
        }
        let standalone = self
            .open
            .iter()
            .filter(|(path, _)| {
                !self.workspaces.keys().any(|root| path.starts_with(root))
                    && !self
                        .documents
                        .values()
                        .any(|document| document.path == **path)
            })
            .map(|(path, value)| (path.clone(), value.clone()))
            .collect::<Vec<_>>();
        for (path, (uri, version, text)) in standalone {
            self.insert_document(connection, &path, uri, version, text, None)?;
        }
        for (key, uri) in old {
            if !self.published.contains_key(&key) {
                publish(connection, uri, None, Vec::new())?;
            }
        }
        Ok(())
    }

    fn scan_workspace(&mut self, connection: &Connection, root: &Path) -> Result<(), String> {
        let mut paths = Vec::new();
        collect_workspace_files(root, &mut paths).map_err(|error| error.to_string())?;

        let config_path = root.join("thp.toml");
        let project = config_path.exists().then_some(root);
        if config_path.exists() {
            let config_uri = path_to_uri(&config_path)?;
            match ProjectConfig::load(root) {
                Ok(config) => {
                    let errors = validate_autoload(root, &config);
                    if errors.is_empty() {
                        for configured in config.autoload().values().flatten() {
                            let directory = if configured.is_absolute() {
                                configured.clone()
                            } else {
                                root.join(configured)
                            };
                            collect_workspace_files(&directory, &mut paths)
                                .map_err(|error| error.to_string())?;
                        }
                    }
                    publish(connection, config_uri.clone(), None, errors)?;
                    self.published
                        .insert(config_uri.as_str().to_owned(), config_uri);
                }
                Err(error) => {
                    let text = fs::read_to_string(&error.path).unwrap_or_default();
                    let span = error
                        .location
                        .as_ref()
                        .and_then(|location| location.span.clone())
                        .unwrap_or(0..0);
                    let diagnostic = config_diagnostic(&text, span, error.message);
                    let uri = path_to_uri(&error.path).unwrap_or(config_uri);
                    publish(connection, uri.clone(), None, vec![diagnostic])?;
                    self.published.insert(uri.as_str().to_owned(), uri);
                }
            }
        }
        paths.sort();
        paths.dedup_by(|left, right| normalize_path(left) == normalize_path(right));

        for path in paths {
            let key = normalize_path(&path);
            if let Some((uri, version, text)) = self.open.get(&key).cloned() {
                self.insert_document(connection, &key, uri, version, text, project)?;
            } else if let Ok(text) = fs::read_to_string(&path) {
                let uri = path_to_uri(&path)?;
                self.insert_document(connection, &key, uri, -1, text, project)?;
            }
        }
        let unlisted_open = self
            .open
            .iter()
            .filter(|(path, _)| {
                path.starts_with(root) && !self.documents.values().any(|d| d.path == **path)
            })
            .map(|(path, value)| (path.clone(), value.clone()))
            .collect::<Vec<_>>();
        for (path, (uri, version, text)) in unlisted_open {
            self.insert_document(connection, &path, uri, version, text, project)?;
        }
        if config_path.exists() {
            self.analyze_project(connection, root)?;
        }
        Ok(())
    }

    fn analyze_project(&mut self, connection: &Connection, root: &Path) -> Result<(), String> {
        let Ok(config) = ProjectConfig::load(root) else {
            return Ok(());
        };
        if !validate_autoload(root, &config).is_empty() {
            return Ok(());
        }
        let mappings = config
            .autoload()
            .iter()
            .map(|(prefix, paths)| AutoloadMapping::new(prefix, paths.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        let Some(seed) = self
            .documents
            .values()
            .find(|document| document.path.starts_with(root))
            .map(|document| document.path.clone())
        else {
            return Ok(());
        };
        let filesystem = FilesystemSourceProvider::new(root, mappings, &seed);
        let modules = match filesystem.enumerate() {
            Ok(modules) => modules,
            Err(error) => {
                let uri = path_to_uri(&root.join("thp.toml"))?;
                publish(
                    connection,
                    uri.clone(),
                    None,
                    vec![config_diagnostic("", 0..0, error.to_string())],
                )?;
                self.published.insert(uri.as_str().to_owned(), uri);
                return Ok(());
            }
        };
        let mapped = modules
            .iter()
            .filter(|module| !module.id.as_str().starts_with("@entry/"))
            .map(|module| normalize_path(&module.canonical_path))
            .collect::<BTreeSet<_>>();
        let entries = self
            .documents
            .values()
            .filter(|document| document.path.starts_with(root) && !mapped.contains(&document.path))
            .map(|document| document.path.clone())
            .collect::<Vec<_>>();
        let entries = if entries.is_empty() {
            vec![root.join(".thp-lsp-entry.thp")]
        } else {
            entries
        };
        let texts = self
            .documents
            .values()
            .map(|document| (document.path.clone(), document.source.text().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let mut diagnostics = BTreeMap::<PathBuf, Vec<LspDiagnostic>>::new();
        for entry in entries {
            let mut provider = InMemorySourceProvider::default();
            for module in &modules {
                if module.id.as_str().starts_with("@entry/") {
                    continue;
                }
                let path = normalize_path(&module.canonical_path);
                let Some(text) = texts.get(&path) else {
                    continue;
                };
                provider.insert(
                    module.id.clone(),
                    path,
                    module.expected_namespace.clone(),
                    text.clone(),
                    false,
                );
            }
            let entry_text = texts
                .get(&entry)
                .cloned()
                .unwrap_or_else(|| "<?thp\n".to_owned());
            let relative = entry.strip_prefix(root).unwrap_or(&entry);
            provider.insert(
                ModuleId::synthetic_entry(relative),
                entry.clone(),
                "",
                entry_text,
                true,
            );
            let request = ProjectRequest::new(root, &entry);
            match compile_project_with_provider(&request, &provider) {
                Ok(compilation) => {
                    for item in compilation.diagnostics {
                        let Some(source) = compilation.sources.get(item.source) else {
                            continue;
                        };
                        diagnostics
                            .entry(normalize_path(source.path()))
                            .or_default()
                            .push(compiler_diagnostic(source, &item.diagnostic));
                    }
                }
                Err(error) => {
                    diagnostics
                        .entry(normalize_path(&error.path))
                        .or_default()
                        .push(config_diagnostic("", 0..0, error.message));
                }
            }
        }
        let affected = self
            .documents
            .iter_mut()
            .filter(|(_, document)| document.path.starts_with(root))
            .map(|(uri, document)| {
                document
                    .diagnostics
                    .retain(|diagnostic| diagnostic.source.as_deref() != Some("thp"));
                if let Some(items) = diagnostics.remove(&document.path) {
                    for diagnostic in items {
                        if !document.diagnostics.contains(&diagnostic) {
                            document.diagnostics.push(diagnostic);
                        }
                    }
                }
                (uri.clone(), document.version, document.diagnostics.clone())
            })
            .collect::<Vec<_>>();
        for (uri, version, diagnostics) in affected {
            publish(
                connection,
                uri,
                (version >= 0).then_some(version),
                diagnostics,
            )?;
        }
        Ok(())
    }

    fn insert_document(
        &mut self,
        connection: &Connection,
        path: &Path,
        uri: Uri,
        version: i32,
        text: String,
        project: Option<&Path>,
    ) -> Result<(), String> {
        let project = project.map_or_else(|| normalize_path(path), normalize_path);
        let diagnostics = if let Some(document) = self.documents.get_mut(&uri) {
            document.projects.insert(project);
            document.diagnostics.clone()
        } else {
            let mut document = DocumentState::new_with_path(path, version, text);
            document.projects.insert(project);
            let diagnostics = document.diagnostics.clone();
            self.documents.insert(uri.clone(), document);
            diagnostics
        };
        let published_version = (version >= 0).then_some(version);
        publish(connection, uri.clone(), published_version, diagnostics)?;
        self.published.insert(uri.as_str().to_owned(), uri);
        Ok(())
    }

    fn handle_request(
        &self,
        connection: &Connection,
        request: ServerRequest,
    ) -> Result<(), String> {
        let id = request.id.clone();
        let result = match self.request_result(&request.method, request.params) {
            Ok(result) => Response::new_ok(id, result),
            Err(RequestFailure::Unsupported) => Response::new_err(
                id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                "unsupported request".to_owned(),
            ),
            Err(RequestFailure::Invalid(message)) => {
                Response::new_err(id, lsp_server::ErrorCode::InvalidParams as i32, message)
            }
        };
        connection
            .sender
            .send(Message::Response(result))
            .map_err(|error| error.to_string())
    }

    fn request_result(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, RequestFailure> {
        let result = match method {
            HoverRequest::METHOD => {
                let params: HoverParams = request_decode(params)?;
                request_encode(self.hover(params))?
            }
            Completion::METHOD => {
                let params: CompletionParams = request_decode(params)?;
                request_encode(self.completion(params))?
            }
            SignatureHelpRequest::METHOD => {
                let params: SignatureHelpParams = request_decode(params)?;
                request_encode(self.signature_help(params))?
            }
            GotoDefinition::METHOD => {
                let params: lsp_types::GotoDefinitionParams = request_decode(params)?;
                request_encode(self.definition(params))?
            }
            References::METHOD => {
                let params: lsp_types::ReferenceParams = request_decode(params)?;
                request_encode(self.references(params))?
            }
            PrepareRenameRequest::METHOD => {
                let params: lsp_types::TextDocumentPositionParams = request_decode(params)?;
                request_encode(self.prepare_rename(&params))?
            }
            Rename::METHOD => {
                let params: lsp_types::RenameParams = request_decode(params)?;
                request_encode(self.rename(params)?)?
            }
            DocumentSymbolRequest::METHOD => {
                let params: lsp_types::DocumentSymbolParams = request_decode(params)?;
                request_encode(self.document_symbols(&params))?
            }
            WorkspaceSymbolRequest::METHOD => {
                let params: lsp_types::WorkspaceSymbolParams = request_decode(params)?;
                request_encode(self.workspace_symbols(&params.query))?
            }
            SemanticTokensFullRequest::METHOD => {
                let params: lsp_types::SemanticTokensParams = request_decode(params)?;
                request_encode(self.semantic_tokens(&params))?
            }
            Formatting::METHOD => {
                let params: lsp_types::DocumentFormattingParams = request_decode(params)?;
                request_encode(self.format(&params))?
            }
            _ => return Err(RequestFailure::Unsupported),
        };
        Ok(result)
    }

    fn hover(&self, params: HoverParams) -> Option<Hover> {
        let target = params.text_document_position_params;
        let document = self.documents.get(&target.text_document.uri)?;
        let offset = document.offset(target.position)?;
        let (name, span, variable) = word_at(document.source.text(), offset)?;
        let markdown = if variable {
            document.variable_hover(&name, offset)?
        } else {
            document
                .declaration_at(offset)
                .and_then(|id| {
                    document
                        .declarations
                        .iter()
                        .find(|declaration| declaration.id.as_str() == id)
                })
                .or_else(|| {
                    let matches = self
                        .declarations(document.project())
                        .filter(|declaration| declaration.name.rsplit('\\').next() == Some(&name))
                        .collect::<Vec<_>>();
                    (matches.len() == 1).then(|| matches[0])
                })
                .map(Declaration::markdown)?
        };
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: Some(document.range(span)),
        })
    }

    fn completion(&self, params: CompletionParams) -> Option<CompletionResponse> {
        let target = params.text_document_position;
        let document = self.documents.get(&target.text_document.uri)?;
        let offset = document.offset(target.position)?;
        let receiver = receiver_before(document.source.text(), offset);
        let mut items: Vec<CompletionItem> = if let Some(variable) = receiver {
            let ty = document.variable_type(&variable, offset)?;
            let ancestors = self.owner_ancestors(&ty, document.project());
            self.declarations(document.project())
                .filter(|declaration| {
                    declaration
                        .owner
                        .as_deref()
                        .is_some_and(|owner| owner == ty || ancestors.contains(owner))
                })
                .map(Declaration::completion)
                .collect()
        } else {
            self.declarations(document.project())
                .filter(|declaration| declaration.owner.is_none())
                .map(Declaration::completion)
                .collect()
        };
        items.sort_by(|left, right| {
            (left.label.as_str(), left.detail.as_deref())
                .cmp(&(right.label.as_str(), right.detail.as_deref()))
        });
        Some(CompletionResponse::Array(items))
    }

    fn signature_help(&self, params: SignatureHelpParams) -> Option<SignatureHelp> {
        let target = params.text_document_position_params;
        let document = self.documents.get(&target.text_document.uri)?;
        let offset = document.offset(target.position)?;
        let call = call_before(document.source.text(), offset)?;
        let declaration = if let Some(receiver) = call.receiver {
            let ty = document.variable_type(&receiver, offset)?;
            let ancestors = self.owner_ancestors(&ty, document.project());
            let matches = self
                .declarations(document.project())
                .filter(|declaration| {
                    declaration.name == call.name
                        && declaration
                            .owner
                            .as_deref()
                            .is_some_and(|owner| owner == ty || ancestors.contains(owner))
                })
                .collect::<Vec<_>>();
            matches
                .iter()
                .find(|declaration| declaration.owner.as_deref() == Some(ty.as_str()))
                .copied()
                .or_else(|| (matches.len() == 1).then(|| matches[0]))
        } else {
            document
                .function_candidates(&call.name)
                .into_iter()
                .find_map(|name| {
                    let matches = self
                        .declarations(document.project())
                        .filter(|declaration| {
                            declaration.owner.is_none()
                                && declaration.name == name
                                && declaration.kind == DeclarationKind::Function
                        })
                        .collect::<Vec<_>>();
                    (matches.len() == 1).then(|| matches[0])
                })
        }?;
        Some(declaration.signature_help(call.argument))
    }

    fn definition(
        &self,
        params: lsp_types::GotoDefinitionParams,
    ) -> Option<lsp_types::GotoDefinitionResponse> {
        let target = params.text_document_position_params;
        let symbol = self.symbol_at(&target.text_document.uri, target.position)?;
        let locations = self.declaration_locations(&symbol);
        (!locations.is_empty()).then(|| lsp_types::GotoDefinitionResponse::Array(locations))
    }

    fn references(&self, params: lsp_types::ReferenceParams) -> Option<Vec<Location>> {
        let target = params.text_document_position;
        let symbol = self.symbol_at(&target.text_document.uri, target.position)?;
        let mut locations = self.occurrences(&symbol);
        if !params.context.include_declaration {
            let declarations = self.declaration_locations(&symbol);
            locations.retain(|location| !declarations.contains(location));
        }
        Some(locations)
    }

    fn prepare_rename(
        &self,
        params: &lsp_types::TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        let document = self.documents.get(&params.text_document.uri)?;
        let offset = document.offset(params.position)?;
        let symbol = self.symbol_at_offset(&params.text_document.uri, offset)?;
        let (name, mut span, _) = word_at(document.source.text(), offset)?;
        if let Some(separator) = name.rfind('\\') {
            span.start += u32::try_from(separator + 1).ok()?;
        }
        Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: document.range(span),
            placeholder: symbol.name,
        })
    }

    fn rename(
        &self,
        params: lsp_types::RenameParams,
    ) -> Result<Option<WorkspaceEdit>, RequestFailure> {
        let target = params.text_document_position;
        let symbol = self
            .symbol_at(&target.text_document.uri, target.position)
            .ok_or_else(|| RequestFailure::Invalid("symbol cannot be renamed safely".to_owned()))?;
        if !valid_identifier(&params.new_name, symbol.key.starts_with("local|")) {
            return Err(RequestFailure::Invalid(
                "rename target must be a valid THP identifier".to_owned(),
            ));
        }
        let mut grouped = BTreeMap::<String, (Uri, Vec<TextEdit>)>::new();
        for location in self.occurrences(&symbol) {
            let Some(document) = self.documents.get(&location.uri) else {
                continue;
            };
            let start = document.offset(location.range.start).unwrap_or(0);
            let replacement = if document.source.text()[start..].starts_with('$') {
                format!("${}", params.new_name)
            } else {
                params.new_name.clone()
            };
            grouped
                .entry(location.uri.as_str().to_owned())
                .or_insert_with(|| (location.uri.clone(), Vec::new()))
                .1
                .push(TextEdit {
                    range: location.range,
                    new_text: replacement,
                });
        }
        let edits = grouped
            .into_iter()
            .map(|(_, (uri, mut edits))| {
                edits.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
                let version = self
                    .documents
                    .get(&uri)
                    .and_then(|document| (document.version >= 0).then_some(document.version));
                TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier { uri, version },
                    edits: edits.into_iter().map(OneOf::Left).collect(),
                }
            })
            .collect();
        Ok(Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(lsp_types::DocumentChanges::Edits(edits)),
            change_annotations: None,
        }))
    }

    fn document_symbols(
        &self,
        params: &lsp_types::DocumentSymbolParams,
    ) -> Option<lsp_types::DocumentSymbolResponse> {
        let document = self.documents.get(&params.text_document.uri)?;
        let mut symbols = document
            .declarations
            .iter()
            .map(|declaration| SymbolInformation {
                name: declaration.name.clone(),
                kind: declaration.kind.symbol_kind(),
                tags: declaration
                    .deprecated()
                    .then(|| vec![lsp_types::SymbolTag::DEPRECATED]),
                #[allow(deprecated)]
                deprecated: None,
                location: Location::new(
                    params.text_document.uri.clone(),
                    document.range(declaration.name_span),
                ),
                container_name: declaration.owner.clone(),
            })
            .collect::<Vec<_>>();
        symbols.extend(
            document
                .local_symbols()
                .into_iter()
                .map(|local| SymbolInformation {
                    name: format!("${}", local.name),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                    location: Location::new(
                        params.text_document.uri.clone(),
                        document.range(local.span),
                    ),
                    container_name: None,
                }),
        );
        symbols.sort_by_key(|symbol| symbol.location.range.start);
        Some(lsp_types::DocumentSymbolResponse::Flat(symbols))
    }

    fn workspace_symbols(&self, query: &str) -> lsp_types::WorkspaceSymbolResponse {
        let query = query.to_ascii_lowercase();
        let mut symbols = Vec::new();
        for (uri, document) in &self.documents {
            symbols.extend(
                document
                    .declarations
                    .iter()
                    .filter(|declaration| declaration.name.to_ascii_lowercase().contains(&query))
                    .map(|declaration| SymbolInformation {
                        name: declaration.name.clone(),
                        kind: declaration.kind.symbol_kind(),
                        tags: declaration
                            .deprecated()
                            .then(|| vec![lsp_types::SymbolTag::DEPRECATED]),
                        #[allow(deprecated)]
                        deprecated: None,
                        location: Location::new(uri.clone(), document.range(declaration.name_span)),
                        container_name: declaration.owner.clone(),
                    }),
            );
            symbols.extend(
                document
                    .local_symbols()
                    .into_iter()
                    .filter(|local| local.name.to_ascii_lowercase().contains(&query))
                    .map(|local| SymbolInformation {
                        name: format!("${}", local.name),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        #[allow(deprecated)]
                        deprecated: None,
                        location: Location::new(uri.clone(), document.range(local.span)),
                        container_name: None,
                    }),
            );
        }
        symbols.sort_by(|left, right| {
            (
                left.name.as_str(),
                left.location.uri.as_str(),
                left.location.range.start,
            )
                .cmp(&(
                    right.name.as_str(),
                    right.location.uri.as_str(),
                    right.location.range.start,
                ))
        });
        lsp_types::WorkspaceSymbolResponse::Flat(symbols)
    }

    fn semantic_tokens(&self, params: &lsp_types::SemanticTokensParams) -> Option<SemanticTokens> {
        let document = self.documents.get(&params.text_document.uri)?;
        Some(SemanticTokens {
            result_id: None,
            data: document.semantic_tokens(),
        })
    }

    fn format(&self, params: &lsp_types::DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let document = self.documents.get(&params.text_document.uri)?;
        if !document.syntax_valid {
            return Some(Vec::new());
        }
        let formatted = format_source(
            document.source.text(),
            &document.tokens,
            params.options.tab_size,
            params.options.insert_spaces,
        );
        if formatted == document.source.text() {
            return Some(Vec::new());
        }
        Some(vec![TextEdit {
            range: Range::new(
                Position::new(0, 0),
                document.position(document.source.len()),
            ),
            new_text: formatted,
        }])
    }

    fn symbol_at(&self, uri: &Uri, position: Position) -> Option<ResolvedSymbol> {
        let document = self.documents.get(uri)?;
        self.symbol_at_offset(uri, document.offset(position)?)
    }

    fn symbol_at_offset(&self, uri: &Uri, offset: usize) -> Option<ResolvedSymbol> {
        let document = self.documents.get(uri)?;
        let (name, span, variable) = word_at(document.source.text(), offset)?;
        if let Some(separator) = name.rfind('\\')
            && offset < span.start as usize + separator + 1
        {
            return None;
        }
        if document
            .program
            .namespace
            .as_ref()
            .is_some_and(|namespace| {
                namespace.name.span.start <= span.start && span.end <= namespace.name.span.end
            })
        {
            return None;
        }
        if let Some(declaration) = document
            .declarations
            .iter()
            .find(|declaration| declaration.name_span == span)
        {
            return Some(Self::resolved_declaration(declaration, document.project()));
        }

        let before = document.source.text()[..span.start as usize].trim_end();
        if variable {
            if before.ends_with("->") || before.ends_with("::") {
                return None;
            }
            let scope = document.local_scope(offset)?;
            return Some(ResolvedSymbol {
                key: format!("local|{}|{scope}|{name}", document.path.display()),
                name,
                method_family: None,
                project: document.project().to_path_buf(),
            });
        }
        if before.ends_with("->") {
            let receiver = receiver_before(document.source.text(), span.start as usize)?;
            let owner = document.variable_type(&receiver, span.start as usize)?;
            let matches = self
                .declarations(document.project())
                .filter(|declaration| {
                    declaration.name == name
                        && declaration.owner.as_deref().is_some_and(|candidate| {
                            candidate == owner
                                || self.owner_is_related(candidate, &owner, document.project())
                        })
                })
                .collect::<Vec<_>>();
            if let Some(exact) = matches
                .iter()
                .find(|declaration| declaration.owner.as_deref() == Some(owner.as_str()))
            {
                return Some(Self::resolved_declaration(exact, document.project()));
            }
            return (matches.len() == 1)
                .then(|| Self::resolved_declaration(matches[0], document.project()));
        }
        if let Some(before_scope) = before.strip_suffix("::") {
            let owner_word = before_scope
                .trim_end()
                .rsplit(|character: char| {
                    !(character.is_ascii_alphanumeric() || character == '_' || character == '\\')
                })
                .next()?;
            let owner = document.resolve_type(owner_word);
            let matches = self
                .declarations(document.project())
                .filter(|declaration| {
                    declaration.name == name && declaration.owner.as_deref() == Some(owner.as_str())
                })
                .collect::<Vec<_>>();
            return (matches.len() == 1)
                .then(|| Self::resolved_declaration(matches[0], document.project()));
        }

        let mut names = document.function_candidates(&name);
        names.push(document.resolve_type(&name));
        names.push(name.clone());
        names.sort();
        names.dedup();
        let matches = self
            .declarations(document.project())
            .filter(|declaration| declaration.owner.is_none() && names.contains(&declaration.name))
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| Self::resolved_declaration(matches[0], document.project()))
    }

    fn resolved_declaration(declaration: &Declaration, project: &Path) -> ResolvedSymbol {
        let family = (declaration.kind == DeclarationKind::Method).then(|| {
            (
                declaration.name.clone(),
                declaration.owner.clone().unwrap_or_default(),
            )
        });
        ResolvedSymbol {
            key: declaration.id.as_str().to_owned(),
            name: declaration
                .name
                .rsplit('\\')
                .next()
                .unwrap_or(&declaration.name)
                .to_owned(),
            method_family: family,
            project: project.to_path_buf(),
        }
    }

    fn same_symbol(&self, left: &ResolvedSymbol, right: &ResolvedSymbol) -> bool {
        if left.project != right.project {
            return false;
        }
        if left.key == right.key {
            return true;
        }
        match (&left.method_family, &right.method_family) {
            (Some((left_name, left_owner)), Some((right_name, right_owner))) => {
                left_name == right_name
                    && self.owner_is_related(left_owner, right_owner, &left.project)
            }
            _ => false,
        }
    }

    fn owner_is_related(&self, left: &str, right: &str, project: &Path) -> bool {
        left == right
            || self.owner_ancestors(left, project).contains(right)
            || self.owner_ancestors(right, project).contains(left)
    }

    fn owner_ancestors(&self, owner: &str, project: &Path) -> BTreeSet<String> {
        let mut pending = vec![owner.to_owned()];
        let mut seen = BTreeSet::new();
        while let Some(current) = pending.pop() {
            for document in self
                .documents
                .values()
                .filter(|document| document.projects.contains(project))
            {
                pending.extend(
                    document
                        .parents_of(&current)
                        .into_iter()
                        .filter(|parent| seen.insert(parent.clone())),
                );
            }
        }
        seen
    }

    fn declaration_locations(&self, symbol: &ResolvedSymbol) -> Vec<Location> {
        if symbol.key.starts_with("local|") {
            return self.occurrences(symbol).into_iter().take(1).collect();
        }
        let mut output = self
            .documents
            .iter()
            .filter(|(_, document)| document.projects.contains(&symbol.project))
            .flat_map(|(uri, document)| {
                document
                    .declarations
                    .iter()
                    .filter(move |declaration| {
                        self.same_symbol(
                            symbol,
                            &Self::resolved_declaration(declaration, &symbol.project),
                        )
                    })
                    .map(move |declaration| {
                        Location::new(uri.clone(), document.range(declaration.name_span))
                    })
            })
            .collect::<Vec<_>>();
        output.sort_by(|left, right| {
            (left.uri.as_str(), left.range.start).cmp(&(right.uri.as_str(), right.range.start))
        });
        output
    }

    fn occurrences(&self, symbol: &ResolvedSymbol) -> Vec<Location> {
        let mut output = Vec::new();
        for (uri, document) in self
            .documents
            .iter()
            .filter(|(_, document)| document.projects.contains(&symbol.project))
        {
            for token in &document.tokens {
                if !matches!(token.kind, TokenKind::Identifier | TokenKind::Variable) {
                    continue;
                }
                let offset = (token.span.start as usize + 1).min(token.span.end as usize);
                if let Some(candidate) = self.symbol_at_offset(uri, offset)
                    && self.same_symbol(symbol, &candidate)
                {
                    output.push(Location::new(uri.clone(), document.range(token.span)));
                }
            }
        }
        output.sort_by(|left, right| {
            (left.uri.as_str(), left.range.start).cmp(&(right.uri.as_str(), right.range.start))
        });
        output.dedup();
        output
    }

    fn declarations<'a>(&'a self, project: &'a Path) -> impl Iterator<Item = &'a Declaration> {
        self.documents
            .values()
            .filter(move |document| document.projects.contains(project))
            .flat_map(|document| &document.declarations)
    }
}

enum RequestFailure {
    Unsupported,
    Invalid(String),
}

fn request_decode<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, RequestFailure> {
    decode(value).map_err(RequestFailure::Invalid)
}

fn request_encode<T: serde::Serialize>(value: T) -> Result<serde_json::Value, RequestFailure> {
    encode(value).map_err(RequestFailure::Invalid)
}

#[derive(Clone, Debug)]
struct ResolvedSymbol {
    key: String,
    name: String,
    method_family: Option<(String, String)>,
    project: PathBuf,
}

fn decode<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|error| error.to_string())
}

fn encode<T: serde::Serialize>(value: T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|error| error.to_string())
}

fn publish(
    connection: &Connection,
    uri: Uri,
    version: Option<i32>,
    diagnostics: Vec<LspDiagnostic>,
) -> Result<(), String> {
    connection
        .sender
        .send(Message::Notification(lsp_server::Notification::new(
            PublishDiagnostics::METHOD.to_owned(),
            PublishDiagnosticsParams::new(uri, diagnostics, version),
        )))
        .map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DeclarationId(String);

impl DeclarationId {
    fn new(path: &str, kind: DeclarationKind, owner: Option<&str>, name: &str) -> Self {
        Self(format!(
            "{path}|{}|{}|{name}",
            kind.as_str(),
            owner.unwrap_or("")
        ))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclarationKind {
    Function,
    Class,
    Interface,
    Trait,
    Method,
    Property,
}

impl DeclarationKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Trait => "trait",
            Self::Method => "method",
            Self::Property => "property",
        }
    }

    const fn completion_kind(self) -> CompletionItemKind {
        match self {
            Self::Function => CompletionItemKind::FUNCTION,
            Self::Class | Self::Trait => CompletionItemKind::CLASS,
            Self::Interface => CompletionItemKind::INTERFACE,
            Self::Method => CompletionItemKind::METHOD,
            Self::Property => CompletionItemKind::PROPERTY,
        }
    }

    const fn symbol_kind(self) -> SymbolKind {
        match self {
            Self::Function => SymbolKind::FUNCTION,
            Self::Class | Self::Trait => SymbolKind::CLASS,
            Self::Interface => SymbolKind::INTERFACE,
            Self::Method => SymbolKind::METHOD,
            Self::Property => SymbolKind::PROPERTY,
        }
    }
}

#[derive(Clone, Debug)]
struct Declaration {
    id: DeclarationId,
    kind: DeclarationKind,
    name: String,
    owner: Option<String>,
    name_span: Span,
    start: u32,
    signature: String,
    parameters: Vec<(String, String)>,
    documentation: Option<Docblock>,
}

impl Declaration {
    fn deprecated(&self) -> bool {
        self.documentation
            .as_ref()
            .is_some_and(|documentation| documentation.deprecated.is_some())
    }

    fn markdown(&self) -> String {
        let mut markdown = format!("```thp\n{}\n```", self.signature);
        if let Some(documentation) = &self.documentation {
            let rendered = documentation.markdown();
            if !rendered.is_empty() {
                markdown.push_str("\n\n");
                markdown.push_str(&rendered);
            }
        }
        markdown
    }

    fn completion(&self) -> CompletionItem {
        CompletionItem {
            label: self.name.clone(),
            kind: Some(self.kind.completion_kind()),
            detail: Some(self.signature.clone()),
            documentation: self.documentation.as_ref().map(|docs| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: docs.markdown(),
                })
            }),
            deprecated: self
                .documentation
                .as_ref()
                .and_then(|docs| docs.deprecated.as_ref().map(|_| true)),
            ..CompletionItem::default()
        }
    }

    fn signature_help(&self, active: u32) -> SignatureHelp {
        let parameter_docs = self.documentation.as_ref().map(|docs| &docs.params);
        SignatureHelp {
            signatures: vec![SignatureInformation {
                label: self.signature.clone(),
                documentation: self.documentation.as_ref().map(|docs| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: docs.prose_markdown(),
                    })
                }),
                parameters: Some(
                    self.parameters
                        .iter()
                        .map(|(name, ty)| ParameterInformation {
                            label: ParameterLabel::Simple(format!("{ty} ${name}")),
                            documentation: parameter_docs
                                .and_then(|docs| docs.iter().find(|param| param.name == *name))
                                .filter(|param| !param.description.is_empty())
                                .map(|param| Documentation::String(param.description.clone())),
                        })
                        .collect(),
                ),
                active_parameter: Some(active),
            }],
            active_signature: Some(0),
            active_parameter: Some(active),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct DocblockIndex {
    blocks: Vec<Docblock>,
    by_declaration: HashMap<DeclarationId, usize>,
    var_hints: Vec<VarHint>,
}

#[derive(Clone, Debug, Default)]
struct Docblock {
    span: Span,
    summary: String,
    description: String,
    params: Vec<ParamDoc>,
    returns: Vec<TypedDoc>,
    throws: Vec<TypedDoc>,
    deprecated: Option<String>,
    internal: Option<String>,
    since: Option<String>,
    see: Vec<String>,
    vars: Vec<VarDoc>,
    unknown: Vec<UnknownTag>,
}

#[derive(Clone, Debug)]
struct ParamDoc {
    ty: String,
    name: String,
    description: String,
    span: Span,
    valid_target: bool,
}

#[derive(Clone, Debug)]
struct VarDoc {
    ty: String,
    name: String,
    description: String,
    span: Span,
    valid_target: bool,
}

#[derive(Clone, Debug)]
struct TypedDoc {
    ty: String,
    description: String,
    span: Span,
}

#[derive(Clone, Debug)]
struct UnknownTag {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    span: Span,
}

#[derive(Clone, Debug)]
struct VarHint {
    name: String,
    ty: String,
    assignment: u32,
    scope_end: u32,
    description: String,
}

impl Docblock {
    fn prose_markdown(&self) -> String {
        match (self.summary.is_empty(), self.description.is_empty()) {
            (true, true) => String::new(),
            (false, true) => self.summary.clone(),
            (true, false) => self.description.clone(),
            (false, false) => format!("{}\n\n{}", self.summary, self.description),
        }
    }

    fn markdown(&self) -> String {
        let mut sections = Vec::new();
        let prose = self.prose_markdown();
        if !prose.is_empty() {
            sections.push(prose);
        }
        if let Some(deprecated) = &self.deprecated {
            sections.push(if deprecated.is_empty() {
                "**Deprecated.**".to_owned()
            } else {
                format!("**Deprecated.** {deprecated}")
            });
        }
        if let Some(internal) = &self.internal {
            sections.push(if internal.is_empty() {
                "**Internal.**".to_owned()
            } else {
                format!("**Internal.** {internal}")
            });
        }
        if let Some(since) = &self.since {
            sections.push(format!("**Since:** {since}"));
        }
        if !self.params.is_empty() {
            sections.push(format!(
                "**Parameters:**\n{}",
                self.params
                    .iter()
                    .map(|parameter| format!(
                        "- `{}` `${}`{}",
                        parameter.ty,
                        parameter.name,
                        suffix(&parameter.description)
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        if let Some(returns) = self.returns.first() {
            sections.push(format!(
                "**Returns:** `{}`{}",
                returns.ty,
                suffix(&returns.description)
            ));
        }
        for thrown in &self.throws {
            sections.push(format!(
                "**Throws:** `{}`{}",
                thrown.ty,
                suffix(&thrown.description)
            ));
        }
        for see in &self.see {
            sections.push(format!("**See:** {see}"));
        }
        sections.join("\n\n")
    }
}

fn suffix(text: &str) -> String {
    if text.is_empty() {
        String::new()
    } else {
        format!(" — {text}")
    }
}

struct DocumentState {
    version: i32,
    path: PathBuf,
    projects: BTreeSet<PathBuf>,
    source: SourceFile,
    #[allow(dead_code)]
    compilation: Compilation,
    program: Program,
    tokens: Vec<Token>,
    syntax_valid: bool,
    declarations: Vec<Declaration>,
    #[allow(dead_code)]
    docblocks: DocblockIndex,
    diagnostics: Vec<LspDiagnostic>,
}

struct LocalSymbol {
    name: String,
    span: Span,
}

impl DocumentState {
    #[cfg(test)]
    fn new(uri: &Uri, version: i32, text: String) -> Self {
        let path = uri_to_path(uri).unwrap_or_else(|_| PathBuf::from(uri.as_str()));
        Self::new_with_path(&path, version, text)
    }

    fn new_with_path(path: &Path, version: i32, text: String) -> Self {
        let source = SourceFile::new(path, text.clone());
        let compilation = compile_text(path, text);
        let (lexed, spans) = lex_with_docblocks(&source);
        let parsed = parse_tokens(&source, lexed.tokens, lexed.diagnostics);
        let syntax_valid = parsed.diagnostics.is_empty();
        let tokens = parsed.tokens.clone();
        let mut diagnostics = compilation
            .diagnostics
            .iter()
            .map(|diagnostic| compiler_diagnostic(&source, diagnostic))
            .collect::<Vec<_>>();
        let path_text = path.to_string_lossy();
        let mut declarations = collect_declarations(&path_text, &parsed.program);
        let mut docblocks = DocblockIndex {
            blocks: spans
                .iter()
                .map(|span| parse_docblock(&source, *span))
                .collect(),
            ..DocblockIndex::default()
        };
        associate_declarations(&parsed.tokens, &mut declarations, &mut docblocks);
        validate_declarations(
            &source,
            &parsed.program,
            compilation.hir.as_ref(),
            &mut declarations,
            &mut diagnostics,
        );
        associate_var_hints(
            &source,
            &parsed.program,
            &parsed.tokens,
            &mut docblocks,
            &mut diagnostics,
        );
        Self {
            version,
            path: normalize_path(path),
            projects: std::iter::once(normalize_path(path)).collect(),
            source,
            compilation,
            program: parsed.program,
            tokens,
            syntax_valid,
            declarations,
            docblocks,
            diagnostics,
        }
    }

    fn offset(&self, position: Position) -> Option<usize> {
        let line = usize::try_from(position.line).ok()?;
        let target = usize::try_from(position.character).ok()?;
        let text = self.source.text();
        let start = if line == 0 {
            0
        } else {
            text.match_indices('\n').nth(line - 1)?.0 + 1
        };
        let end = text[start..]
            .find('\n')
            .map_or(text.len(), |length| start + length);
        let mut utf16 = 0;
        for (relative, character) in text[start..end].char_indices() {
            if utf16 == target {
                return Some(start + relative);
            }
            utf16 += character.len_utf16();
            if utf16 > target {
                return None;
            }
        }
        (utf16 == target).then_some(end)
    }

    fn project(&self) -> &Path {
        self.projects
            .first()
            .expect("every document belongs to at least one project")
    }

    fn position(&self, offset: usize) -> Position {
        position(self.source.text(), offset)
    }

    fn range(&self, span: Span) -> Range {
        Range::new(
            self.position(span.start as usize),
            self.position(span.end as usize),
        )
    }

    fn declaration_at(&self, offset: usize) -> Option<&str> {
        self.declarations
            .iter()
            .find(|declaration| declaration.name_span.range().contains(&offset))
            .map(|declaration| declaration.id.as_str())
    }

    fn variable_type(&self, name: &str, offset: usize) -> Option<String> {
        self.docblocks
            .var_hints
            .iter()
            .filter(|hint| {
                hint.name == name
                    && hint.assignment as usize <= offset
                    && offset <= hint.scope_end as usize
            })
            .max_by_key(|hint| hint.assignment)
            .map(|hint| base_nominal(&hint.ty))
            .or_else(|| {
                self.compiler_local(name, offset)
                    .map(|ty| base_nominal(&ty))
            })
    }

    fn variable_hover(&self, name: &str, offset: usize) -> Option<String> {
        let hinted = self
            .docblocks
            .var_hints
            .iter()
            .filter(|hint| {
                hint.name == name
                    && hint.assignment as usize <= offset
                    && offset <= hint.scope_end as usize
            })
            .max_by_key(|hint| hint.assignment)
            .map(|hint| hint.ty.clone());
        let compiler = self.compiler_local(name, offset);
        let description = self
            .docblocks
            .var_hints
            .iter()
            .filter(|hint| {
                hint.name == name
                    && hint.assignment as usize <= offset
                    && offset <= hint.scope_end as usize
            })
            .max_by_key(|hint| hint.assignment)
            .map(|hint| hint.description.as_str())
            .filter(|description| !description.is_empty());
        let mut rendered = match (hinted, compiler) {
            (Some(hint), Some(compiler)) if hint != compiler => Some(format!(
                "```thp\n{hint} ${name}\n```\n\nCompiler type: `{compiler}`"
            )),
            (Some(hint), _) => Some(format!("```thp\n{hint} ${name}\n```")),
            (None, Some(compiler)) => Some(format!("```thp\n{compiler} ${name}\n```")),
            (None, None) => None,
        }?;
        if let Some(description) = description {
            rendered.push_str("\n\n");
            rendered.push_str(description);
        }
        Some(rendered)
    }

    fn compiler_local(&self, name: &str, offset: usize) -> Option<String> {
        self.compilation
            .hir
            .as_ref()
            .and_then(|hir| {
                hir.functions
                    .iter()
                    .filter(|function| {
                        function.span.start as usize <= offset
                            && offset <= function.span.end as usize
                    })
                    .min_by_key(|function| function.span.end - function.span.start)
                    .or_else(|| {
                        hir.functions
                            .iter()
                            .find(|function| function.owner.is_none())
                    })?
                    .locals
                    .iter()
                    .find(|local| local.name == name)
                    .map(|local| local.ty.to_string())
            })
            .or_else(|| source_local_type(&self.program, name, offset))
            .map(|ty| {
                let base = base_nominal(&ty);
                if matches!(
                    base.as_str(),
                    "int"
                        | "float"
                        | "bool"
                        | "string"
                        | "null"
                        | "void"
                        | "mixed"
                        | "never"
                        | "vector"
                        | "map"
                ) {
                    ty
                } else {
                    let resolved = self.resolve_type(&base);
                    ty.replacen(&base, &resolved, 1)
                }
            })
    }

    fn function_candidates(&self, name: &str) -> Vec<String> {
        if let Some(name) = name.strip_prefix('\\') {
            return vec![name.to_owned()];
        }
        let program = &self.program;
        let (first, rest) = name.split_once('\\').unwrap_or((name, ""));
        if let Some(import) = program
            .imports
            .iter()
            .find(|import| import.kind == UseKind::Function && import.alias == first)
        {
            let target = import
                .target
                .as_string()
                .trim_start_matches('\\')
                .to_owned();
            return vec![if rest.is_empty() {
                target
            } else {
                format!("{target}\\{rest}")
            }];
        }
        let namespace = program
            .namespace
            .as_ref()
            .map_or_else(String::new, |namespace| {
                namespace
                    .name
                    .as_string()
                    .trim_start_matches('\\')
                    .to_owned()
            });
        let qualified = canonical(&namespace, name);
        if name.contains('\\') || qualified == name {
            vec![qualified]
        } else {
            vec![qualified, name.to_owned()]
        }
    }

    fn resolve_type(&self, name: &str) -> String {
        let (namespace, aliases) = resolution_context(&self.program);
        thp_modules::resolve_type_name(name, &namespace, &aliases)
    }

    fn local_scope(&self, offset: usize) -> Option<String> {
        let hir = self.compilation.hir.as_ref()?;
        Some(
            hir.functions
                .iter()
                .filter(|function| {
                    function.span.start as usize <= offset && offset <= function.span.end as usize
                })
                .min_by_key(|function| function.span.end - function.span.start)
                .map_or_else(
                    || format!("0:{}", self.source.len()),
                    |function| format!("{}:{}", function.span.start, function.span.end),
                ),
        )
    }

    fn parents_of(&self, owner: &str) -> Vec<String> {
        let namespace = self
            .program
            .namespace
            .as_ref()
            .map_or_else(String::new, |value| {
                value.name.as_string().trim_start_matches('\\').to_owned()
            });
        self.program
            .statements
            .iter()
            .find_map(|statement| match &statement.kind {
                StmtKind::Class(class) if canonical(&namespace, &class.name) == owner => Some(
                    class
                        .parent
                        .iter()
                        .chain(&class.interfaces)
                        .map(|parent| self.resolve_type(&parent.name))
                        .collect(),
                ),
                StmtKind::Interface(interface)
                    if canonical(&namespace, &interface.name) == owner =>
                {
                    Some(
                        interface
                            .parent
                            .iter()
                            .map(|parent| self.resolve_type(&parent.name))
                            .collect(),
                    )
                }
                _ => None,
            })
            .unwrap_or_default()
    }

    fn semantic_tokens(&self) -> Vec<SemanticToken> {
        let mut raw = Vec::<(Position, u32, u32, u32)>::new();
        let comments = comment_spans(self.source.text());
        for span in comments {
            push_semantic_span(self.source.text(), span, 12, 0, &mut raw);
        }
        for token in &self.tokens {
            if token.kind == TokenKind::Eof {
                continue;
            }
            let (kind, modifiers) = semantic_kind(self, token);
            push_semantic_span(self.source.text(), token.span, kind, modifiers, &mut raw);
        }
        raw.sort_by_key(|(position, _, _, _)| (position.line, position.character));
        let mut previous = Position::new(0, 0);
        raw.into_iter()
            .map(|(position, length, token_type, token_modifiers_bitset)| {
                let delta_line = position.line - previous.line;
                let delta_start = if delta_line == 0 {
                    position.character - previous.character
                } else {
                    position.character
                };
                previous = position;
                SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset,
                }
            })
            .collect()
    }

    fn local_symbols(&self) -> Vec<LocalSymbol> {
        let Some(hir) = &self.compilation.hir else {
            return Vec::new();
        };
        let mut output = Vec::new();
        for function in &hir.functions {
            for local in &function.locals {
                if local.name == "this" {
                    continue;
                }
                let span = self
                    .tokens
                    .iter()
                    .filter(|token| token.kind == TokenKind::Variable)
                    .filter(|token| {
                        function.span.start <= token.span.start
                            && token.span.end <= function.span.end
                    })
                    .find(|token| {
                        self.source.text()[token.span.range()].strip_prefix('$')
                            == Some(local.name.as_str())
                    })
                    .map_or(local.span, |token| token.span);
                output.push(LocalSymbol {
                    name: local.name.clone(),
                    span,
                });
            }
        }
        output.sort_by_key(|local| (local.span.start, local.name.clone()));
        output
    }
}

fn source_local_type(program: &Program, name: &str, offset: usize) -> Option<String> {
    for statement in &program.statements {
        match &statement.kind {
            StmtKind::Function(function)
                if statement.span.start as usize <= offset
                    && offset <= statement.span.end as usize =>
            {
                return function
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == name)
                    .map(|parameter| parameter.ty.to_string())
                    .or_else(|| block_local_type(&function.body, name, offset));
            }
            StmtKind::Class(class) => {
                if let Some(method) = class.methods.iter().find(|method| {
                    method.span.start as usize <= offset && offset <= method.span.end as usize
                }) {
                    return method
                        .function
                        .parameters
                        .iter()
                        .find(|parameter| parameter.name == name)
                        .map(|parameter| parameter.ty.to_string())
                        .or_else(|| block_local_type(&method.function.body, name, offset));
                }
            }
            StmtKind::Trait(trait_decl) => {
                if let Some(method) = trait_decl.methods.iter().find(|method| {
                    method.span.start as usize <= offset && offset <= method.span.end as usize
                }) {
                    return method
                        .function
                        .parameters
                        .iter()
                        .find(|parameter| parameter.name == name)
                        .map(|parameter| parameter.ty.to_string())
                        .or_else(|| block_local_type(&method.function.body, name, offset));
                }
            }
            _ => {}
        }
    }
    block_local_type(&program.statements, name, offset)
}

fn block_local_type(statements: &[Stmt], name: &str, offset: usize) -> Option<String> {
    let mut found = None;
    for statement in statements
        .iter()
        .filter(|statement| statement.span.start as usize <= offset)
    {
        match &statement.kind {
            StmtKind::Assign {
                name: assigned,
                annotation,
                value,
            } if assigned == name => {
                found = annotation.as_ref().map(ToString::to_string).or_else(|| {
                    let ExprKind::New { class_name, .. } = &value.kind else {
                        return None;
                    };
                    Some(class_name.clone())
                });
            }
            StmtKind::If {
                branches,
                otherwise,
            } => {
                found = branches
                    .iter()
                    .find_map(|(_, body)| {
                        block_contains(body, offset)
                            .then(|| block_local_type(body, name, offset))
                            .flatten()
                    })
                    .or_else(|| {
                        otherwise
                            .as_ref()
                            .filter(|body| block_contains(body, offset))
                            .and_then(|body| block_local_type(body, name, offset))
                    })
                    .or(found);
            }
            StmtKind::While { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::Using { body, .. }
            | StmtKind::Block(body)
                if block_contains(body, offset) =>
            {
                found = block_local_type(body, name, offset).or(found);
            }
            StmtKind::Try {
                body,
                catches,
                finally,
            } => {
                if block_contains(body, offset) {
                    found = block_local_type(body, name, offset).or(found);
                }
                for catch in catches.iter().filter(|catch| {
                    catch.span.start as usize <= offset && offset <= catch.span.end as usize
                }) {
                    found = if catch.variable == name {
                        Some(catch.class_name.clone())
                    } else {
                        block_local_type(&catch.body, name, offset).or(found)
                    };
                }
                if let Some(finally) = finally
                    && block_contains(finally, offset)
                {
                    found = block_local_type(finally, name, offset).or(found);
                }
            }
            _ => {}
        }
    }
    found
}

fn block_contains(statements: &[Stmt], offset: usize) -> bool {
    statements
        .first()
        .is_some_and(|statement| statement.span.start as usize <= offset)
        && statements
            .last()
            .is_some_and(|statement| offset <= statement.span.end as usize)
}

fn normalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        path.parent()
            .and_then(|parent| fs::canonicalize(parent).ok())
            .and_then(|parent| path.file_name().map(|name| parent.join(name)))
            .unwrap_or_else(|| path.to_path_buf())
    })
}

fn uri_to_path(uri: &Uri) -> Result<PathBuf, String> {
    let raw = uri.as_str();
    let rest = raw
        .strip_prefix("file://")
        .ok_or_else(|| "only file URIs are supported".to_owned())?;
    if rest.contains(['?', '#']) {
        return Err("file URI cannot contain a query or fragment".to_owned());
    }
    let decoded = percent_decode(rest)?;
    #[cfg(windows)]
    {
        if let Some(unc) = decoded.strip_prefix('/') {
            if unc.as_bytes().get(1) == Some(&b':') {
                return Ok(PathBuf::from(unc.replace('/', "\\")));
            }
        }
        if !decoded.starts_with('/') {
            return Ok(PathBuf::from(format!("\\\\{}", decoded.replace('/', "\\"))));
        }
        Ok(PathBuf::from(decoded.replace('/', "\\")))
    }
    #[cfg(not(windows))]
    {
        if decoded.starts_with('/') {
            Ok(PathBuf::from(decoded))
        } else {
            Ok(PathBuf::from(format!("//{decoded}")))
        }
    }
}

fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let pair = bytes
                .get(index + 1..index + 3)
                .ok_or_else(|| "truncated percent escape in file URI".to_owned())?;
            let text = std::str::from_utf8(pair).map_err(|error| error.to_string())?;
            output.push(
                u8::from_str_radix(text, 16)
                    .map_err(|_| "invalid percent escape in file URI".to_owned())?,
            );
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).map_err(|_| "file URI path is not valid UTF-8".to_owned())
}

fn path_to_uri(path: &Path) -> Result<Uri, String> {
    #[allow(unused_mut)]
    let mut path = normalize_path(path).to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    let prefix = if path.starts_with("//") {
        path = path.trim_start_matches('/').to_owned();
        "file://"
    } else {
        "file:///"
    };
    #[cfg(not(windows))]
    let prefix = "file://";
    let encoded = path
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect::<String>();
    Uri::from_str(&format!("{prefix}{encoded}")).map_err(|error| error.to_string())
}

fn collect_workspace_files(directory: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if let Err(error) = collect_workspace_files(&path, output)
                && error.kind() != std::io::ErrorKind::PermissionDenied
            {
                return Err(error);
            }
        } else if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("thp")
        {
            output.push(path);
        }
    }
    Ok(())
}

fn validate_autoload(root: &Path, config: &ProjectConfig) -> Vec<LspDiagnostic> {
    let mut directories = Vec::<(String, PathBuf)>::new();
    let mut diagnostics = Vec::new();
    for (prefix, paths) in config.autoload() {
        for configured in paths {
            let path = if configured.is_absolute() {
                configured.clone()
            } else {
                root.join(configured)
            };
            if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
                diagnostics.push(config_diagnostic(
                    "",
                    0..0,
                    format!(
                        "autoload directory {} cannot be a symbolic link",
                        path.display()
                    ),
                ));
                continue;
            }
            match fs::canonicalize(&path) {
                Ok(canonical) if canonical.is_dir() => {
                    if let Err(error) = fs::read_dir(&canonical) {
                        diagnostics.push(config_diagnostic(
                            "",
                            0..0,
                            format!(
                                "autoload directory {} is not readable: {error}",
                                path.display()
                            ),
                        ));
                    } else {
                        directories.push((prefix.clone(), canonical));
                    }
                }
                Ok(_) => diagnostics.push(config_diagnostic(
                    "",
                    0..0,
                    format!("autoload path {} is not a directory", path.display()),
                )),
                Err(error) => diagnostics.push(config_diagnostic(
                    "",
                    0..0,
                    format!(
                        "autoload directory {} cannot be resolved: {error}",
                        path.display()
                    ),
                )),
            }
        }
    }
    for (index, (prefix, directory)) in directories.iter().enumerate() {
        for (other_prefix, other) in &directories[index + 1..] {
            if directory.starts_with(other) || other.starts_with(directory) {
                diagnostics.push(config_diagnostic(
                    "",
                    0..0,
                    format!(
                        "overlapping autoload mappings `{prefix}` ({}) and `{other_prefix}` ({})",
                        directory.display(),
                        other.display()
                    ),
                ));
            }
        }
    }
    diagnostics
}

fn config_diagnostic(source: &str, span: std::ops::Range<usize>, message: String) -> LspDiagnostic {
    LspDiagnostic {
        range: Range::new(position(source, span.start), position(source, span.end)),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(lsp_types::NumberOrString::String("CONFIG".to_owned())),
        source: Some("thp-config".to_owned()),
        message,
        ..LspDiagnostic::default()
    }
}

fn position(text: &str, offset: usize) -> Position {
    let offset = offset.min(text.len());
    let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count();
    let start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    Position::new(
        u32::try_from(line).unwrap_or(u32::MAX),
        u32::try_from(text[start..offset].encode_utf16().count()).unwrap_or(u32::MAX),
    )
}

fn compiler_diagnostic(
    source: &SourceFile,
    diagnostic: &thp_diagnostics::Diagnostic,
) -> LspDiagnostic {
    let span = diagnostic
        .labels
        .first()
        .map_or(Span::empty(0), |label| label.span);
    LspDiagnostic {
        range: Range::new(
            position(source.text(), span.start as usize),
            position(source.text(), span.end as usize),
        ),
        severity: Some(match diagnostic.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: Some(lsp_types::NumberOrString::String(
            diagnostic.code.to_owned(),
        )),
        source: Some("thp".to_owned()),
        message: diagnostic.message.clone(),
        ..LspDiagnostic::default()
    }
}

fn doc_diagnostic(
    source: &SourceFile,
    code: &'static str,
    span: Span,
    message: String,
) -> LspDiagnostic {
    LspDiagnostic {
        range: Range::new(
            position(source.text(), span.start as usize),
            position(source.text(), span.end as usize),
        ),
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(lsp_types::NumberOrString::String(code.to_owned())),
        source: Some("thp-doc".to_owned()),
        message,
        ..LspDiagnostic::default()
    }
}

fn collect_declarations(path: &str, program: &Program) -> Vec<Declaration> {
    let namespace = program
        .namespace
        .as_ref()
        .map_or_else(String::new, |declaration| {
            declaration
                .name
                .as_string()
                .trim_start_matches('\\')
                .to_owned()
        });
    let mut output = Vec::new();
    for statement in &program.statements {
        match &statement.kind {
            StmtKind::Function(function) => output.push(function_declaration(
                path,
                &namespace,
                None,
                function,
                statement.span.start,
                DeclarationKind::Function,
            )),
            StmtKind::Class(class) => {
                collect_class(path, &namespace, class, statement.span.start, &mut output);
            }
            StmtKind::Interface(interface) => {
                collect_interface(
                    path,
                    &namespace,
                    interface,
                    statement.span.start,
                    &mut output,
                );
            }
            StmtKind::Trait(trait_decl) => {
                collect_trait(
                    path,
                    &namespace,
                    trait_decl,
                    statement.span.start,
                    &mut output,
                );
            }
            _ => {}
        }
    }
    output.sort_by_key(|declaration| declaration.start);
    output
}

fn canonical(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}\\{name}")
    }
}

fn nominal_declaration(
    path: &str,
    namespace: &str,
    name: &str,
    name_span: Span,
    start: u32,
    kind: DeclarationKind,
) -> Declaration {
    let name = canonical(namespace, name);
    Declaration {
        id: DeclarationId::new(path, kind, None, &name),
        kind,
        signature: format!("{} {name}", kind.as_str()),
        name,
        owner: None,
        name_span,
        start,
        parameters: Vec::new(),
        documentation: None,
    }
}

fn function_declaration(
    path: &str,
    namespace: &str,
    owner: Option<&str>,
    function: &FunctionDecl,
    start: u32,
    kind: DeclarationKind,
) -> Declaration {
    let name = if owner.is_none() {
        canonical(namespace, &function.name)
    } else {
        function.name.clone()
    };
    let parameters = function
        .parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.ty.to_string()))
        .collect::<Vec<_>>();
    let rendered = parameters
        .iter()
        .map(|(name, ty)| format!("{ty} ${name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let prefix = owner.map_or("function ".to_owned(), |owner| format!("{owner}::"));
    Declaration {
        id: DeclarationId::new(path, kind, owner, &name),
        kind,
        signature: format!("{prefix}{name}({rendered}): {}", function.return_type),
        name,
        owner: owner.map(str::to_owned),
        name_span: function.name_span,
        start,
        parameters,
        documentation: None,
    }
}

fn collect_class(
    path: &str,
    namespace: &str,
    class: &ClassDecl,
    start: u32,
    output: &mut Vec<Declaration>,
) {
    let owner = canonical(namespace, &class.name);
    output.push(nominal_declaration(
        path,
        namespace,
        &class.name,
        class.name_span,
        start,
        DeclarationKind::Class,
    ));
    collect_members(
        path,
        namespace,
        &owner,
        &class.properties,
        &class.methods,
        output,
    );
}

fn collect_interface(
    path: &str,
    namespace: &str,
    interface: &InterfaceDecl,
    start: u32,
    output: &mut Vec<Declaration>,
) {
    let owner = canonical(namespace, &interface.name);
    output.push(nominal_declaration(
        path,
        namespace,
        &interface.name,
        interface.name_span,
        start,
        DeclarationKind::Interface,
    ));
    collect_members(path, namespace, &owner, &[], &interface.methods, output);
}

fn collect_trait(
    path: &str,
    namespace: &str,
    trait_decl: &TraitDecl,
    start: u32,
    output: &mut Vec<Declaration>,
) {
    let owner = canonical(namespace, &trait_decl.name);
    output.push(nominal_declaration(
        path,
        namespace,
        &trait_decl.name,
        trait_decl.name_span,
        start,
        DeclarationKind::Trait,
    ));
    collect_members(
        path,
        namespace,
        &owner,
        &trait_decl.properties,
        &trait_decl.methods,
        output,
    );
}

fn collect_members(
    path: &str,
    namespace: &str,
    owner: &str,
    properties: &[PropertyDecl],
    methods: &[MethodDecl],
    output: &mut Vec<Declaration>,
) {
    for property in properties {
        output.push(Declaration {
            id: DeclarationId::new(path, DeclarationKind::Property, Some(owner), &property.name),
            kind: DeclarationKind::Property,
            name: property.name.clone(),
            owner: Some(owner.to_owned()),
            name_span: property.name_span,
            start: property.span.start,
            signature: format!("{} ${}", property.ty, property.name),
            parameters: Vec::new(),
            documentation: None,
        });
    }
    for method in methods {
        output.push(function_declaration(
            path,
            namespace,
            Some(owner),
            &method.function,
            method.span.start,
            DeclarationKind::Method,
        ));
    }
}

fn associate_declarations(
    tokens: &[Token],
    declarations: &mut [Declaration],
    index: &mut DocblockIndex,
) {
    for declaration in declarations {
        let candidate = index
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| block.span.end <= declaration.start)
            .filter(|(_, block)| !has_token_between(tokens, block.span.end, declaration.start))
            .max_by_key(|(_, block)| block.span.end)
            .map(|(id, _)| id);
        if let Some(id) = candidate {
            declaration.documentation = Some(index.blocks[id].clone());
            index.by_declaration.insert(declaration.id.clone(), id);
        }
    }
}

fn has_token_between(tokens: &[Token], start: u32, end: u32) -> bool {
    tokens.iter().any(|token| {
        token.kind != TokenKind::Eof && token.span.start >= start && token.span.end <= end
    })
}

fn parse_docblock(source: &SourceFile, block: DocblockSpan) -> Docblock {
    let raw = block.text(source.text());
    let body = raw
        .strip_prefix("/**")
        .and_then(|text| text.strip_suffix("*/"))
        .unwrap_or(raw);
    let mut lines = body
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(|line| {
            let line = line.trim_start();
            line.strip_prefix('*')
                .map_or(line, |line| line.strip_prefix(' ').unwrap_or(line))
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>();
    while lines.first().is_some_and(String::is_empty) {
        lines.remove(0);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }

    let mut parsed = Docblock {
        span: block.span,
        ..Docblock::default()
    };
    let mut prose = Vec::new();
    let mut current_tag: Option<(String, String)> = None;
    let finish_tag = |tag: Option<(String, String)>, parsed: &mut Docblock| {
        if let Some((name, payload)) = tag {
            parse_tag(parsed, &name, payload.trim(), block.span);
        }
    };
    for line in lines {
        if let Some(tag) = line.strip_prefix('@') {
            finish_tag(current_tag.take(), &mut parsed);
            let (name, payload) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
            current_tag = Some((name.to_owned(), payload.trim().to_owned()));
        } else if let Some((_, payload)) = &mut current_tag {
            if !line.is_empty() {
                if !payload.is_empty() {
                    payload.push(' ');
                }
                payload.push_str(line.trim());
            }
        } else {
            prose.push(line);
        }
    }
    finish_tag(current_tag, &mut parsed);
    let prose = prose.join("\n");
    let mut paragraphs = prose.split("\n\n").filter(|part| !part.trim().is_empty());
    parsed.summary = paragraphs.next().unwrap_or("").replace('\n', " ");
    parsed.description = paragraphs.collect::<Vec<_>>().join("\n\n");
    parsed
}

fn parse_tag(parsed: &mut Docblock, name: &str, payload: &str, span: Span) {
    match name {
        "param" => {
            let (ty, rest) = split_type(payload);
            let (variable, description) = split_word(rest);
            parsed.params.push(ParamDoc {
                ty: ty.to_owned(),
                name: variable.trim_start_matches('$').to_owned(),
                description: description.trim().to_owned(),
                span,
                valid_target: variable.starts_with('$') && variable.len() > 1,
            });
        }
        "return" => {
            let (ty, description) = split_type(payload);
            parsed.returns.push(TypedDoc {
                ty: ty.to_owned(),
                description: description.trim().to_owned(),
                span,
            });
        }
        "throws" => {
            let (ty, description) = split_type(payload);
            parsed.throws.push(TypedDoc {
                ty: ty.to_owned(),
                description: description.trim().to_owned(),
                span,
            });
        }
        "var" => {
            let (ty, rest) = split_type(payload);
            let (variable, description) = split_word(rest);
            parsed.vars.push(VarDoc {
                ty: ty.to_owned(),
                name: variable.trim_start_matches('$').to_owned(),
                description: description.trim().to_owned(),
                span,
                valid_target: variable.starts_with('$') && variable.len() > 1,
            });
        }
        "deprecated" => parsed.deprecated = Some(payload.to_owned()),
        "internal" => parsed.internal = Some(payload.to_owned()),
        "since" => parsed.since = Some(payload.to_owned()),
        "see" => parsed.see.push(payload.to_owned()),
        _ => parsed.unknown.push(UnknownTag {
            name: name.to_owned(),
            span,
        }),
    }
}

fn split_word(text: &str) -> (&str, &str) {
    let text = text.trim_start();
    text.find(char::is_whitespace)
        .map_or((text, ""), |index| (&text[..index], &text[index..]))
}

fn split_type(text: &str) -> (&str, &str) {
    let text = text.trim_start();
    let source = SourceFile::new("<docblock>", text);
    let output = parse_type_prefix(&source, Span::new(0, text.len()));
    if output.ty.is_some() && output.diagnostics.is_empty() {
        let end = output.consumed.end as usize;
        (&text[..end], &text[end..])
    } else {
        split_word(text)
    }
}

#[derive(Clone)]
struct Assignment {
    name: String,
    start: u32,
    scope_end: u32,
}

fn associate_var_hints(
    source: &SourceFile,
    program: &Program,
    tokens: &[Token],
    index: &mut DocblockIndex,
    diagnostics: &mut Vec<LspDiagnostic>,
) {
    let mut assignments = Vec::new();
    collect_assignments(&program.statements, program.span.end, &mut assignments);
    for block in &index.blocks {
        if block.vars.is_empty() {
            continue;
        }
        if block.vars.len() != 1
            || block.params.len() + block.throws.len() + block.returns.len() != 0
        {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                block.span,
                "a local @var block must contain exactly one @var tag".to_owned(),
            ));
            continue;
        }
        let documented = &block.vars[0];
        if documented.ty.is_empty() || !documented.valid_target {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                documented.span,
                "malformed @var; expected `@var Type $name`".to_owned(),
            ));
            continue;
        }
        let assignment = assignments
            .iter()
            .filter(|assignment| block.span.end <= assignment.start)
            .filter(|assignment| !has_token_between(tokens, block.span.end, assignment.start))
            .min_by_key(|assignment| assignment.start);
        let Some(assignment) = assignment else {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                documented.span,
                "@var must immediately precede a local assignment".to_owned(),
            ));
            continue;
        };
        let closest = index
            .blocks
            .iter()
            .filter(|candidate| candidate.span.end <= assignment.start)
            .filter(|candidate| !has_token_between(tokens, candidate.span.end, assignment.start))
            .max_by_key(|candidate| candidate.span.end);
        if closest.is_some_and(|closest| closest.span != block.span) {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                documented.span,
                "@var is superseded by a closer documentation block".to_owned(),
            ));
            continue;
        }
        if assignment.name != documented.name {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                documented.span,
                format!(
                    "@var documents `${}` but the following assignment targets `${}`",
                    documented.name, assignment.name
                ),
            ));
            continue;
        }
        match resolve_documented_type(program, &documented.ty, source, documented.span) {
            Ok(ty) => index.var_hints.push(VarHint {
                name: documented.name.clone(),
                ty: ty.to_string(),
                assignment: assignment.start,
                scope_end: assignment.scope_end,
                description: documented.description.clone(),
            }),
            Err(message) => {
                diagnostics.push(doc_diagnostic(source, "DOC0010", documented.span, message));
            }
        }
    }
    index.var_hints.sort_by_key(|hint| hint.assignment);
}

fn collect_assignments(statements: &[Stmt], scope_end: u32, output: &mut Vec<Assignment>) {
    for statement in statements {
        match &statement.kind {
            StmtKind::Assign { name, .. } => output.push(Assignment {
                name: name.clone(),
                start: statement.span.start,
                scope_end,
            }),
            StmtKind::Function(function) => {
                collect_assignments(&function.body, statement.span.end, output);
            }
            StmtKind::Class(class) => {
                for method in &class.methods {
                    collect_assignments(&method.function.body, method.span.end, output);
                }
            }
            StmtKind::Interface(interface) => {
                for method in &interface.methods {
                    collect_assignments(&method.function.body, method.span.end, output);
                }
            }
            StmtKind::Trait(trait_decl) => {
                for method in &trait_decl.methods {
                    collect_assignments(&method.function.body, method.span.end, output);
                }
            }
            StmtKind::If {
                branches,
                otherwise,
            } => {
                for (_, body) in branches {
                    collect_assignments(body, scope_end, output);
                }
                if let Some(body) = otherwise {
                    collect_assignments(body, scope_end, output);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::Using { body, .. }
            | StmtKind::Block(body) => collect_assignments(body, scope_end, output),
            StmtKind::Try {
                body,
                catches,
                finally,
            } => {
                collect_assignments(body, scope_end, output);
                for catch in catches {
                    collect_assignments(&catch.body, scope_end, output);
                }
                if let Some(body) = finally {
                    collect_assignments(body, scope_end, output);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
struct FunctionSpec {
    parameters: Vec<(String, TypeSyntax)>,
    returns: TypeSyntax,
}

fn validate_declarations(
    source: &SourceFile,
    program: &Program,
    hir: Option<&HirModule>,
    declarations: &mut [Declaration],
    diagnostics: &mut Vec<LspDiagnostic>,
) {
    let mut functions = HashMap::new();
    collect_function_specs(&program.statements, &mut functions);
    for declaration in declarations {
        let Some(documentation) = &mut declaration.documentation else {
            continue;
        };
        if documentation.since.as_ref().is_some_and(String::is_empty) {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                documentation.span,
                "@since requires text".to_owned(),
            ));
        }
        for target in &documentation.see {
            if target.is_empty() {
                diagnostics.push(doc_diagnostic(
                    source,
                    "DOC0001",
                    documentation.span,
                    "@see requires a target".to_owned(),
                ));
            }
        }
        for thrown in &documentation.throws {
            match resolve_documented_type(program, &thrown.ty, source, thrown.span) {
                Err(message) => {
                    diagnostics.push(doc_diagnostic(source, "DOC0008", thrown.span, message));
                }
                Ok(ty) if !throwable_type(&ty, hir) => diagnostics.push(doc_diagnostic(
                    source,
                    "DOC0009",
                    thrown.span,
                    format!("documented throws type `{ty}` is not Throwable"),
                )),
                Ok(_) => {}
            }
        }
        let Some(function) = functions.get(&declaration.name_span.start) else {
            continue;
        };
        validate_params(source, program, function, documentation, diagnostics);
        validate_return(source, program, function, documentation, diagnostics);
    }
}

fn collect_function_specs(statements: &[Stmt], output: &mut HashMap<u32, FunctionSpec>) {
    for statement in statements {
        match &statement.kind {
            StmtKind::Function(function) => insert_function_spec(function, output),
            StmtKind::Class(class) => {
                for method in &class.methods {
                    insert_function_spec(&method.function, output);
                }
            }
            StmtKind::Interface(interface) => {
                for method in &interface.methods {
                    insert_function_spec(&method.function, output);
                }
            }
            StmtKind::Trait(trait_decl) => {
                for method in &trait_decl.methods {
                    insert_function_spec(&method.function, output);
                }
            }
            _ => {}
        }
    }
}

fn insert_function_spec(function: &FunctionDecl, output: &mut HashMap<u32, FunctionSpec>) {
    output.insert(
        function.name_span.start,
        FunctionSpec {
            parameters: function
                .parameters
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                .collect(),
            returns: function.return_type.clone(),
        },
    );
}

fn validate_params(
    source: &SourceFile,
    program: &Program,
    function: &FunctionSpec,
    documentation: &mut Docblock,
    diagnostics: &mut Vec<LspDiagnostic>,
) {
    let mut accepted = Vec::new();
    for parameter in &documentation.params {
        if parameter.ty.is_empty() || !parameter.valid_target {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                parameter.span,
                "malformed @param; expected `@param Type $name`".to_owned(),
            ));
            continue;
        }
        if !function
            .parameters
            .iter()
            .any(|(name, _)| *name == parameter.name)
        {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0003",
                parameter.span,
                format!(
                    "`${}` is not a parameter of this declaration",
                    parameter.name
                ),
            ));
            continue;
        }
        if accepted
            .iter()
            .any(|existing: &ParamDoc| existing.name == parameter.name)
        {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0004",
                parameter.span,
                format!("duplicate documentation for `${}`", parameter.name),
            ));
            continue;
        }
        let documented =
            match resolve_documented_type(program, &parameter.ty, source, parameter.span) {
                Ok(ty) => ty,
                Err(message) => {
                    diagnostics.push(doc_diagnostic(source, "DOC0002", parameter.span, message));
                    continue;
                }
            };
        let source_type = function
            .parameters
            .iter()
            .find(|(name, _)| *name == parameter.name)
            .map(|(_, ty)| resolve_source_type(program, ty.clone()))
            .expect("parameter existence was checked");
        if type_key(&documented) != type_key(&source_type) {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0006",
                parameter.span,
                format!(
                    "documented type `{documented}` does not match source type `{source_type}`"
                ),
            ));
        }
        let mut parameter = parameter.clone();
        parameter.ty = documented.to_string();
        accepted.push(parameter);
    }
    documentation.params = accepted;
}

fn validate_return(
    source: &SourceFile,
    program: &Program,
    function: &FunctionSpec,
    documentation: &mut Docblock,
    diagnostics: &mut Vec<LspDiagnostic>,
) {
    let mut accepted = None;
    for returns in &documentation.returns {
        if returns.ty.is_empty() {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0001",
                returns.span,
                "malformed @return; expected `@return Type`".to_owned(),
            ));
            continue;
        }
        if accepted.is_some() {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0005",
                returns.span,
                "duplicate @return documentation".to_owned(),
            ));
            continue;
        }
        let documented = match resolve_documented_type(program, &returns.ty, source, returns.span) {
            Ok(ty) => ty,
            Err(message) => {
                diagnostics.push(doc_diagnostic(source, "DOC0002", returns.span, message));
                continue;
            }
        };
        let source_type = resolve_source_type(program, function.returns.clone());
        if type_key(&documented) != type_key(&source_type) {
            diagnostics.push(doc_diagnostic(
                source,
                "DOC0007",
                returns.span,
                format!(
                    "documented type `{documented}` does not match source type `{source_type}`"
                ),
            ));
        }
        let mut returns = returns.clone();
        returns.ty = documented.to_string();
        accepted = Some(returns);
    }
    documentation.returns = accepted.into_iter().collect();
}

fn resolve_source_type(program: &Program, mut ty: TypeSyntax) -> TypeSyntax {
    let (namespace, aliases) = resolution_context(program);
    resolve_type_syntax_in_namespace(&mut ty, &namespace, &aliases);
    ty
}

fn resolve_documented_type(
    program: &Program,
    text: &str,
    _source: &SourceFile,
    _span: Span,
) -> Result<TypeSyntax, String> {
    if text.is_empty() {
        return Err("missing THP type".to_owned());
    }
    let fragment = SourceFile::new("<docblock>", text);
    let parsed = parse_type_prefix(&fragment, Span::new(0, text.len()));
    let Some(mut ty) = parsed.ty else {
        return Err("invalid THP type".to_owned());
    };
    if !parsed.diagnostics.is_empty() || parsed.consumed.end as usize != text.len() {
        return Err(format!("invalid THP type `{text}`"));
    }
    let (namespace, aliases) = resolution_context(program);
    resolve_type_syntax_in_namespace(&mut ty, &namespace, &aliases);
    validate_type_arity(program, &ty)?;
    Ok(ty)
}

fn resolution_context(program: &Program) -> (String, BTreeMap<String, String>) {
    let namespace = program
        .namespace
        .as_ref()
        .map_or_else(String::new, |namespace| {
            namespace
                .name
                .as_string()
                .trim_start_matches('\\')
                .to_owned()
        });
    let aliases = program
        .imports
        .iter()
        .filter(|import| import.kind == UseKind::Type)
        .map(|import| {
            (
                import.alias.clone(),
                import
                    .target
                    .as_string()
                    .trim_start_matches('\\')
                    .to_owned(),
            )
        })
        .collect();
    (namespace, aliases)
}

fn validate_type_arity(program: &Program, ty: &TypeSyntax) -> Result<(), String> {
    let (namespace, _) = resolution_context(program);
    let mut arities = BTreeMap::from([
        ("int".to_owned(), 0),
        ("float".to_owned(), 0),
        ("bool".to_owned(), 0),
        ("string".to_owned(), 0),
        ("null".to_owned(), 0),
        ("void".to_owned(), 0),
        ("mixed".to_owned(), 0),
        ("never".to_owned(), 0),
        ("vector".to_owned(), 1),
        ("map".to_owned(), 2),
        ("Traversable".to_owned(), 2),
        ("Iterator".to_owned(), 2),
        ("IteratorAggregate".to_owned(), 2),
    ]);
    for name in [
        "Closeable",
        "ReadableStream",
        "WritableStream",
        "SeekableStream",
        "Throwable",
        "MemoryStream",
        "TempStream",
        "Streams",
        "Files",
        "ReadableFileStream",
        "OpenMode",
        "SeekFrom",
        "Exception",
        "ValueError",
        "IoException",
        "OpenStreamException",
        "ClosedStreamException",
        "UnsupportedStreamOperationException",
        "InvalidStreamUriException",
        "Error",
        "UnhandledMatchError",
    ] {
        arities.insert(name.to_owned(), 0);
    }
    let mut parameters = Vec::new();
    for statement in &program.statements {
        match &statement.kind {
            StmtKind::Class(class) => {
                arities.insert(
                    canonical(&namespace, &class.name),
                    class.type_parameters.len(),
                );
                parameters.extend(
                    class
                        .type_parameters
                        .iter()
                        .map(|parameter| parameter.name.clone()),
                );
            }
            StmtKind::Interface(interface) => {
                arities.insert(
                    canonical(&namespace, &interface.name),
                    interface.type_parameters.len(),
                );
                parameters.extend(
                    interface
                        .type_parameters
                        .iter()
                        .map(|parameter| parameter.name.clone()),
                );
            }
            StmtKind::Trait(trait_decl) => {
                arities.insert(canonical(&namespace, &trait_decl.name), 0);
            }
            _ => {}
        }
    }
    check_type_arity(ty, &arities, &parameters)
}

fn check_type_arity(
    ty: &TypeSyntax,
    arities: &BTreeMap<String, usize>,
    parameters: &[String],
) -> Result<(), String> {
    match &ty.kind {
        TypeSyntaxKind::Named { name, arguments } => {
            if parameters.iter().any(|parameter| parameter == name) {
                if arguments.is_empty() {
                    return Ok(());
                }
                return Err(format!("type parameter `{name}` cannot take arguments"));
            }
            let Some(expected) = arities.get(name) else {
                return Err(format!("unresolved type `{name}`"));
            };
            if *expected != arguments.len() {
                return Err(format!(
                    "type `{name}` expects {expected} generic arguments, found {}",
                    arguments.len()
                ));
            }
            for argument in arguments {
                check_type_arity(argument, arities, parameters)?;
            }
            Ok(())
        }
        TypeSyntaxKind::Nullable(inner) => check_type_arity(inner, arities, parameters),
        TypeSyntaxKind::Union(members) => {
            for member in members {
                check_type_arity(member, arities, parameters)?;
            }
            Ok(())
        }
    }
}

fn type_key(ty: &TypeSyntax) -> String {
    fn members(ty: &TypeSyntax, output: &mut Vec<String>) {
        match &ty.kind {
            TypeSyntaxKind::Nullable(inner) => {
                members(inner, output);
                output.push("null".to_owned());
            }
            TypeSyntaxKind::Union(items) => {
                for item in items {
                    members(item, output);
                }
            }
            TypeSyntaxKind::Named { name, arguments } => output.push(if arguments.is_empty() {
                name.clone()
            } else {
                format!(
                    "{name}<{}>",
                    arguments.iter().map(type_key).collect::<Vec<_>>().join(",")
                )
            }),
        }
    }
    let mut output = Vec::new();
    members(ty, &mut output);
    output.sort();
    output.dedup();
    output.join("|")
}

fn throwable_type(ty: &TypeSyntax, hir: Option<&HirModule>) -> bool {
    match &ty.kind {
        TypeSyntaxKind::Named { name, arguments } if arguments.is_empty() => {
            matches!(name.as_str(), "Throwable" | "Exception" | "Error")
                || hir.is_some_and(|hir| hir.is_nominal_subtype(name, "Throwable"))
        }
        TypeSyntaxKind::Union(members) => members.iter().all(|member| throwable_type(member, hir)),
        TypeSyntaxKind::Nullable(_) | TypeSyntaxKind::Named { .. } => false,
    }
}

fn base_nominal(ty: &str) -> String {
    let ty = ty.trim_start_matches('?');
    let ty = ty.split('|').find(|member| *member != "null").unwrap_or(ty);
    ty.split('<')
        .next()
        .unwrap_or(ty)
        .trim_start_matches('\\')
        .to_owned()
}

fn word_at(text: &str, offset: usize) -> Option<(String, Span, bool)> {
    let offset = offset.min(text.len());
    let bytes = text.as_bytes();
    let mut start = offset;
    while start > 0 && is_word(bytes[start - 1]) {
        start -= 1;
    }
    let variable = start > 0 && bytes[start - 1] == b'$';
    if variable {
        start -= 1;
    }
    let mut end = offset;
    while end < bytes.len() && is_word(bytes[end]) {
        end += 1;
    }
    let raw = &text[start..end];
    let name = raw.trim_start_matches('$');
    (!name.is_empty()).then(|| (name.to_owned(), Span::new(start, end), variable))
}

const fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'\\'
}

fn receiver_before(text: &str, offset: usize) -> Option<String> {
    let prefix = text[..offset.min(text.len())].trim_end();
    let member_start = prefix
        .rfind(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .map_or(0, |index| index + 1);
    let receiver = prefix[..member_start].strip_suffix("->")?.trim_end();
    let end = receiver.len();
    let start = receiver[..end]
        .rfind(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .map_or(0, |index| index + 1);
    let start = start.checked_sub(1)?;
    (receiver.as_bytes().get(start) == Some(&b'$')).then(|| receiver[start + 1..end].to_owned())
}

struct CallSite {
    receiver: Option<String>,
    name: String,
    argument: u32,
}

fn call_before(text: &str, offset: usize) -> Option<CallSite> {
    let offset = offset.min(text.len());
    let prefix = text.get(..offset)?;
    let source = SourceFile::new("<call>", prefix);
    let tokens = lex(&source).tokens;
    let mut depth = 0_u32;
    let mut argument = 0_u32;
    let mut open = None;
    for (index, token) in tokens.iter().enumerate().rev() {
        match token.kind {
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth += 1,
            TokenKind::LParen if depth == 0 => {
                open = Some(index);
                break;
            }
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                depth = depth.saturating_sub(1);
            }
            TokenKind::Comma if depth == 0 => argument += 1,
            _ => {}
        }
    }
    let open = open?;
    let name_end = open.checked_sub(1)?;
    if tokens[name_end].kind != TokenKind::Identifier {
        return None;
    }
    let mut name_start = name_end;
    while name_start >= 2
        && tokens[name_start - 1].kind == TokenKind::NamespaceSeparator
        && tokens[name_start - 2].kind == TokenKind::Identifier
    {
        name_start -= 2;
    }
    if name_start > 0 && tokens[name_start - 1].kind == TokenKind::NamespaceSeparator {
        name_start -= 1;
    }
    let name = prefix[tokens[name_start].span.start as usize..tokens[name_end].span.end as usize]
        .to_owned();
    let receiver = (name_start >= 2
        && tokens[name_start - 1].kind == TokenKind::Arrow
        && tokens[name_start - 2].kind == TokenKind::Variable)
        .then(|| {
            prefix[tokens[name_start - 2].span.range()]
                .trim_start_matches('$')
                .to_owned()
        });
    Some(CallSite {
        receiver,
        name,
        argument,
    })
}

fn valid_identifier(name: &str, variable: bool) -> bool {
    let text = if variable {
        format!("${name}")
    } else {
        name.to_owned()
    };
    let source = SourceFile::new("<rename>", text);
    let lexed = lex(&source);
    let expected = if variable {
        TokenKind::Variable
    } else {
        TokenKind::Identifier
    };
    lexed.diagnostics.is_empty()
        && lexed.tokens.len() == 2
        && lexed.tokens[0].kind == expected
        && lexed.tokens[0].span.range() == (0..source.len())
}

fn semantic_kind(document: &DocumentState, token: &Token) -> (u32, u32) {
    if let Some(declaration) = document
        .declarations
        .iter()
        .find(|declaration| declaration.name_span == token.span)
    {
        let kind = match declaration.kind {
            DeclarationKind::Function => 3,
            DeclarationKind::Class | DeclarationKind::Interface | DeclarationKind::Trait => 1,
            DeclarationKind::Method => 4,
            DeclarationKind::Property => 5,
        };
        let mut modifiers = 1;
        if matches!(
            declaration.kind,
            DeclarationKind::Method | DeclarationKind::Property
        ) && document.tokens.iter().any(|token| {
            token.kind == TokenKind::Static
                && declaration.start <= token.span.start
                && token.span.end <= declaration.name_span.start
        }) {
            modifiers |= 2;
        }
        if declaration.deprecated() {
            modifiers |= 4;
        }
        return (kind, modifiers);
    }
    match token.kind {
        TokenKind::Variable => {
            let parameter =
                document.program.statements.iter().any(|statement| {
                    statement_parameters(statement).any(|span| span == token.span)
                });
            let declaration = parameter
                // ponytail: linear local lookup keeps the 0.1 index simple; cache it if large files profile poorly.
                || document
                    .local_symbols()
                    .iter()
                    .any(|local| local.span == token.span);
            (if parameter { 6 } else { 7 }, u32::from(declaration))
        }
        TokenKind::Identifier => {
            if type_parameter_spans(&document.program).contains(&token.span) {
                return (2, 1);
            }
            if document
                .program
                .namespace
                .as_ref()
                .is_some_and(|namespace| {
                    namespace.name.span.start <= token.span.start
                        && token.span.end <= namespace.name.span.end
                })
            {
                return (0, 0);
            }
            let previous = document
                .tokens
                .iter()
                .take_while(|candidate| candidate.span.start < token.span.start)
                .last()
                .map(|candidate| candidate.kind);
            let next = document
                .tokens
                .iter()
                .find(|candidate| candidate.span.start >= token.span.end)
                .map(|candidate| candidate.kind);
            match (previous, next) {
                (Some(TokenKind::Arrow | TokenKind::DoubleColon), Some(TokenKind::LParen)) => {
                    (4, 0)
                }
                (Some(TokenKind::Arrow | TokenKind::DoubleColon), _) => (5, 0),
                (_, Some(TokenKind::LParen)) => (3, 0),
                (Some(TokenKind::Namespace | TokenKind::Use), _) => (0, 0),
                _ => (1, 0),
            }
        }
        TokenKind::String => (9, 0),
        TokenKind::Integer | TokenKind::Float => (10, 0),
        TokenKind::OpenTag
        | TokenKind::Function
        | TokenKind::Namespace
        | TokenKind::Class
        | TokenKind::Interface
        | TokenKind::Trait
        | TokenKind::Use
        | TokenKind::Const
        | TokenKind::InsteadOf
        | TokenKind::Implements
        | TokenKind::Extends
        | TokenKind::Abstract
        | TokenKind::Final
        | TokenKind::Public
        | TokenKind::Protected
        | TokenKind::Private
        | TokenKind::Static
        | TokenKind::New
        | TokenKind::InstanceOf
        | TokenKind::Throw
        | TokenKind::Try
        | TokenKind::Catch
        | TokenKind::Finally
        | TokenKind::Using
        | TokenKind::Return
        | TokenKind::If
        | TokenKind::ElseIf
        | TokenKind::Else
        | TokenKind::While
        | TokenKind::For
        | TokenKind::Foreach
        | TokenKind::As
        | TokenKind::Break
        | TokenKind::Continue
        | TokenKind::Match
        | TokenKind::Default
        | TokenKind::Echo
        | TokenKind::True
        | TokenKind::False
        | TokenKind::Null => (8, 0),
        _ => (11, 0),
    }
}

fn type_parameter_spans(program: &Program) -> Vec<Span> {
    program
        .statements
        .iter()
        .flat_map(|statement| match &statement.kind {
            StmtKind::Class(class) => class
                .type_parameters
                .iter()
                .map(|parameter| parameter.name_span)
                .collect::<Vec<_>>(),
            StmtKind::Interface(interface) => interface
                .type_parameters
                .iter()
                .map(|parameter| parameter.name_span)
                .collect(),
            _ => Vec::new(),
        })
        .collect()
}

fn statement_parameters(statement: &Stmt) -> Box<dyn Iterator<Item = Span> + '_> {
    match &statement.kind {
        StmtKind::Function(function) => Box::new(
            function
                .parameters
                .iter()
                .map(|parameter| parameter.name_span),
        ),
        StmtKind::Class(class) => Box::new(
            class
                .methods
                .iter()
                .flat_map(|method| &method.function.parameters)
                .map(|parameter| parameter.name_span),
        ),
        StmtKind::Interface(interface) => Box::new(
            interface
                .methods
                .iter()
                .flat_map(|method| &method.function.parameters)
                .map(|parameter| parameter.name_span),
        ),
        StmtKind::Trait(trait_decl) => Box::new(
            trait_decl
                .methods
                .iter()
                .flat_map(|method| &method.function.parameters)
                .map(|parameter| parameter.name_span),
        ),
        _ => Box::new(std::iter::empty()),
    }
}

fn push_semantic_span(
    text: &str,
    span: Span,
    token_type: u32,
    modifiers: u32,
    output: &mut Vec<(Position, u32, u32, u32)>,
) {
    let mut start = span.start as usize;
    for part in text[span.range()].split_inclusive('\n') {
        let content = part.strip_suffix('\n').unwrap_or(part);
        let content = content.strip_suffix('\r').unwrap_or(content);
        if !content.is_empty() {
            output.push((
                position(text, start),
                u32::try_from(content.encode_utf16().count()).unwrap_or(u32::MAX),
                token_type,
                modifiers,
            ));
        }
        start += part.len();
    }
}

fn comment_spans(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if matches!(bytes[index], b'\'' | b'"') {
            let quote = bytes[index];
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index += 2;
                } else if bytes[index] == quote {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
        } else if bytes[index..].starts_with(b"//") {
            let start = index;
            index = text[index..]
                .find('\n')
                .map_or(bytes.len(), |end| index + end);
            spans.push(Span::new(start, index));
        } else if bytes[index..].starts_with(b"/*") {
            let start = index;
            index = text[index + 2..]
                .find("*/")
                .map_or(bytes.len(), |end| index + 2 + end + 2);
            spans.push(Span::new(start, index));
        } else {
            index += 1;
        }
    }
    spans
}

fn format_source(text: &str, tokens: &[Token], tab_size: u32, insert_spaces: bool) -> String {
    let significant = tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
        .collect::<Vec<_>>();
    let mut horizontal = String::with_capacity(text.len());
    let mut cursor = 0;
    for (index, token) in significant.iter().enumerate() {
        let gap = &text[cursor..token.span.start as usize];
        if index == 0 || gap.contains(['\n', '\r']) || !gap.chars().all(char::is_whitespace) {
            horizontal.push_str(gap);
        } else {
            horizontal.push_str(token_gap(significant[index - 1].kind, token.kind));
        }
        horizontal.push_str(&text[token.span.range()]);
        cursor = token.span.end as usize;
    }
    horizontal.push_str(&text[cursor..]);

    let mut depth = 0usize;
    let mut line_depths = Vec::new();
    for line in 0..=text.bytes().filter(|byte| *byte == b'\n').count() {
        let kinds = tokens
            .iter()
            .filter(|token| position(text, token.span.start as usize).line as usize == line)
            .map(|token| token.kind)
            .collect::<Vec<_>>();
        let line_depth = if kinds.first() == Some(&TokenKind::RBrace) {
            depth.saturating_sub(1)
        } else {
            depth
        };
        line_depths.push(line_depth);
        for kind in kinds {
            match kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    let unit = if insert_spaces {
        " ".repeat(tab_size as usize)
    } else {
        "\t".to_owned()
    };
    let protected_lines = comment_spans(text)
        .into_iter()
        .chain(tokens.iter().filter_map(|token| {
            (token.kind == TokenKind::String && text[token.span.range()].contains('\n'))
                .then_some(token.span)
        }))
        .flat_map(|span| {
            let start = position(text, span.start as usize).line;
            let end = position(text, span.end as usize).line;
            start..=end
        })
        .collect::<BTreeSet<_>>();
    let mut output = String::with_capacity(horizontal.len());
    for (line, part) in horizontal.split_inclusive('\n').enumerate() {
        if protected_lines.contains(&u32::try_from(line).unwrap_or(u32::MAX)) {
            output.push_str(part);
            continue;
        }
        let ending = if part.ends_with("\r\n") {
            "\r\n"
        } else if part.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let content = part
            .strip_suffix(ending)
            .unwrap_or(part)
            .trim_end_matches([' ', '\t']);
        if !content.trim().is_empty() {
            output.push_str(&unit.repeat(line_depths.get(line).copied().unwrap_or(0)));
            output.push_str(content.trim_start_matches([' ', '\t']));
        }
        output.push_str(ending);
    }
    output
}

fn token_gap(left: TokenKind, right: TokenKind) -> &'static str {
    if matches!(
        left,
        TokenKind::LParen
            | TokenKind::LBracket
            | TokenKind::NamespaceSeparator
            | TokenKind::Arrow
            | TokenKind::DoubleColon
    ) || matches!(
        right,
        TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::Comma
            | TokenKind::Colon
            | TokenKind::Semicolon
            | TokenKind::NamespaceSeparator
            | TokenKind::Arrow
            | TokenKind::DoubleColon
    ) || right == TokenKind::LParen
        && matches!(
            left,
            TokenKind::Identifier | TokenKind::Variable | TokenKind::RParen | TokenKind::RBracket
        )
        || right == TokenKind::Question
        || left == TokenKind::Question
    {
        ""
    } else {
        " "
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::thread;

    use lsp_server::{Message, Notification, Request, RequestId};
    use lsp_types::{Position, Uri};
    use thp_bytecode::encode as encode_bytecode;
    use thp_compiler::compile_text;

    use super::{
        DocumentState, ProjectConfig, Server, capabilities, format_source, parse_docblock,
        path_to_uri, position, run, uri_to_path, validate_autoload,
    };
    use thp_diagnostics::{SourceFile, Span};
    use thp_syntax::DocblockSpan;

    #[test]
    fn normalizes_docblocks_and_continuation_lines() {
        let text = "/**\r\n * Summary.\r\n *\r\n * Details.\r\n * @param map<string, vector<User>> $items first\r\n *   continued\r\n * @unknown retained\r\n */";
        let source = SourceFile::new("test.thp", text);
        let docs = parse_docblock(
            &source,
            DocblockSpan {
                span: Span::new(0, text.len()),
            },
        );
        assert_eq!(docs.summary, "Summary.");
        assert_eq!(docs.description, "Details.");
        assert_eq!(docs.params[0].ty, "map<string, vector<User>>");
        assert_eq!(docs.params[0].description, "first continued");
        assert_eq!(docs.unknown.len(), 1);
    }

    #[test]
    fn validates_docs_and_keeps_var_hints_detached() {
        let source = r#"<?thp
class User { public function label(): string { return "user"; } }
class Other {}
/**
 * @param int $missing
 * @return int
 * @return string
 */
function load(): Other { return new Other(); }
function inspect(): void {
    /** @var User $value local docs */
    $value = load();
    $value;
}
"#;
        let uri = Uri::from_str("file:///test.thp").unwrap();
        let state = DocumentState::new(&uri, 1, source.to_owned());
        let codes = state
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.code.as_ref())
            .map(|code| format!("{code:?}"))
            .collect::<Vec<_>>();
        assert!(codes.iter().any(|code| code.contains("DOC0003")));
        assert!(codes.iter().any(|code| code.contains("DOC0005")));
        assert!(codes.iter().any(|code| code.contains("DOC0007")));
        let use_offset = source.rfind("$value;").unwrap() + 2;
        let hover = state.variable_hover("value", use_offset).unwrap();
        assert!(hover.contains("User $value"));
        assert!(hover.contains("Compiler type: `Other`"));
        assert!(hover.contains("local docs"));
    }

    #[test]
    fn docblocks_and_var_hints_do_not_change_compilation() {
        let documented = r#"<?thp
class User {}
function make(): User { return new User(); }
function inspect(): void {
    /** @var User $value */
    $value = make();
}
echo "ok";
"#;
        let start = documented.find("/**").unwrap();
        let end = documented[start..].find("*/").unwrap() + start + 2;
        let mut plain = documented.to_owned();
        plain.replace_range(start..end, &" ".repeat(end - start));
        let with_docs = compile_text("test.thp", documented);
        let without_docs = compile_text("test.thp", plain);
        assert_eq!(with_docs.tokens, without_docs.tokens);
        assert_eq!(with_docs.ast, without_docs.ast);
        assert_eq!(
            format!("{:?}", with_docs.hir),
            format!("{:?}", without_docs.hir)
        );
        assert_eq!(
            format!("{:?}", with_docs.mir),
            format!("{:?}", without_docs.mir)
        );
        assert_eq!(
            encode_bytecode(with_docs.bytecode.as_ref().unwrap()),
            encode_bytecode(without_docs.bytecode.as_ref().unwrap())
        );
        assert_eq!(
            thp_vm::execute(
                with_docs.bytecode.as_ref().unwrap(),
                thp_vm::Limits::default()
            )
            .unwrap()
            .output,
            thp_vm::execute(
                without_docs.bytecode.as_ref().unwrap(),
                thp_vm::Limits::default()
            )
            .unwrap()
            .output
        );
    }

    #[test]
    fn var_overlay_drives_member_completion_and_signature_help() {
        let source = r#"<?thp
class User {
    /**
     * Greets a user.
     * @param string $name Display name.
     * @param int $times Repeat count.
     */
    public function greet(string $name, int $times): string { return $name; }
}
class Other {}
function load(): Other { return new Other(); }
function inspect(): void {
    /** @var User $value */
    $value = load();
    $value->greet("Ada", 1);
}
"#;
        let uri = Uri::from_str("file:///members.thp").unwrap();
        let document = DocumentState::new(&uri, 1, source.to_owned());
        let mut server = Server::default();
        server.documents.insert(uri.clone(), document);

        let completion_offset = source.find("greet(\"Ada").unwrap() + 3;
        let completion = server
            .completion(
                serde_json::from_value(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": position(source, completion_offset),
                    "context": { "triggerKind": 1 }
                }))
                .unwrap(),
            )
            .unwrap();
        let lsp_types::CompletionResponse::Array(items) = completion else {
            panic!("expected completion array");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "greet");
        assert!(items[0].documentation.is_some());

        let signature_offset = source.find(", 1").unwrap() + 2;
        let signature = server
            .signature_help(
                serde_json::from_value(serde_json::json!({
                    "textDocument": { "uri": "file:///members.thp" },
                    "position": position(source, signature_offset)
                }))
                .unwrap(),
            )
            .unwrap();
        assert_eq!(signature.active_parameter, Some(1));
        assert_eq!(
            signature.signatures[0].parameters.as_ref().unwrap()[1].documentation,
            Some(lsp_types::Documentation::String("Repeat count.".to_owned()))
        );
    }

    #[test]
    fn signature_help_resolves_namespaces_and_ignores_string_commas() {
        let source = r#"<?thp
namespace App;
function greet(string $name, int $times): string { return $name; }
function inspect(): void { greet("Ada, Lovelace", 1); }
"#;
        let uri = Uri::from_str("file:///signature.thp").unwrap();
        let document = DocumentState::new(&uri, 1, source.to_owned());
        let offset = source.rfind(", 1").unwrap() + 2;
        let mut server = Server::default();
        server.documents.insert(uri, document);
        let signature = server
            .signature_help(
                serde_json::from_value(serde_json::json!({
                    "textDocument": { "uri": "file:///signature.thp" },
                    "position": position(source, offset)
                }))
                .unwrap(),
            )
            .unwrap();
        assert_eq!(signature.active_parameter, Some(1));
        assert!(
            signature.signatures[0]
                .label
                .starts_with("function App\\greet(")
        );
    }

    #[test]
    fn advertises_and_serves_hover_over_memory_transport() {
        let serialized = serde_json::to_value(capabilities()).unwrap();
        assert_eq!(serialized["textDocumentSync"], 1);
        assert_eq!(
            serialized["signatureHelpProvider"]["triggerCharacters"][1],
            ","
        );
        assert_eq!(serialized["definitionProvider"], true);
        assert_eq!(serialized["referencesProvider"], true);
        assert_eq!(serialized["renameProvider"]["prepareProvider"], true);
        assert_eq!(serialized["semanticTokensProvider"]["full"], true);
        assert_eq!(serialized["documentFormattingProvider"], true);

        let (server, client) = lsp_server::Connection::memory();
        let thread = thread::spawn(move || run(&server));
        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(1),
                method: "initialize".to_owned(),
                params: serde_json::json!({ "capabilities": {} }),
            }))
            .unwrap();
        let Message::Response(initialized) = client.receiver.recv().unwrap() else {
            panic!("expected initialize response");
        };
        let value = initialized.response_result.unwrap();
        assert_eq!(value["serverInfo"]["name"], "thp-lsp");
        client
            .sender
            .send(Message::Notification(Notification::new(
                "initialized".to_owned(),
                serde_json::json!({}),
            )))
            .unwrap();
        client
            .sender
            .send(Message::Notification(Notification::new(
                "textDocument/didOpen".to_owned(),
                serde_json::json!({
                    "textDocument": {
                        "uri": "file:///test.thp",
                        "languageId": "thp",
                        "version": 1,
                        "text": "<?thp\n/** Greets. */\nfunction greet(string $name): string { return $name; }\n"
                    }
                }),
            )))
            .unwrap();
        assert!(matches!(
            client.receiver.recv().unwrap(),
            Message::Notification(_)
        ));
        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(2),
                method: "textDocument/hover".to_owned(),
                params: serde_json::json!({
                    "textDocument": { "uri": "file:///test.thp" },
                    "position": { "line": 2, "character": 10 }
                }),
            }))
            .unwrap();
        let Message::Response(hover) = client.receiver.recv().unwrap() else {
            panic!("expected hover response");
        };
        assert!(
            hover
                .response_result
                .unwrap()
                .to_string()
                .contains("Greets.")
        );

        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(30),
                method: "textDocument/hover".to_owned(),
                params: serde_json::json!({ "bad": true }),
            }))
            .unwrap();
        let Message::Response(malformed) = client.receiver.recv().unwrap() else {
            panic!("expected malformed-request response");
        };
        assert_eq!(
            malformed.response_result.unwrap_err().code,
            lsp_server::ErrorCode::InvalidParams as i32
        );

        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(3),
                method: "shutdown".to_owned(),
                params: serde_json::Value::Null,
            }))
            .unwrap();
        assert!(matches!(
            client.receiver.recv().unwrap(),
            Message::Response(_)
        ));
        client
            .sender
            .send(Message::Notification(Notification::new(
                "exit".to_owned(),
                serde_json::Value::Null,
            )))
            .unwrap();
        assert!(thread.join().unwrap().is_ok());
    }

    #[test]
    fn decodes_and_encodes_unicode_file_uris() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("space śource.thp");
        std::fs::write(&path, "<?thp\n").unwrap();
        let uri = path_to_uri(&path).unwrap();
        assert!(uri.as_str().contains("%20"));
        assert_eq!(
            uri_to_path(&uri).unwrap(),
            std::fs::canonicalize(path).unwrap()
        );
        assert!(uri_to_path(&"file:///bad%FF".parse().unwrap()).is_err());
    }

    #[test]
    fn formatting_preserves_source_shape_and_is_idempotent() {
        let source = "<?thp\r\n/**\r\n * keep  docs\r\n */\r\nfunction  greet( string $name ): string {\r\n// keep  comment\r\n  return  \"a  b\" . $name;   \r\n}\r\n\r\n";
        let file = SourceFile::new("format.thp", source);
        let parsed = thp_syntax::parse(&file);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let formatted = format_source(source, &parsed.tokens, 2, true);
        assert!(formatted.contains("/**\r\n * keep  docs\r\n */"));
        assert!(formatted.contains("// keep  comment"));
        assert!(formatted.contains("\"a  b\""));
        assert_eq!(
            formatted.matches("\r\n").count(),
            source.matches("\r\n").count()
        );
        assert!(formatted.ends_with("\r\n\r\n"));
        let reparsed = thp_syntax::parse(&SourceFile::new("format.thp", formatted.clone()));
        assert_eq!(
            format_source(&formatted, &reparsed.tokens, 2, true),
            formatted
        );
    }

    fn server_with_source(source: &str) -> (Server, Uri) {
        let uri = Uri::from_str("file:///regression.thp").unwrap();
        let mut server = Server::default();
        server
            .documents
            .insert(uri.clone(), DocumentState::new(&uri, 1, source.to_owned()));
        (server, uri)
    }

    fn renamed_source(server: &Server, uri: &Uri, offset: usize, name: &str) -> String {
        let document = &server.documents[uri];
        let edit = server
            .rename(
                serde_json::from_value(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": document.position(offset),
                    "newName": name
                }))
                .unwrap(),
            )
            .unwrap_or_else(|_| panic!("rename failed"))
            .unwrap();
        let Some(lsp_types::DocumentChanges::Edits(edits)) = edit.document_changes else {
            panic!("expected document edits");
        };
        assert_eq!(edits.len(), 1);
        assert_eq!(&edits[0].text_document.uri, uri);
        let mut text = document.source.text().to_owned();
        for edit in edits[0].edits.iter().rev() {
            let lsp_types::OneOf::Left(edit) = edit else {
                panic!("expected text edit");
            };
            text.replace_range(
                document.offset(edit.range.start).unwrap()
                    ..document.offset(edit.range.end).unwrap(),
                &edit.new_text,
            );
        }
        text
    }

    #[test]
    fn property_rename_updates_members_without_renaming_locals() {
        let source = "<?thp\nclass User { public string $name; }\nfunction read(User $user): string { return $user->name; }\n$name = \"local\";\n";
        let (server, uri) = server_with_source(source);
        assert!(server.documents[&uri].diagnostics.is_empty());
        let expected = source
            .replacen("$name", "$title", 1)
            .replace("->name", "->title");
        for offset in [
            source.find("$name").unwrap() + 1,
            source.find("->name").unwrap() + 3,
        ] {
            assert_eq!(renamed_source(&server, &uri, offset, "title"), expected);
        }
        let local = source.rfind("$name").unwrap() + 1;
        assert_eq!(
            renamed_source(&server, &uri, local, "title"),
            source.replace("$name =", "$title =")
        );
        let source =
            "<?thp\nclass User { public string $name; }\necho User::$name;\n$name = \"local\";\n";
        let (server, uri) = server_with_source(source);
        // Static properties are rejected by the THP parser, so they cannot be renamed.
        assert!(
            server
                .symbol_at(&uri, position(source, source.find("::$name").unwrap() + 3))
                .is_none()
        );
    }

    #[test]
    fn local_rename_declines_when_syntax_errors_destroy_scope_information() {
        let valid = "<?thp\nfunction first(int $x): int { return $x; }\nfunction second(int $x): int { return $x; }\n";
        let source = format!("{valid}echo ;\n");
        let (server, uri) = server_with_source(&source);
        let offset = source.find("$x").unwrap() + 1;
        let params = serde_json::json!({"textDocument": {"uri": uri}, "position": position(&source, offset)});
        assert!(
            server
                .prepare_rename(&serde_json::from_value(params.clone()).unwrap())
                .is_none()
        );
        let mut params = params;
        params["newName"] = serde_json::json!("value");
        assert!(
            server
                .rename(serde_json::from_value(params).unwrap())
                .is_err()
        );
        let (server, uri) = server_with_source(valid);
        assert_eq!(
            renamed_source(&server, &uri, offset, "value"),
            valid.replacen("$x", "$value", 2)
        );
    }

    #[test]
    fn rename_rejects_keywords_but_allows_keyword_variable_names() {
        let source = "<?thp\nfunction greet(): int { return 1; }\necho greet();\n$value = 1;\n";
        let (server, uri) = server_with_source(source);
        for name in ["while", "function", "true", "null", "bad-name", "", "1name"] {
            let params = serde_json::json!({
                "textDocument": {"uri": uri},
                "position": position(source, source.find("greet").unwrap()),
                "newName": name
            });
            assert!(
                server
                    .rename(serde_json::from_value(params).unwrap())
                    .is_err(),
                "{name}"
            );
        }
        assert_eq!(
            renamed_source(&server, &uri, source.find("greet").unwrap(), "welcome"),
            source.replace("greet", "welcome")
        );
        assert_eq!(
            renamed_source(&server, &uri, source.find("$value").unwrap() + 1, "while"),
            source.replace("$value", "$while")
        );
    }

    #[test]
    fn semantic_tokens_do_not_treat_quoted_comment_markers_as_comments() {
        for literal in [
            "'https://example.com'",
            "'/* text */'",
            r"'it\'s // text'",
            "\"https://example.com\"",
        ] {
            let source = format!("<?thp\necho {literal}; // real comment\n");
            let (_, uri) = server_with_source(&source);
            let document = DocumentState::new(&uri, 1, source.clone());
            assert!(document.syntax_valid);
            let tokens = document.semantic_tokens();
            assert_eq!(
                tokens.iter().filter(|token| token.token_type == 12).count(),
                1
            );
            let mut line = 0;
            let mut start = 0;
            let mut end = 0;
            for token in tokens {
                if token.delta_line == 0 {
                    start += token.delta_start;
                    assert!(start >= end, "overlapping tokens for {source}");
                } else {
                    line += token.delta_line;
                    start = token.delta_start;
                }
                end = start + token.length;
                if token.token_type == 12 {
                    assert_eq!(
                        position(&source, source.find("// real").unwrap()),
                        lsp_types::Position::new(line, start)
                    );
                }
            }
        }
    }

    #[test]
    fn semantic_tokens_do_not_extend_past_unix_lines() {
        let source = "<?thp\n/* first\n second */\necho \"line\n two\";\n";
        let (_, uri) = server_with_source(source);
        let document = DocumentState::new(&uri, 1, source.to_owned());
        assert!(document.syntax_valid);
        let mut position = Position::new(0, 0);
        for token in document.semantic_tokens() {
            position.line += token.delta_line;
            position.character = if token.delta_line == 0 {
                position.character + token.delta_start
            } else {
                token.delta_start
            };
            let line = source
                .split('\n')
                .nth(usize::try_from(position.line).unwrap())
                .unwrap();
            let remaining = u32::try_from(
                line[usize::try_from(position.character).unwrap()..]
                    .encode_utf16()
                    .count(),
            )
            .unwrap();
            assert!(
                token.length <= remaining,
                "token at {position:?} with length {} exceeds {line:?}",
                token.length,
            );
        }
    }

    #[test]
    fn symbol_operations_stay_within_their_project() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for project in ["one", "two"] {
            let folder = root.join(project);
            std::fs::create_dir(&folder).unwrap();
            std::fs::write(folder.join("thp.toml"), "").unwrap();
        }
        std::fs::create_dir(root.join("shared")).unwrap();
        std::fs::write(
            root.join("one/thp.toml"),
            "[autoload]\n\"App\\\\\" = \"../shared\"\n",
        )
        .unwrap();
        let source = "<?thp\nnamespace App;\nfunction greet(): int { return 1; }\n";
        std::fs::write(root.join("shared/Functions.thp"), source).unwrap();
        std::fs::write(root.join("one/call.thp"), "<?thp\necho \\App\\greet();\n").unwrap();
        std::fs::write(root.join("two/call.thp"), "<?thp\necho greet();\n").unwrap();
        let (connection, _client) = lsp_server::Connection::memory();
        let mut server = Server::default();
        for project in ["one", "two"] {
            let path = std::fs::canonicalize(root.join(project)).unwrap();
            server
                .workspaces
                .insert(path.clone(), path_to_uri(&path).unwrap());
        }
        server.rescan(&connection).unwrap();
        let declaration = path_to_uri(&root.join("shared/Functions.thp")).unwrap();
        let local_call = path_to_uri(&root.join("one/call.thp")).unwrap();
        let other_call = path_to_uri(&root.join("two/call.thp")).unwrap();
        assert!(server.documents[&declaration].diagnostics.is_empty());
        assert!(server.documents[&local_call].diagnostics.is_empty());
        let symbol = server
            .symbol_at(
                &declaration,
                position(source, source.find("greet").unwrap()),
            )
            .unwrap();
        let locations = server.occurrences(&symbol);
        assert_eq!(locations.len(), 2, "{locations:?}");
        assert!(
            locations
                .iter()
                .all(|location| location.uri == declaration || location.uri == local_call)
        );
        assert!(
            server
                .symbol_at(&other_call, lsp_types::Position::new(1, 6))
                .is_none()
        );
        for uri in [
            other_call.clone(),
            Uri::from_str("file:///standalone.thp").unwrap(),
        ] {
            if !server.documents.contains_key(&uri) {
                server.documents.insert(
                    uri.clone(),
                    DocumentState::new(&uri, 1, "<?thp\necho greet();\n".to_owned()),
                );
            }
            let params = serde_json::json!({"textDocument": {"uri": uri}, "position": {"line": 1, "character": 6}});
            assert!(
                server
                    .hover(serde_json::from_value(params.clone()).unwrap())
                    .is_none()
            );
            let completion = server
                .completion(serde_json::from_value(params).unwrap())
                .unwrap();
            assert_eq!(
                serde_json::to_value(completion).unwrap(),
                serde_json::json!([])
            );
        }
    }

    #[test]
    fn shared_autoload_modules_remain_visible_to_every_workspace() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir(root.join("shared")).unwrap();
        std::fs::write(
            root.join("shared/Functions.thp"),
            "<?thp\nnamespace App;\nfunction greet(): int { return 1; }\n",
        )
        .unwrap();
        for project in ["one", "two"] {
            let folder = root.join(project);
            std::fs::create_dir(&folder).unwrap();
            std::fs::write(
                folder.join("thp.toml"),
                "[autoload]\n\"App\\\\\" = \"../shared\"\n",
            )
            .unwrap();
            std::fs::write(folder.join("main.thp"), "<?thp\necho \\App\\greet();\n").unwrap();
        }
        let (connection, _client) = lsp_server::Connection::memory();
        let mut server = Server::default();
        for project in ["one", "two"] {
            let path = std::fs::canonicalize(root.join(project)).unwrap();
            server
                .workspaces
                .insert(path.clone(), path_to_uri(&path).unwrap());
        }
        server.rescan(&connection).unwrap();
        for project in ["one", "two"] {
            let uri = path_to_uri(&root.join(project).join("main.thp")).unwrap();
            assert!(
                server
                    .symbol_at(&uri, lsp_types::Position::new(1, 11))
                    .is_some(),
                "{project} lost its shared module"
            );
        }
    }

    #[test]
    fn hover_on_unknown_names_returns_none() {
        let source = "<?thp\nmissing();\n";
        let (server, uri) = server_with_source(source);
        assert!(
            server
                .hover(
                    serde_json::from_value(serde_json::json!({
                        "textDocument": { "uri": uri },
                        "position": position(source, source.find("missing").unwrap())
                    }))
                    .unwrap()
                )
                .is_none()
        );
    }

    #[test]
    fn signature_help_on_unknown_functions_returns_none() {
        let source = "<?thp\nmissing();\n";
        let (server, uri) = server_with_source(source);
        assert!(
            server
                .signature_help(
                    serde_json::from_value(serde_json::json!({
                        "textDocument": { "uri": uri },
                        "position": position(source, source.find("();").unwrap() + 1)
                    }))
                    .unwrap()
                )
                .is_none()
        );
    }

    #[test]
    fn signature_help_on_unknown_methods_returns_none() {
        let source = "<?thp\nclass User {}\n$user = new User();\n$user->missing();\n";
        let (server, uri) = server_with_source(source);
        assert!(
            server
                .signature_help(
                    serde_json::from_value(serde_json::json!({
                        "textDocument": { "uri": uri },
                        "position": position(source, source.rfind("();").unwrap() + 1)
                    }))
                    .unwrap()
                )
                .is_none()
        );
    }

    #[test]
    fn formatting_preserves_multiline_string_bytes() {
        for newline in ["\n", "\r\n"] {
            for quote in ['\'', '"'] {
                let literal = format!(
                    "{quote}first  \t{newline}  \t{newline}    second{newline}  last{quote}"
                );
                let source = format!(
                    "<?thp{newline}function demo(): void {{{newline}echo  {literal};{newline}}}"
                );
                let file = SourceFile::new("format.thp", source.clone());
                let parsed = thp_syntax::parse(&file);
                assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
                let formatted = format_source(&source, &parsed.tokens, 2, true);
                assert!(formatted.contains(&literal), "{formatted:?}");
                let reparsed = thp_syntax::parse(&SourceFile::new("format.thp", formatted.clone()));
                assert!(reparsed.diagnostics.is_empty());
                assert_eq!(
                    format_source(&formatted, &reparsed.tokens, 2, true),
                    formatted
                );
            }
        }
    }

    #[test]
    fn rename_preserves_qualified_name_prefixes() {
        let source = "<?thp\nnamespace App;\nfunction greet(): int { return 1; }\necho \\App\\greet();\necho greet();\n";
        let (server, uri) = server_with_source(source);
        let params = |offset| {
            serde_json::json!({
                "textDocument": { "uri": uri },
                "position": position(source, offset)
            })
        };
        let declaration = source.find("greet").unwrap();
        let call = source.find("\\App\\greet").unwrap();
        let prepared = server
            .prepare_rename(&serde_json::from_value(params(call + 5)).unwrap())
            .unwrap();
        assert_eq!(
            serde_json::to_value(prepared).unwrap()["range"],
            serde_json::json!({
                "start": position(source, call + 5),
                "end": position(source, call + 10)
            })
        );
        assert!(
            server
                .prepare_rename(&serde_json::from_value(params(call + 1)).unwrap())
                .is_none()
        );
        for offset in [declaration, call + 5] {
            let mut rename_params = params(offset);
            rename_params["newName"] = serde_json::json!("welcome");
            let edit = server
                .rename(serde_json::from_value(rename_params).unwrap())
                .unwrap_or_else(|_| panic!("rename request failed"))
                .unwrap();
            let Some(lsp_types::DocumentChanges::Edits(documents)) = edit.document_changes else {
                panic!("expected document edits");
            };
            assert_eq!(documents.len(), 1);
            assert_eq!(documents[0].text_document.version, Some(1));
            assert_eq!(documents[0].edits.len(), 3);
            let mut renamed = source.to_owned();
            for edit in documents[0].edits.iter().rev() {
                let lsp_types::OneOf::Left(edit) = edit else {
                    panic!("expected text edit");
                };
                let document = &server.documents[&uri];
                let start = document.offset(edit.range.start).unwrap();
                let end = document.offset(edit.range.end).unwrap();
                renamed.replace_range(start..end, &edit.new_text);
            }
            assert_eq!(renamed, source.replace("greet", "welcome"));
        }
    }

    #[test]
    fn validates_autoload_directories_and_scans_workspace_overlays() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir_all(root.join("src/nested")).unwrap();
        std::fs::create_dir(root.join("lib")).unwrap();
        std::fs::write(
            root.join("thp.toml"),
            "[autoload]\n\"App\\\\\" = [\"src\", \"lib\"]\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/Service.thp"),
            "<?thp\nnamespace App;\nclass Service {}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("lib/Other.thp"),
            "<?thp\nnamespace App;\nclass Other {}\n",
        )
        .unwrap();
        std::fs::write(root.join("main.thp"), "<?thp\necho \"disk\";\n").unwrap();
        let config = ProjectConfig::load(root).unwrap();
        assert!(validate_autoload(root, &config).is_empty());

        let root_uri = path_to_uri(root).unwrap();
        let (connection, _client) = lsp_server::Connection::memory();
        let mut server = Server::default();
        server.workspaces.insert(root.to_path_buf(), root_uri);
        server.rescan(&connection).unwrap();
        assert_eq!(server.documents.len(), 3);

        let entry = std::fs::canonicalize(root.join("main.thp")).unwrap();
        let entry_uri = path_to_uri(&entry).unwrap();
        server.open.insert(
            entry.clone(),
            (
                entry_uri.clone(),
                2,
                "<?thp\necho \"overlay\";\n".to_owned(),
            ),
        );
        server.rescan(&connection).unwrap();
        assert_eq!(
            server.documents[&entry_uri].source.text(),
            "<?thp\necho \"overlay\";\n"
        );

        let overlap = ProjectConfig::parse(
            root.join("thp.toml"),
            "[autoload]\n\"App\\\\\" = [\"src\", \"src/nested\"]\n",
        )
        .unwrap();
        assert_eq!(validate_autoload(root, &overlap).len(), 1);
        let missing = ProjectConfig::parse(
            root.join("thp.toml"),
            "[autoload]\n\"App\\\\\" = \"missing\"\n",
        )
        .unwrap();
        assert_eq!(validate_autoload(root, &missing).len(), 1);
    }
}
