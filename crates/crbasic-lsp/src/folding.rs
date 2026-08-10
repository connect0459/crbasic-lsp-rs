//! Folding range provider for CRBasic LSP
//!
//! Block statements (`If`/`For`/`Do`/`Function`/`Sub`) already carry a span
//! covering their closing keyword, so each becomes one folding range
//! directly. `BeginProg`/`EndProg`, `DataTable`/`EndTable`, and
//! `ConstTable`/`EndConstTable` are parsed as independent flat statements
//! rather than a single spanning one, so this module pairs them back up
//! itself.

use crbasic_parser::ast::{Program, Statement};
use crbasic_parser::lexer::token::Position;
use tower_lsp_server::ls_types::FoldingRange;

/// Provides Folding Range functionality
pub struct FoldingRangeProvider;

impl FoldingRangeProvider {
    /// Extracts every foldable range from a parsed program
    ///
    /// # Arguments
    /// * `program` - The parsed AST program
    pub fn get_folding_ranges(program: &Program) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();
        Self::collect_from_statements(&program.statements, &mut ranges);
        ranges
    }

    /// Walks a statement list, recursing into block bodies and pairing up
    /// the flat `BeginProg`/`EndProg`, `DataTable`/`EndTable`,
    /// `ConstTable`/`EndConstTable`, `ApplyAndRestartSequence`/
    /// `EndApplyAndRestartSequence`, `ShutDownBegin`/`ShutDownEnd`,
    /// `Scan`/`NextScan`, `SubScan`/`NextSubScan`,
    /// `SlowSequence`/`EndSequence`, `DisplayMenu`/`EndMenu`/
    /// `SubMenu`/`EndSubMenu`, `WebPageBegin`/`WebPageEnd`,
    /// `ModemHangup`/`EndModemHangup`, and `VoiceBeg`/`EndVoice` markers
    fn collect_from_statements(statements: &[Statement], ranges: &mut Vec<FoldingRange>) {
        let mut begin_prog_stack: Vec<Position> = Vec::new();
        let mut data_table_stack: Vec<Position> = Vec::new();
        let mut const_table_stack: Vec<Position> = Vec::new();
        let mut apply_and_restart_sequence_stack: Vec<Position> = Vec::new();
        let mut shutdown_stack: Vec<Position> = Vec::new();
        let mut scan_stack: Vec<Position> = Vec::new();
        let mut subscan_stack: Vec<Position> = Vec::new();
        let mut slow_sequence_stack: Vec<Position> = Vec::new();
        let mut display_menu_stack: Vec<Position> = Vec::new();
        let mut sub_menu_stack: Vec<Position> = Vec::new();
        let mut web_page_stack: Vec<Position> = Vec::new();
        let mut modem_hangup_stack: Vec<Position> = Vec::new();
        let mut voice_stack: Vec<Position> = Vec::new();

        for statement in statements {
            match statement {
                Statement::ProgramStructure { keyword, span, .. } => match keyword.as_str() {
                    "BeginProg" => begin_prog_stack.push(span.start),
                    "DataTable" => data_table_stack.push(span.start),
                    "ConstTable" => const_table_stack.push(span.start),
                    "ApplyAndRestartSequence" => apply_and_restart_sequence_stack.push(span.start),
                    "ShutDownBegin" => shutdown_stack.push(span.start),
                    "SlowSequence" => slow_sequence_stack.push(span.start),
                    "VoiceBeg" => voice_stack.push(span.start),
                    "EndProg" => {
                        if let Some(start) = begin_prog_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndTable" => {
                        if let Some(start) = data_table_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndConstTable" => {
                        if let Some(start) = const_table_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndApplyAndRestartSequence" => {
                        if let Some(start) = apply_and_restart_sequence_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "ShutDownEnd" => {
                        if let Some(start) = shutdown_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "NextScan" => {
                        if let Some(start) = scan_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "NextSubScan" => {
                        if let Some(start) = subscan_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndSequence" => {
                        if let Some(start) = slow_sequence_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndSubMenu" => {
                        if let Some(start) = sub_menu_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndMenu" => {
                        if let Some(start) = display_menu_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "WebPageEnd" => {
                        if let Some(start) = web_page_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndModemHangup" => {
                        if let Some(start) = modem_hangup_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    "EndVoice" => {
                        if let Some(start) = voice_stack.pop() {
                            Self::push_range(ranges, start, span.end);
                        }
                    }
                    _ => {}
                },
                Statement::FunctionCall { name, span, .. } => match name.as_str() {
                    "Scan" => scan_stack.push(span.start),
                    "SubScan" => subscan_stack.push(span.start),
                    "DisplayMenu" => display_menu_stack.push(span.start),
                    "SubMenu" => sub_menu_stack.push(span.start),
                    "WebPageBegin" => web_page_stack.push(span.start),
                    "ModemHangup" => modem_hangup_stack.push(span.start),
                    _ => {}
                },
                Statement::IfStatement {
                    then_branch,
                    else_branch,
                    span,
                    ..
                }
                | Statement::PreprocessorConditional {
                    then_branch,
                    else_branch,
                    span,
                    ..
                } => {
                    Self::push_range(ranges, span.start, span.end);
                    Self::collect_from_statements(then_branch, ranges);
                    if let Some(else_stmts) = else_branch {
                        Self::collect_from_statements(else_stmts, ranges);
                    }
                }
                Statement::ForLoop { body, span, .. }
                | Statement::DoLoop { body, span, .. }
                | Statement::FunctionDefinition { body, span, .. }
                | Statement::SubroutineDefinition { body, span, .. } => {
                    Self::push_range(ranges, span.start, span.end);
                    Self::collect_from_statements(body, ranges);
                }
                Statement::StructureType { span, .. } => {
                    Self::push_range(ranges, span.start, span.end);
                }
                Statement::SelectCase {
                    cases,
                    else_branch,
                    span,
                    ..
                } => {
                    Self::push_range(ranges, span.start, span.end);
                    for case in cases {
                        Self::push_range(ranges, case.span.start, case.span.end);
                        Self::collect_from_statements(&case.body, ranges);
                    }
                    if let Some(else_stmts) = else_branch {
                        Self::collect_from_statements(else_stmts, ranges);
                    }
                }
                _ => {}
            }
        }
    }

    /// Pushes a folding range if it spans more than one line
    fn push_range(ranges: &mut Vec<FoldingRange>, start: Position, end: Position) {
        let start_line = start.line.saturating_sub(1) as u32;
        let end_line = end.line.saturating_sub(1) as u32;

        if end_line > start_line {
            ranges.push(FoldingRange {
                start_line,
                end_line,
                ..Default::default()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crbasic_parser::ast::Expression;
    use crbasic_parser::lexer::token::Span;

    fn pos(line: usize, column: usize) -> Position {
        Position::new(line, column)
    }

    fn span(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span::new(pos(start_line, start_col), pos(end_line, end_col))
    }

    fn program(statements: Vec<Statement>) -> Program {
        Program::new(statements, span(1, 1, 1, 1))
    }

    fn dummy_condition() -> Expression {
        Expression::BooleanLiteral {
            value: true,
            span: span(1, 1, 1, 1),
        }
    }

    fn program_structure(keyword: &str, line: usize) -> Statement {
        Statement::ProgramStructure {
            keyword: keyword.to_string(),
            arguments: None,
            span: span(line, 1, line, keyword.len() + 1),
        }
    }

    fn function_call(name: &str, line: usize) -> Statement {
        Statement::FunctionCall {
            name: name.to_string(),
            arguments: Vec::new(),
            span: span(line, 1, line, name.len() + 1),
        }
    }

    mod get_folding_ranges {
        use super::*;

        #[test]
        fn folds_an_if_statement_from_if_to_endif() {
            let program = program(vec![Statement::IfStatement {
                condition: dummy_condition(),
                then_branch: Vec::new(),
                else_branch: None,
                span: span(1, 1, 5, 5),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 4);
        }

        #[test]
        fn folds_a_for_loop_from_for_to_next() {
            let program = program(vec![Statement::ForLoop {
                variable: "i".to_string(),
                start: dummy_condition(),
                end: dummy_condition(),
                step: None,
                body: Vec::new(),
                span: span(2, 1, 6, 4),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 1);
            assert_eq!(ranges[0].end_line, 5);
        }

        #[test]
        fn folds_a_do_loop_from_do_to_loop() {
            let program = program(vec![Statement::DoLoop {
                condition: None,
                condition_at_start: false,
                body: Vec::new(),
                span: span(3, 1, 9, 4),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 2);
            assert_eq!(ranges[0].end_line, 8);
        }

        #[test]
        fn folds_a_function_definition() {
            let program = program(vec![Statement::FunctionDefinition {
                name: "DoWork".to_string(),
                parameters: Vec::new(),
                body: Vec::new(),
                span: span(1, 1, 10, 11),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 9);
        }

        #[test]
        fn folds_a_subroutine_definition() {
            let program = program(vec![Statement::SubroutineDefinition {
                name: "Init".to_string(),
                parameters: Vec::new(),
                body: Vec::new(),
                span: span(1, 1, 4, 7),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 3);
        }

        #[test]
        fn folds_a_structure_type_block() {
            let program = program(vec![Statement::StructureType {
                name: "TempRHSensor".to_string(),
                members: Vec::new(),
                span: span(1, 1, 4, 16),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 3);
        }

        #[test]
        fn pairs_begin_prog_with_the_matching_end_prog() {
            let program = program(vec![
                program_structure("BeginProg", 1),
                program_structure("EndProg", 20),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 19);
        }

        #[test]
        fn pairs_data_table_with_the_matching_end_table() {
            let program = program(vec![
                program_structure("DataTable", 2),
                program_structure("EndTable", 8),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 1);
            assert_eq!(ranges[0].end_line, 7);
        }

        #[test]
        fn pairs_const_table_with_the_matching_end_const_table() {
            let program = program(vec![
                program_structure("ConstTable", 2),
                program_structure("EndConstTable", 8),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 1);
            assert_eq!(ranges[0].end_line, 7);
        }

        #[test]
        fn pairs_apply_and_restart_sequence_with_its_matching_end() {
            let program = program(vec![
                program_structure("ApplyAndRestartSequence", 1),
                program_structure("EndApplyAndRestartSequence", 5),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 4);
        }

        #[test]
        fn pairs_shutdownbegin_with_its_matching_shutdownend() {
            let program = program(vec![
                program_structure("ShutDownBegin", 1),
                program_structure("ShutDownEnd", 5),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 4);
        }

        #[test]
        fn pairs_two_sequential_data_tables_independently() {
            let program = program(vec![
                program_structure("DataTable", 1),
                program_structure("EndTable", 3),
                program_structure("DataTable", 5),
                program_structure("EndTable", 9),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 2);
            assert_eq!((ranges[0].start_line, ranges[0].end_line), (0, 2));
            assert_eq!((ranges[1].start_line, ranges[1].end_line), (4, 8));
        }

        #[test]
        fn ignores_an_end_prog_without_a_matching_begin_prog() {
            let program = program(vec![program_structure("EndProg", 1)]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert!(ranges.is_empty());
        }

        #[test]
        fn ignores_a_begin_prog_without_a_matching_end_prog() {
            let program = program(vec![program_structure("BeginProg", 1)]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert!(ranges.is_empty());
        }

        #[test]
        fn recurses_into_nested_block_bodies() {
            let program = program(vec![Statement::IfStatement {
                condition: dummy_condition(),
                then_branch: vec![Statement::ForLoop {
                    variable: "i".to_string(),
                    start: dummy_condition(),
                    end: dummy_condition(),
                    step: None,
                    body: Vec::new(),
                    span: span(2, 1, 4, 4),
                }],
                else_branch: None,
                span: span(1, 1, 5, 5),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 2);
        }

        #[test]
        fn pairs_scan_with_the_matching_next_scan() {
            let program = program(vec![
                function_call("Scan", 1),
                program_structure("NextScan", 5),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 4);
        }

        #[test]
        fn pairs_subscan_with_the_matching_next_subscan() {
            let program = program(vec![
                function_call("SubScan", 2),
                program_structure("NextSubScan", 4),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 1);
            assert_eq!(ranges[0].end_line, 3);
        }

        #[test]
        fn pairs_slow_sequence_with_the_matching_end_sequence() {
            let program = program(vec![
                program_structure("SlowSequence", 1),
                function_call("Scan", 2),
                program_structure("NextScan", 8),
                program_structure("EndSequence", 9),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 2);
            assert_eq!((ranges[0].start_line, ranges[0].end_line), (1, 7));
            assert_eq!((ranges[1].start_line, ranges[1].end_line), (0, 8));
        }

        #[test]
        fn pairs_display_menu_with_the_matching_end_menu() {
            let program = program(vec![
                function_call("DisplayMenu", 1),
                program_structure("EndMenu", 6),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 5);
        }

        #[test]
        fn pairs_sub_menu_with_the_matching_end_sub_menu() {
            let program = program(vec![
                function_call("DisplayMenu", 1),
                function_call("SubMenu", 2),
                program_structure("EndSubMenu", 4),
                program_structure("EndMenu", 5),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 2);
            assert_eq!((ranges[0].start_line, ranges[0].end_line), (1, 3));
            assert_eq!((ranges[1].start_line, ranges[1].end_line), (0, 4));
        }

        #[test]
        fn pairs_web_page_begin_with_the_matching_web_page_end() {
            let program = program(vec![
                function_call("WebPageBegin", 1),
                program_structure("WebPageEnd", 4),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 3);
        }

        #[test]
        fn pairs_modem_hangup_with_the_matching_end_modem_hangup() {
            let program = program(vec![
                function_call("ModemHangup", 1),
                program_structure("EndModemHangup", 3),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 2);
        }

        #[test]
        fn pairs_voice_beg_with_the_matching_end_voice() {
            let program = program(vec![
                program_structure("VoiceBeg", 1),
                program_structure("EndVoice", 3),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0].start_line, 0);
            assert_eq!(ranges[0].end_line, 2);
        }

        #[test]
        fn does_not_confuse_a_bare_scan_reference_with_a_subscan_call() {
            let program = program(vec![
                function_call("SubScan", 1),
                program_structure("NextSubScan", 3),
                program_structure("NextScan", 4),
            ]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert_eq!(ranges.len(), 1, "an unmatched NextScan must not be paired");
            assert_eq!((ranges[0].start_line, ranges[0].end_line), (0, 2));
        }

        #[test]
        fn skips_ranges_that_do_not_span_multiple_lines() {
            let program = program(vec![Statement::IfStatement {
                condition: dummy_condition(),
                then_branch: Vec::new(),
                else_branch: None,
                span: span(1, 1, 1, 20),
            }]);

            let ranges = FoldingRangeProvider::get_folding_ranges(&program);

            assert!(ranges.is_empty());
        }
    }
}
