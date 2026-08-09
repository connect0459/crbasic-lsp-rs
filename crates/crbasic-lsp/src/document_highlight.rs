//! Document highlight provider for CRBasic LSP
//!
//! This module provides the Document Highlight functionality, allowing editors
//! to visually mark every occurrence of the symbol under the cursor within the
//! current document.

use crate::references::ReferencesProvider;
use crbasic_parser::lexer::token::{Span, Token};
use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Range};

/// Provides Document Highlight functionality
pub struct DocumentHighlightProvider;

impl DocumentHighlightProvider {
    /// Gets all highlight ranges for the symbol at the given position
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    ///
    /// # Returns
    /// A highlight for every occurrence of the symbol under the cursor,
    /// or `None` if the cursor is not on an identifier
    pub fn get_document_highlights(
        tokens: &[Token],
        position: Position,
    ) -> Option<Vec<DocumentHighlight>> {
        let symbol_name = ReferencesProvider::find_identifier_at_position(tokens, position)?;
        let spans = ReferencesProvider::find_all_references(tokens, &symbol_name);

        if spans.is_empty() {
            return None;
        }

        Some(
            spans
                .into_iter()
                .map(|span| DocumentHighlight {
                    range: Self::span_to_range(span),
                    kind: Some(DocumentHighlightKind::TEXT),
                })
                .collect(),
        )
    }

    /// Converts parser Span to LSP Range
    fn span_to_range(span: Span) -> Range {
        Range {
            start: Position {
                line: span.start.line.saturating_sub(1) as u32,
                character: span.start.column.saturating_sub(1) as u32,
            },
            end: Position {
                line: span.end.line.saturating_sub(1) as u32,
                character: span.end.column.saturating_sub(1) as u32,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::lexer::token::{Position as ParserPosition, TokenKind};

    fn create_span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(
            ParserPosition::new(start_line, start_col),
            ParserPosition::new(end_line, end_col),
        )
    }

    fn create_identifier_token(name: &str, line: usize, start_col: usize) -> Token<'_> {
        let end_col = start_col + name.len();
        Token {
            kind: TokenKind::Identifier(name),
            lexeme: name,
            span: create_span(line, start_col, line, end_col),
        }
    }

    mod get_document_highlights {
        use super::*;

        fn setup_test() -> Vec<Token<'static>> {
            vec![
                create_identifier_token("Temp_C", 1, 8),
                create_identifier_token("Temp_C", 3, 1),
                create_identifier_token("Temp_C", 5, 10),
            ]
        }

        #[test]
        fn highlights_every_occurrence_of_the_symbol_under_the_cursor() {
            let tokens = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = DocumentHighlightProvider::get_document_highlights(&tokens, position);

            let highlights = result.expect("Should return highlights");
            assert_eq!(highlights.len(), 3);
        }

        #[test]
        fn returns_none_when_cursor_is_not_on_an_identifier() {
            let tokens: Vec<Token<'static>> = vec![];
            let position = Position {
                line: 0,
                character: 0,
            };

            let result = DocumentHighlightProvider::get_document_highlights(&tokens, position);

            assert!(result.is_none());
        }

        #[test]
        fn each_highlight_uses_the_text_kind() {
            let tokens = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = DocumentHighlightProvider::get_document_highlights(&tokens, position);

            let highlights = result.expect("Should return highlights");
            for highlight in highlights {
                assert_eq!(highlight.kind, Some(DocumentHighlightKind::TEXT));
            }
        }

        #[test]
        fn highlight_ranges_match_the_symbol_spans() {
            let tokens = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = DocumentHighlightProvider::get_document_highlights(&tokens, position);

            let highlights = result.expect("Should return highlights");
            // First occurrence: line 1, col 8 (0-indexed: line 0, char 7)
            assert_eq!(highlights[0].range.start.line, 0);
            assert_eq!(highlights[0].range.start.character, 7);
            // Second occurrence: line 3, col 1 (0-indexed: line 2, char 0)
            assert_eq!(highlights[1].range.start.line, 2);
            assert_eq!(highlights[1].range.start.character, 0);
        }

        #[test]
        fn ignores_identifiers_with_a_different_name() {
            let tokens = vec![
                create_identifier_token("Temp_C", 1, 1),
                create_identifier_token("Humidity", 2, 1),
            ];
            let position = Position {
                line: 0,
                character: 1,
            };

            let result = DocumentHighlightProvider::get_document_highlights(&tokens, position);

            let highlights = result.expect("Should return highlights");
            assert_eq!(highlights.len(), 1);
        }
    }
}
