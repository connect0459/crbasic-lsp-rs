//! Workspace symbol provider for CRBasic LSP
//!
//! Answers `workspace/symbol` by reusing the existing `textDocument/documentSymbol`
//! extraction (`symbols::extract_document_symbols`) across every open document,
//! flattening each document's nested symbol tree and attaching that document's
//! URI so results can point back to the right file.
//!
//! Search is scoped to currently open documents, not the whole project on
//! disk -- this server has no workspace file-indexing infrastructure, only
//! the documents the client has opened (see `DocumentManager`).

use crate::symbols;
use crbasic_parser::ast::Program;
use tower_lsp_server::ls_types::{DocumentSymbol, Location, SymbolInformation, Uri};

/// Provides Workspace Symbol functionality
pub struct WorkspaceSymbolProvider;

impl WorkspaceSymbolProvider {
    /// Searches every given document's symbols for a case-insensitive
    /// substring match against `query`
    ///
    /// An empty `query` matches every symbol, per the common convention for
    /// "list all symbols" workspace symbol requests.
    ///
    /// # Arguments
    /// * `documents` - Each open document's URI paired with its cached AST
    /// * `query` - The search string
    pub fn search<'a>(
        documents: impl Iterator<Item = (&'a Uri, &'a Program)>,
        query: &str,
    ) -> Vec<SymbolInformation> {
        let query_lower = query.to_lowercase();

        documents
            .flat_map(|(uri, program)| {
                let document_symbols = symbols::extract_document_symbols(program);
                Self::flatten(uri, &document_symbols, &query_lower)
            })
            .collect()
    }

    /// Recursively flattens a document's nested symbol tree into matching
    /// [`SymbolInformation`] entries, attaching `uri` to each
    fn flatten(
        uri: &Uri,
        document_symbols: &[DocumentSymbol],
        query_lower: &str,
    ) -> Vec<SymbolInformation> {
        let mut results = Vec::new();

        for symbol in document_symbols {
            if symbol.name.to_lowercase().contains(query_lower) {
                #[allow(deprecated)]
                results.push(SymbolInformation {
                    name: symbol.name.clone(),
                    kind: symbol.kind,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range: symbol.range,
                    },
                    container_name: None,
                });
            }

            if let Some(children) = &symbol.children {
                results.extend(Self::flatten(uri, children, query_lower));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::ast::Statement;
    use crbasic_parser::lexer::token::{Position, Span};

    fn span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(
            Position::new(start_line, start_col),
            Position::new(end_line, end_col),
        )
    }

    fn program(statements: Vec<Statement>) -> Program {
        Program::new(statements, span(1, 1, 1, 1))
    }

    fn public_var(name: &str) -> Statement {
        Statement::VarDeclaration {
            keyword: "Public".to_string(),
            name: name.to_string(),
            array_dimensions: None,
            type_annotation: None,
            type_size: None,
            initializer: None,
            span: span(1, 1, 1, 10),
        }
    }

    fn uri(name: &str) -> Uri {
        format!("file:///{name}.cr6")
            .parse::<Uri>()
            .expect("Valid URL")
    }

    mod search {
        use super::*;

        #[test]
        fn finds_a_symbol_matching_the_query() {
            let doc_uri = uri("a");
            let program = program(vec![public_var("Temp_C")]);

            let results =
                WorkspaceSymbolProvider::search([(&doc_uri, &program)].into_iter(), "Temp");

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "Temp_C");
        }

        #[test]
        fn matches_case_insensitively() {
            let doc_uri = uri("a");
            let program = program(vec![public_var("Temp_C")]);

            let results =
                WorkspaceSymbolProvider::search([(&doc_uri, &program)].into_iter(), "temp");

            assert_eq!(results.len(), 1);
        }

        #[test]
        fn excludes_symbols_that_do_not_match() {
            let doc_uri = uri("a");
            let program = program(vec![public_var("Temp_C")]);

            let results =
                WorkspaceSymbolProvider::search([(&doc_uri, &program)].into_iter(), "Humidity");

            assert!(results.is_empty());
        }

        #[test]
        fn empty_query_returns_every_symbol() {
            let doc_uri = uri("a");
            let program = program(vec![public_var("Temp_C"), public_var("Humidity")]);

            let results = WorkspaceSymbolProvider::search([(&doc_uri, &program)].into_iter(), "");

            assert_eq!(results.len(), 2);
        }

        #[test]
        fn searches_across_multiple_documents() {
            let uri_a = uri("a");
            let uri_b = uri("b");
            let program_a = program(vec![public_var("Temp_C")]);
            let program_b = program(vec![public_var("Temp_F")]);

            let results = WorkspaceSymbolProvider::search(
                [(&uri_a, &program_a), (&uri_b, &program_b)].into_iter(),
                "Temp",
            );

            assert_eq!(results.len(), 2);
        }

        #[test]
        fn results_carry_the_owning_documents_uri() {
            let doc_uri = uri("a");
            let program = program(vec![public_var("Temp_C")]);

            let results =
                WorkspaceSymbolProvider::search([(&doc_uri, &program)].into_iter(), "Temp");

            assert_eq!(results[0].location.uri, doc_uri);
        }

        #[test]
        fn finds_symbols_nested_inside_a_function_body() {
            let doc_uri = uri("a");
            let program = program(vec![Statement::FunctionDefinition {
                name: "Calc".to_string(),
                parameters: Vec::new(),
                body: vec![public_var("Result")],
                span: span(1, 1, 3, 1),
            }]);

            let results =
                WorkspaceSymbolProvider::search([(&doc_uri, &program)].into_iter(), "Result");

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "Result");
        }
    }
}
