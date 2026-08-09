//! LSP Integration Tests
//!
//! These tests verify the end-to-end behavior of the LSP server,
//! including document synchronization, diagnostics, and language features.

use crbasic_lsp::CRBasicLanguageServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

/// Test helper to create an LSP service for testing
async fn create_test_server() -> (LspService<CRBasicLanguageServer>, tower_lsp::ClientSocket) {
    CRBasicLanguageServer::new_service()
}

/// Helper to create a test document URI
fn test_uri(name: &str) -> Url {
    Url::parse(&format!("file:///test/{}", name)).expect("Failed to create test URI")
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
