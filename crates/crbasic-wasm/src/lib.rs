//! WASM bindings for CRBasic parser and analyzer
//!
//! This crate provides WebAssembly bindings for the CRBasic parser,
//! enabling use in web-based IDEs and editors.

use crbasic_parser::Parser;
use crbasic_parser::lexer::Scanner;
use crbasic_parser::semantic::{DataloggerModel, SemanticAnalyzer};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Result of a parse operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Whether parsing was successful
    pub success: bool,
    /// The parsed AST as JSON (if successful)
    pub ast: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Error location (if failed)
    pub error_location: Option<ErrorLocation>,
}

/// Location information for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

/// A diagnostic message from semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The diagnostic message
    pub message: String,
    /// Severity level: "error" or "warning"
    pub severity: String,
    /// Start line (1-indexed)
    pub start_line: usize,
    /// Start column (1-indexed)
    pub start_column: usize,
    /// End line (1-indexed)
    pub end_line: usize,
    /// End column (1-indexed)
    pub end_column: usize,
}

/// Result of analysis operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Parse result
    pub parse_result: ParseResult,
    /// Semantic diagnostics (only if parse succeeded)
    pub diagnostics: Vec<Diagnostic>,
}

/// Tokenizes CRBasic source code and returns tokens as JSON
///
/// # Arguments
/// * `source` - The CRBasic source code
///
/// # Returns
/// JSON string containing an array of tokens
#[wasm_bindgen]
pub fn tokenize(source: &str) -> String {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    serde_json::to_string(&tokens)
        .unwrap_or_else(|e| format!(r#"{{"error": "Serialization failed: {}"}}"#, e))
}

/// Parses CRBasic source code and returns AST as JSON
///
/// # Arguments
/// * `source` - The CRBasic source code
///
/// # Returns
/// JSON string containing ParseResult with AST or error
#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let result = match parser.parse() {
        Ok(program) => {
            let ast_json = serde_json::to_value(&program).ok();
            ParseResult {
                success: true,
                ast: ast_json,
                error: None,
                error_location: None,
            }
        }
        Err(err) => ParseResult {
            success: false,
            ast: None,
            error: Some(err.message.clone()),
            error_location: Some(ErrorLocation {
                line: err.span.start.line,
                column: err.span.start.column,
            }),
        },
    };
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(
            r#"{{"success": false, "error": "Serialization failed: {}"}}"#,
            e
        )
    })
}

/// Analyzes CRBasic source code and returns diagnostics as JSON
///
/// # Arguments
/// * `source` - The CRBasic source code
/// * `file_path` - The file path (used to detect datalogger model from extension)
///
/// # Returns
/// JSON string containing AnalysisResult with parse result and diagnostics
#[wasm_bindgen]
pub fn analyze(source: &str, file_path: &str) -> String {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let analysis_result = match parser.parse() {
        Ok(program) => {
            let model = detect_model_from_path(file_path);

            let mut analyzer = SemanticAnalyzer::new(model);
            let errors = analyzer.analyze(&program);

            let diagnostics: Vec<Diagnostic> = errors
                .iter()
                .map(|e| Diagnostic {
                    message: e.message.clone(),
                    severity: match e.severity {
                        crbasic_parser::semantic::ErrorSeverity::Error => "error".to_string(),
                        crbasic_parser::semantic::ErrorSeverity::Warning => "warning".to_string(),
                    },
                    start_line: e.span.start.line,
                    start_column: e.span.start.column,
                    end_line: e.span.end.line,
                    end_column: e.span.end.column,
                })
                .collect();

            let ast_json = serde_json::to_value(&program).ok();
            AnalysisResult {
                parse_result: ParseResult {
                    success: true,
                    ast: ast_json,
                    error: None,
                    error_location: None,
                },
                diagnostics,
            }
        }
        Err(err) => AnalysisResult {
            parse_result: ParseResult {
                success: false,
                ast: None,
                error: Some(err.message.clone()),
                error_location: Some(ErrorLocation {
                    line: err.span.start.line,
                    column: err.span.start.column,
                }),
            },
            diagnostics: vec![],
        },
    };

    serde_json::to_string(&analysis_result).unwrap_or_else(|e| {
        format!(
            r#"{{"parse_result": {{"success": false, "error": "Serialization failed: {}"}}, "diagnostics": []}}"#,
            e
        )
    })
}

/// Detects the datalogger model from file path extension
fn detect_model_from_path(file_path: &str) -> DataloggerModel {
    let extension = file_path.rsplit('.').next().unwrap_or("");
    DataloggerModel::from_extension(extension)
}

/// Returns the version of the WASM module
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod tokenize_api {
        use super::*;

        #[test]
        fn returns_json_array_of_tokens() {
            let result = tokenize("Public x");
            let parsed: serde_json::Value =
                serde_json::from_str(&result).expect("Result should be valid JSON");
            assert!(parsed.is_array(), "Result should be an array");
        }

        #[test]
        fn tokenizes_simple_declaration() {
            let result = tokenize("Public x");
            let tokens: Vec<serde_json::Value> =
                serde_json::from_str(&result).expect("Result should be valid JSON array");

            assert!(tokens.len() >= 2, "Should have at least 2 tokens");

            assert_eq!(
                tokens[0]["kind"]["Keyword"].as_str(),
                Some("Public"),
                "First token should be Public keyword"
            );
        }

        #[test]
        fn tokenizes_empty_source() {
            let result = tokenize("");
            let tokens: Vec<serde_json::Value> =
                serde_json::from_str(&result).expect("Result should be valid JSON array");

            assert!(!tokens.is_empty(), "Should have at least EOF token");
        }
    }

    mod parse_api {
        use super::*;

        #[test]
        fn returns_success_for_valid_program() {
            let result = parse("BeginProg\nEndProg");
            let parsed: ParseResult =
                serde_json::from_str(&result).expect("Result should be valid ParseResult JSON");

            assert!(parsed.success, "Parse should succeed");
            assert!(parsed.ast.is_some(), "AST should be present");
            assert!(parsed.error.is_none(), "Error should be None");
        }

        #[test]
        fn returns_error_for_invalid_program() {
            let result = parse("If Then");
            let parsed: ParseResult =
                serde_json::from_str(&result).expect("Result should be valid ParseResult JSON");

            assert!(!parsed.success, "Parse should fail");
            assert!(parsed.error.is_some(), "Error message should be present");
        }

        #[test]
        fn includes_error_location_on_failure() {
            let result = parse("If Then");
            let parsed: ParseResult =
                serde_json::from_str(&result).expect("Result should be valid ParseResult JSON");

            assert!(!parsed.success);
            assert!(
                parsed.error_location.is_some(),
                "Error location should be present"
            );
            let location = parsed.error_location.expect("Location should exist");
            assert!(location.line >= 1, "Line should be 1-indexed");
        }

        #[test]
        fn parses_variable_declaration() {
            let result = parse("Public Temp_C");
            let parsed: ParseResult =
                serde_json::from_str(&result).expect("Result should be valid ParseResult JSON");

            assert!(
                parsed.success,
                "Parse should succeed for variable declaration"
            );
            assert!(parsed.ast.is_some());
        }
    }

    mod analyze_api {
        use super::*;

        #[test]
        fn returns_analysis_result_for_valid_program() {
            let result = analyze("Public x", "test.cr6");
            let parsed: AnalysisResult =
                serde_json::from_str(&result).expect("Result should be valid AnalysisResult JSON");

            assert!(parsed.parse_result.success, "Parse should succeed");
        }

        #[test]
        fn detects_variable_name_length_error_for_cr200x() {
            let result = analyze("Public Temperature_Sensor_1", "test.cr1");
            let parsed: AnalysisResult =
                serde_json::from_str(&result).expect("Result should be valid AnalysisResult JSON");

            assert!(parsed.parse_result.success, "Parse should succeed");

            let errors: Vec<_> = parsed
                .diagnostics
                .iter()
                .filter(|d| d.severity == "error")
                .collect();
            assert!(
                !errors.is_empty(),
                "Should have error for long variable name on CR200X"
            );
        }

        #[test]
        fn returns_warning_for_cr200x_truncation() {
            let result = analyze("Public Temperature_1", "test.cr1"); // 13 chars
            let parsed: AnalysisResult =
                serde_json::from_str(&result).expect("Result should be valid AnalysisResult JSON");

            assert!(parsed.parse_result.success, "Parse should succeed");

            let warnings: Vec<_> = parsed
                .diagnostics
                .iter()
                .filter(|d| d.severity == "warning")
                .collect();
            assert!(
                !warnings.is_empty(),
                "Should have warning for name > 12 chars on CR200X"
            );
        }

        #[test]
        fn no_warnings_for_short_variable_on_cr6() {
            let result = analyze("Public Temperature_1", "test.cr6"); // 13 chars
            let parsed: AnalysisResult =
                serde_json::from_str(&result).expect("Result should be valid AnalysisResult JSON");

            assert!(parsed.parse_result.success, "Parse should succeed");
            assert!(
                parsed.diagnostics.is_empty(),
                "Should have no diagnostics for short name on CR6"
            );
        }

        #[test]
        fn returns_parse_error_for_invalid_syntax() {
            let result = analyze("If Then", "test.cr6");
            let parsed: AnalysisResult =
                serde_json::from_str(&result).expect("Result should be valid AnalysisResult JSON");

            assert!(!parsed.parse_result.success, "Parse should fail");
            assert!(
                parsed.diagnostics.is_empty(),
                "No semantic diagnostics for parse failure"
            );
        }
    }

    mod model_detection {
        use super::*;

        #[test]
        fn detects_cr200x_from_cr1_extension() {
            let model = detect_model_from_path("program.cr1");
            assert_eq!(model, DataloggerModel::CR200X);
        }

        #[test]
        fn detects_cr6_from_cr6_extension() {
            let model = detect_model_from_path("program.cr6");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_granite_from_crb_extension() {
            let model = detect_model_from_path("program.crb");
            assert_eq!(model, DataloggerModel::GRANITE);
        }

        #[test]
        fn handles_path_with_directories() {
            let model = detect_model_from_path("/path/to/program.cr1");
            assert_eq!(model, DataloggerModel::CR200X);
        }

        #[test]
        fn returns_unknown_for_no_extension() {
            let model = detect_model_from_path("program");
            assert_eq!(model, DataloggerModel::Unknown);
        }
    }

    mod version_api {
        use super::*;

        #[test]
        fn returns_package_version() {
            let ver = version();
            assert!(!ver.is_empty(), "Version should not be empty");
            assert!(
                ver.contains('.'),
                "Version should be in semver format (e.g., 0.1.0)"
            );
        }
    }
}
