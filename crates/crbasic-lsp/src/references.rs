//! Find all references provider for CRBasic LSP
//!
//! This module provides the Find All References functionality, allowing users
//! to find all occurrences of a symbol throughout the document.

use crbasic_parser::lexer::token::{Span, Token, TokenKind};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

/// Provides Find All References functionality
pub struct ReferencesProvider;

impl ReferencesProvider {
    /// Finds the identifier at the given position in the token stream
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    ///
    /// # Returns
    /// The identifier name if found at position
    pub fn find_identifier_at_position(tokens: &[Token<'_>], position: Position) -> Option<String> {
        let line = position.line as usize + 1;
        let column = position.character as usize + 1;

        tokens.iter().find_map(|token| {
            let TokenKind::Identifier(_) = &token.kind else {
                return None;
            };

            let start = &token.span.start;
            let end = &token.span.end;

            // Check if position is within token span (half-open interval)
            if line < start.line || line > end.line {
                return None;
            }
            if line == start.line && column < start.column {
                return None;
            }
            if line == end.line && column >= end.column {
                return None;
            }

            Some(token.lexeme.to_string())
        })
    }

    /// Finds all references to a symbol in the token stream
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `symbol_name` - The name of the symbol to find
    ///
    /// # Returns
    /// All token spans where the symbol appears
    pub fn find_all_references(tokens: &[Token<'_>], symbol_name: &str) -> Vec<Span> {
        tokens
            .iter()
            .filter_map(|token| {
                // Match both Identifier and Keyword tokens (for function calls)
                match &token.kind {
                    TokenKind::Identifier(name) if *name == symbol_name => Some(token.span),
                    _ if token.lexeme == symbol_name => {
                        // Also check lexeme for case-insensitive matches
                        if matches!(&token.kind, TokenKind::Identifier(_)) {
                            Some(token.span)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .collect()
    }

    /// Gets all reference locations for a symbol at the given position
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    /// * `uri` - The document URI
    /// * `include_declaration` - Whether to include the declaration in results
    ///
    /// # Returns
    /// All locations where the symbol is referenced
    pub fn get_references(
        tokens: &[Token],
        position: Position,
        uri: Url,
        _include_declaration: bool,
    ) -> Option<Vec<Location>> {
        let symbol_name = Self::find_identifier_at_position(tokens, position)?;
        let spans = Self::find_all_references(tokens, &symbol_name);

        if spans.is_empty() {
            return None;
        }

        let locations = spans
            .into_iter()
            .map(|span| Location {
                uri: uri.clone(),
                range: Self::span_to_range(span),
            })
            .collect();

        Some(locations)
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
    use crbasic_parser::lexer::token::Position as ParserPosition;

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

    mod find_all_references {
        use super::*;

        #[test]
        fn finds_single_reference() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 1)];

            let refs = ReferencesProvider::find_all_references(&tokens, "Temp_C");

            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0].start.line, 1);
            assert_eq!(refs[0].start.column, 1);
        }

        #[test]
        fn finds_multiple_references() {
            let tokens = vec![
                create_identifier_token("Temp_C", 1, 8),
                create_identifier_token("Temp_C", 3, 1),
                create_identifier_token("Temp_C", 5, 10),
            ];

            let refs = ReferencesProvider::find_all_references(&tokens, "Temp_C");

            assert_eq!(refs.len(), 3);
        }

        #[test]
        fn ignores_different_identifiers() {
            let tokens = vec![
                create_identifier_token("Temp_C", 1, 1),
                create_identifier_token("Humidity", 2, 1),
                create_identifier_token("Temp_C", 3, 1),
            ];

            let refs = ReferencesProvider::find_all_references(&tokens, "Temp_C");

            assert_eq!(refs.len(), 2);
        }

        #[test]
        fn returns_empty_for_unknown_symbol() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 1)];

            let refs = ReferencesProvider::find_all_references(&tokens, "Unknown");

            assert!(refs.is_empty());
        }
    }

    mod get_references {
        use super::*;

        fn setup_test() -> (Vec<Token<'static>>, Url) {
            let tokens = vec![
                create_identifier_token("Temp_C", 1, 8),
                create_identifier_token("Temp_C", 3, 1),
                create_identifier_token("Temp_C", 5, 10),
            ];
            let uri = Url::parse("file:///test.cr1").expect("Valid URL");
            (tokens, uri)
        }

        #[test]
        fn returns_all_references_including_declaration() {
            let (tokens, uri) = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = ReferencesProvider::get_references(&tokens, position, uri, true);

            assert!(result.is_some());
            let locations = result.expect("Should return locations");
            assert_eq!(locations.len(), 3);
        }

        #[test]
        fn returns_none_when_not_on_identifier() {
            let (_, uri) = setup_test();
            let tokens = vec![];
            let position = Position {
                line: 0,
                character: 0,
            };

            let result = ReferencesProvider::get_references(&tokens, position, uri, true);

            assert!(result.is_none());
        }

        #[test]
        fn locations_have_correct_uri() {
            let (tokens, uri) = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = ReferencesProvider::get_references(&tokens, position, uri.clone(), true);

            let locations = result.expect("Should return locations");
            for loc in locations {
                assert_eq!(loc.uri, uri);
            }
        }

        #[test]
        fn locations_have_correct_ranges() {
            let (tokens, uri) = setup_test();
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = ReferencesProvider::get_references(&tokens, position, uri, true);

            let locations = result.expect("Should return locations");
            // First reference: line 1, col 8 (0-indexed: line 0, char 7)
            assert_eq!(locations[0].range.start.line, 0);
            assert_eq!(locations[0].range.start.character, 7);
            // Second reference: line 3, col 1 (0-indexed: line 2, char 0)
            assert_eq!(locations[1].range.start.line, 2);
            assert_eq!(locations[1].range.start.character, 0);
        }
    }
}
