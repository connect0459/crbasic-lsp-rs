//! LSP Integration Tests
//!
//! These tests verify the end-to-end behavior of the LSP server,
//! including document synchronization, diagnostics, and language features.

use crbasic_lsp::CRBasicLanguageServer;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{LanguageServer, LspService};

/// Test helper to create an LSP service for testing
async fn create_test_server() -> (
    LspService<CRBasicLanguageServer>,
    tower_lsp_server::ClientSocket,
) {
    CRBasicLanguageServer::new_service()
}

/// Helper to create a test document URI
fn test_uri(name: &str) -> Uri {
    format!("file:///test/{}", name)
        .parse::<Uri>()
        .expect("Failed to create test URI")
}

mod document_synchronization {
    use super::*;

    #[tokio::test]
    async fn opens_and_tracks_document() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nEndProg".to_string(),
            },
        };

        // This should not panic
        service.inner().did_open(params).await;
    }

    #[tokio::test]
    async fn updates_document_content() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            }],
        };

        service.inner().did_change(change_params).await;
    }

    #[tokio::test]
    async fn closes_document() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let close_params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };

        service.inner().did_close(close_params).await;
    }
}

mod diagnostics {
    use super::*;

    #[tokio::test]
    async fn publishes_diagnostics_for_valid_program() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            },
        };

        // Should publish diagnostics (none for valid program)
        service.inner().did_open(params).await;
    }

    #[tokio::test]
    async fn publishes_error_diagnostics_for_invalid_syntax() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic\nEndProg".to_string(), // Missing variable name
            },
        };

        service.inner().did_open(params).await;
    }

    #[tokio::test]
    async fn publishes_warning_for_cr200x_truncation() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR1"); // CR200X model

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic VeryLongVariableName\nEndProg".to_string(),
            },
        };

        service.inner().did_open(params).await;
    }
}

mod completion {
    use super::*;

    #[tokio::test]
    async fn provides_keyword_completions() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\n\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let completion_params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        let result = service.inner().completion(completion_params).await;
        assert!(result.is_ok(), "Completion should succeed");

        if let Ok(Some(CompletionResponse::Array(items))) = result {
            let has_public = items.iter().any(|item| item.label == "Public");
            let has_if = items.iter().any(|item| item.label == "If");
            assert!(has_public, "Should include 'Public' keyword");
            assert!(has_if, "Should include 'If' keyword");
            assert!(!items.is_empty(), "Should return completion items");
        } else {
            panic!("Expected completion array");
        }
    }

    #[tokio::test]
    async fn provides_user_defined_variable_completions() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\n\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let completion_params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        let result = service.inner().completion(completion_params).await;
        assert!(result.is_ok(), "Completion should succeed");

        if let Ok(Some(CompletionResponse::Array(items))) = result {
            let has_temp = items.iter().any(|item| item.label == "Temp");
            assert!(has_temp, "Should include user-defined variable 'Temp'");
        } else {
            panic!("Expected completion array");
        }
    }

    #[tokio::test]
    async fn provides_pattern_snippet_completions() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\n\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let completion_params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        let result = service.inner().completion(completion_params).await;
        assert!(result.is_ok(), "Completion should succeed");

        if let Ok(Some(CompletionResponse::Array(items))) = result {
            let has_scan_loop = items.iter().any(|item| item.label == "ScanLoop");
            let has_new_program = items.iter().any(|item| item.label == "NewProgram");
            assert!(has_scan_loop, "Should include 'ScanLoop' pattern snippet");
            assert!(
                has_new_program,
                "Should include 'NewProgram' pattern snippet"
            );
        } else {
            panic!("Expected completion array");
        }
    }
}

mod hover {
    use super::*;

    #[tokio::test]
    async fn provides_hover_for_keywords() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let hover_params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = service.inner().hover(hover_params).await;
        assert!(result.is_ok(), "Hover should succeed");

        if let Ok(Some(hover)) = result {
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(
                        markup.value.contains("BeginProg"),
                        "Hover should contain BeginProg information"
                    );
                }
                HoverContents::Scalar(marked_string) => match marked_string {
                    MarkedString::String(s) => {
                        assert!(
                            s.contains("BeginProg"),
                            "Hover should contain BeginProg information"
                        );
                    }
                    MarkedString::LanguageString(ls) => {
                        assert!(
                            ls.value.contains("BeginProg"),
                            "Hover should contain BeginProg information"
                        );
                    }
                },
                _ => panic!("Unexpected hover contents type"),
            }
        } else {
            panic!("Expected hover result");
        }
    }
}

mod signature_help {
    use super::*;

    #[tokio::test]
    async fn provides_signature_for_builtin_functions() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nScan(".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let sig_params = SignatureHelpParams {
            context: None,
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 5,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = service.inner().signature_help(sig_params).await;
        assert!(result.is_ok(), "Signature help should succeed");

        if let Ok(Some(sig_help)) = result {
            assert!(
                !sig_help.signatures.is_empty(),
                "Should return at least one signature"
            );
            let scan_sig = sig_help
                .signatures
                .iter()
                .any(|sig| sig.label.contains("Scan"));
            assert!(scan_sig, "Should include Scan function signature");
        } else {
            panic!("Expected signature help result");
        }
    }
}

mod goto_definition {
    use super::*;

    #[tokio::test]
    async fn jumps_to_variable_declaration() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let def_params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().goto_definition(def_params).await;
        assert!(result.is_ok(), "Go to definition should succeed");

        if let Ok(Some(GotoDefinitionResponse::Scalar(location))) = result {
            assert_eq!(location.uri, uri, "Definition should be in the same file");
            assert_eq!(
                location.range.start.line, 1,
                "Definition should point to line 1 (Public Temp)"
            );
        } else {
            panic!("Expected goto definition result");
        }
    }
}

mod find_references {
    use super::*;

    #[tokio::test]
    async fn finds_all_variable_references() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nTemp = Temp + 1\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let ref_params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 7,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        };

        let result = service.inner().references(ref_params).await;
        assert!(result.is_ok(), "Find references should succeed");

        if let Ok(Some(locations)) = result {
            assert!(
                locations.len() >= 3,
                "Should find at least 3 references to Temp (found {})",
                locations.len()
            );
        } else {
            panic!("Expected references result");
        }
    }
}

mod document_highlight {
    use super::*;

    #[tokio::test]
    async fn highlights_all_occurrences_of_a_variable() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nTemp = Temp + 1\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let highlight_params = DocumentHighlightParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 7,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().document_highlight(highlight_params).await;
        assert!(result.is_ok(), "Document highlight should succeed");

        if let Ok(Some(highlights)) = result {
            assert!(
                highlights.len() >= 3,
                "Should highlight at least 3 occurrences of Temp (found {})",
                highlights.len()
            );
        } else {
            panic!("Expected document highlight result");
        }
    }

    #[tokio::test]
    async fn returns_none_when_cursor_is_not_on_an_identifier() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let highlight_params = DocumentHighlightParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().document_highlight(highlight_params).await;
        assert!(result.is_ok(), "Document highlight should succeed");
        assert_eq!(
            result.expect("Should be Ok"),
            None,
            "Should not highlight anything on the BeginProg keyword"
        );
    }
}

mod selection_range {
    use super::*;

    #[tokio::test]
    async fn expands_from_the_identifier_to_the_enclosing_statement_to_the_whole_program() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let selection_params = SelectionRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            positions: vec![Position {
                line: 2,
                character: 0,
            }],
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().selection_range(selection_params).await;
        assert!(result.is_ok(), "Selection range should succeed");

        let ranges = result
            .expect("Should be Ok")
            .expect("Should return selection ranges");
        assert_eq!(ranges.len(), 1);

        let innermost = &ranges[0];
        assert_eq!(innermost.range.start.line, 2);
        let statement_level = innermost
            .parent
            .as_ref()
            .expect("identifier's parent should be the assignment statement");
        assert_eq!(statement_level.range.start.line, 2);
        let mut outermost = statement_level;
        while let Some(parent) = &outermost.parent {
            outermost = parent;
        }
        assert_eq!(
            outermost.range.start.line, 0,
            "outermost range should span the whole program, starting at BeginProg"
        );
    }

    #[tokio::test]
    async fn returns_none_for_a_document_without_a_parsed_ast() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let selection_params = SelectionRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            positions: vec![Position {
                line: 0,
                character: 0,
            }],
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().selection_range(selection_params).await;
        assert!(result.is_ok(), "Selection range should succeed");
        assert_eq!(result.expect("Should be Ok"), None);
    }
}

mod code_action {
    use super::*;

    #[tokio::test]
    async fn offers_a_quick_fix_that_truncates_an_over_length_variable_name() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR1"); // CR200X model: 16 char max

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic VeryLongVariableName\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        // Mirrors the data shape the server embeds when it first publishes
        // this diagnostic (see backend::CRBasicLanguageServer::code_action_data)
        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 7,
                },
                end: Position {
                    line: 1,
                    character: 27,
                },
            },
            code: Some(NumberOrString::String("truncate-variable-name".to_string())),
            data: Some(serde_json::json!({
                "variableName": "VeryLongVariableName",
                "targetLength": 16,
            })),
            ..Default::default()
        };

        let action_params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: diagnostic.range,
            context: CodeActionContext {
                diagnostics: vec![diagnostic],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().code_action(action_params).await;
        assert!(result.is_ok(), "Code action should succeed");

        let actions = result
            .expect("Should be Ok")
            .expect("Should offer a quick fix");
        assert_eq!(actions.len(), 1);

        let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
            panic!("Expected a CodeAction");
        };
        let edit = action.edit.as_ref().expect("Should have a workspace edit");
        let changes = edit.changes.as_ref().expect("Should have changes");
        let edits = changes.get(&uri).expect("Should target the document");

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "VeryLongVariable");
    }

    #[tokio::test]
    async fn returns_none_when_no_diagnostics_carry_quick_fix_data() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let action_params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::default(),
            context: CodeActionContext {
                diagnostics: vec![],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().code_action(action_params).await;
        assert!(result.is_ok(), "Code action should succeed");
        assert_eq!(result.expect("Should be Ok"), None);
    }
}

mod folding_range {
    use super::*;

    #[tokio::test]
    async fn folds_program_structure_and_block_statements() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\n\
                       If 1 > 0 Then\n\
                       For i = 1 To 10\n\
                       Next\n\
                       EndIf\n\
                       EndProg"
                    .to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let folding_params = FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().folding_range(folding_params).await;
        assert!(result.is_ok(), "Folding range should succeed");

        let ranges = result
            .expect("Should be Ok")
            .expect("Should return folding ranges");
        // BeginProg/EndProg (lines 0-5), If/EndIf (lines 1-4), For/Next (lines 2-3)
        assert_eq!(ranges.len(), 3);
    }

    #[tokio::test]
    async fn returns_none_for_a_document_without_a_parsed_ast() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let folding_params = FoldingRangeParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().folding_range(folding_params).await;
        assert!(result.is_ok(), "Folding range should succeed");
        assert_eq!(result.expect("Should be Ok"), None);
    }
}

mod workspace_symbol {
    use super::*;

    #[tokio::test]
    async fn finds_matching_symbols_across_open_documents() {
        let (service, _socket) = create_test_server().await;
        let uri_a = test_uri("a.CR6");
        let uri_b = test_uri("b.CR6");

        service
            .inner()
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri_a.clone(),
                    language_id: "crbasic".to_string(),
                    version: 1,
                    text: "BeginProg\nPublic Temp_C\nEndProg".to_string(),
                },
            })
            .await;
        service
            .inner()
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri_b.clone(),
                    language_id: "crbasic".to_string(),
                    version: 1,
                    text: "BeginProg\nPublic Temp_F\nEndProg".to_string(),
                },
            })
            .await;

        let result = service
            .inner()
            .symbol(WorkspaceSymbolParams {
                query: "Temp".to_string(),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .await;
        assert!(result.is_ok(), "Workspace symbol search should succeed");

        let symbols = match result
            .expect("Should be Ok")
            .expect("Should return symbols")
        {
            WorkspaceSymbolResponse::Flat(symbols) => symbols,
            WorkspaceSymbolResponse::Nested(_) => panic!("Expected a flat symbol response"),
        };
        assert_eq!(symbols.len(), 2);
        let uris: Vec<&Uri> = symbols.iter().map(|s| &s.location.uri).collect();
        assert!(uris.contains(&&uri_a));
        assert!(uris.contains(&&uri_b));
    }

    #[tokio::test]
    async fn excludes_symbols_from_documents_that_do_not_match() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        service
            .inner()
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "crbasic".to_string(),
                    version: 1,
                    text: "BeginProg\nPublic Temp_C\nEndProg".to_string(),
                },
            })
            .await;

        let result = service
            .inner()
            .symbol(WorkspaceSymbolParams {
                query: "Humidity".to_string(),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .await;
        assert!(result.is_ok(), "Workspace symbol search should succeed");

        let symbols = match result
            .expect("Should be Ok")
            .expect("Should return an empty list")
        {
            WorkspaceSymbolResponse::Flat(symbols) => symbols,
            WorkspaceSymbolResponse::Nested(_) => panic!("Expected a flat symbol response"),
        };
        assert!(symbols.is_empty());
    }
}

mod inlay_hint {
    use super::*;

    #[tokio::test]
    async fn shows_parameter_names_for_a_builtin_function_call() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nScan(1, Sec, 0)\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let hint_params = InlayHintParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 10,
                    character: 0,
                },
            },
        };

        let result = service.inner().inlay_hint(hint_params).await;
        assert!(result.is_ok(), "Inlay hint should succeed");

        let hints = result.expect("Should be Ok").expect("Should return hints");
        assert_eq!(hints.len(), 3); // Scan(Interval, Units, BufferOption, Count) has 3 args here
    }

    #[tokio::test]
    async fn returns_none_for_a_document_without_a_parsed_ast() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let hint_params = InlayHintParams {
            work_done_progress_params: WorkDoneProgressParams::default(),
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::default(),
        };

        let result = service.inner().inlay_hint(hint_params).await;
        assert!(result.is_ok(), "Inlay hint should succeed");
        assert!(result.expect("Should be Ok").is_none());
    }
}

mod code_lens {
    use super::*;

    #[tokio::test]
    async fn shows_a_reference_count_lens_for_a_declared_variable() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nTemp = Temp + 1\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let lens_params = CodeLensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().code_lens(lens_params).await;
        assert!(result.is_ok(), "Code lens should succeed");

        let lenses = result.expect("Should be Ok").expect("Should return lenses");
        assert_eq!(lenses.len(), 1);
        let command = lenses[0].command.as_ref().expect("Should have a command");
        assert_eq!(command.title, "3 references"); // Temp = 5, Temp = Temp + 1 (twice)
    }

    #[tokio::test]
    async fn returns_none_for_a_document_without_a_parsed_ast() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let lens_params = CodeLensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().code_lens(lens_params).await;
        assert!(result.is_ok(), "Code lens should succeed");
        assert!(result.expect("Should be Ok").is_none());
    }
}

mod call_hierarchy {
    use super::*;

    async fn open_program_with_a_caller_and_callee(
        service: &LspService<CRBasicLanguageServer>,
        uri: &Uri,
    ) {
        service
            .inner()
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "crbasic".to_string(),
                    version: 1,
                    text:
                        "BeginProg\nSub Caller\nCalc()\nEndSub\nFunction Calc\nEndFunction\nEndProg"
                            .to_string(),
                },
            })
            .await;
    }

    #[tokio::test]
    async fn prepares_incoming_and_outgoing_calls_for_a_function() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");
        open_program_with_a_caller_and_callee(&service, &uri).await;

        let prepare_params = CallHierarchyPrepareParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 4,
                    character: 9,
                }, // "Calc" in "Function Calc"
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let prepare_result = service.inner().prepare_call_hierarchy(prepare_params).await;
        assert!(prepare_result.is_ok(), "Prepare should succeed");
        let items = prepare_result
            .expect("Should be Ok")
            .expect("Should resolve an item");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Calc");

        let incoming_result = service
            .inner()
            .incoming_calls(CallHierarchyIncomingCallsParams {
                item: items[0].clone(),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .await;
        assert!(incoming_result.is_ok(), "Incoming calls should succeed");
        let incoming = incoming_result
            .expect("Should be Ok")
            .expect("Should return incoming calls");
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].from.name, "Caller");

        let outgoing_result = service
            .inner()
            .outgoing_calls(CallHierarchyOutgoingCallsParams {
                item: incoming[0].from.clone(),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .await;
        assert!(outgoing_result.is_ok(), "Outgoing calls should succeed");
        let outgoing = outgoing_result
            .expect("Should be Ok")
            .expect("Should return outgoing calls");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].to.name, "Calc");
    }

    #[tokio::test]
    async fn returns_none_when_the_cursor_is_not_on_a_callable_symbol() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let prepare_params = CallHierarchyPrepareParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 7,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = service.inner().prepare_call_hierarchy(prepare_params).await;
        assert!(result.is_ok(), "Prepare should succeed");
        assert!(result.expect("Should be Ok").is_none());
    }
}

mod document_symbols {
    use super::*;

    #[tokio::test]
    async fn extracts_program_structure_symbols() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nDataTable(Test)\nEndTable\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let symbol_params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().document_symbol(symbol_params).await;
        assert!(result.is_ok(), "Document symbols should succeed");

        if let Ok(Some(DocumentSymbolResponse::Nested(symbols))) = result {
            let has_begin_prog = symbols.iter().any(|s| s.name == "BeginProg");
            assert!(has_begin_prog, "Should include BeginProg symbol");

            let has_temp = symbols.iter().any(|s| s.name == "Temp");
            assert!(has_temp, "Should include Temp variable symbol");

            let has_data_table = symbols.iter().any(|s| s.name.contains("Test"));
            assert!(has_data_table, "Should include DataTable symbol");
        } else {
            panic!("Expected document symbols result");
        }
    }
}

mod rename {
    use super::*;

    #[tokio::test]
    async fn renames_all_occurrences_of_variable() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nTemp = Temp + 1\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let rename_params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 7,
                },
            },
            new_name: "Temperature".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = service.inner().rename(rename_params).await;
        assert!(result.is_ok(), "Rename should succeed");

        let edit = result
            .expect("Rename should succeed")
            .expect("Should produce a workspace edit");
        let changes = edit.changes.expect("Should contain changes");
        let edits = changes.get(&uri).expect("Should have edits for the URI");

        assert!(
            edits.len() >= 3,
            "Should rename at least 3 occurrences of Temp (found {})",
            edits.len()
        );
        assert!(edits.iter().all(|e| e.new_text == "Temperature"));
    }

    #[tokio::test]
    async fn rejects_rename_with_invalid_identifier() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let rename_params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 7,
                },
            },
            new_name: "Invalid Name".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = service.inner().rename(rename_params).await;

        assert!(result.is_err(), "Rename with an invalid name should fail");
    }

    #[tokio::test]
    async fn prepare_rename_returns_range_for_variable() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let position_params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 7,
            },
        };

        let result = service.inner().prepare_rename(position_params).await;
        assert!(result.is_ok(), "Prepare rename should succeed");
        assert!(
            matches!(result, Ok(Some(PrepareRenameResponse::Range(_)))),
            "Should return the range of the identifier under the cursor"
        );
    }
}

mod semantic_tokens {
    use super::*;

    #[tokio::test]
    async fn distinguishes_public_variable_declaration_from_its_reference() {
        let (service, _socket) = create_test_server().await;
        let uri = test_uri("test.CR6");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "crbasic".to_string(),
                version: 1,
                text: "BeginProg\nPublic Temp\nTemp = 5\nEndProg".to_string(),
            },
        };
        service.inner().did_open(open_params).await;

        let params = SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = service.inner().semantic_tokens_full(params).await;
        assert!(result.is_ok(), "Semantic tokens request should succeed");

        let tokens = match result.expect("Should succeed") {
            Some(SemanticTokensResult::Tokens(tokens)) => tokens.data,
            other => panic!("Expected full semantic tokens, got {other:?}"),
        };

        assert_eq!(tokens.len(), 2, "Should classify both Temp occurrences");
        assert_ne!(
            tokens[0].token_modifiers_bitset & 1,
            0,
            "Declaring occurrence should carry the declaration modifier"
        );
        assert_eq!(
            tokens[1].token_modifiers_bitset & 1,
            0,
            "Reference should not carry the declaration modifier"
        );
        assert_ne!(
            tokens[0].token_modifiers_bitset & (1 << 2),
            0,
            "Public variable should carry the global modifier"
        );
    }
}
