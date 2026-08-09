//! Shared function-call-site walker for CRBasic ASTs
//!
//! Recursively collects every function call in a statement list, tracking
//! which named `Function`/`Sub` (if any) each call is made from. Shared by
//! the inlay hint provider (parameter-name hints need each call's
//! arguments) and the call hierarchy provider (incoming calls need to
//! attribute each call to its enclosing callable).

use crbasic_parser::ast::{AssignmentTarget, Expression, Statement};
use crbasic_parser::lexer::token::Span;

/// A single function call found while walking a statement list
#[derive(Debug, Clone, Copy)]
pub struct CallSite<'a> {
    /// The name of the `Function`/`Sub` this call is made from, or `None`
    /// if the call is made directly from the top-level program body
    pub enclosing: Option<&'a str>,
    /// The name of the function being called
    pub name: &'a str,
    /// The call's argument expressions
    pub arguments: &'a [Expression],
    /// The source span of the call expression itself, not its arguments
    pub span: Span,
}

/// Collects every function call in `statements`, including calls nested
/// inside expressions and inside `If`/`For`/`Do` bodies
pub fn collect_call_sites(statements: &[Statement]) -> Vec<CallSite<'_>> {
    let mut sites = Vec::new();
    walk_statements(statements, None, &mut sites);
    sites
}

fn walk_statements<'a>(
    statements: &'a [Statement],
    enclosing: Option<&'a str>,
    sites: &mut Vec<CallSite<'a>>,
) {
    for statement in statements {
        match statement {
            Statement::VarDeclaration {
                array_dimensions,
                initializer,
                ..
            } => {
                if let Some(dimensions) = array_dimensions {
                    for expr in dimensions {
                        walk_expression(expr, enclosing, sites);
                    }
                }
                if let Some(init) = initializer {
                    walk_expression(init, enclosing, sites);
                }
            }
            Statement::Assignment { target, value, .. } => {
                if let AssignmentTarget::ArrayElement { indices, .. } = target {
                    for index in indices {
                        walk_expression(index, enclosing, sites);
                    }
                }
                walk_expression(value, enclosing, sites);
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                walk_expression(condition, enclosing, sites);
                walk_statements(then_branch, enclosing, sites);
                if let Some(else_stmts) = else_branch {
                    walk_statements(else_stmts, enclosing, sites);
                }
            }
            Statement::ForLoop {
                start,
                end,
                step,
                body,
                ..
            } => {
                walk_expression(start, enclosing, sites);
                walk_expression(end, enclosing, sites);
                if let Some(step_expr) = step {
                    walk_expression(step_expr, enclosing, sites);
                }
                walk_statements(body, enclosing, sites);
            }
            Statement::DoLoop {
                condition, body, ..
            } => {
                if let Some(cond) = condition {
                    walk_expression(cond, enclosing, sites);
                }
                walk_statements(body, enclosing, sites);
            }
            Statement::FunctionCall {
                name,
                arguments,
                span,
            } => {
                sites.push(CallSite {
                    enclosing,
                    name,
                    arguments,
                    span: *span,
                });
                for arg in arguments {
                    walk_expression(arg, enclosing, sites);
                }
            }
            Statement::Expression { expression, .. } => {
                walk_expression(expression, enclosing, sites);
            }
            Statement::ProgramStructure { arguments, .. } => {
                if let Some(args) = arguments {
                    for arg in args {
                        walk_expression(arg, enclosing, sites);
                    }
                }
            }
            Statement::FunctionDefinition { name, body, .. }
            | Statement::SubroutineDefinition { name, body, .. } => {
                walk_statements(body, Some(name.as_str()), sites);
            }
        }
    }
}

fn walk_expression<'a>(
    expr: &'a Expression,
    enclosing: Option<&'a str>,
    sites: &mut Vec<CallSite<'a>>,
) {
    match expr {
        Expression::BinaryOp { left, right, .. } => {
            walk_expression(left, enclosing, sites);
            walk_expression(right, enclosing, sites);
        }
        Expression::UnaryOp { operand, .. } => walk_expression(operand, enclosing, sites),
        Expression::FunctionCall {
            name,
            arguments,
            span,
        } => {
            sites.push(CallSite {
                enclosing,
                name,
                arguments,
                span: *span,
            });
            for arg in arguments {
                walk_expression(arg, enclosing, sites);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            walk_expression(array, enclosing, sites);
            walk_expression(index, enclosing, sites);
        }
        Expression::IntegerLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::Identifier { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::lexer::token::Position;

    fn span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(
            Position::new(start_line, start_col),
            Position::new(end_line, end_col),
        )
    }

    fn identifier(name: &str, line: usize, col: usize) -> Expression {
        Expression::Identifier {
            name: name.to_string(),
            span: span(line, col, line, col + name.len()),
        }
    }

    fn call_statement(name: &str, line: usize) -> Statement {
        Statement::FunctionCall {
            name: name.to_string(),
            arguments: Vec::new(),
            span: span(line, 1, line, 1 + name.len() + 2),
        }
    }

    mod collect_call_sites {
        use super::*;

        #[test]
        fn collects_a_top_level_call_with_no_enclosing_function() {
            let statements = vec![call_statement("Scan", 1)];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites.len(), 1);
            assert_eq!(sites[0].name, "Scan");
            assert_eq!(sites[0].enclosing, None);
        }

        #[test]
        fn attributes_a_call_inside_a_function_body_to_that_function() {
            let statements = vec![Statement::FunctionDefinition {
                name: "Calc".to_string(),
                parameters: Vec::new(),
                body: vec![call_statement("Sqrt", 2)],
                span: span(1, 1, 3, 11),
            }];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites.len(), 1);
            assert_eq!(sites[0].name, "Sqrt");
            assert_eq!(sites[0].enclosing, Some("Calc"));
        }

        #[test]
        fn attributes_a_call_inside_a_subroutine_body_to_that_subroutine() {
            let statements = vec![Statement::SubroutineDefinition {
                name: "Init".to_string(),
                parameters: Vec::new(),
                body: vec![call_statement("Scan", 2)],
                span: span(1, 1, 3, 7),
            }];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites.len(), 1);
            assert_eq!(sites[0].enclosing, Some("Init"));
        }

        #[test]
        fn collects_a_call_nested_inside_an_if_branch() {
            let statements = vec![Statement::IfStatement {
                condition: identifier("Flag", 1, 4),
                then_branch: vec![call_statement("Scan", 2)],
                else_branch: None,
                span: span(1, 1, 3, 6),
            }];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites.len(), 1);
            assert_eq!(sites[0].name, "Scan");
        }

        #[test]
        fn collects_a_call_nested_inside_an_expression() {
            let statements = vec![Statement::Assignment {
                target: AssignmentTarget::Identifier {
                    name: "Result".to_string(),
                    span: span(1, 1, 1, 7),
                },
                value: Expression::FunctionCall {
                    name: "Sqrt".to_string(),
                    arguments: vec![identifier("X", 1, 15)],
                    span: span(1, 10, 1, 17),
                },
                span: span(1, 1, 1, 17),
            }];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites.len(), 1);
            assert_eq!(sites[0].name, "Sqrt");
        }

        #[test]
        fn collects_multiple_calls_in_order() {
            let statements = vec![call_statement("Scan", 1), call_statement("CallTable", 2)];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites.len(), 2);
            assert_eq!(sites[0].name, "Scan");
            assert_eq!(sites[1].name, "CallTable");
        }

        #[test]
        fn records_the_calls_own_span_not_an_arguments_span() {
            let statements = vec![Statement::FunctionCall {
                name: "CallTable".to_string(),
                arguments: vec![identifier("Test", 1, 11)],
                span: span(1, 1, 1, 20),
            }];

            let sites = collect_call_sites(&statements);

            assert_eq!(sites[0].span, span(1, 1, 1, 20));
        }
    }
}
