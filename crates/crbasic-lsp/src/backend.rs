//! LSP Backend implementation
//!
//! This module implements the Language Server Protocol backend using tower-lsp.

use crate::completion::CompletionProvider;
use crate::definition::DefinitionProvider;
use crate::document::DocumentManager;
use crate::hover::HoverProvider;
use crate::references::ReferencesProvider;
use crate::rename::RenameProvider;
use crate::signature::SignatureProvider;
use crate::symbols;
use crbasic_parser::SemanticError;
use crbasic_parser::lexer::Scanner;
use crbasic_parser::lexer::token::Position;
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

                // Convert parser Position to LSP Position (0-indexed)
                let start_pos = error.span.start;
                let end_pos = error.span.end;

                Diagnostic {
                    range: Range {
                        start: Self::position_to_lsp(start_pos),
                        end: Self::position_to_lsp(end_pos),
                    },
                    severity: Some(severity),
                    code: None,
                    code_description: None,
                    source: Some("crbasic-lsp".to_string()),
                    message: error.message.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect()
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
            // Run analysis
            if let Err(e) = doc.analyze() {
                // Parse error - publish as diagnostic
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

            // Publish semantic diagnostics
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
                document_symbol_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
                rename_provider: Some(tower_lsp::lsp_types::OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
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

        // Analyze and publish diagnostics
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

            // Analyze and publish diagnostics
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

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            // Tokenize the document
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            // Get hover information at position
            return Ok(HoverProvider::get_hover_at_position(&tokens, position));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

        let manager = self.document_manager.read().await;

        // Get AST if available for user-defined completions
        let ast = manager.get(&uri).and_then(|doc| doc.ast.as_ref());

        let items = CompletionProvider::get_all_completions(ast);

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let manager = self.document_manager.read().await;

        if let Some(doc) = manager.get(&uri) {
            // Calculate cursor offset from position
            let lines: Vec<&str> = doc.text.lines().collect();
            let mut offset = 0usize;

            for (i, line) in lines.iter().enumerate() {
                if i == position.line as usize {
                    offset += position.character as usize;
                    break;
                }
                offset += line.len() + 1; // +1 for newline
            }

            // Extract function name and count parameters
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
            // Tokenize the document
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            // Extract definitions from AST
            if let Some(ast) = &doc.ast {
                let definitions = DefinitionProvider::extract_definitions(ast);

                // Get definition location
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
            // Tokenize the document
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            // Find all references
            return Ok(ReferencesProvider::get_references(
                &tokens,
                position,
                uri,
                include_declaration,
            ));
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
            // Tokenize the document
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
            // Tokenize the document
            let mut scanner = Scanner::new(&doc.text);
            let tokens = scanner.scan_tokens();

            return RenameProvider::get_rename_edit(&tokens, position, &new_name, uri)
                .map_err(tower_lsp::jsonrpc::Error::invalid_params);
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::lexer::token::Span;
    use crbasic_parser::semantic::ErrorSeverity;

    #[test]
    fn converts_error_severity_to_diagnostic_severity() {
        let error = SemanticError {
            message: "Test error".to_string(),
            span: Span::new(Position::new(1, 1), Position::new(1, 10)),
            severity: ErrorSeverity::Error,
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
        };

        let diagnostics = CRBasicLanguageServer::semantic_errors_to_diagnostics(&[error]);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::WARNING));
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
        // Edge case: Position at (1, 1) should convert to (0, 0)
        let parser_pos = Position::new(1, 1);
        let lsp_pos = CRBasicLanguageServer::position_to_lsp(parser_pos);

        assert_eq!(lsp_pos.line, 0);
        assert_eq!(lsp_pos.character, 0);
    }
}
