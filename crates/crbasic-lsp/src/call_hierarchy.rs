//! Call hierarchy provider for CRBasic LSP
//!
//! Answers `textDocument/prepareCallHierarchy` and the two follow-up
//! requests (`callHierarchy/incomingCalls`, `callHierarchy/outgoingCalls`)
//! by combining the existing declaration extraction (`DefinitionProvider`)
//! with the shared call-site walker (`call_sites::collect_call_sites`).
//!
//! Search is scoped to currently open documents -- the same limitation
//! `workspace_symbol.rs` documents: this server has no workspace
//! file-indexing infrastructure beyond `DocumentManager`.
//!
//! Only calls made from inside a named `Function`/`Sub` are represented as
//! incoming calls. A call made directly from the main program body (inside
//! `BeginProg`/`EndProg`) has no enclosing callable symbol to attribute it
//! to, so it is left out rather than invented -- this matches the
//! conventional definition of call hierarchy as edges between named
//! callable symbols, the same way most language servers don't synthesize a
//! "<module>" node for top-level script code.

use crate::call_sites::collect_call_sites;
use crate::definition::{DefinitionProvider, SymbolDefinition, SymbolKind as DefinitionSymbolKind};
use crate::references::ReferencesProvider;
use crbasic_parser::ast::{Program, Statement};
use crbasic_parser::lexer::token::{Span, Token};
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, Position, Range,
    SymbolKind, Url,
};

/// Provides Call Hierarchy functionality
pub struct CallHierarchyProvider;

impl CallHierarchyProvider {
    /// Resolves the callable symbol at `position`, if any, to the item that
    /// starts a call hierarchy session
    ///
    /// # Arguments
    /// * `tokens` - The token stream for the document
    /// * `ast` - The parsed AST program
    /// * `uri` - The document URI
    /// * `position` - The cursor position (LSP 0-indexed)
    pub fn prepare(
        tokens: &[Token],
        ast: &Program,
        uri: &Url,
        position: Position,
    ) -> Option<CallHierarchyItem> {
        let name = ReferencesProvider::find_identifier_at_position(tokens, position)?;
        let definitions = DefinitionProvider::extract_definitions(ast);
        let definition = definitions.get(&name)?;
        Self::item_for_definition(definition, uri)
    }

    /// Finds every call made to `item`'s symbol from another named
    /// `Function`/`Sub`, across every given document
    pub fn incoming_calls<'a>(
        item: &CallHierarchyItem,
        documents: impl Iterator<Item = (&'a Url, &'a Program)>,
    ) -> Vec<CallHierarchyIncomingCall> {
        let mut results = Vec::new();

        for (uri, program) in documents {
            let definitions = DefinitionProvider::extract_definitions(program);
            let sites = collect_call_sites(&program.statements);

            let mut ranges_by_caller: HashMap<&str, Vec<Range>> = HashMap::new();
            for site in &sites {
                if site.name != item.name {
                    continue;
                }
                let Some(caller_name) = site.enclosing else {
                    continue; // Top-level calls have no enclosing symbol to attribute to
                };
                ranges_by_caller
                    .entry(caller_name)
                    .or_default()
                    .push(Self::span_to_range(site.span));
            }

            for (caller_name, ranges) in ranges_by_caller {
                let Some(caller_definition) = definitions.get(caller_name) else {
                    continue;
                };
                let Some(from) = Self::item_for_definition(caller_definition, uri) else {
                    continue;
                };
                results.push(CallHierarchyIncomingCall {
                    from,
                    from_ranges: ranges,
                });
            }
        }

        results
    }

    /// Finds every call `item`'s symbol makes to another user-defined
    /// `Function`/`Sub`, searching the given documents for each callee's
    /// declaration
    pub fn outgoing_calls<'a>(
        item: &CallHierarchyItem,
        documents: impl Iterator<Item = (&'a Url, &'a Program)>,
    ) -> Vec<CallHierarchyOutgoingCall> {
        let documents: Vec<(&Url, &Program)> = documents.collect();

        let Some(body) = Self::find_body(&item.name, &documents) else {
            return Vec::new();
        };

        let mut ranges_by_callee: HashMap<&str, Vec<Range>> = HashMap::new();
        for site in collect_call_sites(body) {
            ranges_by_callee
                .entry(site.name)
                .or_default()
                .push(Self::span_to_range(site.span));
        }

        let mut results = Vec::new();
        for (callee_name, ranges) in ranges_by_callee {
            let Some((callee_uri, callee_definition)) =
                documents.iter().find_map(|(doc_uri, doc_program)| {
                    DefinitionProvider::extract_definitions(doc_program)
                        .get(callee_name)
                        .cloned()
                        .map(|definition| (*doc_uri, definition))
                })
            else {
                continue; // Not a known user-defined Function/Sub (e.g. a built-in)
            };
            let Some(to) = Self::item_for_definition(&callee_definition, callee_uri) else {
                continue;
            };
            results.push(CallHierarchyOutgoingCall {
                to,
                from_ranges: ranges,
            });
        }

        results
    }

    /// Finds the body of the `Function`/`Sub` named `name` across `documents`
    fn find_body<'a>(name: &str, documents: &[(&'a Url, &'a Program)]) -> Option<&'a [Statement]> {
        documents.iter().find_map(|(_, program)| {
            program
                .statements
                .iter()
                .find_map(|statement| match statement {
                    Statement::FunctionDefinition {
                        name: def_name,
                        body,
                        ..
                    }
                    | Statement::SubroutineDefinition {
                        name: def_name,
                        body,
                        ..
                    } if def_name == name => Some(body.as_slice()),
                    _ => None,
                })
        })
    }

    /// Builds a [`CallHierarchyItem`] for a declaration, or `None` if it
    /// isn't callable (only `Function`/`Sub` declarations have a place in a
    /// call hierarchy)
    fn item_for_definition(definition: &SymbolDefinition, uri: &Url) -> Option<CallHierarchyItem> {
        let kind = match definition.kind {
            DefinitionSymbolKind::Function => SymbolKind::FUNCTION,
            DefinitionSymbolKind::Subroutine => SymbolKind::METHOD,
            DefinitionSymbolKind::Variable => return None,
        };
        let range = Self::span_to_range(definition.span);

        Some(CallHierarchyItem {
            name: definition.name.clone(),
            kind,
            tags: None,
            detail: None,
            uri: uri.clone(),
            range,
            selection_range: range,
            data: None,
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

    fn identifier_token(name: &str, line: usize, start_col: usize) -> Token<'_> {
        let end_col = start_col + name.len();
        Token {
            kind: TokenKind::Identifier(name),
            lexeme: name,
            span: span(line, start_col, line, end_col),
        }
    }

    fn call_statement(name: &str, line: usize) -> Statement {
        Statement::FunctionCall {
            name: name.to_string(),
            arguments: Vec::new(),
            span: span(line, 1, line, 1 + name.len() + 2),
        }
    }

    fn function_def(
        name: &str,
        body: Vec<Statement>,
        start_line: usize,
        end_line: usize,
    ) -> Statement {
        Statement::FunctionDefinition {
            name: name.to_string(),
            parameters: Vec::new(),
            body,
            span: span(start_line, 1, end_line, 1 + "EndFunction".len()),
        }
    }

    fn subroutine_def(
        name: &str,
        body: Vec<Statement>,
        start_line: usize,
        end_line: usize,
    ) -> Statement {
        Statement::SubroutineDefinition {
            name: name.to_string(),
            parameters: Vec::new(),
            body,
            span: span(start_line, 1, end_line, 1 + "EndSub".len()),
        }
    }

    fn uri(name: &str) -> Url {
        Url::parse(&format!("file:///{name}.cr6")).expect("Valid URL")
    }

    mod prepare {
        use super::*;

        #[test]
        fn resolves_a_function_declaration_at_its_own_position() {
            let program = program(vec![function_def("Calc", Vec::new(), 1, 3)]);
            let tokens = vec![identifier_token("Calc", 1, 10)];
            let position = Position {
                line: 0,
                character: 10,
            };

            let item = CallHierarchyProvider::prepare(&tokens, &program, &uri("a"), position)
                .expect("Should resolve to a call hierarchy item");

            assert_eq!(item.name, "Calc");
            assert_eq!(item.kind, SymbolKind::FUNCTION);
        }

        #[test]
        fn resolves_a_subroutine_declaration_to_the_method_kind() {
            let program = program(vec![subroutine_def("Init", Vec::new(), 1, 3)]);
            let tokens = vec![identifier_token("Init", 1, 5)];
            let position = Position {
                line: 0,
                character: 5,
            };

            let item = CallHierarchyProvider::prepare(&tokens, &program, &uri("a"), position)
                .expect("Should resolve to a call hierarchy item");

            assert_eq!(item.kind, SymbolKind::METHOD);
        }

        #[test]
        fn resolves_a_reference_to_its_declaration() {
            let program = program(vec![
                function_def("Calc", Vec::new(), 1, 3),
                call_statement("Calc", 5),
            ]);
            let tokens = vec![identifier_token("Calc", 5, 1)];
            let position = Position {
                line: 4,
                character: 1,
            };

            let item = CallHierarchyProvider::prepare(&tokens, &program, &uri("a"), position)
                .expect("Should resolve to a call hierarchy item");

            assert_eq!(item.range.start.line, 0); // Points at the declaration, not the call site
        }

        #[test]
        fn returns_none_for_a_variable() {
            let program = program(vec![Statement::VarDeclaration {
                keyword: "Public".to_string(),
                name: "Temp_C".to_string(),
                array_dimensions: None,
                type_annotation: None,
                initializer: None,
                span: span(1, 1, 1, 15),
            }]);
            let tokens = vec![identifier_token("Temp_C", 1, 8)];
            let position = Position {
                line: 0,
                character: 8,
            };

            let item = CallHierarchyProvider::prepare(&tokens, &program, &uri("a"), position);

            assert!(item.is_none());
        }

        #[test]
        fn returns_none_when_not_on_an_identifier() {
            let program = program(vec![function_def("Calc", Vec::new(), 1, 3)]);
            let tokens: Vec<Token<'static>> = Vec::new();
            let position = Position {
                line: 0,
                character: 0,
            };

            let item = CallHierarchyProvider::prepare(&tokens, &program, &uri("a"), position);

            assert!(item.is_none());
        }
    }

    mod incoming_calls {
        use super::*;

        fn function_item(name: &str, doc_uri: Url) -> CallHierarchyItem {
            CallHierarchyItem {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: None,
                uri: doc_uri,
                range: Range::default(),
                selection_range: Range::default(),
                data: None,
            }
        }

        #[test]
        fn finds_a_caller_that_is_a_named_function() {
            let program = program(vec![
                function_def("Caller", vec![call_statement("Calc", 2)], 1, 3),
                function_def("Calc", Vec::new(), 5, 7),
            ]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::incoming_calls(
                &function_item("Calc", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].from.name, "Caller");
        }

        #[test]
        fn groups_multiple_calls_from_the_same_caller_into_one_entry() {
            let program = program(vec![function_def(
                "Caller",
                vec![call_statement("Calc", 2), call_statement("Calc", 3)],
                1,
                4,
            )]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::incoming_calls(
                &function_item("Calc", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].from_ranges.len(), 2);
        }

        #[test]
        fn excludes_calls_made_directly_from_the_top_level_program() {
            let program = program(vec![call_statement("Calc", 1)]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::incoming_calls(
                &function_item("Calc", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert!(calls.is_empty());
        }

        #[test]
        fn ignores_calls_to_a_different_function() {
            let program = program(vec![function_def(
                "Caller",
                vec![call_statement("SomethingElse", 2)],
                1,
                3,
            )]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::incoming_calls(
                &function_item("Calc", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert!(calls.is_empty());
        }
    }

    mod outgoing_calls {
        use super::*;

        fn function_item(name: &str, doc_uri: Url) -> CallHierarchyItem {
            CallHierarchyItem {
                name: name.to_string(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                detail: None,
                uri: doc_uri,
                range: Range::default(),
                selection_range: Range::default(),
                data: None,
            }
        }

        #[test]
        fn finds_a_callee_that_is_a_user_defined_function() {
            let program = program(vec![
                function_def("Caller", vec![call_statement("Calc", 2)], 1, 3),
                function_def("Calc", Vec::new(), 5, 7),
            ]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::outgoing_calls(
                &function_item("Caller", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].to.name, "Calc");
        }

        #[test]
        fn skips_calls_to_unrecognized_built_in_functions() {
            let program = program(vec![function_def(
                "Caller",
                vec![call_statement("Scan", 2)],
                1,
                3,
            )]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::outgoing_calls(
                &function_item("Caller", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert!(calls.is_empty());
        }

        #[test]
        fn groups_multiple_calls_to_the_same_callee_into_one_entry() {
            let program = program(vec![
                function_def(
                    "Caller",
                    vec![call_statement("Calc", 2), call_statement("Calc", 3)],
                    1,
                    4,
                ),
                function_def("Calc", Vec::new(), 6, 8),
            ]);
            let doc_uri = uri("a");

            let calls = CallHierarchyProvider::outgoing_calls(
                &function_item("Caller", doc_uri.clone()),
                [(&doc_uri, &program)].into_iter(),
            );

            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].from_ranges.len(), 2);
        }

        #[test]
        fn finds_a_callee_declared_in_a_different_open_document() {
            let caller_program = program(vec![function_def(
                "Caller",
                vec![call_statement("Calc", 2)],
                1,
                3,
            )]);
            let callee_program = program(vec![function_def("Calc", Vec::new(), 1, 3)]);
            let caller_uri = uri("a");
            let callee_uri = uri("b");

            let calls = CallHierarchyProvider::outgoing_calls(
                &function_item("Caller", caller_uri.clone()),
                [
                    (&caller_uri, &caller_program),
                    (&callee_uri, &callee_program),
                ]
                .into_iter(),
            );

            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].to.uri, callee_uri);
        }
    }
}
