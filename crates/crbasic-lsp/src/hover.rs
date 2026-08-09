//! Hover information provider for CRBasic
//!
//! This module provides hover information for keywords, built-in functions,
//! and user-defined symbols in CRBasic source code.

use crbasic_parser::lexer::token::{Token, TokenKind};
use tower_lsp_server::ls_types::{
    Hover, HoverContents, MarkupContent, MarkupKind, Position, Range,
};

/// Provides hover information for CRBasic symbols
pub struct HoverProvider;

impl HoverProvider {
    /// Returns hover information for a token at the given position
    ///
    /// # Arguments
    /// * `tokens` - The list of tokens from the lexer
    /// * `position` - The LSP position (0-indexed)
    ///
    /// # Returns
    /// * `Some(Hover)` if hover information is available
    /// * `None` if no hover information is available
    pub fn get_hover_at_position(tokens: &[Token], position: Position) -> Option<Hover> {
        let token = Self::find_token_at_position(tokens, position)?;
        Self::get_hover_for_token(token)
    }

    /// Finds the token at the given LSP position
    ///
    /// # Arguments
    /// * `tokens` - The list of tokens from the lexer
    /// * `position` - The LSP position (0-indexed)
    ///
    /// # Returns
    /// * `Some(&Token)` if a token is found at the position
    /// * `None` if no token is found
    fn find_token_at_position<'a>(
        tokens: &'a [Token<'a>],
        position: Position,
    ) -> Option<&'a Token<'a>> {
        // Convert LSP position (0-indexed) to parser position (1-indexed)
        let line = position.line as usize + 1;
        let column = position.character as usize + 1;

        tokens.iter().find(|token| {
            let start = &token.span.start;
            let end = &token.span.end;

            // Token spans use half-open interval [start, end)
            if line < start.line || line > end.line {
                return false;
            }

            if line == start.line && column < start.column {
                return false;
            }

            // end.column is exclusive (one past the last character)
            if line == end.line && column >= end.column {
                return false;
            }

            true
        })
    }

    /// Returns hover information for a token
    fn get_hover_for_token(token: &Token) -> Option<Hover> {
        match &token.kind {
            TokenKind::Keyword(kw) => {
                let description = Self::get_keyword_description(kw)?;
                let range = Self::token_to_lsp_range(token);

                Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: description.to_string(),
                    }),
                    range: Some(range),
                })
            }
            _ => None,
        }
    }

    /// Converts a token's span to an LSP range
    fn token_to_lsp_range(token: &Token) -> Range {
        Range {
            start: Position {
                line: token.span.start.line.saturating_sub(1) as u32,
                character: token.span.start.column.saturating_sub(1) as u32,
            },
            end: Position {
                line: token.span.end.line.saturating_sub(1) as u32,
                character: token.span.end.column.saturating_sub(1) as u32,
            },
        }
    }

    /// Returns hover information for a keyword
    ///
    /// # Arguments
    /// * `keyword` - The keyword to get hover information for (case-insensitive)
    ///
    /// # Returns
    /// * `Some(Hover)` if the keyword is recognized
    /// * `None` if the keyword is not recognized
    pub fn get_keyword_hover(keyword: &str) -> Option<Hover> {
        let description = Self::get_keyword_description(keyword)?;

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: description.to_string(),
            }),
            range: None,
        })
    }

    /// Returns the description for a keyword
    fn get_keyword_description(keyword: &str) -> Option<&'static str> {
        match keyword.to_lowercase().as_str() {
            "if" => Some(
                "**If**\n\nConditional statement. Executes code block if condition is true.\n\n```crbasic\nIf condition Then\n  ' statements\nEndIf\n```",
            ),
            "then" => Some("**Then**\n\nPart of If statement. Follows the condition."),
            "else" => Some(
                "**Else**\n\nAlternative branch in If statement. Executes when condition is false.",
            ),
            "elseif" => Some("**ElseIf**\n\nAdditional conditional branch in If statement."),
            "endif" => Some("**EndIf**\n\nTerminates an If block."),
            "for" => Some(
                "**For**\n\nLoop statement with counter.\n\n```crbasic\nFor i = 1 To 10 Step 1\n  ' statements\nNext i\n```",
            ),
            "to" => Some("**To**\n\nSpecifies the end value in a For loop."),
            "step" => {
                Some("**Step**\n\nSpecifies the increment value in a For loop. Default is 1.")
            }
            "next" => Some("**Next**\n\nTerminates a For loop and increments the counter."),
            "do" => Some(
                "**Do**\n\nLoop statement.\n\n```crbasic\nDo While condition\n  ' statements\nLoop\n```",
            ),
            "loop" => Some("**Loop**\n\nTerminates a Do block."),
            "while" => Some("**While**\n\nLoop condition. Can be used with Do or Loop."),
            "exitfor" => Some("**ExitFor**\n\nImmediately exits a For loop."),
            "exitdo" => Some("**ExitDo**\n\nImmediately exits a Do loop."),
            "continue" => Some("**Continue**\n\nSkips to the next iteration of the current loop."),
            "break" => Some("**Break**\n\nImmediately exits the current loop."),
            "select" => Some(
                "**Select**\n\nMulti-way branch statement.\n\n```crbasic\nSelect Case expression\n  Case value1\n    ' statements\n  Case Else\n    ' default\nEndSelect\n```",
            ),
            "case" => Some("**Case**\n\nSpecifies a branch in a Select statement."),
            "is" => Some(
                "**Is**\n\nUsed with a comparison operator in a Case clause (e.g. `Case Is > 10`).",
            ),
            "endselect" => Some("**EndSelect**\n\nTerminates a Select block."),
            "exitselect" => Some("**ExitSelect**\n\nImmediately exits a Select block."),
            "goto" => Some("**GoTo**\n\nUnconditional jump to a labeled line. Use sparingly."),
            "nextscan" => Some("**NextScan**\n\nMarks the end of a Scan loop."),

            "public" => Some(
                "**Public**\n\nDeclares a public (global) variable that can be monitored and logged.\n\n```crbasic\nPublic Temp_C As Float\n```",
            ),
            "dim" => Some(
                "**Dim**\n\nDeclares a local (scratch) variable. Not accessible for monitoring.\n\n```crbasic\nDim i As Long\n```",
            ),
            "const" => Some(
                "**Const**\n\nDeclares a constant value that cannot be changed.\n\n```crbasic\nConst PI = 3.14159\n```",
            ),
            "alias" => Some("**Alias**\n\nDefines an alternative name for a variable."),
            "as" => Some(
                "**As**\n\nSpecifies the data type in a variable declaration.\n\nTypes: Float, Long, String, Boolean",
            ),
            "units" => Some("**Units**\n\nSpecifies the engineering units for a variable."),

            "beginprog" => {
                Some("**BeginProg**\n\nMarks the start of the main program execution block.")
            }
            "endprog" => Some("**EndProg**\n\nMarks the end of the main program execution block."),
            "datatable" => Some(
                "**DataTable**\n\nDefines a data table for storing measurements.\n\n```crbasic\nDataTable(TableName, TriggerCondition, Size)\n  ' output instructions\nEndTable\n```",
            ),
            "endtable" => Some("**EndTable**\n\nTerminates a DataTable block."),

            "function" => Some(
                "**Function**\n\nDefines a user-defined function that returns a value.\n\n```crbasic\nFunction MyFunc(param As Float) As Float\n  MyFunc = param * 2\nEndFunction\n```",
            ),
            "endfunction" => Some("**EndFunction**\n\nTerminates a Function block."),
            "sub" => Some(
                "**Sub**\n\nDefines a subroutine (procedure without return value).\n\n```crbasic\nSub MySub(ByRef param As Float)\n  param = param * 2\nEndSub\n```",
            ),
            "endsub" => Some("**EndSub**\n\nTerminates a Sub block."),

            "and" => {
                Some("**AND**\n\nLogical AND operator. Returns true if both operands are true.")
            }
            "or" => Some("**OR**\n\nLogical OR operator. Returns true if either operand is true."),
            "not" => Some("**NOT**\n\nLogical NOT operator. Negates a boolean value."),
            "xor" => Some(
                "**XOR**\n\nLogical XOR operator. Returns true if exactly one operand is true.",
            ),
            "mod" => Some("**MOD**\n\nModulo operator. Returns the remainder of integer division."),

            "true" => Some("**True**\n\nBoolean literal representing true (-1 in CRBasic)."),
            "false" => Some("**False**\n\nBoolean literal representing false (0 in CRBasic)."),

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod keyword_hover {
        use super::*;

        #[test]
        fn returns_hover_for_recognized_keyword() {
            let hover = HoverProvider::get_keyword_hover("If");

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert_eq!(markup.kind, MarkupKind::Markdown);
                    assert!(markup.value.contains("**If**"));
                    assert!(markup.value.contains("Conditional statement"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn returns_none_for_unrecognized_identifier() {
            let hover = HoverProvider::get_keyword_hover("my_variable");

            assert!(hover.is_none());
        }

        #[test]
        fn is_case_insensitive() {
            let hover_lower = HoverProvider::get_keyword_hover("if");
            let hover_upper = HoverProvider::get_keyword_hover("IF");
            let hover_mixed = HoverProvider::get_keyword_hover("iF");

            assert!(hover_lower.is_some());
            assert!(hover_upper.is_some());
            assert!(hover_mixed.is_some());
        }

        mod control_flow_keywords {
            use super::*;

            #[test]
            fn all_control_flow_keywords_have_hover_info() {
                let keywords = [
                    "If",
                    "Then",
                    "Else",
                    "ElseIf",
                    "EndIf",
                    "For",
                    "To",
                    "Step",
                    "Next",
                    "Do",
                    "Loop",
                    "While",
                    "ExitFor",
                    "ExitDo",
                    "Select",
                    "Case",
                    "ExitSelect",
                    "GoTo",
                ];

                for keyword in keywords {
                    let hover = HoverProvider::get_keyword_hover(keyword);
                    assert!(
                        hover.is_some(),
                        "Expected hover info for keyword: {}",
                        keyword
                    );
                }
            }
        }

        mod declaration_keywords {
            use super::*;

            #[test]
            fn all_declaration_keywords_have_hover_info() {
                let keywords = ["Public", "Dim", "Const", "Alias", "As", "Units"];

                for keyword in keywords {
                    let hover = HoverProvider::get_keyword_hover(keyword);
                    assert!(
                        hover.is_some(),
                        "Expected hover info for keyword: {}",
                        keyword
                    );
                }
            }
        }

        mod program_structure_keywords {
            use super::*;

            #[test]
            fn all_program_structure_keywords_have_hover_info() {
                let keywords = ["BeginProg", "EndProg", "DataTable", "EndTable"];

                for keyword in keywords {
                    let hover = HoverProvider::get_keyword_hover(keyword);
                    assert!(
                        hover.is_some(),
                        "Expected hover info for keyword: {}",
                        keyword
                    );
                }
            }
        }

        mod function_keywords {
            use super::*;

            #[test]
            fn all_function_keywords_have_hover_info() {
                let keywords = ["Function", "EndFunction", "Sub", "EndSub"];

                for keyword in keywords {
                    let hover = HoverProvider::get_keyword_hover(keyword);
                    assert!(
                        hover.is_some(),
                        "Expected hover info for keyword: {}",
                        keyword
                    );
                }
            }
        }

        mod logical_operators {
            use super::*;

            #[test]
            fn all_logical_operators_have_hover_info() {
                let keywords = ["AND", "OR", "NOT", "XOR", "MOD"];

                for keyword in keywords {
                    let hover = HoverProvider::get_keyword_hover(keyword);
                    assert!(
                        hover.is_some(),
                        "Expected hover info for keyword: {}",
                        keyword
                    );
                }
            }
        }

        mod boolean_literals {
            use super::*;

            #[test]
            fn all_boolean_literals_have_hover_info() {
                let keywords = ["True", "False"];

                for keyword in keywords {
                    let hover = HoverProvider::get_keyword_hover(keyword);
                    assert!(
                        hover.is_some(),
                        "Expected hover info for keyword: {}",
                        keyword
                    );
                }
            }
        }

        #[test]
        fn every_language_keyword_has_hover_info() {
            let missing: Vec<&str> = crbasic_parser::LANGUAGE_KEYWORDS
                .iter()
                .map(|(name, _)| *name)
                .filter(|name| HoverProvider::get_keyword_hover(name).is_none())
                .collect();

            assert!(
                missing.is_empty(),
                "Missing hover info for language keywords: {:?}",
                missing
            );
        }
    }

    mod position_based_hover {
        use super::*;
        use crbasic_parser::lexer::Scanner;

        fn tokenize(source: &str) -> Vec<Token<'_>> {
            let mut scanner = Scanner::new(source);
            scanner.scan_tokens()
        }

        #[test]
        fn returns_hover_for_keyword_at_position() {
            let tokens = tokenize("If x Then");
            let position = Position {
                line: 0,
                character: 0,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**If**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
            assert!(hover.range.is_some());
        }

        #[test]
        fn returns_hover_within_keyword() {
            // Token span is [0,0) to [0,2) in LSP (0-indexed), so character 1 is inside
            let tokens = tokenize("If x Then");
            let position = Position {
                line: 0,
                character: 1,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
        }

        #[test]
        fn returns_none_for_identifier() {
            let tokens = tokenize("If x Then");
            let position = Position {
                line: 0,
                character: 3,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            // Identifiers don't have hover info yet
            assert!(hover.is_none());
        }

        #[test]
        fn returns_none_for_whitespace() {
            let tokens = tokenize("If x Then");
            let position = Position {
                line: 0,
                character: 2,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_none());
        }

        #[test]
        fn returns_hover_for_keyword_on_second_line() {
            // Position of "If" on line 2 (line 1 in 0-indexed)
            let tokens = tokenize("Public x\nIf y Then");
            let position = Position {
                line: 1,
                character: 0,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**If**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn hover_range_matches_token_position() {
            let tokens = tokenize("  If x Then");
            let position = Position {
                line: 0,
                character: 2,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            let range = hover.range.expect("range should be set");

            assert_eq!(range.start.line, 0);
            assert_eq!(range.start.character, 2);
            assert_eq!(range.end.line, 0);
            assert_eq!(range.end.character, 4);
        }
    }
}
