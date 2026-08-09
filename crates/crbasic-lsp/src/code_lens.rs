//! Code lens provider for CRBasic LSP
//!
//! Shows a "N references" lens above every `Public`/`Dim`/`Const` variable
//! and `Function`/`Sub` declaration, reusing the existing declaration
//! extraction (`DefinitionProvider`) and reference search
//! (`ReferencesProvider`) instead of re-deriving either from scratch.

use crate::definition::{DefinitionProvider, SymbolDefinition};
use crate::references::ReferencesProvider;
use crbasic_parser::ast::Program;
use crbasic_parser::lexer::token::{Span, Token};
use tower_lsp_server::ls_types::{CodeLens, Command, Location, Position, Range, Uri};

/// Provides Code Lens functionality
pub struct CodeLensProvider;

impl CodeLensProvider {
    /// Builds a "N references" lens for every declared symbol in the program
    ///
    /// # Arguments
    /// * `ast` - The parsed AST program
    /// * `tokens` - The token stream for the document
    /// * `uri` - The document URI
    pub fn get_code_lenses(ast: &Program, tokens: &[Token], uri: &Uri) -> Vec<CodeLens> {
        let mut definitions: Vec<SymbolDefinition> = DefinitionProvider::extract_definitions(ast)
            .into_values()
            .collect();
        definitions
            .sort_by_key(|definition| (definition.span.start.line, definition.span.start.column));

        definitions
            .iter()
            .map(|definition| Self::lens_for_definition(definition, tokens, uri))
            .collect()
    }

    /// Builds the "N references" lens for a single declared symbol
    fn lens_for_definition(definition: &SymbolDefinition, tokens: &[Token], uri: &Uri) -> CodeLens {
        let locations = Self::reference_locations(definition, tokens, uri);
        let count = locations.len();
        let title = format!("{count} reference{}", if count == 1 { "" } else { "s" });
        let range = Self::span_to_range(definition.span);

        CodeLens {
            range,
            command: Some(Command {
                title,
                command: "editor.action.showReferences".to_string(),
                arguments: Some(vec![
                    serde_json::json!(uri),
                    serde_json::json!(range.start),
                    serde_json::json!(locations),
                ]),
            }),
            data: None,
        }
    }

    /// Finds every reference to `definition`'s symbol, excluding the
    /// declaring occurrence itself
    ///
    /// The declaring occurrence is identified the same way
    /// `semantic_tokens.rs` does: the first identifier token matching the
    /// symbol's name on its declaration's source line -- CRBasic requires
    /// declaration before use, so this holds for every real program.
    fn reference_locations(
        definition: &SymbolDefinition,
        tokens: &[Token],
        uri: &Uri,
    ) -> Vec<Location> {
        let declaration_line = definition.span.start.line;
        let mut skipped_declaration = false;

        ReferencesProvider::find_all_references(tokens, &definition.name)
            .into_iter()
            .filter(|span| {
                if !skipped_declaration && span.start.line == declaration_line {
                    skipped_declaration = true;
                    false
                } else {
                    true
                }
            })
            .map(|span| Location {
                uri: uri.clone(),
                range: Self::span_to_range(span),
            })
            .collect()
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
    use crbasic_parser::ast::Statement;
    use crbasic_parser::lexer::token::{Position as ParserPosition, TokenKind};

    fn span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(
            ParserPosition::new(start_line, start_col),
            ParserPosition::new(end_line, end_col),
        )
    }

    fn program(statements: Vec<Statement>) -> Program {
        Program::new(statements, span(1, 1, 1, 1))
    }

    fn public_var(name: &str, line: usize) -> Statement {
        Statement::VarDeclaration {
            keyword: "Public".to_string(),
            name: name.to_string(),
            array_dimensions: None,
            type_annotation: None,
            type_size: None,
            initializer: None,
            span: span(line, 1, line, 8 + name.len()),
        }
    }

    fn identifier_token(name: &str, line: usize, start_col: usize) -> Token<'_> {
        let end_col = start_col + name.len();
        Token {
            kind: TokenKind::Identifier(name),
            lexeme: name,
            span: span(line, start_col, line, end_col),
        }
    }

    fn uri() -> Uri {
        "file:///test.cr6".parse::<Uri>().expect("Valid URL")
    }

    fn command_title(lens: &CodeLens) -> &str {
        lens.command
            .as_ref()
            .expect("Should have a command")
            .title
            .as_str()
    }

    mod get_code_lenses {
        use super::*;

        #[test]
        fn builds_one_lens_per_declared_symbol() {
            let program = program(vec![public_var("Temp_C", 1)]);
            let tokens = vec![identifier_token("Temp_C", 1, 8)];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(lenses.len(), 1);
        }

        #[test]
        fn counts_references_excluding_the_declaration() {
            let program = program(vec![public_var("Temp_C", 1)]);
            let tokens = vec![
                identifier_token("Temp_C", 1, 8), // declaration
                identifier_token("Temp_C", 2, 1), // usage
                identifier_token("Temp_C", 3, 1), // usage
            ];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(command_title(&lenses[0]), "2 references");
        }

        #[test]
        fn uses_singular_wording_for_exactly_one_reference() {
            let program = program(vec![public_var("Temp_C", 1)]);
            let tokens = vec![
                identifier_token("Temp_C", 1, 8),
                identifier_token("Temp_C", 2, 1),
            ];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(command_title(&lenses[0]), "1 reference");
        }

        #[test]
        fn shows_zero_references_for_an_unused_symbol() {
            let program = program(vec![public_var("Temp_C", 1)]);
            let tokens = vec![identifier_token("Temp_C", 1, 8)];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(command_title(&lenses[0]), "0 references");
        }

        #[test]
        fn lens_range_matches_the_declaration_span() {
            let program = program(vec![public_var("Temp_C", 5)]);
            let tokens = vec![identifier_token("Temp_C", 5, 8)];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(lenses[0].range.start.line, 4);
        }

        #[test]
        fn includes_a_lens_for_function_definitions() {
            let program = program(vec![Statement::FunctionDefinition {
                name: "Calc".to_string(),
                parameters: Vec::new(),
                body: Vec::new(),
                span: span(1, 1, 3, 11),
            }]);
            let tokens = vec![identifier_token("Calc", 1, 10)];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(lenses.len(), 1);
        }

        #[test]
        fn includes_a_lens_for_subroutine_definitions() {
            let program = program(vec![Statement::SubroutineDefinition {
                name: "Init".to_string(),
                parameters: Vec::new(),
                body: Vec::new(),
                span: span(1, 1, 3, 7),
            }]);
            let tokens = vec![identifier_token("Init", 1, 5)];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            assert_eq!(lenses.len(), 1);
        }

        #[test]
        fn command_arguments_carry_the_given_uri() {
            let program = program(vec![public_var("Temp_C", 1)]);
            let tokens = vec![
                identifier_token("Temp_C", 1, 8),
                identifier_token("Temp_C", 2, 1),
            ];
            let doc_uri = uri();

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &doc_uri);

            let command = lenses[0].command.as_ref().expect("Should have a command");
            let arguments = command.arguments.as_ref().expect("Should have arguments");
            let serialized_uri: serde_json::Value = serde_json::json!(doc_uri);
            assert_eq!(arguments[0], serialized_uri);
        }

        #[test]
        fn command_locations_exclude_the_declaration_occurrence() {
            let program = program(vec![public_var("Temp_C", 1)]);
            let tokens = vec![
                identifier_token("Temp_C", 1, 8),
                identifier_token("Temp_C", 2, 1),
            ];

            let lenses = CodeLensProvider::get_code_lenses(&program, &tokens, &uri());

            let command = lenses[0].command.as_ref().expect("Should have a command");
            let arguments = command.arguments.as_ref().expect("Should have arguments");
            let locations = arguments[2]
                .as_array()
                .expect("Third argument should be an array");
            assert_eq!(locations.len(), 1);
        }
    }
}
