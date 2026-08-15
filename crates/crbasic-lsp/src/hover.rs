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
                .or_else(|| Self::get_preprocessor_constant_description(name))
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

    /// Returns the description for `LoggerType`, the predefined constant
    /// compared inside `#If`/`#ElseIf` conditions, or `None` if `name` isn't
    /// it (e.g. an ordinary variable name).
    ///
    /// Not part of `LANGUAGE_KEYWORDS` for the same reason as
    /// `get_data_type_description`: `LoggerType` is lexed as a plain
    /// identifier, and reclassifying it as a keyword would break `#If
    /// LoggerType = ...` parsing (only `True`/`False` have a primary-
    /// expression fallback for bare keyword tokens today).
    fn get_preprocessor_constant_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "loggertype" => Some(
                "**LoggerType**\n\nPredefined constant for `#If`/`#ElseIf` model comparisons (e.g. `#If LoggerType = CR1000X`), letting one program compile differently per datalogger model.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in function name (any category),
    /// or `None` if `name` isn't one of `BUILTIN_FUNCTIONS` (e.g. an
    /// ordinary variable name).
    fn get_builtin_function_description(name: &str) -> Option<&'static str> {
        Self::get_scan_function_description(name)
            .or_else(|| Self::get_measurement_function_description(name))
            .or_else(|| Self::get_communication_function_description(name))
            .or_else(|| Self::get_data_function_description(name))
            .or_else(|| Self::get_string_function_description(name))
            .or_else(|| Self::get_math_function_description(name))
            .or_else(|| Self::get_time_function_description(name))
            .or_else(|| Self::get_logical_function_description(name))
            .or_else(|| Self::get_menu_function_description(name))
    }

    /// Returns the description for the built-in `IIf` conditional-expression
    /// function, or `None` if `name` isn't it.
    fn get_logical_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "iif" => Some(
                "**IIf**\n\nEvaluates a Boolean expression and returns TrueValue if true, otherwise FalseValue.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in custom-menu function name, or
    /// `None` if `name` isn't one of them.
    fn get_menu_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "displaymenu" => Some(
                "**DisplayMenu**\n\nMarks the beginning of a custom on-screen menu definition.",
            ),
            "submenu" => Some(
                "**SubMenu**\n\nMarks the beginning of a nested custom menu within a DisplayMenu block.",
            ),
            "menuitem" => Some(
                "**MenuItem**\n\nDefines an editable custom-menu entry showing the name and value of a variable.",
            ),
            "menupick" => Some(
                "**MenuPick**\n\nCreates a fixed pick-list of selectable values for the preceding MenuItem.",
            ),
            "menurecompile" => Some(
                "**MenuRecompile**\n\nCreates a custom menu item that triggers a program recompile after Constant Table edits.",
            ),
            "displayvalue" => Some(
                "**DisplayValue**\n\nDefines a read-only custom-menu entry showing a data-table field, variable, or expression.",
            ),
            "displayline" => Some(
                "**DisplayLine**\n\nDisplays a single line of read-only text in a custom menu.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in time/system function name,
    /// or `None` if `name` isn't one of them.
    fn get_time_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "realtime" => Some("**RealTime**\n\nReturns the current real-time clock values."),
            "clockchange" => Some(
                "**ClockChange**\n\nReturns the number of milliseconds the datalogger's clock has changed since the last time this instruction was executed.",
            ),
            "clockset" => Some(
                "**ClockSet**\n\nSets the datalogger clock from the values in a 7-element array.",
            ),
            "timeintointerval" => {
                Some("**TimeIntoInterval**\n\nReturns true when the interval boundary is crossed.")
            }
            "iftime" => Some("**IfTime**\n\nReturns true at specified time intervals."),
            "timer" => Some("**Timer**\n\nReturns elapsed time from a timer."),
            "delay" => Some("**Delay**\n\nPauses execution for a specified time."),
            "setstatus" => Some(
                "**SetStatus**\n\nChanges the value of a field in the datalogger's Status table.",
            ),
            "setsetting" => Some(
                "**SetSetting**\n\nChanges the value of a field in the datalogger's Settings table.",
            ),
            "movebytes" => Some(
                "**MoveBytes**\n\nMoves binary bytes of data from one memory location to another.",
            ),
            "arraylength" => Some(
                "**ArrayLength**\n\nReturns the total number of elements across all dimensions of an array.",
            ),
            "nan" => Some(
                "**NaN**\n\nRepresents the IEEE-754 Not-a-Number value used to flag an invalid measurement. Takes no parentheses.",
            ),
            "secssince1990" => Some(
                "**SecsSince1990**\n\nConverts between a date/time string and the number of seconds since January 1, 1990.",
            ),
            "timeisbetween" => Some(
                "**TimeIsBetween**\n\nReturns true when the datalogger's real-time clock falls within a specified time range.",
            ),
            "daylightsaving" => Some(
                "**DaylightSaving**\n\nDetects a custom-rule daylight saving transition and returns the clock adjustment.",
            ),
            "daylightsavingus" => Some(
                "**DaylightSavingUS**\n\nDetects a US-rule daylight saving transition and returns the clock adjustment.",
            ),
            "instructiontimes" => Some(
                "**InstructionTimes**\n\nPopulates an array with the processing time, in microseconds, of every program line; must precede BeginProg.",
            ),
            "linenum" => {
                Some("**LineNum**\n\nReturns the current program line number, for debugging.")
            }
            "signature" => Some(
                "**Signature**\n\nReturns a pseudo-random signature of the program code between two Signature markers.",
            ),
            "arrayindex" => Some(
                "**ArrayIndex**\n\nReturns the numeric index of a named array element whose position isn't known in advance.",
            ),
            "debug" => Some(
                "**Debug**\n\nConfigures breakpoint and trace-history behavior for the CRBasic debugger; used together with DebugBreak.",
            ),
            "move" => Some(
                "**Move**\n\nCopies a run of values from a source array/variable into a destination range.",
            ),
            "semaphoreget" => Some(
                "**SemaphoreGet**\n\nWaits for and claims a semaphore, blocking until it is free.",
            ),
            "semaphorerelease" => Some(
                "**SemaphoreRelease**\n\nReleases a semaphore previously claimed with SemaphoreGet.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in math function name, or
    /// `None` if `name` isn't one of them.
    fn get_math_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "abs" => Some("**Abs**\n\nReturns the absolute value."),
            "sgn" => Some("**Sgn**\n\nReturns the sign of a number as -1, 0, or 1."),
            "sqr" => Some("**Sqr**\n\nReturns the square root."),
            "exp" => Some("**Exp**\n\nReturns e raised to a power."),
            "ln" => Some("**Ln**\n\nReturns the natural logarithm."),
            "log" => Some("**Log**\n\nReturns the natural logarithm."),
            "log10" => Some("**Log10**\n\nReturns the base-10 logarithm."),
            "sin" => Some("**Sin**\n\nReturns the sine."),
            "cos" => Some("**Cos**\n\nReturns the cosine."),
            "tan" => Some("**Tan**\n\nReturns the tangent."),
            "sinh" => Some("**Sinh**\n\nReturns the hyperbolic sine."),
            "cosh" => Some("**Cosh**\n\nReturns the hyperbolic cosine."),
            "tanh" => Some("**Tanh**\n\nReturns the hyperbolic tangent."),
            "asin" => Some("**Asin**\n\nReturns the arc sine."),
            "acos" => Some("**Acos**\n\nReturns the arc cosine."),
            "atn" => Some("**Atn**\n\nReturns the arc tangent."),
            "atn2" => Some("**Atn2**\n\nReturns the arc tangent of Y/X."),
            "int" => {
                Some("**Int**\n\nReturns the integer part (truncates toward negative infinity).")
            }
            "fix" => Some("**Fix**\n\nReturns the integer part (truncates toward zero)."),
            "frac" => Some("**Frac**\n\nReturns the fractional portion of a number."),
            "round" => Some("**Round**\n\nRounds to specified decimal places."),
            "rnd" => Some(
                "**Rnd**\n\nReturns a random value between 0 (inclusive) and 1 (exclusive). Takes no parentheses.",
            ),
            "randomize" => Some(
                "**Randomize**\n\nInitializes the random-number generator used by Rnd with a new seed value.",
            ),
            "ceiling" => Some("**Ceiling**\n\nRounds a number up to the nearest integer."),
            "floor" => Some("**Floor**\n\nRounds a number down to the nearest integer."),
            "matrix" => Some(
                "**Matrix**\n\nPerforms matrix math (add, subtract, multiply, transpose, invert) on 2-D arrays.",
            ),
            "minspa" => Some(
                "**MinSpa**\n\nFinds the minimum value across a spatial array and its index, writing both into a 2-element Dest array.",
            ),
            "sortspa" => Some(
                "**SortSpa**\n\nSorts array elements in ascending order, with NaN and infinite values sorted to the beginning.",
            ),
            "findspa" => Some(
                "**FindSpa**\n\nSearches an array for a value within a specified range, returning its position, or 0 if not found.",
            ),
            "rectpolar" => Some(
                "**RectPolar**\n\nConverts rectangular coordinates (X, Y) into polar coordinates (vector length and angle in radians).",
            ),
            "satvp" => Some(
                "**SatVP**\n\nCalculates saturation vapor pressure, in kilopascals, from a temperature measurement.",
            ),
            "straincalc" => Some(
                "**StrainCalc**\n\nConverts the mV/V output from a bridge measurement into microstrain (µε), using the specified bridge configuration.",
            ),
            "vaporpressure" => Some(
                "**VaporPressure**\n\nCalculates vapor pressure, in kilopascals, from temperature and relative humidity.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in string-manipulation function
    /// name, or `None` if `name` isn't one of them.
    fn get_string_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "splitstr" => Some("**SplitStr**\n\nSplits a string by delimiter."),
            "formatfloat" => Some("**FormatFloat**\n\nFormats a float as a string."),
            "formatlong" => Some(
                "**FormatLong**\n\nConverts a Long value to a decimal, hexadecimal, or octal string.",
            ),
            "formatlonglong" => Some(
                "**FormatLongLong**\n\nConverts a 64-bit value, stored across two adjacent Long variables, into its decimal string representation.",
            ),
            "mid" => Some("**Mid**\n\nExtracts a substring."),
            "left" => Some("**Left**\n\nReturns leftmost characters."),
            "right" => Some("**Right**\n\nReturns rightmost characters."),
            "len" => Some("**Len**\n\nReturns the length of a string."),
            "instr" => Some("**InStr**\n\nFinds a substring within a string."),
            "lowercase" => Some("**LowerCase**\n\nConverts to lowercase."),
            "uppercase" => Some("**UpperCase**\n\nConverts to uppercase."),
            "trim" => Some("**Trim**\n\nRemoves leading and trailing spaces."),
            "rtrim" => Some("**RTrim**\n\nRemoves trailing spaces."),
            "ltrim" => Some("**LTrim**\n\nRemoves leading spaces."),
            "replace" => Some("**Replace**\n\nReplaces occurrences in a string."),
            "chr" => Some("**Chr**\n\nReturns a character in the extended ASCII character set."),
            "ascii" => Some("**ASCII**\n\nReturns the ASCII value of a character in a string."),
            "strcomp" => Some(
                "**StrComp**\n\nCompares two strings to determine if they are identical or their sort order.",
            ),
            "checksum" => {
                Some("**CheckSum**\n\nReturns a checksum signature for the characters in a string.")
            }
            "hextodec" => {
                Some("**HexToDec**\n\nConverts a hexadecimal string to a float or integer.")
            }
            "hex" => {
                Some("**Hex**\n\nReturns a hexadecimal string representation of a Long value.")
            }
            "sprintf" => {
                Some("**Sprintf**\n\nWrites a formatted output string to a destination variable.")
            }
            "typeof" => Some("**TypeOf**\n\nReturns an integer data-type code for a variable."),
            _ => None,
        }
    }

    /// Returns the description for a built-in data-table/output-processing
    /// or file-management function name, or `None` if `name` isn't one of
    /// them.
    fn get_data_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "sample" => Some("**Sample**\n\nSamples and stores a value in the data table."),
            "average" => Some("**Average**\n\nCalculates and stores the average of values."),
            "stddev" => Some("**StdDev**\n\nCalculates and stores the standard deviation."),
            "minimum" => Some("**Minimum**\n\nStores the minimum value over the output interval."),
            "maximum" => Some("**Maximum**\n\nStores the maximum value over the output interval."),
            "totalize" => {
                Some("**Totalize**\n\nStores the sum of values over the output interval.")
            }
            "histogram" => Some(
                "**Histogram**\n\nStores a frequency distribution of input data across a set of bins.",
            ),
            "histogram4d" => Some(
                "**Histogram4D**\n\nProcesses input data as a standard or weighted-value histogram of up to four dimensions.",
            ),
            "median" => Some(
                "**Median**\n\nStores the median of a variable, over time, in an output table.",
            ),
            "moment" => Some(
                "**Moment**\n\nOutputs a central moment (variance, skewness, kurtosis, etc.) of a value over the measurement interval.",
            ),
            "samplemaxmin" => Some(
                "**SampleMaxMin**\n\nRecords the value of a companion variable at the moment a preceding Maximum or Minimum reaches its extremum.",
            ),
            "peakvalley" => Some(
                "**PeakValley**\n\nDetects local maxima and minima in signal data with a hysteresis threshold.",
            ),
            "fft" => Some(
                "**FFT**\n\nPerforms a Fast Fourier Transform on time-series measurement data.",
            ),
            "covariance" => Some(
                "**Covariance**\n\nComputes time-series covariance among array elements, for eddy-flux systems.",
            ),
            "levelcrossing" => Some(
                "**LevelCrossing**\n\nBuilds a 1D or 2D level-crossing histogram for fatigue counting.",
            ),
            "worstcase" => Some(
                "**WorstCase**\n\nSaves ranked worst-case data events into separate clone tables.",
            ),
            "fieldnames" => Some(
                "**FieldNames**\n\nOverrides the default field names for the preceding output-processing instruction.",
            ),
            "cardout" => {
                Some("**CardOut**\n\nCreates a new data table that is stored on a memory card.")
            }
            "newfile" => Some(
                "**NewFile**\n\nDetermines whether a monitored file has been newly written since this instruction last ran.",
            ),
            "filemanage" => Some(
                "**FileManage**\n\nPerforms a management operation, such as delete, hide, run, or format, on a file or device.",
            ),
            "fileopen" => Some(
                "**FileOpen**\n\nOpens a file for reading or writing and returns a file handle.",
            ),
            "fileclose" => Some("**FileClose**\n\nCloses a file previously opened with FileOpen."),
            "fileread" => {
                Some("**FileRead**\n\nReads data from an open file into a variable or array.")
            }
            "filewrite" => {
                Some("**FileWrite**\n\nWrites data from a variable or array to an open file.")
            }
            "fileencrypt" => Some(
                "**FileEncrypt**\n\nEncrypts a file in place; the datalogger automatically decrypts it at compile time when referenced.",
            ),
            "filecopy" => {
                Some("**FileCopy**\n\nCopies a file from one drive on the datalogger to another.")
            }
            "filerename" => Some("**FileRename**\n\nRenames a file stored on the datalogger."),
            "filesize" => Some("**FileSize**\n\nReturns the size, in bytes, of a specified file."),
            "filetime" => {
                Some("**FileTime**\n\nReturns the last-modified timestamp of a specified file.")
            }
            "filelist" => Some(
                "**FileList**\n\nWrites the list of file names on a device into a destination array.",
            ),
            "datainterval" => Some(
                "**DataInterval**\n\nSets the real-time-clock-based interval on which a data table's records are generated.",
            ),
            "cardflush" => Some(
                "**CardFlush**\n\nImmediately writes buffered data to an external storage device.",
            ),
            "dataevent" => Some(
                "**DataEvent**\n\nConditionally starts and stops data storage to a table based on trigger conditions.",
            ),
            "data" => {
                Some("**Data**\n\nDefines a list of Float constants for later retrieval with Read.")
            }
            "datalong" => Some(
                "**DataLong**\n\nDefines a list of Long constants for later retrieval with Read.",
            ),
            "datatime" => Some(
                "**DataTime**\n\nSelects whether a data table's records are timestamped at scan time or at storage time.",
            ),
            "resettable" => Some(
                "**ResetTable**\n\nErases all records from a specified data table during program execution.",
            ),
            "tablefile" => Some(
                "**TableFile**\n\nWrites a data table's contents to external storage media; placed inside a DataTable/EndTable declaration.",
            ),
            "filemark" => Some(
                "**FileMark**\n\nInserts a filemark into a data table, signaling file-splitting software to start a new file.",
            ),
            "filereadline" => Some(
                "**FileReadLine**\n\nReads one line from an open file into a destination variable.",
            ),
            "erase" => Some("**Erase**\n\nSets all bytes of a variable or array to zero."),
            "getrecord" => Some(
                "**GetRecord**\n\nRetrieves one complete record from a data table into an array.",
            ),
            "gzip" => Some(
                "**Gzip**\n\nCompresses one or more files on datalogger storage into a .gz or .tar.gz archive.",
            ),
            "newfieldnames" => Some(
                "**NewFieldNames**\n\nRenames the auto-generated field name(s) of a variable or array for table output.",
            ),
            "rainflow" => Some(
                "**RainFlow**\n\nPerforms rainflow cycle-counting on a signal, building an amplitude/mean histogram for fatigue analysis.",
            ),
            "read" => Some(
                "**Read**\n\nReads the next value(s) sequentially from a Data/DataLong constant list into a variable, advancing the read pointer.",
            ),
            "restore" => Some(
                "**Restore**\n\nResets the read pointer to the start of a Data/DataLong constant list, so a following Read restarts from the beginning.",
            ),
            "runprogram" => Some(
                "**RunProgram**\n\nRuns a specified datalogger program file, replacing the currently active program.",
            ),
            "stationname" => Some(
                "**StationName**\n\nAssigns a name to the station, stored in the Status table; declared once near the top of the program.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in communication function name
    /// (serial, TCP/UDP, Modbus, SDI-12, email, PPP, FTP, HTTP), or `None`
    /// if `name` isn't one of them.
    fn get_communication_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "serialopen" => Some("**SerialOpen**\n\nOpens a serial communication port."),
            "serialclose" => Some("**SerialClose**\n\nCloses a serial communication port."),
            "serialin" => Some("**SerialIn**\n\nReads data from a serial port."),
            "serialout" => Some("**SerialOut**\n\nSends data to a serial port."),
            "serialinrecord" => Some(
                "**SerialInRecord**\n\nReads and parses incoming serial data using begin/end markers.",
            ),
            "serialoutblock" => Some("**SerialOutBlock**\n\nSends binary data out a serial port."),
            "serialflush" => {
                Some("**SerialFlush**\n\nClears any characters in the serial input buffer.")
            }
            "serialbrk" => Some(
                "**SerialBrk**\n\nSends a break signal of a specified duration to a serial communication port.",
            ),
            "modbusmaster" => Some(
                "**ModbusMaster**\n\nSets up the datalogger as a Modbus client to send or retrieve data from a Modbus server.",
            ),
            "sdi12recorder" => {
                Some("**SDI12Recorder**\n\nRetrieves measurement results from an SDI-12 sensor.")
            }
            "tcpopen" => Some("**TCPOpen**\n\nSets up a TCP/IP socket for communication."),
            "tcpclose" => {
                Some("**TCPClose**\n\nCloses a TCP/IP socket that was set up for communication.")
            }
            "udpopen" => Some("**UDPOpen**\n\nOpens a port for transferring UDP packets."),
            "udpsocketopen" => Some(
                "**UDPSocketOpen**\n\nOpens a UDP socket, relating a UDP source port to an ID.",
            ),
            "udpsocketsend" => Some(
                "**UDPSocketSend**\n\nSends a UDP datagram to a remote device via an opened UDP socket.",
            ),
            "udpsocketrecv" => Some(
                "**UDPSocketRecv**\n\nRetrieves incoming UDP packets sent to a socket's listening port.",
            ),
            "udpsocketclose" => Some(
                "**UDPSocketClose**\n\nCloses an opened UDP socket and frees its associated memory.",
            ),
            "emailrelay" => Some(
                "**EmailRelay**\n\nSends an email message to one or more addresses via a Campbell Scientific relay service.",
            ),
            "emailsend" => {
                Some("**EmailSend**\n\nSends an email message directly via an SMTP server.")
            }
            "dialmodem" => Some(
                "**DialModem**\n\nDials a modem device over a communications port and checks for an expected response.",
            ),
            "dialsequence" => Some(
                "**DialSequence**\n\nDeclares a PakBus dial-out route/sequence, closed by EndDialSequence.",
            ),
            "enddialsequence" => Some(
                "**EndDialSequence**\n\nCloses a DialSequence block, reporting whether the dial sequence succeeded.",
            ),
            "modemhangup" => Some(
                "**ModemHangup**\n\nDeclares code to run when a communications port hangs up, closed by EndModemHangup.",
            ),
            "dialvoice" => Some(
                "**DialVoice**\n\nDials out over a voice modem, or waits for an incoming call if no dial string is given. Returns -1 (success), 0 (failure), or -3 (no voice modem present).",
            ),
            "voicehangup" => Some(
                "**VoiceHangup**\n\nHangs up the voice modem after a DialVoice connection; not needed inside a VoiceBeg/EndVoice block, which hangs up automatically via EndVoice.",
            ),
            "voicekey" => Some(
                "**VoiceKey**\n\nWaits for and returns a single DTMF key (0-9, *, #) pressed by the caller, or a timeout/carrier-loss/no-modem/disconnect code.",
            ),
            "voicenumber" => Some(
                "**VoiceNumber**\n\nWaits for and returns a multi-digit DTMF number entered by the caller (terminated by # or timeout), or a timeout/carrier-loss/no-modem/disconnect code.",
            ),
            "voicephrases" => Some(
                "**VoicePhrases**\n\nSpeaks a comma-separated list of vocabulary words/phrases over the voice modem.",
            ),
            "voicesetup" => Some(
                "**VoiceSetup**\n\nConfigures the DTMF keys and timing behavior used by the voice-modem instructions.",
            ),
            "voicespeak" => Some(
                "**VoiceSpeak**\n\nSpeaks a string built by concatenating literal text and variable values over the voice modem.",
            ),
            "smsrecv" => {
                Some("**SMSRecv**\n\nPolls a CELL2XX cellular modem for a pending SMS message.")
            }
            "smssend" => Some("**SMSSend**\n\nSends an SMS message via a CELL2XX cellular modem."),
            "pppopen" => Some(
                "**PPPOpen**\n\nEnables a PPP network connection through an external modem and returns its IP address.",
            ),
            "pppclose" => Some("**PPPClose**\n\nCloses an open PPP connection with a server."),
            "ftpclient" => {
                Some("**FTPClient**\n\nManages files on a server using FTP, FTPS, or SFTP.")
            }
            "httpget" => Some("**HTTPGet**\n\nSends a GET request to an HTTP server."),
            "httppost" => {
                Some("**HTTPPost**\n\nSends files or text to a URL via an HTTP POST request.")
            }
            "httpput" => {
                Some("**HTTPPut**\n\nSends files or text to a URL via an HTTP PUT request.")
            }
            "gps" => Some(
                "**GPS**\n\nSynchronizes the datalogger clock with a GPS receiver and stores its position/timing data.",
            ),
            "ethernetpower" => {
                Some("**EthernetPower**\n\nTurns power to all Ethernet devices on or off.")
            }
            "i2copen" => Some(
                "**I2COpen**\n\nConfigures a port pair for I2C communication at a specified clock rate.",
            ),
            "i2cread" => Some("**I2CRead**\n\nReads bytes from an I2C peripheral device."),
            "i2cwrite" => Some("**I2CWrite**\n\nWrites bytes to an I2C peripheral device."),
            "spiopen" => Some(
                "**SPIOpen**\n\nConfigures the datalogger as an SPI controller for communication with peripheral devices.",
            ),
            "spiread" => Some(
                "**SPIRead**\n\nSynchronously reads a specified number of bytes from an SPI peripheral device.",
            ),
            "spiwrite" => Some(
                "**SPIWrite**\n\nSynchronously transmits a specified number of bytes to an SPI peripheral device.",
            ),
            "acceptdatarecords" => Some(
                "**AcceptDataRecords**\n\nConfigures the datalogger to receive and store data records pushed from a remote PakBus datalogger.",
            ),
            "broadcast" => Some(
                "**Broadcast**\n\nSends a broadcast message to all devices on a PakBus network.",
            ),
            "clockreport" => Some(
                "**ClockReport**\n\nSends this datalogger's clock value to a specified PakBus device.",
            ),
            "datagram" => Some(
                "**DataGram**\n\nInitializes a SerialServer/DataGram application that tunnels serial traffic through a PakBus network.",
            ),
            "encryptexempt" => Some(
                "**EncryptExempt**\n\nDeclares a PakBus address range exempt from PakBus encryption.",
            ),
            "getdatarecord" => Some(
                "**GetDataRecord**\n\nRetrieves the most recent record(s) from a table on a remote PakBus datalogger into a local table.",
            ),
            "getfile" => Some(
                "**GetFile**\n\nRetrieves a file from a remote PakBus datalogger and stores it locally.",
            ),
            "getvariables" => Some(
                "**GetVariables**\n\nRetrieves one or more variable values from a data table on a remote PakBus device.",
            ),
            "pakbusclock" => Some(
                "**PakBusClock**\n\nConfigures the datalogger to accept and synchronize its clock from time broadcasts sent by a specified PakBus device.",
            ),
            "route" => Some(
                "**Route**\n\nReturns the neighbor address of, or the route to, a PakBus datalogger.",
            ),
            "routes" => Some(
                "**Routes**\n\nRetrieves the datalogger's list of known dynamic PakBus routes into an array.",
            ),
            "routersneighbors" => Some(
                "**RoutersNeighbors**\n\nReturns a list of all PakBus routers and their neighbors known to the datalogger.",
            ),
            "senddata" => Some(
                "**SendData**\n\nSends the most recent record from a data table to a destination PakBus device.",
            ),
            "sendfile" => Some(
                "**SendFile**\n\nSends a file from the datalogger to another PakBus datalogger.",
            ),
            "sendgetvariables" => Some(
                "**SendGetVariables**\n\nSends and/or retrieves an array of values to/from the host datalogger during its assigned time slot.",
            ),
            "sendtabledef" => Some(
                "**SendTableDef**\n\nSends a data table's definition to a destination device on the PakBus network.",
            ),
            "sendvariables" => Some(
                "**SendVariables**\n\nSends one or more variable values to a table in a destination PakBus device.",
            ),
            "staticroute" => Some(
                "**StaticRoute**\n\nDefines a fixed route to a PakBus datalogger, for use when dynamic routing is unavailable.",
            ),
            "timeuntiltransmit" => Some(
                "**TimeUntilTransmit**\n\nReturns the seconds remaining until the datalogger's assigned communication time slot with its host.",
            ),
            "dnp" => Some(
                "**DNP**\n\nConfigures a communications port to set up the datalogger as a DNP3 outstation device.",
            ),
            "dnpupdate" => Some(
                "**DNPUpdate**\n\nSets up the datalogger as a DNP3 outstation and determines when it updates its arrays of DNP elements.",
            ),
            "dnpvariable" => Some(
                "**DNPVariable**\n\nMaps a variable or array to a DNP3 object, variation, and class within the datalogger's outstation configuration.",
            ),
            "argosdata" => Some(
                "**ArgosData**\n\nSpecifies the data to be transmitted to the Argos satellite.",
            ),
            "argosdatarepeat" => {
                Some("**ArgosDataRepeat**\n\nSets the repeat rate for the ArgosData instruction.")
            }
            "argoserror" => Some(
                "**ArgosError**\n\nRequests and clears the current error message from the Argos transmitter.",
            ),
            "argossetup" => Some(
                "**ArgosSetup**\n\nSets up the datalogger for transmitting data via an Argos satellite.",
            ),
            "argostransmit" => Some(
                "**ArgosTransmit**\n\nInitiates a single transmission to an Argos satellite when the instruction is executed.",
            ),
            "goesdata" => Some(
                "**GOESData**\n\nTransmits data from a data table to a GOES satellite transmitter.",
            ),
            "goesfield" => Some(
                "**GOESField**\n\nDeclares an output field to include in a GOES transmission; precedes the data-table field instruction it applies to.",
            ),
            "goesgps" => Some(
                "**GOESGPS**\n\nRetrieves GPS data from a compatible GOES satellite transmitter and stores it in two variable arrays.",
            ),
            "goessetup" => Some(
                "**GOESSetup**\n\nConfigures a GOES satellite transmitter for communication with the satellite.",
            ),
            "goesstatus" => Some(
                "**GOESStatus**\n\nRequests status and diagnostic information from a GOES satellite transmitter.",
            ),
            "goestable" => Some(
                "**GOESTable**\n\nFormats and outputs a data table's records to a TX325/TX326 GOES satellite transmitter.",
            ),
            "sdmao4" => Some(
                "**SDMAO4**\n\nSets the output voltage on an SDM-AO4 four-channel analog output device.",
            ),
            "sdmao4a" => Some(
                "**SDMAO4A**\n\nSets the output voltage on an SDM-AO4A four-channel analog output device.",
            ),
            "sdmbeginport" => Some(
                "**SDMBeginPort**\n\nDesignates an alternate set of datalogger terminals to use as an SDM port.",
            ),
            "sdmcan" => Some(
                "**SDMCAN**\n\nConfigures and operates the SDM-CAN interface between a CAN-bus network and the datalogger.",
            ),
            "sdmcd16ac" => Some(
                "**SDMCD16AC**\n\nEnables or disables the relay ports of an SDM-CD16AC relay control device.",
            ),
            "sdmcd16mask" => Some(
                "**SDMCD16Mask**\n\nEnables or disables specific relay ports of an SDM-CD16AC device via a bit-mask filter.",
            ),
            "timedcontrol" => Some(
                "**TimedControl**\n\nRuns a timed sequence of binary output values through an SDMCD16 peripheral, synchronized to a specified interval.",
            ),
            "sdmcvo4" => Some(
                "**SDMCVO4**\n\nControls the SDM-CVO4 four-channel current/voltage output device.",
            ),
            "sdmgeneric" => Some(
                "**SDMGeneric**\n\nSends raw commands to an SDM device with no dedicated CRBasic instruction support.",
            ),
            "sdmint8" => Some(
                "**SDMINT8**\n\nPrograms and controls the SDM-INT8 eight-channel interval timer.",
            ),
            "sdmio16" => Some(
                "**SDMIO16**\n\nSets up and operates an SDM-IO16 16-port digital I/O expansion device.",
            ),
            "sdmsio4" => Some(
                "**SDMSIO4**\n\nControls and transfers data with a legacy SDM-SIO4 four-port serial I/O device.",
            ),
            "sdmspeed" => Some(
                "**SDMSpeed**\n\nChanges the bit period the datalogger uses to clock SDM bus communication.",
            ),
            "sdmsw8a" => Some(
                "**SDMSW8A**\n\nReads channels from an SDM-SW8A eight-channel switch closure module.",
            ),
            "sdmtrigger" => Some(
                "**SDMTrigger**\n\nBroadcasts a simultaneous \"measure now\" group trigger to all SDM devices that support it.",
            ),
            "sdmx50" => {
                Some("**SDMX50**\n\nSwitches an SDMX50 coaxial multiplexer to a specified channel.")
            }
            "cpiaddmodule" => Some(
                "**CPIAddModule**\n\nStatically assigns a CPI-bus address to a GRANITE/CDM/VWIRE module.",
            ),
            "cpifilesend" => Some(
                "**CPIFileSend**\n\nSends an OS file to a GRANITE/CDM module over the CPI bus via memory card, USR drive, or USB.",
            ),
            "cpispeed" => Some(
                "**CPISpeed**\n\nAdjusts the CPI bus bit rate, needed when the bus load or cable length requires a slower speed.",
            ),
            "mqttpublishtable" => Some(
                "**MQTTPublishTable**\n\nPublishes a data table's contents to an MQTT broker; placed inside a DataTable/EndTable declaration.",
            ),
            "mqttpublishconsttable" => Some(
                "**MQTTPublishConstTable**\n\nEnables remote editing of ConstTable values via MQTT; placed inside a ConstTable/EndConstTable declaration.",
            ),
            "mqttconnect" => Some(
                "**MQTTConnect**\n\nOverrides the default MQTT-publish retry schedule, forcing a connect or disconnect attempt over an available IP connection.",
            ),
            "cwbdiagnostics" => Some(
                "**CWBDiagnostics**\n\nReturns diagnostic information about a CWB100 wireless sensor base's network performance.",
            ),
            "checkport" => Some(
                "**CheckPort**\n\nRetrieves the current status (high/low) of a specified digital port or terminal.",
            ),
            "comportisactive" => Some(
                "**ComPortIsActive**\n\nReturns a Boolean indicating whether activity is currently detected on a communications port.",
            ),
            "dhcprenew" => Some(
                "**DHCPRenew**\n\nRestarts DHCP on the Ethernet interface to request a new IP address lease.",
            ),
            "encryption" => Some(
                "**Encryption**\n\nPerforms AES-128 encryption or decryption on the contents of a variable.",
            ),
            "httpout" => Some(
                "**HTTPOut**\n\nEmits a line of HTML for a datalogger-generated web page; used inside WebPageBegin/WebPageEnd.",
            ),
            "ipinfo" => Some(
                "**IPInfo**\n\nRetrieves the IP address of a datalogger interface, or the remote IP of a socket handle.",
            ),
            "ipnetpower" => {
                Some("**IPNetPower**\n\nPowers on or off a specific IP-capable network interface.")
            }
            "iproute" => Some(
                "**IPRoute**\n\nDirects outgoing IP traffic through a specified network interface when multiple interfaces are active.",
            ),
            "iptrace" => Some(
                "**IPTrace**\n\nWrites IP debug/troubleshooting messages to a string variable; can be used as a DataTable output trigger.",
            ),
            "monitorcomms" => Some(
                "**MonitorComms**\n\nCaptures communication traffic from a specified port into a string variable, for debugging.",
            ),
            "pingip" => Some(
                "**PingIP**\n\nPings an IP address and returns the response time in milliseconds.",
            ),
            "portbridge" => Some(
                "**PortBridge**\n\nEstablishes a bidirectional data channel between two communications ports.",
            ),
            "portget" => Some(
                "**PortGet**\n\nReads the status of a control or universal port into a destination variable.",
            ),
            "portpairconfig" => Some(
                "**PortPairConfig**\n\nConfigures the voltage level and pull-resistor mode for a terminal pair.",
            ),
            "portsconfig" => Some(
                "**PortsConfig**\n\nConfigures digital ports as input or output using a bitmask.",
            ),
            "snmpvariable" => Some(
                "**SNMPVariable**\n\nDefines a custom MIB entry exposing a datalogger variable through SNMP.",
            ),
            "tcpactiveconnections" => Some(
                "**TCPActiveConnections**\n\nMonitors active TCP connections and polling activity on a listening port.",
            ),
            "udpdatagram" => Some("**UDPDataGram**\n\nSends and receives UDP packets."),
            "webpagebegin" => Some(
                "**WebPageBegin**\n\nDeclares a datalogger-served HTML page, closed by WebPageEnd.",
            ),
            "xmlparse" => Some("**XMLParse**\n\nParses an XML file or string on the datalogger."),
            "emailrecv" => Some(
                "**EMailRecv**\n\nPolls a POP3 mail server for a message matching the given criteria and stores its body in a string variable.",
            ),
            "modbusserver" => Some(
                "**ModbusServer**\n\nConfigures the datalogger as a Modbus server, exposing variables/coils to a Modbus client (formerly ModbusSlave).",
            ),
            "modemcallback" => Some(
                "**ModemCallBack**\n\nDials out via a phone modem so the datalogger initiates a callback connection to a computer.",
            ),
            "networktimeprotocol" => Some(
                "**NetworkTimeProtocol**\n\nSynchronizes the datalogger clock with an NTP server, returning the pre-adjustment clock error in milliseconds.",
            ),
            "serialinblock" => Some(
                "**SerialInBlock**\n\nReads a block of raw serial bytes into Dest without waiting for a delimiter, returning the byte count received.",
            ),
            _ => None,
        }
    }

    /// Returns the description for a built-in `Scan`/`SubScan` function
    /// name, or `None` if `name` isn't one of them.
    fn get_scan_function_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "scan" => Some("**Scan**\n\nInitiates a measurement scan at specified intervals."),
            "subscan" => Some(
                "**SubScan**\n\nBegins a nested sub-scan for faster measurement or multiplexer control.",
            ),
            "triggersequence" => Some(
                "**TriggerSequence**\n\nTriggers execution of a SlowSequence at its WaitTriggerSequence point, after an optional delay.",
            ),
            "waitdigtrig" => Some(
                "**WaitDigTrig**\n\nTriggers a measurement scan using an external digital signal instead of the datalogger's internal clock.",
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
            "battery" => {
                Some("**Battery**\n\nMeasures the voltage of the battery powering the datalogger.")
            }
            "paneltemp" => Some(
                "**PanelTemp**\n\nMeasures the temperature of the datalogger wiring panel in degrees Celsius.",
            ),
            "voltse" => Some("**VoltSe**\n\nMeasures single-ended voltage."),
            "voltdiff" => Some("**VoltDiff**\n\nMeasures differential voltage."),
            "brhalf" => Some(
                "**BrHalf**\n\nApplies an excitation voltage to a half bridge and measures the single-ended voltage output.",
            ),
            "brfull" => Some(
                "**BrFull**\n\nApplies an excitation voltage to a full bridge and measures the differential voltage output.",
            ),
            "therm107" => Some("**Therm107**\n\nMeasures temperature using a 107 thermistor."),
            "therm108" => Some("**Therm108**\n\nMeasures temperature using a 108 thermistor."),
            "therm109" => Some("**Therm109**\n\nMeasures temperature using a 109 thermistor."),
            "thermistor" => Some(
                "**Thermistor**\n\nPerforms a bridge measurement of a thermistor, returning resistance in Ohms, or temperature in Celsius if Steinhart-Hart coefficients are given.",
            ),
            "tcpsyc" => Some(
                "**TCPsyc**\n\nMeasures one or more Peltier-style thermocouple psychrometers, directly or via an AM16/32B multiplexer.",
            ),
            "periodavg" => Some(
                "**PeriodAvg**\n\nMeasures the period or frequency of a signal on a single-ended channel.",
            ),
            "pulsecount" => Some("**PulseCount**\n\nMeasures pulse count from a sensor."),
            "timerinput" => Some(
                "**TimerInput**\n\nMeasures time intervals between edges, or frequency, on digital ports; can span across scan intervals.",
            ),
            "portset" => Some("**PortSet**\n\nSets a control port to a high or low state."),
            "pulseport" => Some(
                "**PulsePort**\n\nToggles a port, delays, toggles it back, and delays again to generate a clocking pulse.",
            ),
            "readio" => Some(
                "**ReadIO**\n\nReads the status of one or more digital ports or terminals, storing the result in Dest.",
            ),
            "writeio" => Some(
                "**WriteIO**\n\nSets the status of one or more digital control ports or universal terminals from Source.",
            ),
            "excitev" => Some(
                "**ExciteV**\n\nSets an excitation channel output to a specified voltage for a specified duration.",
            ),
            "excitei" => {
                Some("**ExciteI**\n\nApplies a current excitation to an excitation channel.")
            }
            "swvx" => Some(
                "**SWVX**\n\nSets a switched, regulated VX excitation channel high or low to power external peripherals or toggle control lines.",
            ),
            "brhalf3w" => Some(
                "**BrHalf3W**\n\nApplies an excitation voltage and measures a 3-wire half bridge to calculate the resistance ratio.",
            ),
            "brhalf4w" => Some(
                "**BrHalf4W**\n\nApplies an excitation voltage and makes two differential voltage measurements to measure a 4-wire half bridge.",
            ),
            "brfull6w" => Some(
                "**BrFull6W**\n\nApplies an excitation voltage and makes two differential voltage measurements to measure a 6-wire full bridge.",
            ),
            "tcse" => Some(
                "**TCSE**\n\nMeasures a thermocouple on a single-ended channel and converts the reading to degrees Celsius.",
            ),
            "tcdiff" => Some(
                "**TCDiff**\n\nMeasures a thermocouple on a differential channel and converts the result to degrees Celsius.",
            ),
            "resistance" => Some(
                "**Resistance**\n\nMeasures the resistance of a basic or full-bridge circuit using current excitation.",
            ),
            "resistance3w" => Some(
                "**Resistance3W**\n\nPerforms a 3-wire resistance measurement, using differential and reverse excitation to cancel voltage offset errors.",
            ),
            "watchdogtimer" => Some(
                "**WatchdogTimer**\n\nEnables a user-programmed watchdog timer that guards the program against lockup.",
            ),
            "pwm" => Some(
                "**PWM**\n\nGenerates a pulse-width-modulated signal on a digital port at a specified duty cycle.",
            ),
            "dewpoint" => Some(
                "**DewPoint**\n\nCalculates dew point temperature from air temperature and relative humidity.",
            ),
            "csat3" => Some(
                "**CSAT3**\n\nControls and retrieves wind and sonic temperature data from a CSAT3 3D sonic anemometer via SDM.",
            ),
            "csat3b" => Some(
                "**CSAT3B**\n\nControls and retrieves wind and sonic temperature data from a CSAT3B 3D sonic anemometer via SDM or CPI.",
            ),
            "csat3bmonitor" => Some(
                "**CSAT3BMonitor**\n\nRetrieves enclosure temperature, relative humidity, and inclination diagnostics from a CSAT3B.",
            ),
            "ec100" => Some(
                "**EC100**\n\nRetrieves measurement data from an EC100-based gas analyzer (EC150, EC155, IRGASON) via SDM.",
            ),
            "ec100configure" => Some(
                "**EC100Configure**\n\nReads or writes configuration settings on an EC100-based gas analyzer via SDM.",
            ),
            "li7200" => Some(
                "**LI7200**\n\nMeasures CO2 and H2O concentration from an LI-7200 closed-path gas analyzer via SDM.",
            ),
            "li7700" => Some(
                "**LI7700**\n\nMeasures methane concentration from an LI-7700 open-path gas analyzer via SDM.",
            ),
            "cdm_acpower" => Some(
                "**CDM_ACPower**\n\nMeasures real AC power and power-quality parameters via a CDM module in single-phase, split-phase, or three-phase configurations.",
            ),
            "cdm_battery" => Some(
                "**CDM_Battery**\n\nReads and returns a CDM module's own power-supply voltage.",
            ),
            "cdm_brfull" => {
                Some("**CDM_BrFull**\n\nMakes a 4-wire full-bridge measurement via a CDM module.")
            }
            "cdm_brfull6w" => {
                Some("**CDM_BrFull6W**\n\nMakes a 6-wire full-bridge measurement via a CDM module.")
            }
            "cdm_brhalf" => Some(
                "**CDM_BrHalf**\n\nMakes a single-ended half-bridge measurement via a CDM module.",
            ),
            "cdm_brhalf3w" => {
                Some("**CDM_BrHalf3W**\n\nMakes a 3-wire half-bridge measurement via a CDM module.")
            }
            "cdm_brhalf4w" => Some(
                "**CDM_BrHalf4W**\n\nMakes a 4-wire half-bridge measurement via a CDM module, commonly used with PRTCalc for RTDs.",
            ),
            "cdm_currentdiff" => Some(
                "**CDM_CurrentDiff**\n\nMakes a differential current-loop measurement via a CDM module.",
            ),
            "cdm_delay" => Some(
                "**CDM_Delay**\n\nDelays a CDM module's measurement or processing task sequence for a specified time.",
            ),
            "cdm_excitei" => Some(
                "**CDM_ExciteI**\n\nApplies a current excitation to an excitation channel on a CDM module.",
            ),
            "cdm_excitev" => Some(
                "**CDM_ExciteV**\n\nApplies a voltage excitation to an excitation channel on a CDM module.",
            ),
            "cdm_muxselect" => Some(
                "**CDM_MuxSelect**\n\nWakes and clocks an AM16/32A or AM16/32B multiplexer to a starting channel via a CDM module.",
            ),
            "cdm_paneltemp" => Some(
                "**CDM_PanelTemp**\n\nReads a CDM wiring-panel thermistor, for use as a thermocouple reference temperature.",
            ),
            "cdm_periodavg" => Some(
                "**CDM_PeriodAvg**\n\nMeasures the period or frequency of a signal on a CDM single-ended channel.",
            ),
            "cdm_pulseport" => Some(
                "**CDM_PulsePort**\n\nToggles a CDM switched-5V digital port, delays, and toggles it again to generate a clock signal.",
            ),
            "cdm_resistance" => Some(
                "**CDM_Resistance**\n\nMeasures resistance via current excitation on a CDM module.",
            ),
            "cdm_resistance3w" => Some(
                "**CDM_Resistance3W**\n\nMeasures resistance via current excitation using a 3-wire connection on a CDM module.",
            ),
            "cdm_sw12" => Some(
                "**CDM_SW12**\n\nSets a CDM switched-12V output port high or low to power external peripherals.",
            ),
            "cdm_sw5" => Some(
                "**CDM_SW5**\n\nSets a CDM switched-5V output port high or low to power external peripherals.",
            ),
            "cdm_swpower" => Some(
                "**CDM_SWPower**\n\nSets the ganged switched-12V and switched-5V power output on a VOLT408 isolation module.",
            ),
            "cdm_tccomp" => Some(
                "**CDM_TCComp**\n\nMakes a differential thermocouple measurement with automatic cold-junction compensation via a CDM module.",
            ),
            "cdm_tcdiff" => Some(
                "**CDM_TCDiff**\n\nMakes a differential thermocouple measurement via a CDM module.",
            ),
            "cdm_tcse" => Some(
                "**CDM_TCSE**\n\nMakes a single-ended thermocouple measurement via a CDM module.",
            ),
            "cdm_therm107" => {
                Some("**CDM_Therm107**\n\nMeasures a 107 thermistor probe via a CDM module.")
            }
            "cdm_therm108" => {
                Some("**CDM_Therm108**\n\nMeasures a 108 thermistor probe via a CDM module.")
            }
            "cdm_therm109" => {
                Some("**CDM_Therm109**\n\nMeasures a 109 thermistor probe via a CDM module.")
            }
            "cdm_voltdiff" => Some(
                "**CDM_VoltDiff**\n\nMakes a differential voltage measurement via a CDM module.",
            ),
            "cdm_voltse" => {
                Some("**CDM_VoltSE**\n\nMakes a single-ended voltage measurement via a CDM module.")
            }
            "cdm_vw300config" => Some(
                "**CDM_VW300Config**\n\nSends configuration settings to a CDM-VW300 vibrating-wire spectrum analyzer; must precede BeginProg.",
            ),
            "cdm_vw300dynamic" => Some(
                "**CDM_VW300Dynamic**\n\nCaptures the dynamic resonant frequency output of a CDM-VW300 vibrating-wire spectrum analyzer.",
            ),
            "cdm_vw300rainflow" => Some(
                "**CDM_VW300RainFlow**\n\nCaptures rainflow-histogram data from a CDM-VW300 vibrating-wire spectrum analyzer.",
            ),
            "cdm_vw300static" => Some(
                "**CDM_VW300Static**\n\nCaptures the static resonant frequency, thermistor temperature, and frequency standard deviation from a CDM-VW300 vibrating-wire spectrum analyzer.",
            ),
            "calibrate" => Some(
                "**Calibrate**\n\nForces calibration of all analog channels under program control to compensate for temperature-related measurement errors.",
            ),
            "fieldcal" => Some(
                "**FieldCal**\n\nSets up the datalogger to perform calibration of one or more variables in an array.",
            ),
            "fieldcalstrain" => Some(
                "**FieldCalStrain**\n\nSets up the datalogger to perform a zero or shunt calibration for a strain measurement.",
            ),
            "loadfieldcal" => Some(
                "**LoadFieldCal**\n\nLoads values from the FieldCal file into datalogger variables, returning True if successful.",
            ),
            "samplefieldcal" => Some(
                "**SampleFieldCal**\n\nStores the values in the FieldCal file to a data table; used inside a DataTable/EndTable declaration.",
            ),
            "calfile" => Some(
                "**CalFile**\n\nWrites an array to a calibration file, or reads a calibration file back into an array if its signature matches.",
            ),
            "newfieldcal" => Some(
                "**NewFieldCal**\n\nA Boolean DataTable trigger that is true for one scan cycle after a new field calibration has been performed.",
            ),
            "acpower" => Some(
                "**ACPower**\n\nMeasures real AC power and power-quality parameters for single-phase, split-phase, or three-phase Y systems.",
            ),
            "am25t" => {
                Some("**AM25T**\n\nControls and measures the AM25T thermocouple multiplexer.")
            }
            "avw200" => {
                Some("**AVW200**\n\nReads vibrating-wire sensors via an AVW200 spectrum analyzer.")
            }
            "vibratingwire" => Some(
                "**VibratingWire**\n\nMeasures one or more vibrating-wire sensors by sweeping an excitation frequency and detecting the sensor's resonant frequency.",
            ),
            "cs616" => {
                Some("**CS616**\n\nEnables and measures a CS616/CS625 water content reflectometer.")
            }
            "cs7500" => Some("**CS7500**\n\nCommunicates with a LI-7500(A) gas analyzer via SDM."),
            "currentse" => Some(
                "**CurrentSE**\n\nMeasures single-ended current via the datalogger's internal shunt resistor.",
            ),
            "hydraprobe" => Some(
                "**HydraProbe**\n\nConverts raw voltages from a Stevens Hydra Probe sensor into soil measurements.",
            ),
            "tdr100" => {
                Some("**TDR100**\n\nMeasures time-domain-reflectometry probes via a TDR100 device.")
            }
            "tdr200" => {
                Some("**TDR200**\n\nMeasures time-domain-reflectometry probes via a TDR200 device.")
            }
            "tga" => {
                Some("**TGA**\n\nMeasures a TGA100A/TGA200/TGA200A trace gas analyzer via SDM.")
            }
            "quadrature" => Some(
                "**Quadrature**\n\nMeasures a shaft quadrature encoder to determine displacement and rotational direction.",
            ),
            "sw12" => Some(
                "**SW12**\n\nEnables or disables a switched-12V output channel to power external peripherals.",
            ),
            "etsz" => {
                Some("**ETsz**\n\nCalculates the ASCE standardized reference evapotranspiration.")
            }
            "solarposition" => Some(
                "**SolarPosition**\n\nCalculates solar azimuth, elevation, hour angle, declination, and air mass.",
            ),
            "wetdrybulb" => Some(
                "**WetDryBulb**\n\nComputes vapor pressure from wet-bulb and dry-bulb temperatures and barometric pressure.",
            ),
            "muxselect" => Some(
                "**MuxSelect**\n\nSelects a channel on an AM16/32A or AM16/32B multiplexer and readies it for measurement.",
            ),
            "pulsecountreset" => Some(
                "**PulseCountReset**\n\nResets the pulse counter and running-average values associated with pulse count measurements.",
            ),
            "prt" => Some(
                "**PRT**\n\nConverts RTD resistance measurements to temperature using the DIN 43760 standard.",
            ),
            "prtcalc" => Some(
                "**PRTCalc**\n\nConverts RTD resistance measurements to temperature using the Callendar-Van Dusen equation.",
            ),
            "moveprecise" => Some(
                "**MovePrecise**\n\nTransfers a value into a variable as a high-precision (56-bit mantissa) number.",
            ),
            "addprecise" => Some(
                "**AddPrecise**\n\nPerforms high-precision addition, reducing floating-point rounding error in running totals.",
            ),
            "pwr" => {
                Some("**PWR**\n\nRaises X to the power of Y and returns a floating-point result.")
            }
            "ctype" => Some(
                "**CType**\n\nConverts an expression to a specified data type (Float, IEEE4, Long, String, or Double).",
            ),
            "avgrun" => Some(
                "**AvgRun**\n\nComputes a running average over the last Number values of a measurement.",
            ),
            "maxrun" => Some(
                "**MaxRun**\n\nComputes a running maximum over the last Number values of a measurement.",
            ),
            "minrun" => Some(
                "**MinRun**\n\nComputes a running minimum over the last Number values of a measurement.",
            ),
            "totalrun" => Some(
                "**TotalRun**\n\nComputes a running total over the last Number values of a measurement.",
            ),
            "avgspa" => Some(
                "**AvgSpa**\n\nComputes the spatial average of a measurement across array elements.",
            ),
            "covspa" => Some(
                "**CovSpa**\n\nComputes spatial covariance between a reference data set and one or more comparison data sets.",
            ),
            "maxspa" => {
                Some("**MaxSpa**\n\nLocates the maximum value and its position within an array.")
            }
            "rmsspa" => Some(
                "**RMSSpa**\n\nComputes the spatial root-mean-square value across array elements.",
            ),
            "stddevspa" => Some(
                "**StdDevSpa**\n\nComputes the spatial standard deviation across array elements.",
            ),
            "fftspa" => Some(
                "**FFTSpa**\n\nPerforms a Fast Fourier Transform on time-series data, for use mid-program rather than as a table entry.",
            ),
            "serialinchk" => Some(
                "**SerialInChk**\n\nReturns the number of characters currently available in the serial input buffer.",
            ),
            "setsecurity" => Some(
                "**SetSecurity**\n\nEstablishes up to three hierarchical security levels restricting access to datalogger functions.",
            ),
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

        mod measurement_functions {
            use super::*;

            #[test]
            fn all_measurement_functions_have_hover_info() {
                for name in [
                    "Battery",
                    "PanelTemp",
                    "VoltSe",
                    "VoltDiff",
                    "BrHalf",
                    "BrFull",
                    "Therm107",
                    "Therm108",
                    "Therm109",
                    "TCDiff",
                    "Resistance",
                    "PeriodAvg",
                    "PulseCount",
                    "PortSet",
                    "PulsePort",
                    "ExciteV",
                    "BrHalf3W",
                    "BrHalf4W",
                    "BrFull6W",
                    "TCSE",
                    "WatchdogTimer",
                    "PWM",
                    "DewPoint",
                    "CSAT3",
                    "CSAT3B",
                    "CSAT3BMonitor",
                    "EC100",
                    "EC100Configure",
                    "LI7200",
                    "LI7700",
                    "CDM_ACPower",
                    "CDM_Battery",
                    "CDM_BrFull",
                    "CDM_BrFull6W",
                    "CDM_BrHalf",
                    "CDM_BrHalf3W",
                    "CDM_BrHalf4W",
                    "CDM_CurrentDiff",
                    "CDM_Delay",
                    "CDM_ExciteI",
                    "CDM_ExciteV",
                    "CDM_MuxSelect",
                    "CDM_PanelTemp",
                    "CDM_PeriodAvg",
                    "CDM_PulsePort",
                    "CDM_Resistance",
                    "CDM_Resistance3W",
                    "CDM_SW12",
                    "CDM_SW5",
                    "CDM_SWPower",
                    "CDM_TCComp",
                    "CDM_TCDiff",
                    "CDM_TCSE",
                    "CDM_Therm107",
                    "CDM_Therm108",
                    "CDM_Therm109",
                    "CDM_VoltDiff",
                    "CDM_VoltSE",
                    "CDM_VW300Config",
                    "CDM_VW300Dynamic",
                    "CDM_VW300RainFlow",
                    "CDM_VW300Static",
                    "Calibrate",
                    "FieldCal",
                    "FieldCalStrain",
                    "LoadFieldCal",
                    "SampleFieldCal",
                    "ACPower",
                    "AM25T",
                    "AVW200",
                    "CS616",
                    "CS7500",
                    "CurrentSE",
                    "HydraProbe",
                    "TDR100",
                    "TDR200",
                    "TGA",
                    "Quadrature",
                    "SW12",
                    "ETsz",
                    "SolarPosition",
                    "WetDryBulb",
                    "MuxSelect",
                    "PulseCountReset",
                    "PRT",
                    "PRTCalc",
                ] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        mod communication_functions {
            use super::*;

            #[test]
            fn all_communication_functions_have_hover_info() {
                for name in [
                    "SerialOpen",
                    "SerialClose",
                    "SerialIn",
                    "SerialOut",
                    "SerialInRecord",
                    "SerialInChk",
                    "SerialOutBlock",
                    "SerialFlush",
                    "ModbusMaster",
                    "SDI12Recorder",
                    "TCPOpen",
                    "TCPClose",
                    "UDPOpen",
                    "UDPSocketOpen",
                    "UDPSocketSend",
                    "UDPSocketRecv",
                    "UDPSocketClose",
                    "EmailRelay",
                    "EmailSend",
                    "DialModem",
                    "DialSequence",
                    "EndDialSequence",
                    "ModemHangup",
                    "SMSRecv",
                    "SMSSend",
                    "PPPOpen",
                    "PPPClose",
                    "FTPClient",
                    "HTTPGet",
                    "HTTPPost",
                    "HTTPPut",
                    "GPS",
                    "EthernetPower",
                    "I2COpen",
                    "I2CRead",
                    "I2CWrite",
                    "SPIOpen",
                    "SPIRead",
                    "SPIWrite",
                    "AcceptDataRecords",
                    "Broadcast",
                    "ClockReport",
                    "DataGram",
                    "EncryptExempt",
                    "GetDataRecord",
                    "GetFile",
                    "GetVariables",
                    "PakBusClock",
                    "Route",
                    "Routes",
                    "SendData",
                    "SendFile",
                    "SendGetVariables",
                    "SendTableDef",
                    "SendVariables",
                    "StaticRoute",
                    "TimeUntilTransmit",
                    "DNP",
                    "DNPUpdate",
                    "DNPVariable",
                    "ArgosData",
                    "ArgosDataRepeat",
                    "ArgosError",
                    "ArgosSetup",
                    "ArgosTransmit",
                    "GOESData",
                    "GOESField",
                    "GOESGPS",
                    "GOESSetup",
                    "GOESStatus",
                    "GOESTable",
                    "SDMAO4",
                    "SDMAO4A",
                    "SDMBeginPort",
                    "SDMCAN",
                    "SDMCD16AC",
                    "SDMCD16Mask",
                    "SDMCVO4",
                    "SDMGeneric",
                    "SDMINT8",
                    "SDMIO16",
                    "SDMSIO4",
                    "SDMSpeed",
                    "SDMSW8A",
                    "SDMTrigger",
                    "SDMX50",
                    "CPIAddModule",
                    "CPIFileSend",
                    "CPISpeed",
                    "MQTTPublishTable",
                    "MQTTPublishConstTable",
                    "CheckPort",
                    "ComPortIsActive",
                    "DHCPRenew",
                    "Encryption",
                    "HTTPOut",
                    "IPInfo",
                    "IPNetPower",
                    "IPRoute",
                    "MonitorComms",
                    "PingIP",
                    "PortBridge",
                    "PortGet",
                    "PortPairConfig",
                    "PortsConfig",
                    "SNMPVariable",
                    "TCPActiveConnections",
                    "UDPDataGram",
                    "WebPageBegin",
                    "XMLParse",
                ] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        mod data_functions {
            use super::*;

            #[test]
            fn all_data_functions_have_hover_info() {
                for name in [
                    "Sample",
                    "Average",
                    "StdDev",
                    "Minimum",
                    "Maximum",
                    "Totalize",
                    "WindVector",
                    "Histogram",
                    "Median",
                    "Moment",
                    "SampleMaxMin",
                    "PeakValley",
                    "FFT",
                    "Covariance",
                    "LevelCrossing",
                    "WorstCase",
                    "FieldNames",
                    "CardOut",
                    "NewFile",
                    "FileManage",
                    "FileOpen",
                    "FileClose",
                    "FileRead",
                    "FileWrite",
                    "FileCopy",
                    "FileRename",
                    "FileSize",
                    "FileTime",
                    "FileList",
                    "DataInterval",
                    "CardFlush",
                    "DataEvent",
                    "Data",
                    "DataLong",
                    "DataTime",
                    "ResetTable",
                    "TableFile",
                    "FileMark",
                    "FileReadLine",
                    "Erase",
                ] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        mod string_functions {
            use super::*;

            #[test]
            fn all_string_functions_have_hover_info() {
                for name in [
                    "SplitStr",
                    "FormatFloat",
                    "FormatLong",
                    "Mid",
                    "Left",
                    "Right",
                    "Len",
                    "InStr",
                    "LowerCase",
                    "UpperCase",
                    "Trim",
                    "RTrim",
                    "LTrim",
                    "Replace",
                    "Chr",
                    "ASCII",
                    "StrComp",
                    "CheckSum",
                    "HexToDec",
                    "Hex",
                    "Sprintf",
                ] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        mod math_functions {
            use super::*;

            #[test]
            fn all_math_functions_have_hover_info() {
                for name in [
                    "Abs",
                    "Sgn",
                    "Sqr",
                    "Exp",
                    "Ln",
                    "Log",
                    "Log10",
                    "Sin",
                    "Cos",
                    "Tan",
                    "Sinh",
                    "Cosh",
                    "Tanh",
                    "Asin",
                    "Acos",
                    "Atn",
                    "Atn2",
                    "Int",
                    "Fix",
                    "Frac",
                    "Round",
                    "Rnd",
                    "Randomize",
                    "Ceiling",
                    "Floor",
                    "MovePrecise",
                    "PWR",
                    "CType",
                    "AvgRun",
                    "MaxRun",
                    "MinRun",
                    "TotalRun",
                    "AvgSpa",
                    "CovSpa",
                    "MaxSpa",
                    "RMSSpa",
                    "StdDevSpa",
                    "FFTSpa",
                ] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        mod time_functions {
            use super::*;

            #[test]
            fn all_time_functions_have_hover_info() {
                for name in [
                    "RealTime",
                    "TimeIntoInterval",
                    "IfTime",
                    "Timer",
                    "Delay",
                    "SetStatus",
                    "SetSetting",
                    "MoveBytes",
                    "ArrayLength",
                    "NaN",
                    "SecsSince1990",
                    "TimeIsBetween",
                    "SetSecurity",
                    "DaylightSaving",
                    "DaylightSavingUS",
                    "InstructionTimes",
                    "LineNum",
                    "Signature",
                ] {
                    let description = HoverProvider::get_builtin_function_description(name);
                    assert!(
                        description.is_some_and(|d| d.contains(&format!("**{}**", name))),
                        "Expected hover info for builtin function: {}",
                        name
                    );
                }
            }
        }

        mod logical_and_menu_functions {
            use super::*;

            #[test]
            fn all_logical_and_menu_functions_have_hover_info() {
                for name in [
                    "IIf",
                    "DisplayMenu",
                    "SubMenu",
                    "MenuItem",
                    "MenuPick",
                    "MenuRecompile",
                    "DisplayValue",
                    "DisplayLine",
                ] {
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
        fn returns_hover_for_loggertype_in_preprocessor_condition() {
            // "#If LoggerType = CR1000X": character 4 is the start of "LoggerType"
            let tokens = tokenize("#If LoggerType = CR1000X");
            let position = Position {
                line: 0,
                character: 4,
            };

            let hover = HoverProvider::get_hover_at_position(&tokens, position);

            assert!(hover.is_some());
            let hover = hover.expect("hover should be Some");
            match hover.contents {
                HoverContents::Markup(markup) => {
                    assert!(markup.value.contains("**LoggerType**"));
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
