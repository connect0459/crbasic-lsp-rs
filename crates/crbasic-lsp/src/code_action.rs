//! Code action provider for CRBasic LSP
//!
//! Offers quick fixes for semantic diagnostics that carry actionable data.
//! A diagnostic becomes actionable by embedding a [`TruncateVariableNameData`]
//! payload in its `data` field when first published (see
//! `backend::CRBasicLanguageServer::semantic_errors_to_diagnostics`); this
//! provider only reads that payload back rather than re-deriving it from the
//! diagnostic's message text.

use crate::references::ReferencesProvider;
use crbasic_parser::lexer::token::{Span, Token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp_server::ls_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Position, Range, TextEdit, Uri,
    WorkspaceEdit,
};

/// Structured payload embedded in [`Diagnostic::data`] for the "truncate an
/// over-length variable name" quick fix
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TruncateVariableNameData {
    /// The over-length variable name to rename
    pub variable_name: String,
    /// The length to truncate the name to
    pub target_length: usize,
}

/// Provides Code Action (quick fix) functionality
pub struct CodeActionProvider;

impl CodeActionProvider {
    /// Builds one quick fix per diagnostic that carries a
    /// [`TruncateVariableNameData`] payload
    ///
    /// # Arguments
    /// * `tokens` - The token stream for the document
    /// * `diagnostics` - The diagnostics the client is requesting actions for
    /// * `uri` - The document URI
    ///
    /// # Returns
    /// A quick fix for every actionable diagnostic; diagnostics without a
    /// recognized payload are skipped
    pub fn get_code_actions(
        tokens: &[Token],
        diagnostics: &[Diagnostic],
        uri: &Uri,
    ) -> Vec<CodeActionOrCommand> {
        diagnostics
            .iter()
            .filter_map(|diagnostic| Self::truncate_variable_name_action(tokens, diagnostic, uri))
            .collect()
    }

    /// Builds a "truncate this variable name" quick fix for a single
    /// diagnostic, if it carries a [`TruncateVariableNameData`] payload
    fn truncate_variable_name_action(
        tokens: &[Token],
        diagnostic: &Diagnostic,
        uri: &Uri,
    ) -> Option<CodeActionOrCommand> {
        let data: TruncateVariableNameData =
            serde_json::from_value(diagnostic.data.clone()?).ok()?;

        let new_name: String = data
            .variable_name
            .chars()
            .take(data.target_length)
            .collect();
        if new_name.is_empty() || new_name == data.variable_name {
            return None;
        }

        let spans = ReferencesProvider::find_all_references(tokens, &data.variable_name);
        if spans.is_empty() {
            return None;
        }

        let edits: Vec<TextEdit> = spans
            .into_iter()
            .map(|span| TextEdit {
                range: Self::span_to_range(span),
                new_text: new_name.clone(),
            })
            .collect();

        let mut changes = HashMap::new();
        changes.insert(uri.clone(), edits);

        Some(CodeActionOrCommand::CodeAction(CodeAction {
            title: format!(
                "Truncate '{}' to '{}' ({} characters)",
                data.variable_name, new_name, data.target_length
            ),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            is_preferred: Some(true),
            ..Default::default()
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

    fn truncate_diagnostic(variable_name: &str, target_length: usize) -> Diagnostic {
        let data = TruncateVariableNameData {
            variable_name: variable_name.to_string(),
            target_length,
        };
        Diagnostic {
            data: Some(serde_json::to_value(data).expect("Should serialize")),
            ..Default::default()
        }
    }

    fn test_uri() -> Uri {
        "file:///test.cr1".parse::<Uri>().expect("Valid URL")
    }

    mod get_code_actions {
        use super::*;

        #[test]
        fn builds_a_quick_fix_for_a_truncatable_diagnostic() {
            let tokens = vec![
                create_identifier_token("Temperature_Sensor_1", 1, 8),
                create_identifier_token("Temperature_Sensor_1", 3, 1),
            ];
            let diagnostics = vec![truncate_diagnostic("Temperature_Sensor_1", 16)];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            assert_eq!(actions.len(), 1);
        }

        #[test]
        fn renames_every_occurrence_to_the_truncated_name() {
            let tokens = vec![
                create_identifier_token("Temperature_Sensor_1", 1, 8),
                create_identifier_token("Temperature_Sensor_1", 3, 1),
            ];
            let diagnostics = vec![truncate_diagnostic("Temperature_Sensor_1", 16)];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
                panic!("Expected a CodeAction");
            };
            let edit = action.edit.as_ref().expect("Should have an edit");
            let changes = edit.changes.as_ref().expect("Should have changes");
            let edits = changes.get(&test_uri()).expect("Should target the URI");

            assert_eq!(edits.len(), 2);
            assert!(edits.iter().all(|e| e.new_text == "Temperature_Sens"));
        }

        #[test]
        fn ignores_diagnostics_without_data() {
            let tokens = vec![create_identifier_token("Temperature_Sensor_1", 1, 8)];
            let diagnostics = vec![Diagnostic::default()];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            assert!(actions.is_empty());
        }

        #[test]
        fn ignores_diagnostics_with_an_unrecognized_data_shape() {
            let tokens = vec![create_identifier_token("Temperature_Sensor_1", 1, 8)];
            let diagnostics = vec![Diagnostic {
                data: Some(serde_json::json!({ "unrelated": true })),
                ..Default::default()
            }];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            assert!(actions.is_empty());
        }

        #[test]
        fn ignores_diagnostics_for_symbols_absent_from_the_document() {
            let tokens = vec![create_identifier_token("Other_Var", 1, 8)];
            let diagnostics = vec![truncate_diagnostic("Temperature_Sensor_1", 16)];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            assert!(actions.is_empty());
        }

        #[test]
        fn skips_when_the_truncated_name_would_equal_the_original() {
            let tokens = vec![create_identifier_token("Short", 1, 8)];
            let diagnostics = vec![truncate_diagnostic("Short", 16)];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            assert!(actions.is_empty());
        }

        #[test]
        fn quick_fix_title_mentions_both_names() {
            let tokens = vec![create_identifier_token("Temperature_Sensor_1", 1, 8)];
            let diagnostics = vec![truncate_diagnostic("Temperature_Sensor_1", 16)];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
                panic!("Expected a CodeAction");
            };
            assert!(action.title.contains("Temperature_Sensor_1"));
            assert!(action.title.contains("Temperature_Sens"));
        }

        #[test]
        fn quick_fix_is_marked_as_a_quickfix_kind() {
            let tokens = vec![create_identifier_token("Temperature_Sensor_1", 1, 8)];
            let diagnostics = vec![truncate_diagnostic("Temperature_Sensor_1", 16)];

            let actions = CodeActionProvider::get_code_actions(&tokens, &diagnostics, &test_uri());

            let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
                panic!("Expected a CodeAction");
            };
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
        }
    }
}
