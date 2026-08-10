//! Inlay hint provider for CRBasic LSP
//!
//! Shows each recognized function's parameter name inline before its
//! corresponding argument at a call site (e.g. `Scan(⟨Interval:⟩1, ...)`),
//! reusing the parameter names already maintained for signature help
//! (`SignatureProvider`) for built-in functions, and reading them straight
//! from the AST for user-defined `Function`/`Sub` definitions.

use crate::call_sites::collect_call_sites;
use crate::signature::SignatureProvider;
use crbasic_parser::ast::{Expression, Program, Statement};
use crbasic_parser::lexer::token::Position as ParserPosition;
use std::collections::HashMap;
use tower_lsp_server::ls_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range};

/// Provides Inlay Hint functionality
pub struct InlayHintProvider;

impl InlayHintProvider {
    /// Builds a parameter-name inlay hint for every recognized function call
    /// argument whose position falls within `range`
    ///
    /// # Arguments
    /// * `program` - The parsed AST program
    /// * `range` - The visible document range the client requested hints for
    pub fn get_inlay_hints(program: &Program, range: Range) -> Vec<InlayHint> {
        let user_defined = Self::collect_user_defined_parameters(&program.statements);

        collect_call_sites(&program.statements)
            .into_iter()
            .filter_map(|site| {
                let parameter_names = Self::resolve_parameter_names(site.name, &user_defined)?;
                Some(Self::hints_for_call(&parameter_names, site.arguments))
            })
            .flatten()
            .filter(|hint| Self::within_range(hint.position, range))
            .collect()
    }

    /// Resolves the ordered parameter names for a call site's function name,
    /// checking user-defined `Function`/`Sub` definitions before falling
    /// back to the built-in signature database
    fn resolve_parameter_names(
        name: &str,
        user_defined: &HashMap<String, Vec<String>>,
    ) -> Option<Vec<String>> {
        if let Some(parameters) = user_defined.get(name) {
            return Some(parameters.clone());
        }

        SignatureProvider::get_function_signature(name)
            .map(|signature| signature.parameters.into_iter().map(|p| p.name).collect())
    }

    /// Pairs each argument with its parameter name, dropping any argument
    /// past the last known parameter
    fn hints_for_call(parameter_names: &[String], arguments: &[Expression]) -> Vec<InlayHint> {
        parameter_names
            .iter()
            .zip(arguments)
            .map(|(parameter_name, argument)| InlayHint {
                position: Self::position_to_lsp(argument.span().start),
                label: InlayHintLabel::String(format!("{parameter_name}:")),
                kind: Some(InlayHintKind::PARAMETER),
                text_edits: None,
                tooltip: None,
                padding_left: Some(false),
                padding_right: Some(true),
                data: None,
            })
            .collect()
    }

    fn within_range(position: Position, range: Range) -> bool {
        position >= range.start && position <= range.end
    }

    /// Converts parser Position (1-indexed) to LSP Position (0-indexed)
    fn position_to_lsp(pos: ParserPosition) -> Position {
        Position {
            line: pos.line.saturating_sub(1) as u32,
            character: pos.column.saturating_sub(1) as u32,
        }
    }

    /// Collects the parameter names declared by every user-defined
    /// `Function`/`Sub` in the program
    fn collect_user_defined_parameters(statements: &[Statement]) -> HashMap<String, Vec<String>> {
        statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::FunctionDefinition {
                    name, parameters, ..
                }
                | Statement::SubroutineDefinition {
                    name, parameters, ..
                } => Some((name.clone(), parameters.clone())),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::lexer::token::Span;

    fn pos(line: usize, column: usize) -> ParserPosition {
        ParserPosition::new(line, column)
    }

    fn span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(pos(start_line, start_col), pos(end_line, end_col))
    }

    fn program(statements: Vec<Statement>) -> Program {
        Program::new(statements, span(1, 1, 1, 1))
    }

    fn int_literal(value: i64, line: usize, col: usize) -> Expression {
        Expression::IntegerLiteral {
            value,
            span: span(line, col, line, col + value.to_string().len()),
        }
    }

    fn identifier(name: &str, line: usize, col: usize) -> Expression {
        Expression::Identifier {
            name: name.to_string(),
            span: span(line, col, line, col + name.len()),
        }
    }

    fn call_statement(name: &str, arguments: Vec<Expression>, line: usize) -> Statement {
        Statement::FunctionCall {
            name: name.to_string(),
            arguments,
            span: span(line, 1, line, 20),
        }
    }

    fn label_text(hint: &InlayHint) -> &str {
        let InlayHintLabel::String(text) = &hint.label else {
            panic!("Expected a string label");
        };
        text.as_str()
    }

    /// A range wide enough to contain any position used in these tests
    fn whole_document_range() -> Range {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 1000,
                character: 0,
            },
        }
    }

    mod get_inlay_hints {
        use super::*;

        #[test]
        fn builds_a_hint_for_each_builtin_parameter() {
            let program = program(vec![call_statement(
                "Scan",
                vec![
                    int_literal(1, 1, 6),
                    identifier("Sec", 1, 9),
                    int_literal(0, 1, 14),
                ],
                1,
            )]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert_eq!(hints.len(), 3);
            assert_eq!(label_text(&hints[0]), "Interval:");
            assert_eq!(label_text(&hints[1]), "Units:");
            assert_eq!(label_text(&hints[2]), "BufferOption:");
        }

        #[test]
        fn positions_a_hint_at_the_start_of_its_argument() {
            let program = program(vec![call_statement(
                "Abs",
                vec![identifier("Test", 1, 11)],
                1,
            )]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert_eq!(hints.len(), 1);
            assert_eq!(hints[0].position.line, 0);
            assert_eq!(hints[0].position.character, 10);
        }

        #[test]
        fn hint_kind_is_parameter() {
            let program = program(vec![call_statement(
                "Abs",
                vec![identifier("Test", 1, 11)],
                1,
            )]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert_eq!(hints[0].kind, Some(InlayHintKind::PARAMETER));
        }

        #[test]
        fn builds_a_hint_for_user_defined_function_parameters() {
            let program = program(vec![
                Statement::FunctionDefinition {
                    name: "Calc".to_string(),
                    parameters: vec!["X".to_string(), "Y".to_string()],
                    body: Vec::new(),
                    span: span(1, 1, 1, 1),
                },
                call_statement("Calc", vec![int_literal(1, 2, 6), int_literal(2, 2, 9)], 2),
            ]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert_eq!(hints.len(), 2);
            assert_eq!(label_text(&hints[0]), "X:");
            assert_eq!(label_text(&hints[1]), "Y:");
        }

        #[test]
        fn skips_calls_to_unrecognized_functions() {
            let program = program(vec![call_statement(
                "SomeUnknownFunction",
                vec![int_literal(1, 1, 21)],
                1,
            )]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert!(hints.is_empty());
        }

        #[test]
        fn finds_calls_nested_inside_an_assignment_expression() {
            let program = program(vec![Statement::Assignment {
                target: crbasic_parser::ast::AssignmentTarget::Identifier {
                    name: "Result".to_string(),
                    span: span(1, 1, 1, 7),
                },
                value: Expression::FunctionCall {
                    name: "Sqr".to_string(),
                    arguments: vec![identifier("X", 1, 14)],
                    span: span(1, 10, 1, 16),
                },
                span: span(1, 1, 1, 16),
            }]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert_eq!(hints.len(), 1);
        }

        #[test]
        fn only_hints_arguments_that_have_a_corresponding_parameter() {
            let program = program(vec![call_statement(
                "Abs",
                vec![identifier("Test", 1, 11), identifier("Extra", 1, 17)],
                1,
            )]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            // Abs only has one documented parameter (Value)
            assert_eq!(hints.len(), 1);
        }

        #[test]
        fn excludes_hints_outside_the_requested_range() {
            let program = program(vec![
                call_statement("Abs", vec![identifier("A", 1, 11)], 1),
                call_statement("Abs", vec![identifier("B", 50, 11)], 50),
            ]);
            let narrow_range = Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 5,
                    character: 0,
                },
            };

            let hints = InlayHintProvider::get_inlay_hints(&program, narrow_range);

            assert_eq!(hints.len(), 1);
            assert_eq!(hints[0].position.line, 0);
        }

        #[test]
        fn ignores_array_dimension_and_initializer_expressions_without_calls() {
            let program = program(vec![Statement::VarDeclaration {
                keyword: "Public".to_string(),
                name: "Data".to_string(),
                array_dimensions: Some(vec![int_literal(10, 1, 13)]),
                type_annotation: None,
                type_size: None,
                initializer: None,
                span: span(1, 1, 1, 16),
            }]);

            let hints = InlayHintProvider::get_inlay_hints(&program, whole_document_range());

            assert!(hints.is_empty());
        }
    }
}
