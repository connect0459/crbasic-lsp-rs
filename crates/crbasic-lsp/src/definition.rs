//! Go to Definition provider for CRBasic LSP
//!
//! This module provides the Go to Definition functionality, allowing navigation
//! from symbol references to their definitions.

use crbasic_parser::ast::{Program, Statement};
use crbasic_parser::lexer::token::{Span, Token, TokenKind};
use std::collections::HashMap;
use tower_lsp_server::ls_types::{Location, Position, Range, Uri};

/// Symbol kind for definition lookup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// Variable symbol
    Variable,
    /// Function symbol
    Function,
    /// Subroutine symbol
    Subroutine,
}

/// Symbol definition information
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolDefinition {
    /// The symbol name
    pub name: String,
    /// The kind of symbol (Variable, Function, or Subroutine)
    pub kind: SymbolKind,
    /// The source code location of the definition
    pub span: Span,
}

/// Provides Go to Definition functionality
pub struct DefinitionProvider;

impl DefinitionProvider {
    /// Extracts all symbol definitions from the AST
    ///
    /// # Arguments
    /// * `ast` - The parsed AST program
    ///
    /// # Returns
    /// A map of symbol names to their definitions
    pub fn extract_definitions(ast: &Program) -> HashMap<String, SymbolDefinition> {
        let mut definitions = HashMap::new();

        for statement in &ast.statements {
            Self::extract_from_statement(statement, &mut definitions);
        }

        definitions
    }

    /// Extracts definitions from a single statement recursively
    fn extract_from_statement(
        statement: &Statement,
        definitions: &mut HashMap<String, SymbolDefinition>,
    ) {
        match statement {
            Statement::VarDeclaration { name, span, .. } => {
                definitions.insert(
                    name.clone(),
                    SymbolDefinition {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        span: *span,
                    },
                );
            }
            Statement::FunctionDefinition {
                name, body, span, ..
            } => {
                definitions.insert(
                    name.clone(),
                    SymbolDefinition {
                        name: name.clone(),
                        kind: SymbolKind::Function,
                        span: *span,
                    },
                );
                for stmt in body {
                    Self::extract_from_statement(stmt, definitions);
                }
            }
            Statement::SubroutineDefinition {
                name, body, span, ..
            } => {
                definitions.insert(
                    name.clone(),
                    SymbolDefinition {
                        name: name.clone(),
                        kind: SymbolKind::Subroutine,
                        span: *span,
                    },
                );
                for stmt in body {
                    Self::extract_from_statement(stmt, definitions);
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
                    Self::extract_from_statement(stmt, definitions);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        Self::extract_from_statement(stmt, definitions);
                    }
                }
            }
            Statement::ForLoop { body, .. } | Statement::DoLoop { body, .. } => {
                for stmt in body {
                    Self::extract_from_statement(stmt, definitions);
                }
            }
            Statement::SelectCase {
                cases, else_branch, ..
            } => {
                for case in cases {
                    for stmt in &case.body {
                        Self::extract_from_statement(stmt, definitions);
                    }
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        Self::extract_from_statement(stmt, definitions);
                    }
                }
            }
            _ => {}
        }
    }

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

    /// Gets the definition location for a symbol
    ///
    /// # Arguments
    /// * `tokens` - The token stream
    /// * `position` - The cursor position (LSP 0-indexed)
    /// * `definitions` - The symbol definitions map
    /// * `uri` - The document URI
    ///
    /// # Returns
    /// The location of the definition if found
    pub fn get_definition(
        tokens: &[Token],
        position: Position,
        definitions: &HashMap<String, SymbolDefinition>,
        uri: Uri,
    ) -> Option<Location> {
        let identifier = Self::find_identifier_at_position(tokens, position)?;
        let definition = definitions.get(&identifier)?;

        Some(Location {
            uri,
            range: Self::span_to_range(definition.span),
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
    use crbasic_parser::lexer::token::Position as ParserPosition;

    fn create_span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(
            ParserPosition::new(start_line, start_col),
            ParserPosition::new(end_line, end_col),
        )
    }

    mod extract_definitions {
        use super::*;

        #[test]
        fn extracts_variable_declarations() {
            let ast = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: "Temp_C".to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_span(1, 1, 1, 14),
                }],
                create_span(1, 1, 1, 14),
            );

            let definitions = DefinitionProvider::extract_definitions(&ast);

            assert_eq!(definitions.len(), 1);
            let def = definitions.get("Temp_C").expect("Should find Temp_C");
            assert_eq!(def.kind, SymbolKind::Variable);
            assert_eq!(def.span.start.line, 1);
        }

        #[test]
        fn extracts_function_definitions() {
            let ast = Program::new(
                vec![Statement::FunctionDefinition {
                    name: "Calculate".to_string(),
                    parameters: vec!["x".to_string()],
                    body: vec![],
                    span: create_span(1, 1, 3, 12),
                }],
                create_span(1, 1, 3, 12),
            );

            let definitions = DefinitionProvider::extract_definitions(&ast);

            assert_eq!(definitions.len(), 1);
            let def = definitions.get("Calculate").expect("Should find Calculate");
            assert_eq!(def.kind, SymbolKind::Function);
        }

        #[test]
        fn extracts_subroutine_definitions() {
            let ast = Program::new(
                vec![Statement::SubroutineDefinition {
                    name: "Initialize".to_string(),
                    parameters: vec![],
                    body: vec![],
                    span: create_span(1, 1, 3, 7),
                }],
                create_span(1, 1, 3, 7),
            );

            let definitions = DefinitionProvider::extract_definitions(&ast);

            assert_eq!(definitions.len(), 1);
            let def = definitions
                .get("Initialize")
                .expect("Should find Initialize");
            assert_eq!(def.kind, SymbolKind::Subroutine);
        }

        #[test]
        fn extracts_nested_definitions() {
            let ast = Program::new(
                vec![Statement::FunctionDefinition {
                    name: "Outer".to_string(),
                    parameters: vec![],
                    body: vec![Statement::VarDeclaration {
                        keyword: "Dim".to_string(),
                        name: "local_var".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_span(2, 3, 2, 15),
                    }],
                    span: create_span(1, 1, 3, 12),
                }],
                create_span(1, 1, 3, 12),
            );

            let definitions = DefinitionProvider::extract_definitions(&ast);

            assert_eq!(definitions.len(), 2);
            assert!(definitions.contains_key("Outer"));
            assert!(definitions.contains_key("local_var"));
        }
    }

    mod find_identifier_at_position {
        use super::*;

        fn create_identifier_token(name: &str, line: usize, start_col: usize) -> Token<'_> {
            let end_col = start_col + name.len();
            Token {
                kind: TokenKind::Identifier(name),
                lexeme: name,
                span: create_span(line, start_col, line, end_col),
            }
        }

        #[test]
        fn finds_identifier_at_exact_start() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 1)];
            let position = Position {
                line: 0,
                character: 0,
            };

            let result = DefinitionProvider::find_identifier_at_position(&tokens, position);

            assert_eq!(result, Some("Temp_C".to_string()));
        }

        #[test]
        fn finds_identifier_at_middle() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 1)];
            let position = Position {
                line: 0,
                character: 3,
            };

            let result = DefinitionProvider::find_identifier_at_position(&tokens, position);

            assert_eq!(result, Some("Temp_C".to_string()));
        }

        #[test]
        fn returns_none_outside_identifier() {
            let tokens = vec![create_identifier_token("Temp_C", 1, 1)];
            let position = Position {
                line: 0,
                character: 10,
            };

            let result = DefinitionProvider::find_identifier_at_position(&tokens, position);

            assert_eq!(result, None);
        }

        #[test]
        fn ignores_non_identifier_tokens() {
            let tokens = vec![Token {
                kind: TokenKind::Keyword("Public"),
                lexeme: "Public",
                span: create_span(1, 1, 1, 7),
            }];
            let position = Position {
                line: 0,
                character: 0,
            };

            let result = DefinitionProvider::find_identifier_at_position(&tokens, position);

            assert_eq!(result, None);
        }
    }

    mod get_definition {
        use super::*;

        fn setup_test() -> (Vec<Token<'static>>, HashMap<String, SymbolDefinition>, Uri) {
            let tokens = vec![Token {
                kind: TokenKind::Identifier("Temp_C"),
                lexeme: "Temp_C",
                span: create_span(5, 10, 5, 16),
            }];

            let mut definitions = HashMap::new();
            definitions.insert(
                "Temp_C".to_string(),
                SymbolDefinition {
                    name: "Temp_C".to_string(),
                    kind: SymbolKind::Variable,
                    span: create_span(1, 8, 1, 14),
                },
            );

            let uri = "file:///test.cr1".parse::<Uri>().expect("Valid URL");

            (tokens, definitions, uri)
        }

        #[test]
        fn returns_definition_location_for_known_symbol() {
            let (tokens, definitions, uri) = setup_test();
            let position = Position {
                line: 4,
                character: 10,
            };

            let result =
                DefinitionProvider::get_definition(&tokens, position, &definitions, uri.clone());

            assert!(result.is_some());
            let location = result.expect("Should return location");
            assert_eq!(location.uri, uri);
            assert_eq!(location.range.start.line, 0);
            assert_eq!(location.range.start.character, 7);
        }

        #[test]
        fn returns_none_for_unknown_symbol() {
            let tokens = vec![Token {
                kind: TokenKind::Identifier("Unknown"),
                lexeme: "Unknown",
                span: create_span(5, 10, 5, 17),
            }];
            let definitions = HashMap::new();
            let uri = "file:///test.cr1".parse::<Uri>().expect("Valid URL");
            let position = Position {
                line: 4,
                character: 10,
            };

            let result = DefinitionProvider::get_definition(&tokens, position, &definitions, uri);

            assert!(result.is_none());
        }

        #[test]
        fn returns_none_when_not_on_identifier() {
            let (_, definitions, uri) = setup_test();
            let tokens = vec![];
            let position = Position {
                line: 4,
                character: 10,
            };

            let result = DefinitionProvider::get_definition(&tokens, position, &definitions, uri);

            assert!(result.is_none());
        }
    }

    mod span_to_range {
        use super::*;

        #[test]
        fn converts_parser_span_to_lsp_range() {
            let span = create_span(5, 10, 5, 20);

            let range = DefinitionProvider::span_to_range(span);

            // Parser is 1-indexed, LSP is 0-indexed
            assert_eq!(range.start.line, 4);
            assert_eq!(range.start.character, 9);
            assert_eq!(range.end.line, 4);
            assert_eq!(range.end.character, 19);
        }
    }
}
