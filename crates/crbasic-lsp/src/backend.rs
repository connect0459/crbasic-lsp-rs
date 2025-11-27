//! LSP Backend implementation
//!
//! This module implements the Language Server Protocol backend using tower-lsp.

use crate::completion::CompletionProvider;
use crate::document::DocumentManager;
use crate::hover::HoverProvider;
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(tower_lsp::lsp_types::OneOf::Left(true)),
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
            let mut scanner = Scanner::new(doc.text.clone());
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
