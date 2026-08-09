//! LSP Backend implementation
//!
//! This module implements the Language Server Protocol backend using tower-lsp.

use crate::call_hierarchy::CallHierarchyProvider;
use crate::code_action::{CodeActionProvider, TruncateVariableNameData};
use crate::code_lens::CodeLensProvider;
use crate::completion::CompletionProvider;
use crate::definition::DefinitionProvider;
use crate::document::DocumentManager;
use crate::document_highlight::DocumentHighlightProvider;
use crate::folding::FoldingRangeProvider;
use crate::hover::HoverProvider;
use crate::inlay_hint::InlayHintProvider;
use crate::references::ReferencesProvider;
use crate::rename::RenameProvider;
use crate::semantic_tokens::SemanticTokensProvider;
use crate::signature::SignatureProvider;
use crate::symbols;
use crate::workspace_symbol::WorkspaceSymbolProvider;
use crbasic_parser::SemanticError;
use crbasic_parser::lexer::Scanner;
use crbasic_parser::lexer::token::Position;
use crbasic_parser::semantic::SemanticErrorKind;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// The CRBasic Language Server backend
pub struct CRBasicLanguageServer {
    client: Client,
    document_manager: Arc<RwLock<DocumentManager>>,
}

impl CRBasicLanguageServer {
    /// Creates a new CRBasic Language Server
    ///
    /// # Arguments
    /// * `client` - The LSP client handle for sending notifications
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_manager: Arc::new(RwLock::new(DocumentManager::new())),
        }
    }

    /// Creates a new LSP service for testing
    ///
    /// This method is primarily intended for integration tests.
    /// It returns a service and client socket pair that can be used
    /// to test LSP protocol communication.
    ///
    /// # Returns
    /// A tuple of (LspService, ClientSocket) for testing
    ///
    /// # Example
    /// ```no_run
    /// use crbasic_lsp::CRBasicLanguageServer;
    ///
    /// # async fn example() {
    /// let (service, _socket) = CRBasicLanguageServer::new_service();
    /// // Use service for testing...
    /// # }
    /// ```
    pub fn new_service() -> (tower_lsp::LspService<Self>, tower_lsp::ClientSocket) {
        tower_lsp::LspService::build(Self::new).finish()
    }

    /// Converts semantic errors to LSP diagnostics
    fn semantic_errors_to_diagnostics(errors: &[SemanticError]) -> Vec<Diagnostic> {
        errors
            .iter()
            .map(|error| {
                let severity = match error.severity {
                    crbasic_parser::semantic::ErrorSeverity::Error => DiagnosticSeverity::ERROR,
                    crbasic_parser::semantic::ErrorSeverity::Warning => DiagnosticSeverity::WARNING,
                };

                let start_pos = error.span.start;
                let end_pos = error.span.end;
                let (code, data) = Self::code_action_data(&error.kind);

                Diagnostic {
                    range: Range {
                        start: Self::position_to_lsp(start_pos),
                        end: Self::position_to_lsp(end_pos),
                    },
                    severity: Some(severity),
                    code,
                    code_description: None,
                    source: Some("crbasic-lsp".to_string()),
                    message: error.message.clone(),
                    related_information: None,
                    tags: None,
                    data,
                }
            })
            .collect()
    }

    /// Builds the diagnostic `code` and `data` needed to offer a quick fix
    /// for a semantic error, if one exists for its kind
    ///
    /// `data` round-trips through the client back to
    /// [`code_action`](Self::code_action), so the quick fix never has to
    /// re-derive what the analyzer already computed.
    fn code_action_data(
        kind: &SemanticErrorKind,
    ) -> (Option<NumberOrString>, Option<serde_json::Value>) {
        let (code, payload) = match kind {
            SemanticErrorKind::MaxLengthExceeded {
                variable_name,
                max_length,
            } => (
                "truncate-variable-name",
                TruncateVariableNameData {
                    variable_name: variable_name.clone(),
                    target_length: *max_length,
                },
            ),
            SemanticErrorKind::RecommendedLengthExceeded {
                variable_name,
                recommended_length,
            } => (
                "truncate-variable-name",
                TruncateVariableNameData {
                    variable_name: variable_name.clone(),
                    target_length: *recommended_length,
                },
            ),
            SemanticErrorKind::TruncationCollision { .. } => return (None, None),
        };

        let data = serde_json::to_value(payload).ok();
        (Some(NumberOrString::String(code.to_string())), data)
    }

    /// Converts parser Position (1-indexed) to LSP Position (0-indexed)
    fn position_to_lsp(pos: Position) -> tower_lsp::lsp_types::Position {
        tower_lsp::lsp_types::Position {
            line: pos.line.saturating_sub(1) as u32,
            character: pos.column.saturating_sub(1) as u32,
        }
    }

    /// Analyzes a document and publishes diagnostics
    async fn analyze_and_publish_diagnostics(&self, uri: Url) {
        let mut manager = self.document_manager.write().await;

        if let Some(doc) = manager.get_mut(&uri) {
            if let Err(e) = doc.analyze() {
                let diagnostic = Diagnostic {
                    range: Range {
                        start: tower_lsp::lsp_types::Position {
                            line: 0,
                            character: 0,
                        },
                        end: tower_lsp::lsp_types::Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("crbasic-lsp".to_string()),
                    message: e,
                    related_information: None,
                    tags: None,
                    data: None,
                };

                self.client
                    .publish_diagnostics(uri, vec![diagnostic], None)
                    .await;
                return;
            }

            let diagnostics = Self::semantic_errors_to_diagnostics(doc.errors());
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CRBasicLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                references_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                document_highlight_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                document_symbol_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        ..Default::default()
                    },
                )),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                workspace_symbol_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
                rename_provider: Some(tower_lsp::lsp_types::OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensProvider::legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "crbasic-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "CRBasic LSP Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        {
            let mut manager = self.document_manager.write().await;
            manager.open(uri.clone(), text, version);
        }

        self.analyze_and_publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get the new text (full sync mode)
        if let Some(change) = params.content_changes.first() {
            {
                let mut manager = self.document_manager.write().await;
                manager.update(&uri, change.text.clone(), version);
            }

            self.analyze_and_publish_diagnostics(uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let mut manager = self.document_manager.write().await;
        manager.close(&uri);
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri)
            && let Some(ast) = &doc.ast
        {
            let symbols = symbols::extract_document_symbols(ast);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;
        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri)
            && let Some(ast) = &doc.ast
        {
            return Ok(Some(FoldingRangeProvider::get_folding_ranges(ast)));
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let manager = self.document_manager.read().await;

        let results = WorkspaceSymbolProvider::search(manager.analyzed_documents(), &params.query);

        Ok(Some(results))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;
        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri)
            && let Some(ast) = &doc.ast
        {
            return Ok(Some(InlayHintProvider::get_inlay_hints(ast, params.range)));
        }

        Ok(None)
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri)
            && let Some(ast) = &doc.ast
        {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            return Ok(Some(CodeLensProvider::get_code_lenses(ast, &tokens, &uri)));
        }

        Ok(None)
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri)
            && let Some(ast) = &doc.ast
        {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            if let Some(item) = CallHierarchyProvider::prepare(&tokens, ast, &uri, position) {
                return Ok(Some(vec![item]));
            }
        }

        Ok(None)
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        let manager = self.document_manager.read().await;

        Ok(Some(CallHierarchyProvider::incoming_calls(
            &params.item,
            manager.analyzed_documents(),
        )))
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        let manager = self.document_manager.read().await;

        Ok(Some(CallHierarchyProvider::outgoing_calls(
            &params.item,
            manager.analyzed_documents(),
        )))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            return Ok(HoverProvider::get_hover_at_position(&tokens, position));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

        let manager = self.document_manager.read().await;

        let ast = manager.get(&uri).and_then(|doc| doc.ast.as_ref());

        let items = CompletionProvider::get_all_completions(ast);

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let lines: Vec<&str> = doc.text.lines().collect();
            let mut offset = 0usize;

            for (i, line) in lines.iter().enumerate() {
                if i == position.line as usize {
                    offset += position.character as usize;
                    break;
                }
                offset += line.len() + 1; // +1 for newline
            }

            if let Some(func_name) = SignatureProvider::extract_function_name(&doc.text, offset) {
                let active_param =
                    SignatureProvider::count_parameters_before_cursor(&doc.text, offset);
                return Ok(SignatureProvider::get_signature_help(
                    &func_name,
                    active_param,
                ));
            }
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            if let Some(ast) = &doc.ast {
                let definitions = DefinitionProvider::extract_definitions(ast);

                if let Some(location) =
                    DefinitionProvider::get_definition(&tokens, position, &definitions, uri)
                {
                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            return Ok(ReferencesProvider::get_references(
                &tokens,
                position,
                uri,
                include_declaration,
            ));
        }

        Ok(None)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            return Ok(DocumentHighlightProvider::get_document_highlights(
                &tokens, position,
            ));
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let diagnostics = params.context.diagnostics;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &uri);
            if !actions.is_empty() {
                return Ok(Some(actions));
            }
        }

        Ok(None)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            if let Some(range) = RenameProvider::prepare_rename(&tokens, position) {
                return Ok(Some(PrepareRenameResponse::Range(range)));
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            return RenameProvider::get_rename_edit(&tokens, position, &new_name, uri)
                .map_err(tower_lsp::jsonrpc::Error::invalid_params);
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri)
            && let Some(ast) = &doc.ast
        {
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            let data = SemanticTokensProvider::get_semantic_tokens(ast, &tokens);
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::lexer::token::Span;
    use crbasic_parser::semantic::{ErrorSeverity, SemanticErrorKind};

    #[test]
    fn converts_error_severity_to_diagnostic_severity() {
        let error = SemanticError {
            message: "Test error".to_string(),
            span: Span::new(Position::new(1, 1), Position::new(1, 10)),
            severity: ErrorSeverity::Error,
            kind: SemanticErrorKind::TruncationCollision {
                variable_name: "Test".to_string(),
            },
        };

        let diagnostics = CRBasicLanguageServer::semantic_errors_to_diagnostics(&[error]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[0].message, "Test error");
    }

    #[test]
    fn converts_warning_severity_to_diagnostic_severity() {
        let error = SemanticError {
            message: "Test warning".to_string(),
            span: Span::new(Position::new(1, 1), Position::new(1, 10)),
            severity: ErrorSeverity::Warning,
            kind: SemanticErrorKind::TruncationCollision {
                variable_name: "Test".to_string(),
            },
        };

        let diagnostics = CRBasicLanguageServer::semantic_errors_to_diagnostics(&[error]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn embeds_quick_fix_data_for_max_length_exceeded() {
        let error = SemanticError {
            message: "Test error".to_string(),
            span: Span::new(Position::new(1, 1), Position::new(1, 10)),
            severity: ErrorSeverity::Error,
            kind: SemanticErrorKind::MaxLengthExceeded {
                variable_name: "Temperature_Sensor_1".to_string(),
                max_length: 16,
            },
        };

        let diagnostics = CRBasicLanguageServer::semantic_errors_to_diagnostics(&[error]);

        assert_eq!(
            diagnostics[0].code,
            Some(NumberOrString::String("truncate-variable-name".to_string()))
        );
        let data: TruncateVariableNameData =
            serde_json::from_value(diagnostics[0].data.clone().expect("Should have data"))
                .expect("Should deserialize");
        assert_eq!(data.variable_name, "Temperature_Sensor_1");
        assert_eq!(data.target_length, 16);
    }

    #[test]
    fn embeds_quick_fix_data_for_recommended_length_exceeded() {
        let error = SemanticError {
            message: "Test warning".to_string(),
            span: Span::new(Position::new(1, 1), Position::new(1, 10)),
            severity: ErrorSeverity::Warning,
            kind: SemanticErrorKind::RecommendedLengthExceeded {
                variable_name: "Temperature_1".to_string(),
                recommended_length: 12,
            },
        };

        let diagnostics = CRBasicLanguageServer::semantic_errors_to_diagnostics(&[error]);

        let data: TruncateVariableNameData =
            serde_json::from_value(diagnostics[0].data.clone().expect("Should have data"))
                .expect("Should deserialize");
        assert_eq!(data.variable_name, "Temperature_1");
        assert_eq!(data.target_length, 12);
    }

    #[test]
    fn omits_quick_fix_data_for_truncation_collision() {
        let error = SemanticError {
            message: "Test error".to_string(),
            span: Span::new(Position::new(1, 1), Position::new(1, 10)),
            severity: ErrorSeverity::Error,
            kind: SemanticErrorKind::TruncationCollision {
                variable_name: "Temperature_S1".to_string(),
            },
        };

        let diagnostics = CRBasicLanguageServer::semantic_errors_to_diagnostics(&[error]);

        assert_eq!(diagnostics[0].code, None);
        assert_eq!(diagnostics[0].data, None);
    }

    #[test]
    fn converts_parser_position_to_lsp_position() {
        // Parser Position is 1-indexed
        let parser_pos = Position::new(5, 10);

        // LSP Position should be 0-indexed
        let lsp_pos = CRBasicLanguageServer::position_to_lsp(parser_pos);

        assert_eq!(lsp_pos.line, 4);
        assert_eq!(lsp_pos.character, 9);
    }

    #[test]
    fn handles_zero_position_gracefully() {
        let parser_pos = Position::new(1, 1);
        let lsp_pos = CRBasicLanguageServer::position_to_lsp(parser_pos);

        assert_eq!(lsp_pos.line, 0);
        assert_eq!(lsp_pos.character, 0);
    }
}
