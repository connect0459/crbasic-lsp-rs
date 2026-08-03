//! Rename provider for CRBasic LSP
//!
//! This module provides the Rename Symbol functionality, allowing users to
//! rename a variable, function, or subroutine and have all its occurrences
//! in the document updated consistently.

use crbasic_parser::lexer::token::{Span, Token, TokenKind};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range, TextEdit, Url, WorkspaceEdit};

/// Provides Rename Symbol functionality
pub struct RenameProvider;

impl RenameProvider {
    /// Checks whether `name` is a syntactically valid CRBasic identifier
    ///
    /// A valid identifier starts with an ASCII letter or underscore and
    /// contains only ASCII alphanumeric characters or underscores, matching
    /// the lexer's identifier grammar (see `Scanner::scan_identifier`).
    pub fn is_valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Finds the identifier token at the given position, if any
    fn find_identifier_token_at_position<'a>(
        tokens: &'a [Token<'a>],
        position: Position,
    ) -> Option<&'a Token<'a>> {
        let line = position.line as usize + 1;
        let column = position.character as usize + 1;

        tokens.iter().find(|token| {
            let TokenKind::Identifier(_) = &token.kind else {
                return false;
            };

            let start = &token.span.start;
            let end = &token.span.end;

            // Check if position is within token span (half-open interval)
            if line < start.line || line > end.line {
                return false;
            }
            if line == start.line && column < start.column {
                return false;
            }
            if line == end.line && column >= end.column {
                return false;
            }

            true
        })
    }

    /// Returns the range of the identifier at the given position
    ///
    /// Used to answer `textDocument/prepareRename`, letting the client
    /// highlight and validate the symbol before the user provides a new name.
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    pub fn prepare_rename(tokens: &[Token<'_>], position: Position) -> Option<Range> {
        let token = Self::find_identifier_token_at_position(tokens, position)?;
        Some(Self::span_to_range(token.span))
    }

    /// Builds a workspace edit that renames every occurrence of the symbol
    /// at the given position to `new_name`
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    /// * `new_name` - The requested new identifier name
    /// * `uri` - The document URI
    ///
    /// # Returns
    /// * `Ok(Some(edit))` - The symbol was found and the edit was built
    /// * `Ok(None)` - No identifier was found at `position`
    /// * `Err(message)` - `new_name` is not a valid CRBasic identifier
    pub fn get_rename_edit(
        tokens: &[Token<'_>],
        position: Position,
        new_name: &str,
        uri: Url,
    ) -> Result<Option<WorkspaceEdit>, String> {
        if !Self::is_valid_identifier(new_name) {
            return Err(format!("\"{new_name}\" is not a valid CRBasic identifier"));
        }

        let Some(token) = Self::find_identifier_token_at_position(tokens, position) else {
            return Ok(None);
        };
        let TokenKind::Identifier(symbol_name) = &token.kind else {
            return Ok(None);
        };

        let edits: Vec<TextEdit> = tokens
            .iter()
            .filter_map(|t| match &t.kind {
                TokenKind::Identifier(name) if name == symbol_name => Some(TextEdit {
                    range: Self::span_to_range(t.span),
                    new_text: new_name.to_string(),
                }),
                _ => None,
            })
            .collect();

        let mut changes = HashMap::new();
        changes.insert(uri, edits);

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
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

    mod is_valid_identifier {
        use super::*;

        #[test]
        fn validates_identifier_syntax() {
            let cases = [
                ("Temp_C", true),
                ("_temp", true),
                ("temp123", true),
                ("", false),
                ("123abc", false),
                ("temp-c", false),
                ("temp c", false),
                ("temp.c", false),
            ];

            for (input, expected) in cases {
                assert_eq!(
                    RenameProvider::is_valid_identifier(input),
                    expected,
                    "is_valid_identifier({input:?}) should be {expected}"
                );
            }
        }
    }

    mod prepare_rename {
        use super::*;

        #[test]
        fn returns_range_of_identifier_at_cursor() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 8)];
            let position = Position {
                line: 0,
                character: 8,
            };

            let range = RenameProvider::prepare_rename(&tokens, position);

            assert_eq!(range, Some(RenameProvider::span_to_range(tokens[0].span)));
        }

        #[test]
        fn returns_none_when_cursor_not_on_identifier() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 8)];
            let position = Position {
                line: 5,
                character: 0,
            };

            let range = RenameProvider::prepare_rename(&tokens, position);

            assert_eq!(range, None);
        }
    }

    mod get_rename_edit {
        use super::*;

        fn test_uri() -> Url {
            Url::parse("file:///test.cr1").expect("Valid URL")
        }

        #[test]
        fn renames_all_occurrences_of_symbol() {
            let tokens = vec![
                create_identifier_token("Temp_C", 1, 8),    // Declaration
                create_identifier_token("Temp_C", 3, 1),    // Reference 1
                create_identifier_token("Humidity", 3, 10), // Unrelated identifier
                create_identifier_token("Temp_C", 5, 10),   // Reference 2
            ];
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = RenameProvider::get_rename_edit(&tokens, position, "Temp_F", test_uri())
                .expect("Valid new name should not error");

            let edit = result.expect("Should find the symbol to rename");
            let changes = edit.changes.expect("Should contain changes");
            let edits = changes.get(&test_uri()).expect("Should have edits for URI");

            assert_eq!(edits.len(), 3);
            assert!(edits.iter().all(|e| e.new_text == "Temp_F"));
        }

        #[test]
        fn returns_none_when_cursor_not_on_identifier() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 8)];
            let position = Position {
                line: 5,
                character: 0,
            };

            let result = RenameProvider::get_rename_edit(&tokens, position, "Temp_F", test_uri())
                .expect("Valid new name should not error");

            assert_eq!(result, None);
        }

        #[test]
        fn rejects_invalid_new_name() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 8)];
            let position = Position {
                line: 0,
                character: 8,
            };

            let result = RenameProvider::get_rename_edit(&tokens, position, "Temp F", test_uri());

            assert!(result.is_err());
        }
    }
}
