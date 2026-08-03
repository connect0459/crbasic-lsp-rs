//! Performance regression tests for the Lexer/Parser/Diagnostics pipeline
//!
//! Targets (see docs/todo.md): Lexer <1ms, Parser <10ms, Diagnostics <50ms
//! for a 1000-line CRBasic program. These targets describe release-build
//! performance; `cargo test` runs in debug mode by default, which lacks the
//! optimizations the targets assume, so each budget is relaxed under
//! `cfg!(debug_assertions)` to avoid flaking on unoptimized builds while
//! still catching gross regressions (e.g. an accidental O(n^2) scan).
//! Run `cargo test --release --test performance` to verify against the
//! real spec'd numbers.

use crbasic_parser::Parser;
use crbasic_parser::lexer::Scanner;
use crbasic_parser::semantic::{DataloggerModel, SemanticAnalyzer};
use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 5;

fn debug_relaxed(target: Duration) -> Duration {
    if cfg!(debug_assertions) {
        target * 20
    } else {
        target
    }
}

/// Builds a deterministic CRBasic program with at least `min_lines` lines,
/// mixing variable declarations, arithmetic assignments, and `If` blocks.
fn generate_large_program(min_lines: usize) -> String {
    let mut source = String::from("BeginProg\n");
    let mut lines = 1;
    let mut i = 0;
    while lines < min_lines {
        source.push_str(&format!("  Public Var_{i} As Float\n"));
        source.push_str(&format!("  Var_{i} = ({i} + 1.5) * (Var_{i} - 2) / 3\n"));
        source.push_str(&format!("  If Var_{i} > 0 Then\n"));
        source.push_str(&format!("    Var_{i} = Var_{i} + 1\n"));
        source.push_str("  EndIf\n");
        lines += 5;
        i += 1;
    }
    source.push_str("EndProg\n");
    source
}

fn fastest_of<F: FnMut() -> Duration>(mut run_once: F) -> Duration {
    (0..SAMPLE_COUNT)
        .map(|_| run_once())
        .min()
        .expect("SAMPLE_COUNT is greater than zero")
}

#[test]
fn fixture_has_at_least_1000_lines() {
    let source = generate_large_program(1000);
    assert!(
        source.lines().count() >= 1000,
        "generated fixture should have at least 1000 lines"
    );
}

#[test]
fn lexer_tokenizes_1000_line_program_within_budget() {
    let source = generate_large_program(1000);

    let elapsed = fastest_of(|| {
        let src = source.clone();
        let start = Instant::now();
        let mut scanner = Scanner::new(&src);
        let tokens = scanner.scan_tokens();
        let elapsed = start.elapsed();
        assert!(!tokens.is_empty());
        elapsed
    });

    let budget = debug_relaxed(Duration::from_millis(1));
    assert!(
        elapsed < budget,
        "lexer should tokenize a 1000-line program within {budget:?}, took {elapsed:?}"
    );
}

#[test]
fn parser_parses_1000_line_program_within_budget() {
    let source = generate_large_program(1000);

    let elapsed = fastest_of(|| {
        let mut scanner = Scanner::new(&source);
        let tokens = scanner.scan_tokens();

        let start = Instant::now();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("generated fixture should parse");
        let elapsed = start.elapsed();
        assert!(!program.statements.is_empty());
        elapsed
    });

    let budget = debug_relaxed(Duration::from_millis(10));
    assert!(
        elapsed < budget,
        "parser should parse a 1000-line program within {budget:?}, took {elapsed:?}"
    );
}

#[test]
fn diagnostics_validate_1000_line_program_within_budget() {
    let source = generate_large_program(1000);

    let elapsed = fastest_of(|| {
        let src = source.clone();
        let start = Instant::now();

        let mut scanner = Scanner::new(&src);
        let tokens = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("generated fixture should parse");
        let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
        let errors = analyzer.analyze(&program);

        let elapsed = start.elapsed();
        assert!(errors.is_empty());
        elapsed
    });

    let budget = debug_relaxed(Duration::from_millis(50));
    assert!(
        elapsed < budget,
        "full-file validation should complete within {budget:?}, took {elapsed:?}"
    );
}
