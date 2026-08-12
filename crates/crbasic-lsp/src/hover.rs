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
        let description = match &token.kind {
            TokenKind::Keyword(kw) => Self::get_keyword_description(kw),
            // Data type names (Float, Long, FP2, ...) are lexed as plain
            // identifiers, not keywords -- reclassifying them as keywords
            // would break `Public x As Float` parsing (see
            // `data_type_completions` in `completion.rs` for why). Hover
            // still needs to recognize them, scoped to exactly this known
            // set so ordinary variable identifiers keep returning `None`.
            TokenKind::Identifier(name) => Self::get_data_type_description(name)
                .or_else(|| Self::get_output_processing_data_type_description(name))
                .or_else(|| Self::get_builtin_function_description(name)),
            _ => None,
        }?;
        let range = Self::token_to_lsp_range(token);

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: description.to_string(),
            }),
            range: Some(range),
        })
    }

    /// Returns the description for a data type name valid after `As`, or
    /// `None` if `name` isn't one of them (e.g. an ordinary variable name)
    fn get_data_type_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "float" => Some(
                "**Float**\n\nSingle-precision floating point number. The default type if `As` is omitted.",
            ),
            "double" => Some(
                "**Double**\n\nDouble-precision floating point number. Reduces error accumulation in calculations.",
            ),
            "long" => Some("**Long**\n\n32-bit signed integer."),
            "boolean" => Some("**Boolean**\n\nStores `True` (-1) or `False` (0)."),
            "string" => Some("**String**\n\nNull-terminated array of characters."),
            "uint1" => Some("**UINT1**\n\n8-bit unsigned integer."),
            _ => None,
        }
    }

    /// Returns the description for a data type name valid only as a
    /// `Sample()`/`Average()`-style instruction argument (final data
    /// storage), or `None` if `name` isn't one of them.
    ///
    /// Distinct from `get_data_type_description`: per Campbell Scientific's
    /// "Data Types" documentation, `Long`/`UINT1`/`Boolean`/`String` are
    /// valid in both positions and already covered there, so they aren't
    /// repeated here.
    fn get_output_processing_data_type_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "fp2" => Some(
                "**FP2**\n\nCampbell Scientific proprietary format; 3 or 4 significant digits (2 bytes).",
            ),
            "ieee4" => Some(
                "**IEEE4**\n\nSingle-precision floating point number (4 bytes); same precision as `Float`.",
            ),
            "ieee8" => Some(
                "**IEEE8**\n\nDouble-precision floating point number (8 bytes); same precision as `Double`.",
            ),
            "uint2" => Some("**UINT2**\n\n16-bit unsigned integer."),
            "uint4" => Some("**UINT4**\n\n32-bit unsigned integer."),
            "bool8" => Some("**Bool8**\n\nArray of eight 1-bit Boolean values packed into 1 byte."),
            "nsec" => Some("**NSEC**\n\nNanosecond-resolution time stamp (8 bytes)."),
            _ => None,
        }
    }

    /// Returns the description for a built-in function name (any category),
    /// or `None` if `name` isn't one of `BUILTIN_FUNCTIONS` (e.g. an
    /// ordinary variable name).
    fn get_builtin_function_description(name: &str) -> Option<&'static str> {
        Self::get_scan_function_description(name)
            .or_else(|| Self::get_measurement_function_description(name))
    }

    /// Returns the description for a built-in `Scan`/`SubScan` function
    /// name, or `None` if `name` isn't one of them.
    fn get_scan_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "scan" => Some("**Scan**\n\nInitiates a measurement scan at specified intervals."),
            "subscan" => Some(
                "**SubScan**\n\nBegins a nested sub-scan for faster measurement or multiplexer control.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in measurement/output-processing
    /// function name, or `None` if `name` isn't one of them (e.g. an
    /// ordinary variable name).
    fn get_measurement_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "windvector" => Some(
                "**WindVector**\n\nProcesses raw wind speed/direction samples into mean wind speed, mean wind vector magnitude and direction, and standard deviation of wind direction.",
            ),
            "tcdiff" => Some(
                "**TCDiff**\n\nMeasures a thermocouple on a differential channel and converts the result to degrees Celsius.",
            ),
            "resistance" => Some(
                "**Resistance**\n\nMeasures the resistance of a basic or full-bridge circuit using current excitation.",
            ),
            "sdi12recorder" => {
                Some("**SDI12Recorder**\n\nRetrieves measurement results from an SDI-12 sensor.")
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
            "#if" => Some(
                "**#If**\n\nCompile-time conditional. Includes the following code only when the condition holds -- compared against `LoggerType` or a `Const`.\n\n```crbasic\n#If LoggerType = GRANITE6\n  ' statements\n#EndIf\n```",
            ),
            "#elseif" => {
                Some("**#ElseIf**\n\nAdditional compile-time conditional branch, following `#If`.")
            }
            "#else" => {
                Some("**#Else**\n\nAlternative compile-time branch, following `#If`/`#IfDef`.")
            }
            "#endif" => Some("**#EndIf**\n\nTerminates a `#If`/`#IfDef` block."),
            "#ifdef" => Some(
                "**#IfDef**\n\nCompile-time check for whether a `Const` has already been declared.\n\n```crbasic\nConst FINAL = 1\n#IfDef FINAL Then\n  Public Testing\n#EndIf\n```",
            ),
            "#undef" => Some(
                "**#UnDef**\n\nUn-declares a `Const` so it can be redeclared -- typically used with `Include` to stitch together library files that each define their own same-named constants.",
            ),
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
            "while" => Some(
                "**While**\n\nLoop condition. Used with `Do`/`Loop`, or as the start of a standalone `While`/`Wend` loop.\n\n```crbasic\nWhile condition\n  ' statements\nWend\n```",
            ),
            "wend" => Some("**Wend**\n\nTerminates a While loop."),
            "until" => Some(
                "**Until**\n\nLoop condition, the inverse of `While`. Used with `Do`/`Loop` at either the start or end of the block.\n\n```crbasic\nDo Until condition\n  ' statements\nLoop\n```",
            ),
            "exitfor" => Some("**ExitFor**\n\nImmediately exits a For loop."),
            "exitdo" => Some("**ExitDo**\n\nImmediately exits a Do loop."),
            "debugbreak" => Some(
                "**DebugBreak**\n\nSuspends program execution at this line when running under the CRBasic debugger.",
            ),
            "restart" => Some("**Restart**\n\nStops and restarts the running program."),
            "select" => Some(
                "**Select**\n\nMulti-way branch statement.\n\n```crbasic\nSelect Case expression\n  Case value1\n    ' statements\n  Case Else\n    ' default\nEndSelect\n```",
            ),
            "case" => Some("**Case**\n\nSpecifies a branch in a Select statement."),
            "is" => Some(
                "**Is**\n\nUsed with a comparison operator in a Case clause (e.g. `Case Is > 10`).",
            ),
            "endselect" => Some("**EndSelect**\n\nTerminates a Select block."),
            "endmenu" => Some(
                "**EndMenu**\n\nTerminates a `DisplayMenu` block.\n\n```crbasic\nDisplayMenu(\"MenuName\", 1, 1)\n  ' menu items\nEndMenu\n```",
            ),
            "endsubmenu" => Some(
                "**EndSubMenu**\n\nTerminates a `SubMenu` block nested inside a `DisplayMenu`.",
            ),
            "nextscan" => Some("**NextScan**\n\nMarks the end of a Scan loop."),
            "continuescan" => Some(
                "**ContinueScan**\n\nJumps to `NextScan`, skipping the remaining instructions in the current scan.",
            ),
            "nextsubscan" => Some("**NextSubScan**\n\nMarks the end of a SubScan block."),
            "slowsequence" => Some(
                "**SlowSequence**\n\nBegins a slow sequence scan block, for measurements at a slower rate than the main scan.\n\n```crbasic\nSlowSequence\n  Scan(...)\n    ' statements\n  NextScan\nEndSequence\n```",
            ),
            "endsequence" => Some("**EndSequence**\n\nTerminates a SlowSequence block."),
            "waittriggersequence" => Some(
                "**WaitTriggerSequence**\n\nMarks a resume-point inside a `SlowSequence`, suspending execution there until the trigger condition is met again.",
            ),
            "exitscan" => Some(
                "**ExitScan**\n\nExits the current Scan loop immediately, regardless of the scan Count.",
            ),

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
            "readonly" => Some(
                "**ReadOnly**\n\nMarks one or more previously declared `Public` variables as visible for monitoring but not externally editable.\n\n```crbasic\nPublic Mult, Offset\nReadOnly Mult, Offset\n```",
            ),
            "optional" => Some(
                "**Optional**\n\nMarks a `Function`/`Sub` parameter as optional.\n\n```crbasic\nFunction Scale(a, Optional b)\n  ...\nEndFunction\n```",
            ),
            "include" => Some(
                "**Include**\n\nPulls in an external CRBasic source file.\n\n```crbasic\nInclude \"cpu:Sensor_PT500_Lib.crb\"\n```",
            ),

            "beginprog" => {
                Some("**BeginProg**\n\nMarks the start of the main program execution block.")
            }
            "endprog" => Some("**EndProg**\n\nMarks the end of the main program execution block."),
            "sequentialmode" => Some(
                "**SequentialMode**\n\nForces the datalogger to run in sequential execution mode. Placed before `BeginProg`.",
            ),
            "pipelinemode" => Some(
                "**PipeLineMode**\n\nForces the datalogger to run in pipeline execution mode. Placed before `BeginProg`.",
            ),
            "preservevariables" => Some(
                "**PreserveVariables**\n\nRetains the values of all `Dim`/`Public` variables in memory across a power loss. Placed before `BeginProg`.",
            ),
            "angledegrees" => Some(
                "**AngleDegrees**\n\nSwitches `ATN`/`ATN2`/`ACOS`/`ASIN`/`RectPolar` to return degrees (instead of radians) and `COS`/`TAN`/`SIN` to interpret their arguments as degrees. Placed before `BeginProg`.",
            ),
            "applyandrestartsequence" => Some(
                "**ApplyAndRestartSequence**\n\nRuns arbitrary code when the `ConstTable` it follows has its `ApplyAndRestart` setting externally set (e.g. via `SetSetting`), typically to validate the table's new constant values before triggering the restart itself. Declared after the `ConstTable` it applies to, both before `BeginProg`.\n\n```crbasic\nConstTable(TableName, Hidden)\n  Const A = 1\nEndConstTable\nApplyAndRestartSequence\n  ' validation code\n  SetSetting(\"TableName.ApplyAndRestart\", 1)\nEndApplyAndRestartSequence\n```",
            ),
            "endapplyandrestartsequence" => Some(
                "**EndApplyAndRestartSequence**\n\nTerminates an ApplyAndRestartSequence block.",
            ),
            "shutdownbegin" => Some(
                "**ShutDownBegin**\n\nRuns cleanup code (e.g. closing a serial port) when the program stops normally. Placed before `BeginProg`.\n\n```crbasic\nShutDownBegin\n  SerialClose(ComC1)\nShutDownEnd\n```",
            ),
            "shutdownend" => Some("**ShutDownEnd**\n\nTerminates a ShutDownBegin block."),
            "essinitialize" => Some(
                "**ESSInitialize**\n\nInitializes the NTCIP Environmental Sensor Station SNMP agent for roadway-weather/DOT telemetry programs. Takes an optional SNMP read/write community string. Placed directly after `BeginProg`.\n\n```crbasic\nBeginProg\n  ESSInitialize(\"private, public\")\nEndProg\n```",
            ),
            "essvariables" => Some(
                "**ESSVariables**\n\nAuto-declares the standard set of NTCIP Environmental Sensor Station variables (used with `ESSInitialize` for roadway-weather/DOT telemetry programs). Takes an optional `Public` or `Dim` modifier (defaults to `Public`).\n\n```crbasic\nESSVariables Dim\n```",
            ),
            "webpageend" => Some(
                "**WebPageEnd**\n\nTerminates a `WebPageBegin` block.\n\n```crbasic\nWebPageBegin(\"Page1\", 1)\n  HTTPOut(\"Hello\", \"text/html\")\nWebPageEnd\n```",
            ),
            "endmodemhangup" => Some(
                "**EndModemHangup**\n\nTerminates a `ModemHangup` block.\n\n```crbasic\nModemHangup(ComC1)\n  ' cleanup instructions\nEndModemHangup\n```",
            ),
            "voicebeg" => Some(
                "**VoiceBeg**\n\nBegins a block of voice-modem response code.\n\n```crbasic\nVoiceBeg\n  ' voice response instructions\nEndVoice\n```",
            ),
            "endvoice" => Some("**EndVoice**\n\nTerminates a VoiceBeg block."),
            "datatable" => Some(
                "**DataTable**\n\nDefines a data table for storing measurements.\n\n```crbasic\nDataTable(TableName, TriggerCondition, Size)\n  ' output instructions\nEndTable\n```",
            ),
            "endtable" => Some("**EndTable**\n\nTerminates a DataTable block."),
            "tablehide" => Some(
                "**TableHide**\n\nSuppresses the display and data collection of this DataTable in datalogger memory. Placed immediately after the DataTable statement.",
            ),
            "openinterval" => Some(
                "**OpenInterval**\n\nMakes time series processing include all measurements since the last data storage, spanning any missed output intervals, instead of only the current interval.",
            ),
            "fillstop" => Some(
                "**FillStop**\n\nStops data storage once this DataTable reaches its configured size, instead of the default ring-memory behavior of overwriting the oldest records. Used within the DataTable declaration.",
            ),
            "calltable" => Some(
                "**CallTable**\n\nInvokes a previously declared DataTable: checks its trigger condition and stores a record if it fires. A bare keyword, not a parenthesized call.\n\n```crbasic\nCallTable TableName\n```",
            ),
            "consttable" => Some(
                "**ConstTable**\n\nDefines a block of constants that field technicians can edit and recompile without touching the constants' use sites. The second parameter, `Hidden`, is 1 to create a table visible only at the highest security level, or 0 (or omitted) for a standard visible table.\n\n```crbasic\nConstTable(TableName, Hidden)\n  Const A = 1\nEndConstTable\n```",
            ),
            "structuretype" => Some(
                "**StructureType**\n\nDefines a reusable data structure. Instances are declared with `Public`/`Dim ... As TypeName` and members are accessed with dot notation.\n\n```crbasic\nStructureType TempRHSensor\n  Temp As Float\n  RH As Float\nEndStructureType\n\nPublic CS215(3) As TempRHSensor\n' CS215(1).Temp\n```",
            ),
            "endstructuretype" => Some("**EndStructureType**\n\nTerminates a StructureType block."),
            "endconsttable" => Some("**EndConstTable**\n\nTerminates a ConstTable block."),

            "function" => Some(
                "**Function**\n\nDefines a user-defined function that returns a value.\n\n```crbasic\nFunction MyFunc(param As Float) As Float\n  MyFunc = param * 2\nEndFunction\n```",
            ),
            "endfunction" => Some("**EndFunction**\n\nTerminates a Function block."),
            "return" => Some(
                "**Return**\n\nReturns a value from a Function and exits it immediately.\n\n```crbasic\nReturn(expression)\n```",
            ),
            "exitfunction" => Some("**ExitFunction**\n\nExits a Function immediately."),
            "sub" => Some(
                "**Sub**\n\nDefines a subroutine (procedure without return value).\n\n```crbasic\nSub MySub(ByRef param As Float)\n  param = param * 2\nEndSub\n```",
            ),
            "endsub" => Some("**EndSub**\n\nTerminates a Sub block."),
            "exit" => {
                Some("**Exit**\n\nUsed with `Sub` to exit a Subroutine immediately: `Exit Sub`.")
            }
            "call" => Some(
                "**Call**\n\nInvokes a subroutine. Optional -- `Call MySub(x, y)` behaves the same as `MySub(x, y)`.\n\n```crbasic\nCall MySub(x, y)\n```",
            ),

            "and" => {
                Some("**AND**\n\nLogical AND operator. Returns true if both operands are true.")
            }
            "or" => Some("**OR**\n\nLogical OR operator. Returns true if either operand is true."),
            "not" => Some("**NOT**\n\nLogical NOT operator. Negates a boolean value."),
            "xor" => Some(
                "**XOR**\n\nLogical XOR operator. Returns true if exactly one operand is true.",
            ),
            "mod" => Some(
                "**MOD**\n\nModulo operator. Returns the remainder of `A / B`. Operands can be any number, not just integers (e.g. `19 MOD 6.7` = `5.6`).",
            ),
            "imp" => Some(
                "**IMP**\n\nLogical implication operator. `A IMP B` is equivalent to `(NOT A) OR B`.",
            ),
            "eqv" => Some(
                "**EQV**\n\nLogical equivalence operator. `A EQV B` is true when A and B have the same truth value (equivalent to `NOT (A XOR B)`).",
            ),
            "intdv" => Some(
                "**INTDV**\n\nInteger division operator. Keyword-form synonym for `\\`: `A INTDV B` divides A by B and truncates to an integer.",
            ),

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
                    "Is",
                    "EndSelect",
                    "Restart",
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
                let keywords = [
                    "Public", "Dim", "Const", "Alias", "As", "Units", "ReadOnly", "Optional",
                    "Include",
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

        mod program_structure_keywords {
            use super::*;

            #[test]
            fn all_program_structure_keywords_have_hover_info() {
                let keywords = [
                    "BeginProg",
                    "EndProg",
                    "DataTable",
                    "EndTable",
                    "TableHide",
                    "OpenInterval",
                    "FillStop",
                    "ConstTable",
                    "EndConstTable",
                    "StructureType",
                    "EndStructureType",
                    "PreserveVariables",
                    "AngleDegrees",
                    "ApplyAndRestartSequence",
                    "EndApplyAndRestartSequence",
                    "ShutDownBegin",
                    "ShutDownEnd",
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

        mod documentation_accuracy {
            use super::*;

            fn hover_text(keyword: &str) -> String {
                let hover = HoverProvider::get_keyword_hover(keyword)
                    .unwrap_or_else(|| panic!("expected hover info for keyword: {}", keyword));
                match hover.contents {
                    HoverContents::Markup(markup) => markup.value,
                    _ => panic!("expected MarkupContent"),
                }
            }

            #[test]
            fn mod_hover_does_not_claim_operands_must_be_integers() {
                let text = hover_text("MOD");

                assert!(
                    !text.to_lowercase().contains("integer division"),
                    "MOD accepts non-integer operands (e.g. 19 MOD 6.7 = 5.6 per \
                     help.campbellsci.com/crbasic/cr6/Content/Instructions/mod.htm), so its \
                     hover text must not describe it as integer division: {}",
                    text
                );
            }

            #[test]
            fn applyandrestartsequence_hover_is_declared_after_consttable() {
                let text = hover_text("ApplyAndRestartSequence");

                let consttable_pos = text.find("ConstTable(TableName");
                let sequence_pos = text.find("ApplyAndRestartSequence\n");
                assert!(
                    consttable_pos.is_some() && sequence_pos.is_some(),
                    "expected the example to contain both ConstTable and \
                     ApplyAndRestartSequence blocks: {}",
                    text
                );
                assert!(
                    consttable_pos.unwrap() < sequence_pos.unwrap(),
                    "the official example \
                     (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/applyandrestartsequence.htm) \
                     declares ConstTable before ApplyAndRestartSequence, not after: {}",
                    text
                );
            }

            #[test]
            fn consttable_hover_names_its_second_parameter_hidden_not_enabled() {
                let text = hover_text("ConstTable");

                assert!(
                    !text.contains("Enabled") && text.contains("Hidden"),
                    "the official syntax \
                     (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/consttableendconsttable.htm) \
                     names the second parameter Hidden (1 = visible only at highest security \
                     level, 0/omitted = standard visible table), not Enabled: {}",
                    text
                );
            }

            #[test]
            fn fillstop_hover_does_not_claim_it_must_immediately_follow_datatable() {
                let text = hover_text("FillStop");

                assert!(
                    !text.contains("immediately after the DataTable statement"),
                    "the official example \
                     (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/fillstop.htm) \
                     places FillStop after DataInterval(...), not immediately after DataTable: {}",
                    text
                );
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

    mod builtin_function_hover {
        use super::*;

        mod scan_functions {
            use super::*;

            #[test]
            fn all_scan_functions_have_hover_info() {
                for name in ["Scan", "SubScan"] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        #[test]
        fn every_canonical_builtin_function_has_hover_info() {
            let missing: Vec<&str> = crbasic_parser::BUILTIN_FUNCTIONS
                .iter()
                .map(|(name, _)| *name)
                .filter(|name| HoverProvider::get_builtin_function_description(name).is_none())
                .collect();

            assert!(
                missing.is_empty(),
                "Missing hover info for builtin functions: {:?}",
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
        fn returns_hover_for_data_type_after_as() {
            // "As Float": A(0)s(1) (2)F(3) -- character 3 is the start of "Float"
            let tokens = tokenize("As Float");
            let position = Position {
                line: 0,
                character: 3,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**Float**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn returns_hover_for_output_processing_data_type_in_sample_call() {
            // "Sample(1,Var,FP2)": character 13 is the start of "FP2"
            let tokens = tokenize("Sample(1,Var,FP2)");
            let position = Position {
                line: 0,
                character: 13,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**FP2**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn returns_hover_for_windvector_function_name() {
            let tokens = tokenize("WindVector(1,WS_ms,WindDir,IEEE4,0,0,0,0)");
            let position = Position {
                line: 0,
                character: 0,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**WindVector**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn returns_hover_for_tcdiff_function_name() {
            let tokens = tokenize("TCDiff(Dest,1,mV200C,1,TypeT,PTemp,True,0,15000,1.0,0)");
            let position = Position {
                line: 0,
                character: 0,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**TCDiff**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn returns_hover_for_resistance_function_name() {
            let tokens = tokenize("Resistance(Dest,1,mV5000,U1,U7,3,2500,True,True,0,60,1.0,0)");
            let position = Position {
                line: 0,
                character: 0,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**Resistance**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
        }

        #[test]
        fn returns_hover_for_sdi12recorder_function_name() {
            let tokens = tokenize("SDI12Recorder(SR50A(),C1,\"0\",\"C1!\",1,0)");
            let position = Position {
                line: 0,
                character: 0,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**SDI12Recorder**"));
                }
                _ => panic!("Expected MarkupContent"),
            }
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
