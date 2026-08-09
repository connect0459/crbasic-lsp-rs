//! Semantic token classification for CRBasic
//!
//! A regex-based grammar can't tell a declaration from a reference or a
//! `Public` variable from a `Dim` one; this module walks the AST to add
//! that distinction on top of the baseline TextMate highlighting.

use crbasic_parser::ast::{Program, Statement};
use crbasic_parser::lexer::token::{Token, TokenKind};
use std::collections::{HashMap, HashSet};
use tower_lsp_server::ls_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend,
};

// Indices/bit positions here must match the order returned by `legend()`;
// reordering one without the other silently breaks the client's rendering.
const TOKEN_TYPE_FUNCTION: u32 = 0;
const TOKEN_TYPE_VARIABLE: u32 = 1;

const MODIFIER_DECLARATION: u32 = 1 << 0;
const MODIFIER_READONLY: u32 = 1 << 1;
const MODIFIER_GLOBAL: u32 = 1 << 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolCategory {
    Variable { is_global: bool, is_readonly: bool },
    Callable,
}

#[derive(Debug, Clone, Copy)]
struct SymbolEntry {
    category: SymbolCategory,
    declaration_line: usize,
}

/// Classifies CRBasic identifiers into LSP semantic tokens.
pub struct SemanticTokensProvider;

impl SemanticTokensProvider {
    /// The legend advertised in `ServerCapabilities`.
    ///
    /// The order of `token_types` and `token_modifiers` must match the
    /// indices/bit positions used internally by [`Self::get_semantic_tokens`].
    pub fn legend() -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: vec![SemanticTokenType::FUNCTION, SemanticTokenType::VARIABLE],
            token_modifiers: vec![
                SemanticTokenModifier::DECLARATION,
                SemanticTokenModifier::READONLY,
                SemanticTokenModifier::new("global"),
            ],
        }
    }

    /// Classifies every tracked identifier occurrence in `tokens` and
    /// encodes them as relative-delta LSP semantic tokens.
    ///
    /// Only identifiers that resolve to a `Public`/`Dim`/`Const` variable or
    /// a `Function`/`Sub` declared in `ast` are emitted; built-in functions,
    /// keywords, and `For` loop variables are left to the TextMate grammar.
    pub fn get_semantic_tokens(ast: &Program, tokens: &[Token]) -> Vec<SemanticToken> {
        let symbol_table = Self::build_symbol_table(ast);
        let mut declared = HashSet::new();
        let mut prev_line = 0usize;
        let mut prev_start = 0usize;
        let mut result = Vec::new();

        for token in tokens {
            let TokenKind::Identifier(name) = &token.kind else {
                continue;
            };

            let Some(entry) = symbol_table.get(*name) else {
                continue;
            };

            let is_declaration =
                !declared.contains(*name) && token.span.start.line == entry.declaration_line;
            if is_declaration {
                declared.insert((*name).to_string());
            }

            let (token_type, mut modifiers) = match entry.category {
                SymbolCategory::Callable => (TOKEN_TYPE_FUNCTION, 0),
                SymbolCategory::Variable {
                    is_global,
                    is_readonly,
                } => {
                    let mut bits = 0;
                    if is_global {
                        bits |= MODIFIER_GLOBAL;
                    }
                    if is_readonly {
                        bits |= MODIFIER_READONLY;
                    }
                    (TOKEN_TYPE_VARIABLE, bits)
                }
            };
            if is_declaration {
                modifiers |= MODIFIER_DECLARATION;
            }

            let line = token.span.start.line.saturating_sub(1);
            let start = token.span.start.column.saturating_sub(1);
            let length = (token.span.end.column - token.span.start.column) as u32;

            let delta_line = (line - prev_line) as u32;
            let delta_start = if delta_line == 0 {
                (start - prev_start) as u32
            } else {
                start as u32
            };

            result.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: modifiers,
            });

            prev_line = line;
            prev_start = start;
        }

        result
    }

    fn build_symbol_table(ast: &Program) -> HashMap<String, SymbolEntry> {
        let mut table = HashMap::new();
        for statement in &ast.statements {
            Self::collect_from_statement(statement, &mut table);
        }
        table
    }

    fn collect_from_statement(statement: &Statement, table: &mut HashMap<String, SymbolEntry>) {
        match statement {
            Statement::VarDeclaration {
                keyword,
                name,
                span,
                ..
            } => {
                table.insert(
                    name.clone(),
                    SymbolEntry {
                        category: SymbolCategory::Variable {
                            is_global: keyword == "Public",
                            is_readonly: keyword == "Const",
                        },
                        declaration_line: span.start.line,
                    },
                );
            }
            Statement::FunctionDefinition {
                name, body, span, ..
            }
            | Statement::SubroutineDefinition {
                name, body, span, ..
            } => {
                table.insert(
                    name.clone(),
                    SymbolEntry {
                        category: SymbolCategory::Callable,
                        declaration_line: span.start.line,
                    },
                );
                for stmt in body {
                    Self::collect_from_statement(stmt, table);
                }
            }
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
                for stmt in then_branch {
                    Self::collect_from_statement(stmt, table);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        Self::collect_from_statement(stmt, table);
                    }
                }
            }
            Statement::ForLoop { body, .. } | Statement::DoLoop { body, .. } => {
                for stmt in body {
                    Self::collect_from_statement(stmt, table);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::Parser;
    use crbasic_parser::lexer::Scanner;

    fn parse(source: &str) -> Program {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        parser.parse().expect("Parse should succeed")
    }

    fn tokenize(source: &str) -> Vec<Token<'_>> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()
    }

    mod legend {
        use super::*;

        #[test]
        fn declares_function_and_variable_token_types_in_order() {
            let legend = SemanticTokensProvider::legend();

            assert_eq!(
                legend.token_types,
                vec![SemanticTokenType::FUNCTION, SemanticTokenType::VARIABLE]
            );
        }

        #[test]
        fn declares_declaration_readonly_and_global_modifiers_in_order() {
            let legend = SemanticTokensProvider::legend();

            assert_eq!(
                legend.token_modifiers,
                vec![
                    SemanticTokenModifier::DECLARATION,
                    SemanticTokenModifier::READONLY,
                    SemanticTokenModifier::new("global"),
                ]
            );
        }
    }

    mod get_semantic_tokens {
        use super::*;

        #[test]
        fn marks_declaration_modifier_only_on_the_declaring_occurrence() {
            let source = "Dim i\ni = 1";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert_eq!(result.len(), 2);
            assert_eq!(
                result[0].token_modifiers_bitset & MODIFIER_DECLARATION,
                MODIFIER_DECLARATION
            );
            assert_eq!(result[1].token_modifiers_bitset & MODIFIER_DECLARATION, 0);
        }

        #[test]
        fn marks_public_variables_as_global_at_every_occurrence() {
            let source = "Public Temp_C\nTemp_C = 1";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert_eq!(result.len(), 2);
            for token in &result {
                assert_eq!(
                    token.token_type, TOKEN_TYPE_VARIABLE,
                    "Public variable occurrences should be classified as variables"
                );
                assert_eq!(
                    token.token_modifiers_bitset & MODIFIER_GLOBAL,
                    MODIFIER_GLOBAL
                );
            }
        }

        #[test]
        fn does_not_mark_dim_variables_as_global() {
            let source = "Dim i\ni = 1";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            for token in &result {
                assert_eq!(token.token_modifiers_bitset & MODIFIER_GLOBAL, 0);
            }
        }

        #[test]
        fn marks_const_variables_as_readonly() {
            let source = "Const PI = 3.14";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert_eq!(result.len(), 1);
            assert_eq!(
                result[0].token_modifiers_bitset & MODIFIER_READONLY,
                MODIFIER_READONLY
            );
        }

        #[test]
        fn classifies_function_declaration_and_call_site_as_function_type() {
            // CRBasic functions return a value by assigning to their own
            // name (`Square = x * x`), so "Square" legitimately occurs three
            // times: the declaration header, the return assignment, and the
            // call site.
            let source = "Function Square(x)\nSquare = x * x\nEndFunction\nSquare(2)";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            let square_occurrences: Vec<_> = result
                .iter()
                .filter(|t| t.token_type == TOKEN_TYPE_FUNCTION)
                .collect();

            assert_eq!(square_occurrences.len(), 3);
            assert_eq!(
                square_occurrences[0].token_modifiers_bitset & MODIFIER_DECLARATION,
                MODIFIER_DECLARATION
            );
            assert_eq!(
                square_occurrences[1].token_modifiers_bitset & MODIFIER_DECLARATION,
                0
            );
            assert_eq!(
                square_occurrences[2].token_modifiers_bitset & MODIFIER_DECLARATION,
                0
            );
        }

        #[test]
        fn classifies_subroutine_the_same_way_as_function() {
            let source = "Sub Initialize()\nEndSub\nInitialize()";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            let occurrences: Vec<_> = result
                .iter()
                .filter(|t| t.token_type == TOKEN_TYPE_FUNCTION)
                .collect();
            assert_eq!(occurrences.len(), 2);
        }

        #[test]
        fn ignores_identifiers_with_no_matching_declaration() {
            // TimeIntoInterval is a built-in function call with no VarDeclaration
            // or FunctionDefinition/SubroutineDefinition backing it.
            let source = "TimeIntoInterval(1, 1, 0)";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert!(result.is_empty());
        }

        #[test]
        fn encodes_first_token_with_absolute_line_and_column() {
            let source = "Dim i";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].delta_line, 0);
            assert_eq!(result[0].delta_start, 4);
            assert_eq!(result[0].length, 1);
        }

        #[test]
        fn encodes_second_token_on_a_new_line_with_absolute_column() {
            let source = "Dim i\ni = 1";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert_eq!(result.len(), 2);
            assert_eq!(result[1].delta_line, 1);
            assert_eq!(result[1].delta_start, 0);
        }

        #[test]
        fn encodes_same_line_tokens_with_relative_column_delta() {
            let source = "Public A\nA = A + 1";
            let ast = parse(source);
            let tokens = tokenize(source);

            let result = SemanticTokensProvider::get_semantic_tokens(&ast, &tokens);

            assert_eq!(result.len(), 3);
            assert_eq!(result[1].delta_line, 1);
            assert_eq!(result[2].delta_line, 0);
            // The second "A" on line 2 starts further right than the
            // first; delta_start is relative to the previous token's start
            // column on the same line, not absolute.
            assert!(result[2].delta_start > 0);
        }
    }
}
