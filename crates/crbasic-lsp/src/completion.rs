//! Code completion provider for CRBasic
//!
//! This module provides IntelliSense completion items for keywords,
//! built-in functions, and user-defined symbols.

use crbasic_parser::ast::{Program, Statement};
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};

/// Provides completion items for CRBasic
pub struct CompletionProvider;

impl CompletionProvider {
    /// Returns all keyword completion items
    pub fn get_keyword_completions() -> Vec<CompletionItem> {
        let mut items = Vec::new();

        items.extend(Self::control_flow_keywords());
        items.extend(Self::declaration_keywords());
        items.extend(Self::program_structure_keywords());
        items.extend(Self::function_definition_keywords());
        items.extend(Self::logical_operator_keywords());

        items
    }

    /// Returns built-in function completion items
    pub fn get_builtin_function_completions() -> Vec<CompletionItem> {
        vec![
            Self::create_function_completion(
                "Scan",
                "Scan(${1:Interval}, ${2:Units}, ${3:BufferOption}, ${4:Count})",
                "Initiates a measurement scan at specified intervals.",
            ),
            Self::create_function_completion(
                "SlowSequence",
                "SlowSequence",
                "Begins a slow sequence scan block.",
            ),
            Self::create_function_completion(
                "EndSequence",
                "EndSequence",
                "Ends a slow sequence scan block.",
            ),
            Self::create_function_completion(
                "CallTable",
                "CallTable(${1:TableName})",
                "Calls a data table to process and store data.",
            ),
            Self::create_function_completion(
                "Sample",
                "Sample(${1:Reps}, ${2:Source}, ${3:DataType})",
                "Samples and stores a value in the data table.",
            ),
            Self::create_function_completion(
                "Average",
                "Average(${1:Reps}, ${2:Source}, ${3:DataType}, ${4:DisableVar})",
                "Calculates and stores the average of values.",
            ),
            Self::create_function_completion(
                "Minimum",
                "Minimum(${1:Reps}, ${2:Source}, ${3:DataType}, ${4:DisableVar}, ${5:Time})",
                "Stores the minimum value over the output interval.",
            ),
            Self::create_function_completion(
                "Maximum",
                "Maximum(${1:Reps}, ${2:Source}, ${3:DataType}, ${4:DisableVar}, ${5:Time})",
                "Stores the maximum value over the output interval.",
            ),
            Self::create_function_completion(
                "Totalize",
                "Totalize(${1:Reps}, ${2:Source}, ${3:DataType}, ${4:DisableVar})",
                "Stores the sum of values over the output interval.",
            ),
            Self::create_function_completion(
                "StdDev",
                "StdDev(${1:Reps}, ${2:Source}, ${3:DataType}, ${4:DisableVar})",
                "Calculates and stores the standard deviation.",
            ),
            Self::create_function_completion(
                "PulseCount",
                "PulseCount(${1:Dest}, ${2:Reps}, ${3:PChan}, ${4:PConfig}, ${5:POption}, ${6:Mult}, ${7:Offset})",
                "Measures pulse count from a sensor.",
            ),
            Self::create_function_completion(
                "VoltSe",
                "VoltSe(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:MeasOfs}, ${6:SettlingTime}, ${7:Integ}, ${8:Mult}, ${9:Offset})",
                "Measures single-ended voltage.",
            ),
            Self::create_function_completion(
                "VoltDiff",
                "VoltDiff(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:RevDiff}, ${6:SettlingTime}, ${7:Integ}, ${8:Mult}, ${9:Offset})",
                "Measures differential voltage.",
            ),
            Self::create_function_completion(
                "Therm107",
                "Therm107(${1:Dest}, ${2:Reps}, ${3:SEChan}, ${4:Excite}, ${5:Mult}, ${6:Offset})",
                "Measures temperature using a 107 thermistor.",
            ),
            Self::create_function_completion(
                "Therm108",
                "Therm108(${1:Dest}, ${2:Reps}, ${3:SEChan}, ${4:Excite}, ${5:Mult}, ${6:Offset})",
                "Measures temperature using a 108 thermistor.",
            ),
            Self::create_function_completion(
                "Therm109",
                "Therm109(${1:Dest}, ${2:Reps}, ${3:SEChan}, ${4:Excite}, ${5:Mult}, ${6:Offset})",
                "Measures temperature using a 109 thermistor.",
            ),
            Self::create_function_completion(
                "TCDiff",
                "TCDiff(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:TCType}, ${6:TRef}, ${7:RevDiff}, ${8:SettlingTime}, ${9:Integ}, ${10:Mult}, ${11:Offset})",
                "Measures thermocouple temperature (differential).",
            ),
            Self::create_function_completion(
                "SerialOpen",
                "SerialOpen(${1:ComPort}, ${2:BaudRate}, ${3:Format}, ${4:TXDelay}, ${5:BufferSize})",
                "Opens a serial communication port.",
            ),
            Self::create_function_completion(
                "SerialClose",
                "SerialClose(${1:ComPort})",
                "Closes a serial communication port.",
            ),
            Self::create_function_completion(
                "SerialIn",
                "SerialIn(${1:Dest}, ${2:ComPort}, ${3:TimeOut}, ${4:TerminationChar}, ${5:MaxNumChars})",
                "Reads data from a serial port.",
            ),
            Self::create_function_completion(
                "SerialOut",
                "SerialOut(${1:ComPort}, ${2:OutString}, ${3:WaitString}, ${4:NumberTries}, ${5:TimeOut})",
                "Sends data to a serial port.",
            ),
            Self::create_function_completion(
                "Abs",
                "Abs(${1:Value})",
                "Returns the absolute value.",
            ),
            Self::create_function_completion("Sqr", "Sqr(${1:Value})", "Returns the square root."),
            Self::create_function_completion(
                "Sqrt",
                "Sqrt(${1:Value})",
                "Returns the square root (alias for Sqr).",
            ),
            Self::create_function_completion(
                "Exp",
                "Exp(${1:Value})",
                "Returns e raised to a power.",
            ),
            Self::create_function_completion(
                "Log",
                "Log(${1:Value})",
                "Returns the natural logarithm.",
            ),
            Self::create_function_completion(
                "Log10",
                "Log10(${1:Value})",
                "Returns the base-10 logarithm.",
            ),
            Self::create_function_completion("Sin", "Sin(${1:Radians})", "Returns the sine."),
            Self::create_function_completion("Cos", "Cos(${1:Radians})", "Returns the cosine."),
            Self::create_function_completion("Tan", "Tan(${1:Radians})", "Returns the tangent."),
            Self::create_function_completion("Asin", "Asin(${1:Value})", "Returns the arc sine."),
            Self::create_function_completion("Acos", "Acos(${1:Value})", "Returns the arc cosine."),
            Self::create_function_completion("Atn", "Atn(${1:Value})", "Returns the arc tangent."),
            Self::create_function_completion(
                "Atn2",
                "Atn2(${1:Y}, ${2:X})",
                "Returns the arc tangent of Y/X.",
            ),
            Self::create_function_completion(
                "Int",
                "Int(${1:Value})",
                "Returns the integer part (truncates toward negative infinity).",
            ),
            Self::create_function_completion(
                "Fix",
                "Fix(${1:Value})",
                "Returns the integer part (truncates toward zero).",
            ),
            Self::create_function_completion(
                "Round",
                "Round(${1:Value}, ${2:NumDigits})",
                "Rounds to specified decimal places.",
            ),
            Self::create_function_completion(
                "Len",
                "Len(${1:String})",
                "Returns the length of a string.",
            ),
            Self::create_function_completion(
                "Mid",
                "Mid(${1:String}, ${2:Start}, ${3:Length})",
                "Extracts a substring.",
            ),
            Self::create_function_completion(
                "Left",
                "Left(${1:String}, ${2:Length})",
                "Returns leftmost characters.",
            ),
            Self::create_function_completion(
                "Right",
                "Right(${1:String}, ${2:Length})",
                "Returns rightmost characters.",
            ),
            Self::create_function_completion(
                "InStr",
                "InStr(${1:Start}, ${2:String}, ${3:SearchString}, ${4:CaseSensitive})",
                "Finds a substring within a string.",
            ),
            Self::create_function_completion(
                "Replace",
                "Replace(${1:String}, ${2:Find}, ${3:ReplaceWith})",
                "Replaces occurrences in a string.",
            ),
            Self::create_function_completion(
                "Trim",
                "Trim(${1:String})",
                "Removes leading and trailing spaces.",
            ),
            Self::create_function_completion(
                "LTrim",
                "LTrim(${1:String})",
                "Removes leading spaces.",
            ),
            Self::create_function_completion(
                "RTrim",
                "RTrim(${1:String})",
                "Removes trailing spaces.",
            ),
            Self::create_function_completion(
                "UpperCase",
                "UpperCase(${1:String})",
                "Converts to uppercase.",
            ),
            Self::create_function_completion(
                "LowerCase",
                "LowerCase(${1:String})",
                "Converts to lowercase.",
            ),
            Self::create_function_completion(
                "SplitStr",
                "SplitStr(${1:Result}, ${2:SearchString}, ${3:Delimiter}, ${4:NumSplits}, ${5:SplitOption})",
                "Splits a string by delimiter.",
            ),
            Self::create_function_completion(
                "FormatFloat",
                "FormatFloat(${1:Value}, ${2:FormatString})",
                "Formats a float as a string.",
            ),
            Self::create_function_completion(
                "Timer",
                "Timer(${1:TimerNumber}, ${2:Units}, ${3:TimerOption})",
                "Returns elapsed time from a timer.",
            ),
            Self::create_function_completion(
                "TimeIntoInterval",
                "TimeIntoInterval(${1:Interval}, ${2:Units})",
                "Returns true when the interval boundary is crossed.",
            ),
            Self::create_function_completion(
                "IfTime",
                "IfTime(${1:TintoInt}, ${2:Interval}, ${3:Units})",
                "Returns true at specified time intervals.",
            ),
            Self::create_function_completion(
                "RealTime",
                "RealTime(${1:Dest})",
                "Returns the current real-time clock values.",
            ),
            Self::create_function_completion(
                "Delay",
                "Delay(${1:Duration}, ${2:Units})",
                "Pauses execution for a specified time.",
            ),
            Self::create_function_completion(
                "ExitScan",
                "ExitScan",
                "Exits the current scan immediately.",
            ),
        ]
    }

    /// Extracts user-defined symbols from the AST
    pub fn get_user_defined_completions(ast: &Program) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        for statement in &ast.statements {
            match statement {
                Statement::VarDeclaration {
                    name,
                    keyword,
                    type_annotation,
                    ..
                } => {
                    let kind = match keyword.as_str() {
                        "Const" => CompletionItemKind::CONSTANT,
                        _ => CompletionItemKind::VARIABLE,
                    };
                    let detail = type_annotation
                        .as_ref()
                        .map(|t| format!("{} {} As {}", keyword, name, t))
                        .unwrap_or_else(|| format!("{} {}", keyword, name));

                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(kind),
                        detail: Some(detail),
                        ..Default::default()
                    });
                }
                Statement::FunctionDefinition {
                    name, parameters, ..
                } => {
                    let params_str = parameters.join(", ");
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(format!("Function {}({})", name, params_str)),
                        insert_text: Some(format!("{}(${{1}})", name)),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    });
                }
                Statement::SubroutineDefinition {
                    name, parameters, ..
                } => {
                    let params_str = parameters.join(", ");
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::METHOD),
                        detail: Some(format!("Sub {}({})", name, params_str)),
                        insert_text: Some(format!("{}(${{1}})", name)),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    });
                }
                _ => {}
            }
        }

        items
    }

    /// Returns multi-statement pattern snippets for common CRBasic idioms
    ///
    /// Unlike the single-keyword snippets in [`Self::get_keyword_completions`],
    /// these combine several statements into a ready-to-edit block (e.g. a
    /// full Scan/NextScan loop paired with its CallTable), matching how the
    /// pattern is actually written in real datalogger programs.
    pub fn get_pattern_snippet_completions() -> Vec<CompletionItem> {
        vec![
            Self::create_pattern_snippet(
                "ScanLoop",
                "Scan(${1:Interval}, ${2:Sec}, ${3:0}, ${4:0})\n\t$0\n\tCallTable(${5:TableName})\nNextScan",
                "Scan/NextScan loop with a CallTable call",
            ),
            Self::create_pattern_snippet(
                "SlowSequenceLoop",
                "SlowSequence\n\tScan(${1:Interval}, ${2:Sec}, ${3:0}, ${4:0})\n\t\t$0\n\t\tCallTable(${5:TableName})\n\tNextScan\nEndSequence",
                "Low-priority scan sequence for non-time-critical measurements",
            ),
            Self::create_pattern_snippet(
                "DataTableSample",
                "DataTable(${1:TableName}, ${2:True}, ${3:-1})\n\tSample(${4:1}, ${5:SourceVariable}, ${6:FP2})\n\t$0\nEndTable",
                "DataTable definition with a Sample output field",
            ),
            Self::create_pattern_snippet(
                "NewProgram",
                "Const ${1:ScanIntervalSec} = ${2:5}\nPublic ${3:VarName} As ${4:Float}\n\nDataTable(${5:TableName},True,-1)\n\tSample(1,${3:VarName},FP2)\nEndTable\n\nBeginProg\n\tScan(${1:ScanIntervalSec},Sec,0,0)\n\t\t$0\n\t\tCallTable(${5:TableName})\n\tNextScan\nEndProg",
                "Starter program skeleton: declarations, DataTable, Scan loop",
            ),
        ]
    }

    /// Returns all completion items (keywords, built-ins, pattern snippets, and user-defined)
    pub fn get_all_completions(ast: Option<&Program>) -> Vec<CompletionItem> {
        let mut items = Self::get_keyword_completions();
        items.extend(Self::get_builtin_function_completions());
        items.extend(Self::get_pattern_snippet_completions());
        items.extend(Self::data_type_completions());

        if let Some(ast) = ast {
            items.extend(Self::get_user_defined_completions(ast));
        }

        items
    }

    /// Returns completion items for the data types valid after `As` in a
    /// `Public`/`Dim` declaration.
    ///
    /// Per Campbell Scientific's own "Data Types" documentation, exactly
    /// these six are valid there (`Float` is the default if `As` is
    /// omitted) -- distinct from the larger output-processing type set
    /// (`FP2`, `IEEE4`, `IEEE8`, `UINT2`, `UINT4`, `Bool8`, `NSEC`, ...)
    /// that's only valid as a `Sample()`/`Average()`-style instruction
    /// argument, a different position this project doesn't offer type
    /// completions for.
    ///
    /// These aren't part of `LANGUAGE_KEYWORDS`: the parser reads a type
    /// annotation as a plain identifier (`Public x As Float` already
    /// parses correctly today), and reclassifying them as lexer keywords
    /// would break that.
    fn data_type_completions() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_completion("Float", "Single-precision floating point (default)"),
            Self::create_keyword_completion("Double", "Double-precision floating point"),
            Self::create_keyword_completion("Long", "32-bit signed integer"),
            Self::create_keyword_completion("Boolean", "True (-1) or False (0)"),
            Self::create_keyword_completion("String", "Null-terminated array of characters"),
            Self::create_keyword_completion("UINT1", "8-bit unsigned integer"),
        ]
    }

    fn control_flow_keywords() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_snippet(
                "If",
                "If ${1:condition} Then\n\t$0\nEndIf",
                "Conditional statement block",
            ),
            Self::create_keyword_completion("Then", "Part of If statement"),
            Self::create_keyword_completion("Else", "Alternative branch in If statement"),
            Self::create_keyword_completion("ElseIf", "Additional conditional branch"),
            Self::create_keyword_completion("EndIf", "Terminates If block"),
            Self::create_keyword_snippet(
                "#If",
                "#If ${1:condition}\n\t$0\n#EndIf",
                "Compile-time conditional block",
            ),
            Self::create_keyword_completion(
                "#ElseIf",
                "Additional compile-time conditional branch",
            ),
            Self::create_keyword_completion("#Else", "Alternative compile-time branch"),
            Self::create_keyword_completion("#EndIf", "Terminates a #If/#IfDef block"),
            Self::create_keyword_snippet(
                "#IfDef",
                "#IfDef ${1:ConstName} Then\n\t$0\n#EndIf",
                "Compile-time check for a declared Const",
            ),
            Self::create_keyword_completion(
                "#UnDef",
                "Un-declares a Const so it can be redeclared",
            ),
            Self::create_keyword_snippet(
                "For",
                "For ${1:i} = ${2:1} To ${3:10}\n\t$0\nNext ${1:i}",
                "Counter-based loop",
            ),
            Self::create_keyword_completion("To", "End value in For loop"),
            Self::create_keyword_completion("Step", "Increment value in For loop"),
            Self::create_keyword_completion("Next", "Terminates For loop"),
            Self::create_keyword_snippet(
                "Do While",
                "Do While ${1:condition}\n\t$0\nLoop",
                "While loop (condition at start)",
            ),
            Self::create_keyword_snippet("Do", "Do\n\t$0\nLoop While ${1:condition}", "Do loop"),
            Self::create_keyword_completion("Loop", "Terminates Do block"),
            Self::create_keyword_completion("While", "Loop condition"),
            Self::create_keyword_completion("Wend", "Terminates a While loop"),
            Self::create_keyword_completion("ExitFor", "Exit For loop immediately"),
            Self::create_keyword_completion("ExitDo", "Exit Do loop immediately"),
            Self::create_keyword_completion("Continue", "Skip to the next loop iteration"),
            Self::create_keyword_completion("Break", "Exit the current loop immediately"),
            Self::create_keyword_snippet(
                "Select Case",
                "Select Case ${1:expression}\n\tCase ${2:value}\n\t\t$0\n\tCase Else\n\t\t\nEndSelect",
                "Multi-way branch statement",
            ),
            Self::create_keyword_completion("Select", "Starts a Select Case block"),
            Self::create_keyword_completion("Case", "Branch in Select statement"),
            Self::create_keyword_completion("Is", "Comparison operator in a Case clause"),
            Self::create_keyword_completion("EndSelect", "Terminates Select block"),
            Self::create_keyword_completion("ExitSelect", "Exit Select block immediately"),
            Self::create_keyword_completion("GoTo", "Unconditional jump (use sparingly)"),
            Self::create_keyword_completion("NextScan", "Terminates a Scan loop"),
        ]
    }

    fn declaration_keywords() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_snippet(
                "Public",
                "Public ${1:VarName} As ${2:Float}",
                "Declare public (global) variable",
            ),
            Self::create_keyword_snippet(
                "Dim",
                "Dim ${1:VarName} As ${2:Float}",
                "Declare local (scratch) variable",
            ),
            Self::create_keyword_snippet(
                "Const",
                "Const ${1:NAME} = ${2:value}",
                "Declare constant value",
            ),
            Self::create_keyword_completion("Alias", "Define alternative variable name"),
            Self::create_keyword_completion("As", "Specify data type"),
            Self::create_keyword_completion("Units", "Specify engineering units"),
        ]
    }

    fn program_structure_keywords() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_snippet(
                "BeginProg",
                "BeginProg\n\t$0\nEndProg",
                "Main program execution block",
            ),
            Self::create_keyword_completion("EndProg", "End of program execution block"),
            Self::create_keyword_snippet(
                "DataTable",
                "DataTable(${1:TableName}, ${2:TriggerCondition}, ${3:Size})\n\t$0\nEndTable",
                "Define data storage table",
            ),
            Self::create_keyword_completion("EndTable", "Terminates DataTable block"),
        ]
    }

    fn function_definition_keywords() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_snippet(
                "Function",
                "Function ${1:FuncName}(${2:params}) As ${3:Float}\n\t$0\nEndFunction",
                "Define user function (returns value)",
            ),
            Self::create_keyword_completion("EndFunction", "Terminates Function block"),
            Self::create_keyword_snippet(
                "Sub",
                "Sub ${1:SubName}(${2:params})\n\t$0\nEndSub",
                "Define subroutine (no return value, copy-back params)",
            ),
            Self::create_keyword_completion("EndSub", "Terminates Sub block"),
        ]
    }

    fn logical_operator_keywords() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_completion("AND", "Logical AND operator"),
            Self::create_keyword_completion("OR", "Logical OR operator"),
            Self::create_keyword_completion("NOT", "Logical NOT operator"),
            Self::create_keyword_completion("XOR", "Logical XOR operator"),
            Self::create_keyword_completion("MOD", "Modulo operator"),
            Self::create_keyword_completion("True", "Boolean true (-1)"),
            Self::create_keyword_completion("False", "Boolean false (0)"),
        ]
    }

    fn create_keyword_completion(label: &str, detail: &str) -> CompletionItem {
        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        }
    }

    fn create_keyword_snippet(label: &str, insert_text: &str, detail: &str) -> CompletionItem {
        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            insert_text: Some(insert_text.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }
    }

    fn create_pattern_snippet(label: &str, insert_text: &str, detail: &str) -> CompletionItem {
        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some(detail.to_string()),
            insert_text: Some(insert_text.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }
    }

    fn create_function_completion(label: &str, insert_text: &str, doc: &str) -> CompletionItem {
        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("Built-in: {}", label)),
            insert_text: Some(insert_text.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.to_string(),
            })),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod keyword_completions {
        use super::*;

        #[test]
        fn returns_keyword_completions() {
            let completions = CompletionProvider::get_keyword_completions();

            assert!(!completions.is_empty());
        }

        #[test]
        fn includes_control_flow_keywords() {
            let completions = CompletionProvider::get_keyword_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"If"));
            assert!(labels.contains(&"For"));
            assert!(labels.contains(&"Do"));
            assert!(labels.contains(&"Select Case"));
        }

        #[test]
        fn includes_declaration_keywords() {
            let completions = CompletionProvider::get_keyword_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"Public"));
            assert!(labels.contains(&"Dim"));
            assert!(labels.contains(&"Const"));
        }

        #[test]
        fn includes_program_structure_keywords() {
            let completions = CompletionProvider::get_keyword_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"BeginProg"));
            assert!(labels.contains(&"EndProg"));
            assert!(labels.contains(&"DataTable"));
            assert!(labels.contains(&"EndTable"));
        }

        #[test]
        fn control_flow_keywords_have_snippets() {
            let completions = CompletionProvider::get_keyword_completions();
            let if_completion = completions.iter().find(|c| c.label == "If").unwrap();

            assert!(if_completion.insert_text.is_some());
            assert_eq!(
                if_completion.insert_text_format,
                Some(InsertTextFormat::SNIPPET)
            );
        }

        #[test]
        fn keywords_have_correct_kind() {
            let completions = CompletionProvider::get_keyword_completions();

            for completion in &completions {
                assert_eq!(completion.kind, Some(CompletionItemKind::KEYWORD));
            }
        }

        #[test]
        fn every_language_keyword_has_a_completion_item() {
            let completions = CompletionProvider::get_keyword_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            let missing: Vec<&str> = crbasic_parser::LANGUAGE_KEYWORDS
                .iter()
                .map(|(name, _)| *name)
                .filter(|name| !labels.contains(name))
                .collect();

            assert!(
                missing.is_empty(),
                "Missing completion items for language keywords: {:?}",
                missing
            );
        }
    }

    mod data_type_completions {
        use super::*;

        #[test]
        fn returns_data_type_completions() {
            let completions = CompletionProvider::data_type_completions();

            assert!(!completions.is_empty());
        }

        #[test]
        fn includes_every_type_valid_after_as() {
            let completions = CompletionProvider::data_type_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            // Per Campbell Scientific's own "Data Types" documentation,
            // these six are valid after `As` in a Public/Dim declaration --
            // distinct from the larger output-processing type set (FP2,
            // IEEE4, IEEE8, UINT2, UINT4, Bool8, NSEC, ...) that's only
            // valid as a Sample()/Average()-style instruction argument.
            for expected in ["Float", "Double", "Long", "Boolean", "String", "UINT1"] {
                assert!(
                    labels.contains(&expected),
                    "Missing data type completion: {}",
                    expected
                );
            }
        }

        #[test]
        fn data_types_have_correct_kind() {
            let completions = CompletionProvider::data_type_completions();

            for completion in &completions {
                assert_eq!(completion.kind, Some(CompletionItemKind::KEYWORD));
            }
        }
    }

    mod builtin_function_completions {
        use super::*;

        #[test]
        fn returns_builtin_function_completions() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert!(!completions.is_empty());
        }

        #[test]
        fn includes_scan_functions() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"Scan"));
        }

        #[test]
        fn includes_data_table_functions() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"CallTable"));
            assert!(labels.contains(&"Sample"));
            assert!(labels.contains(&"Average"));
        }

        #[test]
        fn includes_measurement_functions() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"PulseCount"));
            assert!(labels.contains(&"VoltSe"));
            assert!(labels.contains(&"VoltDiff"));
        }

        #[test]
        fn includes_math_functions() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"Abs"));
            assert!(labels.contains(&"Sqr"));
            assert!(labels.contains(&"Sin"));
            assert!(labels.contains(&"Cos"));
        }

        #[test]
        fn includes_string_functions() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"Len"));
            assert!(labels.contains(&"Mid"));
            assert!(labels.contains(&"SplitStr"));
        }

        #[test]
        fn builtin_functions_have_correct_kind() {
            let completions = CompletionProvider::get_builtin_function_completions();

            for completion in &completions {
                assert_eq!(completion.kind, Some(CompletionItemKind::FUNCTION));
            }
        }

        #[test]
        fn every_builtin_function_completion_is_a_known_canonical_name() {
            let completions = CompletionProvider::get_builtin_function_completions();

            let unknown: Vec<&str> = completions
                .iter()
                .map(|c| c.label.as_str())
                .filter(|label| {
                    !crbasic_parser::BUILTIN_FUNCTIONS
                        .iter()
                        .any(|(name, _)| name == label)
                })
                .collect();

            assert!(
                unknown.is_empty(),
                "Completion labels not found (or wrong casing) in BUILTIN_FUNCTIONS: {:?}",
                unknown
            );
        }

        #[test]
        fn builtin_functions_have_snippet_format() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let scan = completions.iter().find(|c| c.label == "Scan").unwrap();

            assert!(scan.insert_text.is_some());
            assert_eq!(scan.insert_text_format, Some(InsertTextFormat::SNIPPET));
            assert!(scan.insert_text.as_ref().unwrap().contains("${1:"));
        }

        #[test]
        fn builtin_functions_have_documentation() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let scan = completions.iter().find(|c| c.label == "Scan").unwrap();

            assert!(scan.documentation.is_some());
        }
    }

    mod user_defined_completions {
        use super::*;
        use crbasic_parser::lexer::token::{Position, Span};

        fn dummy_span() -> Span {
            Span::new(Position::new(1, 1), Position::new(1, 1))
        }

        fn create_test_ast() -> Program {
            Program {
                statements: vec![
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature".to_string(),
                        array_dimensions: None,
                        type_annotation: Some("Float".to_string()),
                        initializer: None,
                        span: dummy_span(),
                    },
                    Statement::VarDeclaration {
                        keyword: "Const".to_string(),
                        name: "PI".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        initializer: None,
                        span: dummy_span(),
                    },
                    Statement::FunctionDefinition {
                        name: "CalcAverage".to_string(),
                        parameters: vec!["a".to_string(), "b".to_string()],
                        body: vec![],
                        span: dummy_span(),
                    },
                    Statement::SubroutineDefinition {
                        name: "InitSensors".to_string(),
                        parameters: vec![],
                        body: vec![],
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            }
        }

        #[test]
        fn extracts_public_variables() {
            let ast = create_test_ast();
            let completions = CompletionProvider::get_user_defined_completions(&ast);

            let temp = completions.iter().find(|c| c.label == "Temperature");
            assert!(temp.is_some());
            assert_eq!(temp.unwrap().kind, Some(CompletionItemKind::VARIABLE));
        }

        #[test]
        fn extracts_constants() {
            let ast = create_test_ast();
            let completions = CompletionProvider::get_user_defined_completions(&ast);

            let pi = completions.iter().find(|c| c.label == "PI");
            assert!(pi.is_some());
            assert_eq!(pi.unwrap().kind, Some(CompletionItemKind::CONSTANT));
        }

        #[test]
        fn extracts_functions() {
            let ast = create_test_ast();
            let completions = CompletionProvider::get_user_defined_completions(&ast);

            let func = completions.iter().find(|c| c.label == "CalcAverage");
            assert!(func.is_some());
            assert_eq!(func.unwrap().kind, Some(CompletionItemKind::FUNCTION));
        }

        #[test]
        fn extracts_subroutines() {
            let ast = create_test_ast();
            let completions = CompletionProvider::get_user_defined_completions(&ast);

            let sub = completions.iter().find(|c| c.label == "InitSensors");
            assert!(sub.is_some());
            assert_eq!(sub.unwrap().kind, Some(CompletionItemKind::METHOD));
        }

        #[test]
        fn function_completions_have_snippets() {
            let ast = create_test_ast();
            let completions = CompletionProvider::get_user_defined_completions(&ast);

            let func = completions
                .iter()
                .find(|c| c.label == "CalcAverage")
                .unwrap();
            assert!(func.insert_text.is_some());
            assert_eq!(func.insert_text_format, Some(InsertTextFormat::SNIPPET));
        }

        #[test]
        fn variable_details_include_type() {
            let ast = create_test_ast();
            let completions = CompletionProvider::get_user_defined_completions(&ast);

            let temp = completions
                .iter()
                .find(|c| c.label == "Temperature")
                .unwrap();
            assert!(temp.detail.as_ref().unwrap().contains("Float"));
        }
    }

    mod pattern_snippet_completions {
        use super::*;

        #[test]
        fn returns_pattern_snippet_completions() {
            let completions = CompletionProvider::get_pattern_snippet_completions();

            assert!(!completions.is_empty());
        }

        #[test]
        fn includes_expected_patterns_with_snippet_kind() {
            let completions = CompletionProvider::get_pattern_snippet_completions();

            let cases = [
                ("ScanLoop", "NextScan"),
                ("SlowSequenceLoop", "EndSequence"),
                ("DataTableSample", "EndTable"),
                ("NewProgram", "EndProg"),
            ];

            for (label, expected_fragment) in cases {
                let item = completions
                    .iter()
                    .find(|c| c.label == label)
                    .unwrap_or_else(|| panic!("Missing pattern snippet: {label}"));

                assert_eq!(item.kind, Some(CompletionItemKind::SNIPPET));
                assert_eq!(item.insert_text_format, Some(InsertTextFormat::SNIPPET));
                let insert_text = item.insert_text.as_ref().expect("Should have insert text");
                assert!(
                    insert_text.contains(expected_fragment),
                    "{label} snippet should contain {expected_fragment}"
                );
            }
        }

        #[test]
        fn new_program_links_table_name_placeholder_across_declaration_and_usage() {
            let completions = CompletionProvider::get_pattern_snippet_completions();
            let new_program = completions
                .iter()
                .find(|c| c.label == "NewProgram")
                .expect("Should include NewProgram snippet");
            let insert_text = new_program
                .insert_text
                .as_ref()
                .expect("Should have insert text");

            // The table name tabstop must be reused between DataTable and CallTable
            // so the client fills both in sync as the user edits the placeholder.
            assert_eq!(insert_text.matches("TableName}").count(), 2);
        }
    }

    mod all_completions {
        use super::*;
        use crbasic_parser::lexer::token::{Position, Span};

        fn dummy_span() -> Span {
            Span::new(Position::new(1, 1), Position::new(1, 1))
        }

        #[test]
        fn combines_all_completion_sources() {
            let completions = CompletionProvider::get_all_completions(None);

            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
            assert!(labels.contains(&"If"));
            assert!(labels.contains(&"Public"));

            assert!(labels.contains(&"Scan"));
            assert!(labels.contains(&"Abs"));
            assert!(labels.contains(&"Float"));
        }

        #[test]
        fn includes_user_defined_when_ast_provided() {
            let ast = Program {
                statements: vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: "MyVar".to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    initializer: None,
                    span: dummy_span(),
                }],
                span: dummy_span(),
            };

            let completions = CompletionProvider::get_all_completions(Some(&ast));
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"MyVar"));
        }
    }
}
