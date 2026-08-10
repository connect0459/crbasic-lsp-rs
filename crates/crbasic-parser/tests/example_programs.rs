//! Integration tests for the curated example programs in `docs/examples/`
//!
//! Unlike `tests/sample_files.rs` (parser regression fixtures drawn from
//! real-world programs), these files are end-user documentation showcasing
//! specific LSP features. This test keeps their documented behavior --
//! which diagnostics fire, and which don't -- in sync with the analyzer.

use crbasic_parser::lexer::Scanner;
use crbasic_parser::semantic::ErrorSeverity;
use crbasic_parser::{DataloggerModel, Parser, SemanticAnalyzer, SemanticError};
use std::fs;
use std::path::PathBuf;

/// Returns the path to the examples directory
fn examples_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../docs/examples")
}

/// Helper to read an example file
fn read_example_file(filename: &str) -> String {
    let path = examples_dir().join(filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e))
}

/// Tokenizes, parses, and semantically analyzes an example program
fn analyze(source: &str, model: DataloggerModel) -> Vec<SemanticError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = parser
        .parse()
        .unwrap_or_else(|e| panic!("Failed to parse example program: {:?}", e));

    let mut analyzer = SemanticAnalyzer::new(model);
    analyzer.analyze(&program)
}

mod getting_started {
    use super::*;

    #[test]
    fn has_no_semantic_errors_or_warnings() {
        let source = read_example_file("01-getting-started.CR6");
        let errors = analyze(&source, DataloggerModel::CR6);

        assert!(
            errors.is_empty(),
            "Expected no diagnostics, got: {errors:?}"
        );
    }
}

mod scope_and_copyback {
    use super::*;

    #[test]
    fn has_no_semantic_errors_or_warnings() {
        let source = read_example_file("02-scope-and-copyback.CR6");
        let errors = analyze(&source, DataloggerModel::CR6);

        assert!(
            errors.is_empty(),
            "Expected no diagnostics, got: {errors:?}"
        );
    }
}

mod cr200x_length_pitfalls {
    use super::*;

    #[test]
    fn demonstrates_the_documented_diagnostics() {
        let source = read_example_file("03-cr200x-length-pitfalls.CR2");
        let errors = analyze(&source, DataloggerModel::CR200X);

        let error_count = errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Error)
            .count();
        let warning_count = errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Warning)
            .count();

        assert_eq!(
            error_count, 3,
            "Expected 1 max-length error + 2 truncation-collision errors, got: {errors:?}"
        );
        assert_eq!(
            warning_count, 4,
            "Expected 4 recommended-length warnings, got: {errors:?}"
        );
    }
}
