//! Selection range provider for CRBasic LSP
//!
//! Builds nested `textDocument/selectionRange` ranges by walking the AST's
//! statement spans from the whole program down to the innermost enclosing
//! block, then adding the identifier token under the cursor (if any) as the
//! final, innermost step -- the same occurrence-lookup approach
//! `document_highlight`/`rename` already use for identifiers.

use crbasic_parser::ast::{Program, Statement};
use crbasic_parser::lexer::token::{Span, Token, TokenKind};
use tower_lsp_server::ls_types::{Position, Range, SelectionRange};

/// Provides Selection Range functionality
pub struct SelectionRangeProvider;

impl SelectionRangeProvider {
    /// Builds one selection range chain per requested position
    ///
    /// # Arguments
    /// * `program` - The parsed AST program
    /// * `tokens` - The token stream
    /// * `positions` - The cursor positions to build ranges for (LSP 0-indexed)
    pub fn get_selection_ranges(
        program: &Program,
        tokens: &[Token],
        positions: &[Position],
    ) -> Vec<SelectionRange> {
        positions
            .iter()
            .map(|&position| Self::build_selection_range(program, tokens, position))
            .collect()
    }

    /// Builds the full outermost-to-innermost chain for a single position
    fn build_selection_range(
        program: &Program,
        tokens: &[Token],
        position: Position,
    ) -> SelectionRange {
        let mut spans = vec![program.span];
        Self::collect_enclosing_statement_spans(&program.statements, position, &mut spans);

        if let Some(identifier_span) = Self::find_identifier_span_at(tokens, position) {
            spans.push(identifier_span);
        }

        spans.dedup();
        Self::chain_from_outermost(spans)
    }

    /// Walks a statement list, pushing the span of whichever statement
    /// contains `position` and recursing into its body, mirroring
    /// `FoldingRangeProvider`'s block traversal
    fn collect_enclosing_statement_spans(
        statements: &[Statement],
        position: Position,
        spans: &mut Vec<Span>,
    ) {
        for statement in statements {
            let span = statement.span();
            if !Self::span_contains(span, position) {
                continue;
            }
            spans.push(span);

            match statement {
                Statement::IfStatement {
                    then_branch,
                    else_branch,
                    ..
                }
                | Statement::PreprocessorConditional {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    Self::collect_enclosing_statement_spans(then_branch, position, spans);
                    if let Some(else_stmts) = else_branch {
                        Self::collect_enclosing_statement_spans(else_stmts, position, spans);
                    }
                }
                Statement::ForLoop { body, .. }
                | Statement::DoLoop { body, .. }
                | Statement::FunctionDefinition { body, .. }
                | Statement::SubroutineDefinition { body, .. } => {
                    Self::collect_enclosing_statement_spans(body, position, spans);
                }
                Statement::SelectCase {
                    cases, else_branch, ..
                } => {
                    for case in cases {
                        if Self::span_contains(case.span, position) {
                            spans.push(case.span);
                            Self::collect_enclosing_statement_spans(&case.body, position, spans);
                        }
                    }
                    if let Some(else_stmts) = else_branch {
                        Self::collect_enclosing_statement_spans(else_stmts, position, spans);
                    }
                }
                _ => {}
            }

            // Sibling statements never overlap, so the containing one found
            // above is the only one that can match at this level.
            return;
        }
    }

    /// Finds the span of the identifier token at `position`, if any
    fn find_identifier_span_at(tokens: &[Token], position: Position) -> Option<Span> {
        tokens.iter().find_map(|token| {
            let TokenKind::Identifier(_) = &token.kind else {
                return None;
            };
            Self::span_contains(token.span, position).then_some(token.span)
        })
    }

    /// Checks whether an LSP position (0-indexed) falls within a parser span
    /// (1-indexed, half-open at the end)
    fn span_contains(span: Span, position: Position) -> bool {
        let line = position.line as usize + 1;
        let column = position.character as usize + 1;

        if line < span.start.line || line > span.end.line {
            return false;
        }
        if line == span.start.line && column < span.start.column {
            return false;
        }
        if line == span.end.line && column >= span.end.column {
            return false;
        }
        true
    }

    /// Builds the linked `SelectionRange` chain from a list of spans ordered
    /// outermost-first. The returned value's own `range` is the last
    /// (innermost) entry; each larger span wraps it as `parent`, ending with
    /// the outermost span at the tail of the chain (`parent: None`).
    fn chain_from_outermost(spans: Vec<Span>) -> SelectionRange {
        let mut iter = spans.into_iter();
        let outermost = iter
            .next()
            .expect("the program span is always pushed as the outermost entry");

        let mut current = SelectionRange {
            range: Self::span_to_range(outermost),
            parent: None,
        };

        for span in iter {
            current = SelectionRange {
                range: Self::span_to_range(span),
                parent: Some(Box::new(current)),
            };
        }

        current
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
    use crbasic_parser::ast::Expression;
    use crbasic_parser::lexer::token::Position as ParserPosition;

    fn pos(line: usize, column: usize) -> ParserPosition {
        ParserPosition::new(line, column)
    }

    fn span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(pos(start_line, start_col), pos(end_line, end_col))
    }

    fn program(statements: Vec<Statement>, program_span: Span) -> Program {
        Program::new(statements, program_span)
    }

    fn dummy_condition() -> Expression {
        Expression::BooleanLiteral {
            value: true,
            span: span(1, 1, 1, 1),
        }
    }

    fn create_identifier_token(
        name: &'static str,
        line: usize,
        start_col: usize,
    ) -> Token<'static> {
        let end_col = start_col + name.len();
        Token {
            kind: TokenKind::Identifier(name),
            lexeme: name,
            span: span(line, start_col, line, end_col),
        }
    }

    mod get_selection_ranges {
        use super::*;

        #[test]
        fn returns_one_chain_per_requested_position() {
            let stmts = vec![Statement::Expression {
                expression: dummy_condition(),
                span: span(1, 1, 1, 10),
            }];
            let program = program(stmts, span(1, 1, 1, 10));
            let positions = vec![
                Position {
                    line: 0,
                    character: 0,
                },
                Position {
                    line: 0,
                    character: 5,
                },
            ];

            let ranges = SelectionRangeProvider::get_selection_ranges(&program, &[], &positions);

            assert_eq!(ranges.len(), 2);
        }

        #[test]
        fn innermost_range_is_the_enclosing_statement() {
            let stmts = vec![
                Statement::Expression {
                    expression: dummy_condition(),
                    span: span(1, 1, 1, 5),
                },
                Statement::Expression {
                    expression: dummy_condition(),
                    span: span(2, 1, 2, 8),
                },
            ];
            let program = program(stmts, span(1, 1, 2, 8));
            let positions = vec![Position {
                line: 1,
                character: 2,
            }];

            let ranges = SelectionRangeProvider::get_selection_ranges(&program, &[], &positions);

            assert_eq!(ranges[0].range.start.line, 1);
            assert_eq!(ranges[0].range.end.line, 1);
            assert_eq!(ranges[0].range.end.character, 7);
        }

        #[test]
        fn outermost_ancestor_is_the_whole_program() {
            let stmts = vec![Statement::Expression {
                expression: dummy_condition(),
                span: span(3, 1, 3, 5),
            }];
            let program = program(stmts, span(1, 1, 10, 1));
            let positions = vec![Position {
                line: 2,
                character: 1,
            }];

            let ranges = SelectionRangeProvider::get_selection_ranges(&program, &[], &positions);

            let mut outermost = &ranges[0];
            while let Some(parent) = &outermost.parent {
                outermost = parent;
            }
            assert_eq!(outermost.range.start.line, 0);
            assert_eq!(outermost.range.end.line, 9);
        }

        #[test]
        fn recurses_into_nested_block_bodies() {
            let program = program(
                vec![Statement::IfStatement {
                    condition: dummy_condition(),
                    then_branch: vec![Statement::Expression {
                        expression: dummy_condition(),
                        span: span(2, 3, 2, 9),
                    }],
                    else_branch: None,
                    span: span(1, 1, 3, 6),
                }],
                span(1, 1, 3, 6),
            );
            let positions = vec![Position {
                line: 1,
                character: 4,
            }];

            let ranges = SelectionRangeProvider::get_selection_ranges(&program, &[], &positions);

            assert_eq!(ranges[0].range.start.line, 1);
            assert_eq!(ranges[0].range.end.character, 8);
            let parent = ranges[0]
                .parent
                .as_ref()
                .expect("If statement is the parent");
            assert_eq!(parent.range.start.line, 0);
            assert_eq!(parent.range.end.line, 2);
        }

        #[test]
        fn innermost_range_is_the_identifier_token_when_cursor_is_on_one() {
            let stmts = vec![Statement::Assignment {
                target: crbasic_parser::ast::AssignmentTarget::Identifier {
                    name: "Temp_C".to_string(),
                    span: span(1, 1, 1, 7),
                },
                value: Expression::IntegerLiteral {
                    value: 5,
                    span: span(1, 10, 1, 11),
                },
                span: span(1, 1, 1, 11),
            }];
            let program = program(stmts, span(1, 1, 1, 11));
            let tokens = vec![create_identifier_token("Temp_C", 1, 1)];
            let positions = vec![Position {
                line: 0,
                character: 2,
            }];

            let ranges =
                SelectionRangeProvider::get_selection_ranges(&program, &tokens, &positions);

            assert_eq!(ranges[0].range.start.character, 0);
            assert_eq!(ranges[0].range.end.character, 6);
            let parent = ranges[0]
                .parent
                .as_ref()
                .expect("assignment statement is the parent");
            assert_eq!(parent.range.end.character, 10);
        }

        #[test]
        fn falls_back_to_the_whole_program_outside_any_statement() {
            let stmts = vec![Statement::Expression {
                expression: dummy_condition(),
                span: span(2, 1, 2, 5),
            }];
            let program = program(stmts, span(1, 1, 5, 1));
            let positions = vec![Position {
                line: 4,
                character: 0,
            }];

            let ranges = SelectionRangeProvider::get_selection_ranges(&program, &[], &positions);

            assert!(ranges[0].parent.is_none());
            assert_eq!(ranges[0].range.start.line, 0);
            assert_eq!(ranges[0].range.end.line, 4);
        }
    }
}
