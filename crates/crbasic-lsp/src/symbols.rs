//! Symbol extraction for document outline
//!
//! This module extracts symbols from the AST to provide document outline
//! functionality (VSCode's outline view).

use crbasic_parser::ast::{Program, Statement};
use crbasic_parser::lexer::token::Position;
use tower_lsp::lsp_types::{DocumentSymbol, Range, SymbolKind};

/// Extracts document symbols from a parsed program
///
/// # Arguments
/// * `program` - The parsed AST program
///
/// # Returns
/// A vector of DocumentSymbol representing the program structure
pub fn extract_document_symbols(program: &Program) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for statement in &program.statements {
        if let Some(symbol) = extract_symbol(statement) {
            symbols.push(symbol);
        }
    }

    symbols
}

/// Extracts a single symbol from a statement
fn extract_symbol(statement: &Statement) -> Option<DocumentSymbol> {
    match statement {
        Statement::ProgramStructure {
            keyword,
            arguments,
            span,
        } => {
            // BeginProg, EndProg, DataTable, EndTable, etc.
            let name = if keyword == "DataTable" {
                // Extract table name from arguments if available
                if let Some(args) = arguments {
                    if !args.is_empty() {
                        format!("DataTable({})", format_args(args))
                    } else {
                        keyword.clone()
                    }
                } else {
                    keyword.clone()
                }
            } else {
                keyword.clone()
            };

            Some(create_symbol(
                name,
                SymbolKind::NAMESPACE,
                *span,
                *span,
                Vec::new(),
            ))
        }
        Statement::FunctionDefinition {
            name,
            parameters,
            body,
            span,
        } => {
            let full_name = if parameters.is_empty() {
                format!("{}()", name)
            } else {
                format!("{}({})", name, parameters.join(", "))
            };

            let children = body.iter().filter_map(extract_symbol).collect::<Vec<_>>();

            Some(create_symbol(
                full_name,
                SymbolKind::FUNCTION,
                *span,
                *span,
                children,
            ))
        }
        Statement::SubroutineDefinition {
            name,
            parameters,
            body,
            span,
        } => {
            let full_name = if parameters.is_empty() {
                format!("{}()", name)
            } else {
                format!("{}({})", name, parameters.join(", "))
            };

            let children = body.iter().filter_map(extract_symbol).collect::<Vec<_>>();

            Some(create_symbol(
                full_name,
                SymbolKind::METHOD,
                *span,
                *span,
                children,
            ))
        }
        Statement::VarDeclaration {
            keyword,
            name,
            type_annotation,
            span,
            ..
        } => {
            let full_name = if let Some(type_ann) = type_annotation {
                format!("{} As {}", name, type_ann)
            } else {
                name.clone()
            };

            let kind = match keyword.as_str() {
                "Const" => SymbolKind::CONSTANT,
                _ => SymbolKind::VARIABLE,
            };

            Some(create_symbol(full_name, kind, *span, *span, Vec::new()))
        }
        _ => None, // Other statements don't contribute to document symbols
    }
}

/// Creates a DocumentSymbol with LSP Position conversion
fn create_symbol(
    name: String,
    kind: SymbolKind,
    span: crbasic_parser::lexer::token::Span,
    selection_span: crbasic_parser::lexer::token::Span,
    children: Vec<DocumentSymbol>,
) -> DocumentSymbol {
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range: span_to_range(span),
        selection_range: span_to_range(selection_span),
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
    }
}

/// Converts parser Span to LSP Range (with 0-indexed positions)
fn span_to_range(span: crbasic_parser::lexer::token::Span) -> Range {
    Range {
        start: position_to_lsp(span.start),
        end: position_to_lsp(span.end),
    }
}

/// Converts parser Position (1-indexed) to LSP Position (0-indexed)
fn position_to_lsp(pos: Position) -> tower_lsp::lsp_types::Position {
    tower_lsp::lsp_types::Position {
        line: pos.line.saturating_sub(1) as u32,
        character: pos.column.saturating_sub(1) as u32,
    }
}

/// Formats expression arguments for display (simplified)
fn format_args(args: &[crbasic_parser::ast::Expression]) -> String {
    use crbasic_parser::ast::Expression;

    args.iter()
        .map(|arg| match arg {
            Expression::IntegerLiteral { value, .. } => value.to_string(),
            Expression::FloatLiteral { value, .. } => value.to_string(),
            Expression::StringLiteral { value, .. } => format!("\"{}\"", value),
            Expression::Identifier { name, .. } => name.clone(),
            _ => "...".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::Parser;

    fn parse(source: &str) -> Program {
        let mut scanner = crbasic_parser::lexer::Scanner::new(source.to_string());
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        parser.parse().expect("Parse should succeed")
    }

    #[test]
    fn extracts_begin_prog_symbol() {
        let program = parse("BeginProg\nEndProg");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "BeginProg");
        assert_eq!(symbols[0].kind, SymbolKind::NAMESPACE);
    }

    #[test]
    fn extracts_data_table_symbol() {
        let program = parse("DataTable\nEndTable");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "DataTable");
        assert_eq!(symbols[0].kind, SymbolKind::NAMESPACE);
        assert_eq!(symbols[1].name, "EndTable");
    }

    #[test]
    fn extracts_function_definition() {
        let program = parse("Function MyFunc()\nEndFunction");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyFunc()");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn extracts_function_with_parameters() {
        let program = parse("Function Calculate(x, y)\nEndFunction");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Calculate(x, y)");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn extracts_subroutine_definition() {
        let program = parse("Sub MySub(param1)\nEndSub");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MySub(param1)");
        assert_eq!(symbols[0].kind, SymbolKind::METHOD);
    }

    #[test]
    fn extracts_variable_declarations() {
        let program = parse("Public Temp_C\nDim i\nConst PI = 3.14");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 3);

        // Public variable
        assert_eq!(symbols[0].name, "Temp_C");
        assert_eq!(symbols[0].kind, SymbolKind::VARIABLE);

        // Dim variable
        assert_eq!(symbols[1].name, "i");
        assert_eq!(symbols[1].kind, SymbolKind::VARIABLE);

        // Const
        assert_eq!(symbols[2].name, "PI");
        assert_eq!(symbols[2].kind, SymbolKind::CONSTANT);
    }

    #[test]
    fn extracts_variable_with_type_annotation() {
        let program = parse("Public Temp_C As Float");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Temp_C As Float");
    }

    #[test]
    fn extracts_nested_symbols_in_function() {
        let program = parse("Function MyFunc()\nDim local_var\nEndFunction");
        let symbols = extract_document_symbols(&program);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyFunc()");

        // Check for nested symbol
        assert!(symbols[0].children.is_some());
        let children = symbols[0].children.as_ref().expect("Should have children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "local_var");
    }

    #[test]
    fn position_conversion_is_zero_indexed() {
        // Parser uses 1-indexed, LSP uses 0-indexed
        let parser_pos = Position::new(5, 10);
        let lsp_pos = position_to_lsp(parser_pos);

        assert_eq!(lsp_pos.line, 4);
        assert_eq!(lsp_pos.character, 9);
    }
}
