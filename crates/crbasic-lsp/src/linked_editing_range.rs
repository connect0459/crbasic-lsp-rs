//! Linked editing range provider for CRBasic LSP
//!
//! This module provides the Linked Editing Range functionality, letting
//! editors update every occurrence of a symbol in the current document as
//! the user types, using the same identifier-occurrence lookup as
//! `document_highlight`/`rename`.

use crate::references::ReferencesProvider;
use crbasic_parser::lexer::token::{Span, Token};
use tower_lsp_server::ls_types::{LinkedEditingRanges, Position, Range};

/// A CRBasic identifier: an ASCII letter or underscore followed by any
/// number of alphanumeric characters or underscores.
const IDENTIFIER_WORD_PATTERN: &str = "^[A-Za-z_][A-Za-z0-9_]*$";

/// Provides Linked Editing Range functionality
pub struct LinkedEditingRangeProvider;

impl LinkedEditingRangeProvider {
    /// Gets every range that must be edited together with the symbol at the
    /// given position
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    ///
    /// # Returns
    /// The ranges of every occurrence of the symbol under the cursor, or
    /// `None` if the cursor is not on an identifier
    pub fn get_linked_editing_ranges(
        tokens: &[Token],
        position: Position,
    ) -> Option<LinkedEditingRanges> {
        let symbol_name = ReferencesProvider::find_identifier_at_position(tokens, position)?;
        let spans = ReferencesProvider::find_all_references(tokens, &symbol_name);

        if spans.is_empty() {
            return None;
        }

        Some(LinkedEditingRanges {
            ranges: spans.into_iter().map(Self::span_to_range).collect(),
            word_pattern: Some(IDENTIFIER_WORD_PATTERN.to_string()),
        })
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

    mod get_linked_editing_ranges {
        use super::*;

        fn setup_test() -> Vec<Token<'static>> {
            vec![
                create_identifier_token("Temp_C", 1, 8),
                create_identifier_token("Temp_C", 3, 1),
                create_identifier_token("Temp_C", 5, 10),
            ]
        }

        #[test]
        fn links_every_occurrence_of_the_symbol_under_the_cursor() {
            let tokens = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = LinkedEditingRangeProvider::get_linked_editing_ranges(&tokens, position);

            let linked = result.expect("Should return linked editing ranges");
            assert_eq!(linked.ranges.len(), 3);
        }

        #[test]
        fn returns_none_when_cursor_is_not_on_an_identifier() {
            let tokens: Vec<Token<'static>> = vec![];
            let position = Position {
                line: 0,
                character: 0,
            };

            let result = LinkedEditingRangeProvider::get_linked_editing_ranges(&tokens, position);

            assert!(result.is_none());
        }

        #[test]
        fn includes_a_crbasic_identifier_word_pattern() {
            let tokens = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = LinkedEditingRangeProvider::get_linked_editing_ranges(&tokens, position)
                .expect("Should return linked editing ranges");

            assert_eq!(
                result.word_pattern.as_deref(),
                Some(IDENTIFIER_WORD_PATTERN)
            );
        }

        #[test]
        fn ranges_match_the_symbol_spans() {
            let tokens = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = LinkedEditingRangeProvider::get_linked_editing_ranges(&tokens, position)
                .expect("Should return linked editing ranges");

            // First occurrence: line 1, col 8 (0-indexed: line 0, char 7)
            assert_eq!(result.ranges[0].start.line, 0);
            assert_eq!(result.ranges[0].start.character, 7);
            // Second occurrence: line 3, col 1 (0-indexed: line 2, char 0)
            assert_eq!(result.ranges[1].start.line, 2);
            assert_eq!(result.ranges[1].start.character, 0);
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

            let result = LinkedEditingRangeProvider::get_linked_editing_ranges(&tokens, position)
                .expect("Should return linked editing ranges");

            assert_eq!(result.ranges.len(), 1);
        }
    }
}
