//! Signature help provider for CRBasic
//!
//! This module provides function signature information including parameter
//! names, types, and documentation for built-in and user-defined functions.

use tower_lsp_server::ls_types::{
    Documentation, MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureInformation,
};

/// Provides signature help for CRBasic functions
pub struct SignatureProvider;

/// Represents a function signature with parameters
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Function description
    pub documentation: String,
    /// Parameter information
    pub parameters: Vec<ParameterInfo>,
}

/// Represents a parameter in a function signature
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub documentation: String,
}

impl SignatureProvider {
    /// Returns signature help for a function
    ///
    /// # Arguments
    /// * `function_name` - The name of the function (case-insensitive)
    ///
    /// # Returns
    /// * `Some(SignatureHelp)` if the function is recognized
    /// * `None` if the function is not recognized
    pub fn get_signature_help(function_name: &str, active_parameter: u32) -> Option<SignatureHelp> {
        let signature = Self::get_function_signature(function_name)?;

        Some(SignatureHelp {
            signatures: vec![Self::to_signature_information(&signature)],
            active_signature: Some(0),
            active_parameter: Some(active_parameter),
        })
    }

    /// Returns signature information for a function by name
    pub fn get_function_signature(function_name: &str) -> Option<FunctionSignature> {
        match function_name.to_lowercase().as_str() {
            "scan" => Some(FunctionSignature {
                name: "Scan".to_string(),
                documentation: "Initiates a measurement scan at specified intervals.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The time interval between scans.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units: uSec, mSec, Sec, Min, Hr.".to_string(),
                    },
                    ParameterInfo {
                        name: "BufferOption".to_string(),
                        documentation: "Buffer size option (0-3).".to_string(),
                    },
                    ParameterInfo {
                        name: "Count".to_string(),
                        documentation: "Number of scans (0 = continuous).".to_string(),
                    },
                ],
            }),

            "subscan" => Some(FunctionSignature {
                name: "SubScan".to_string(),
                documentation:
                    "Begins a nested sub-scan for faster measurement or multiplexer control."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "SubInterval".to_string(),
                        documentation: "The time interval between sub-scans.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units: uSec, mSec, Sec, Min, Hr.".to_string(),
                    },
                    ParameterInfo {
                        name: "Count".to_string(),
                        documentation: "Number of sub-scans (0 = continuous).".to_string(),
                    },
                ],
            }),

            "iif" => Some(FunctionSignature {
                name: "IIf".to_string(),
                documentation:
                    "Evaluates a Boolean expression and returns TrueValue if true, otherwise FalseValue."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Expression".to_string(),
                        documentation: "The Boolean expression to evaluate.".to_string(),
                    },
                    ParameterInfo {
                        name: "TrueValue".to_string(),
                        documentation: "Value returned when Expression is true.".to_string(),
                    },
                    ParameterInfo {
                        name: "FalseValue".to_string(),
                        documentation: "Value returned when Expression is false.".to_string(),
                    },
                ],
            }),

            "sample" => Some(FunctionSignature {
                name: "Sample".to_string(),
                documentation: "Samples and stores a value in the data table.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of values to sample.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable or array to sample from.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type (IEEE4, FP2, etc.).".to_string(),
                    },
                ],
            }),

            "average" => Some(FunctionSignature {
                name: "Average".to_string(),
                documentation: "Calculates and stores the average of values.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of values to average.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable or array to average.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type (IEEE4, FP2, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Variable to disable output (0 = enabled).".to_string(),
                    },
                ],
            }),

            "minimum" => Some(FunctionSignature {
                name: "Minimum".to_string(),
                documentation: "Stores the minimum value over the output interval.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of values to check.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable or array to check.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type.".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Variable to disable output.".to_string(),
                    },
                    ParameterInfo {
                        name: "Time".to_string(),
                        documentation: "Store time of minimum (True/False).".to_string(),
                    },
                ],
            }),

            "maximum" => Some(FunctionSignature {
                name: "Maximum".to_string(),
                documentation: "Stores the maximum value over the output interval.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of values to check.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable or array to check.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type.".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Variable to disable output.".to_string(),
                    },
                    ParameterInfo {
                        name: "Time".to_string(),
                        documentation: "Store time of maximum (True/False).".to_string(),
                    },
                ],
            }),

            "stddev" => Some(FunctionSignature {
                name: "StdDev".to_string(),
                documentation:
                    "Calculates and stores the standard deviation of Source values over the output interval."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of Source variables to calculate a standard deviation for (Source must be an array if greater than 1).".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable or expression whose standard deviation is calculated.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type (IEEE4, FP2, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Variable to disable output (0 = enabled); stores NAN if disabled for the whole interval.".to_string(),
                    },
                ],
            }),

            "totalize" => Some(FunctionSignature {
                name: "Totalize".to_string(),
                documentation:
                    "Calculates and stores the sum of Source values over the output interval."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of Source variables to total (Source must be an array if greater than 1).".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable or expression whose values are summed.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type (IEEE4, FP2, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Variable to disable output (0 = enabled); stores 0, not NAN, if disabled the whole interval.".to_string(),
                    },
                ],
            }),

            "histogram" => Some(FunctionSignature {
                name: "Histogram".to_string(),
                documentation:
                    "Stores a frequency distribution of BinSelect values across a set of bins between LoLim and UpLim."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BinSelect".to_string(),
                        documentation: "Variable whose value determines which bin is incremented.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output data type for bin totals (IEEE4, FP2, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Variable to disable output (0 = enabled); ±12345 resets the histogram.".to_string(),
                    },
                    ParameterInfo {
                        name: "Bins".to_string(),
                        documentation: "Number of bins spanning the range from LoLim to UpLim.".to_string(),
                    },
                    ParameterInfo {
                        name: "Form".to_string(),
                        documentation: "Three-digit code (ABC) controlling reset, output form, and bin-limit inclusion.".to_string(),
                    },
                    ParameterInfo {
                        name: "WtVal".to_string(),
                        documentation: "Constant or variable weight added on each increment (1 = simple frequency count).".to_string(),
                    },
                    ParameterInfo {
                        name: "LoLim".to_string(),
                        documentation: "Lower limit of the histogram's measurement range.".to_string(),
                    },
                    ParameterInfo {
                        name: "UpLim".to_string(),
                        documentation: "Upper limit of the histogram's measurement range.".to_string(),
                    },
                ],
            }),

            "fieldnames" => Some(FunctionSignature {
                name: "FieldNames".to_string(),
                documentation:
                    "Overrides the default field names, and optionally adds descriptions, for the immediately preceding output-processing instruction."
                        .to_string(),
                parameters: vec![ParameterInfo {
                    name: "FieldNameDescriptionList".to_string(),
                    documentation: "Quoted, comma-separated \"Fieldname:Description\" pairs, one per output field.".to_string(),
                }],
            }),

            "cardout" => Some(FunctionSignature {
                name: "CardOut".to_string(),
                documentation: "Creates a data table that is stored on a memory card.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "StopRing".to_string(),
                        documentation: "Table fill mode: 0 = ring (overwrite oldest), 1 = fill and stop.".to_string(),
                    },
                    ParameterInfo {
                        name: "Size".to_string(),
                        documentation: "Number of records to allocate on the card (-1 = auto-allocate).".to_string(),
                    },
                ],
            }),

            "newfile" => Some(FunctionSignature {
                name: "NewFile".to_string(),
                documentation:
                    "Determines whether a monitored file has been newly written since this instruction last ran."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "NewFileVar".to_string(),
                        documentation: "Variable receiving the result; set to 0 when a new file is detected.".to_string(),
                    },
                    ParameterInfo {
                        name: "FileName".to_string(),
                        documentation: "Device:FileName of the file to monitor; wildcards ? and * are supported.".to_string(),
                    },
                    ParameterInfo {
                        name: "NewFileName".to_string(),
                        documentation: "Optional variable that receives the name of the newly detected file.".to_string(),
                    },
                ],
            }),

            "filemanage" => Some(FunctionSignature {
                name: "FileManage".to_string(),
                documentation:
                    "Performs a management operation, such as delete, hide, run, or format, on a file or device."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "DeviceFileName".to_string(),
                        documentation: "Device:FileName string identifying the file or device to manage.".to_string(),
                    },
                    ParameterInfo {
                        name: "Attribute".to_string(),
                        documentation: "Bit-field code selecting the operation (e.g., delete, hide, run, run on power-up).".to_string(),
                    },
                ],
            }),

            "fileopen" => Some(FunctionSignature {
                name: "FileOpen".to_string(),
                documentation: "Opens a file for reading or writing and returns a file handle."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FileName".to_string(),
                        documentation: "Device:FileName of the file to open.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode".to_string(),
                        documentation: "File access mode string (e.g., \"r\", \"w\", \"a\", with optional \"b\"/\"+\").".to_string(),
                    },
                    ParameterInfo {
                        name: "SeekPoint".to_string(),
                        documentation: "Byte offset at which to begin reading or writing (-1 = append at end of file).".to_string(),
                    },
                ],
            }),

            "fileclose" => Some(FunctionSignature {
                name: "FileClose".to_string(),
                documentation: "Closes a file previously opened with FileOpen.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "FileHandle".to_string(),
                    documentation: "Handle of the open file to close, as returned by FileOpen."
                        .to_string(),
                }],
            }),

            "fileread" => Some(FunctionSignature {
                name: "FileRead".to_string(),
                documentation: "Reads data from an open file into a variable or array."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FileHandle".to_string(),
                        documentation: "Handle of the open file to read from, as returned by FileOpen.".to_string(),
                    },
                    ParameterInfo {
                        name: "Destination".to_string(),
                        documentation: "String variable (or array) that receives the data read from the file.".to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "Maximum number of characters to read into Destination.".to_string(),
                    },
                ],
            }),

            "filewrite" => Some(FunctionSignature {
                name: "FileWrite".to_string(),
                documentation: "Writes data from a variable or array to an open file."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FileHandle".to_string(),
                        documentation: "Handle of the open file to write to, as returned by FileOpen.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Constant, variable, or array whose data is written to the file.".to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "Maximum number of characters from Source to write (0 = write a string up to its null terminator).".to_string(),
                    },
                ],
            }),

            "filecopy" => Some(FunctionSignature {
                name: "FileCopy".to_string(),
                documentation: "Copies a file from one drive on the datalogger to another."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FromFileName".to_string(),
                        documentation: "Device:FileName of the source file to copy.".to_string(),
                    },
                    ParameterInfo {
                        name: "ToFileName".to_string(),
                        documentation: "Device:FileName of the destination for the copy."
                            .to_string(),
                    },
                ],
            }),

            "filerename" => Some(FunctionSignature {
                name: "FileRename".to_string(),
                documentation: "Renames a file stored on the datalogger.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "OldFileName".to_string(),
                        documentation: "Device:FileName of the file to rename.".to_string(),
                    },
                    ParameterInfo {
                        name: "NewFileName".to_string(),
                        documentation: "Device:FileName specifying the new name (and optionally a different device).".to_string(),
                    },
                ],
            }),

            "filesize" => Some(FunctionSignature {
                name: "FileSize".to_string(),
                documentation: "Returns the size, in bytes, of a specified file.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "FileHandle".to_string(),
                    documentation: "Handle of the file (from FileOpen) or a Device:FileName string identifying it.".to_string(),
                }],
            }),

            "filetime" => Some(FunctionSignature {
                name: "FileTime".to_string(),
                documentation: "Returns the last-modified timestamp of a specified file."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "FileHandle".to_string(),
                    documentation: "Handle of the file (from FileOpen) or a Device:FileName string identifying it.".to_string(),
                }],
            }),

            "filelist" => Some(FunctionSignature {
                name: "FileList".to_string(),
                documentation: "Writes the list of file names on a device into a destination array.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Device".to_string(),
                        documentation: "Device to query for file names (CPU, CRD, USR, or USB).".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "String array that receives the file names found on the device.".to_string(),
                    },
                ],
            }),

            "datainterval" => Some(FunctionSignature {
                name: "DataInterval".to_string(),
                documentation:
                    "Sets the real-time-clock-based interval on which a data table's records are generated."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "TintoInt".to_string(),
                        documentation: "Offset, in Units, into the interval at which output occurs.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "Length of the output interval, in Units (0 = same as the scan interval).".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for TintoInt and Interval: mSec, Sec, Min, Hr, Day, or Mon.".to_string(),
                    },
                    ParameterInfo {
                        name: "Lapses".to_string(),
                        documentation: "Timestamp-overhead mode: positive uses an efficient header, 0 timestamps every record, negative disables lapse adjustment.".to_string(),
                    },
                ],
            }),

            "cardflush" => Some(FunctionSignature {
                name: "CardFlush".to_string(),
                documentation: "Immediately writes buffered data to an external storage device.".to_string(),
                parameters: vec![],
            }),

            "dataevent" => Some(FunctionSignature {
                name: "DataEvent".to_string(),
                documentation: "Conditionally starts and stops data storage to a table based on trigger conditions.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "RecBefore".to_string(),
                        documentation: "The number of records to store before the triggering event occurs.".to_string(),
                    },
                    ParameterInfo {
                        name: "StartTrig".to_string(),
                        documentation: "An expression that begins data storage when it becomes true or non-zero.".to_string(),
                    },
                    ParameterInfo {
                        name: "EndTrig".to_string(),
                        documentation: "An expression that stops data storage when it becomes true; 0 means storage never stops.".to_string(),
                    },
                    ParameterInfo {
                        name: "RecAfter".to_string(),
                        documentation: "The number of records to store after the event stops.".to_string(),
                    },
                ],
            }),

            "data" => Some(FunctionSignature {
                name: "Data".to_string(),
                documentation: "Defines a list of Float constants for later retrieval with Read.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Value1".to_string(),
                        documentation: "The first constant in the list.".to_string(),
                    },
                    ParameterInfo {
                        name: "Value2".to_string(),
                        documentation: "Additional comma-separated constants in the list.".to_string(),
                    },
                ],
            }),

            "datalong" => Some(FunctionSignature {
                name: "DataLong".to_string(),
                documentation: "Defines a list of Long constants for later retrieval with Read.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Value1".to_string(),
                        documentation: "The first constant in the list.".to_string(),
                    },
                    ParameterInfo {
                        name: "Value2".to_string(),
                        documentation: "Additional comma-separated constants in the list.".to_string(),
                    },
                ],
            }),

            "datatime" => Some(FunctionSignature {
                name: "DataTime".to_string(),
                documentation: "Selects whether a data table's records are timestamped at scan time or at storage time.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "DataTimeOpt".to_string(),
                    documentation: "0 timestamps at scan time (top of scan); 1 timestamps at storage time.".to_string(),
                }],
            }),

            "resettable" => Some(FunctionSignature {
                name: "ResetTable".to_string(),
                documentation: "Erases all records from a specified data table during program execution.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "TableName".to_string(),
                    documentation: "The name of the data table to erase.".to_string(),
                }],
            }),

            "tablefile" => Some(FunctionSignature {
                name: "TableFile".to_string(),
                documentation: "Writes a data table's contents to external storage media; placed inside a DataTable/EndTable declaration.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FileName".to_string(),
                        documentation: "The output device and file name (e.g. CRD:, USR:, or USB:).".to_string(),
                    },
                    ParameterInfo {
                        name: "Options".to_string(),
                        documentation: "The file format and metadata options (e.g. TOB1, TOA5, CSIXML, CSIJSON).".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxFiles".to_string(),
                        documentation: "The maximum number of files to retain; -1 is ring mode, -2 fills and stops.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "The number of records per file, or the offset time for interval-based writes.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The write trigger: 0 for record-based, non-zero for time-based.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval.".to_string(),
                    },
                    ParameterInfo {
                        name: "OutStat".to_string(),
                        documentation: "An optional variable that receives the file-write status (-1 = written, 0 = not written).".to_string(),
                    },
                    ParameterInfo {
                        name: "LastFileName".to_string(),
                        documentation: "An optional string variable that receives the most recently written file name.".to_string(),
                    },
                ],
            }),

            "filemark" => Some(FunctionSignature {
                name: "FileMark".to_string(),
                documentation: "Inserts a filemark into a data table, signaling file-splitting software to start a new file.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Tablename".to_string(),
                    documentation: "The output table that receives the filemark.".to_string(),
                }],
            }),

            "filereadline" => Some(FunctionSignature {
                name: "FileReadLine".to_string(),
                documentation: "Reads one line from an open file into a destination variable.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FileHandle".to_string(),
                        documentation: "The handle of an open file, created by FileOpen.".to_string(),
                    },
                    ParameterInfo {
                        name: "Destination".to_string(),
                        documentation: "The string variable that receives the line read from the file.".to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "The maximum number of characters to read into Destination.".to_string(),
                    },
                ],
            }),

            "erase" => Some(FunctionSignature {
                name: "Erase".to_string(),
                documentation: "Sets all bytes of a variable or array to zero.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "EraseVar".to_string(),
                    documentation: "The variable or array to clear.".to_string(),
                }],
            }),

            "pulsecount" => Some(FunctionSignature {
                name: "PulseCount".to_string(),
                documentation: "Measures pulse count from a sensor.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable for the result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "PChan".to_string(),
                        documentation: "Pulse input channel.".to_string(),
                    },
                    ParameterInfo {
                        name: "PConfig".to_string(),
                        documentation: "Pulse configuration code.".to_string(),
                    },
                    ParameterInfo {
                        name: "POption".to_string(),
                        documentation: "Pulse option (0-2).".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset for scaling.".to_string(),
                    },
                ],
            }),

            "voltse" => Some(FunctionSignature {
                name: "VoltSe".to_string(),
                documentation: "Measures single-ended voltage.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable for the result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Voltage range (mV5000, mV2500, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasOff".to_string(),
                        documentation: "Measure offset (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Settling time in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset for scaling.".to_string(),
                    },
                ],
            }),

            "voltdiff" => Some(FunctionSignature {
                name: "VoltDiff".to_string(),
                documentation: "Measures differential voltage.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable for the result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Voltage range.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "Differential channel number.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Reverse differential (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Settling time in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset for scaling.".to_string(),
                    },
                ],
            }),

            "tcdiff" => Some(FunctionSignature {
                name: "TCDiff".to_string(),
                documentation: "Measures a thermocouple on a differential channel and converts the result to degrees Celsius.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable (or array, if Reps > 1) for the result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Voltage range for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "Differential channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCType".to_string(),
                        documentation: "Thermocouple type (TypeT, TypeE, TypeK, TypeJ, TypeB, TypeR, TypeS, or TypeN).".to_string(),
                    },
                    ParameterInfo {
                        name: "TRef".to_string(),
                        documentation: "Reference temperature variable, in degrees C.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Reverse differential (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Settling time in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset for scaling.".to_string(),
                    },
                ],
            }),

            "resistance" => Some(FunctionSignature {
                name: "Resistance".to_string(),
                documentation: "Measures the resistance of a basic or full-bridge circuit using current excitation.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable (or array, if Reps > 1) for the result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Voltage range (mV5000, mV1000, or mV200).".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "Differential channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "IexChan".to_string(),
                        documentation: "Current excitation channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "Number of sensors to excite per excitation channel before advancing.".to_string(),
                    },
                    ParameterInfo {
                        name: "EXuA".to_string(),
                        documentation: "Excitation current, in microamps.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Reverse excitation (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Reverse differential (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Settling time in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasCurrent".to_string(),
                        documentation: "Optional: include the excitation current as the last value in Dest.".to_string(),
                    },
                ],
            }),

            "sdi12recorder" => Some(FunctionSignature {
                name: "SDI12Recorder".to_string(),
                documentation: "Retrieves measurement results from an SDI-12 sensor.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination array for the sensor's returned values.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDIPort".to_string(),
                        documentation: "SDI-12 port the sensor is connected to.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDIAddress".to_string(),
                        documentation: "SDI-12 address of the sensor (0-9, a-z, A-Z).".to_string(),
                    },
                    ParameterInfo {
                        name: "SDICommand".to_string(),
                        documentation: "SDI-12 command string to send, in quotes (e.g., \"M!\").".to_string(),
                    },
                    ParameterInfo {
                        name: "Multiplier".to_string(),
                        documentation: "Multiplier for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset for scaling.".to_string(),
                    },
                    ParameterInfo {
                        name: "FillNAN".to_string(),
                        documentation: "Optional: how to record NAN values from a bad sensor reading.".to_string(),
                    },
                    ParameterInfo {
                        name: "WaitonTimeout".to_string(),
                        documentation: "Optional: wait inside the instruction for a C! command to finish.".to_string(),
                    },
                ],
            }),

            "windvector" => Some(FunctionSignature {
                name: "WindVector".to_string(),
                documentation: "Processes raw wind speed/direction samples into mean wind speed, mean wind vector magnitude and direction, and standard deviation of wind direction.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of wind vector averages to calculate.".to_string(),
                    },
                    ParameterInfo {
                        name: "Speed/East".to_string(),
                        documentation: "Wind speed (polar sensor) or East component (orthogonal sensor); an array if Reps > 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Direction/North".to_string(),
                        documentation: "Wind direction (polar sensor) or North component (orthogonal sensor); an array if Reps > 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "Output storage format (IEEE4, FP2, IEEE8, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "Excludes the current measurement from output when nonzero.".to_string(),
                    },
                    ParameterInfo {
                        name: "Subinterval".to_string(),
                        documentation: "Scan interval for subinterval standard-deviation processing (0 to use every sample).".to_string(),
                    },
                    ParameterInfo {
                        name: "SensorType".to_string(),
                        documentation: "Sensor configuration: 0 for polar, 1 for orthogonal.".to_string(),
                    },
                    ParameterInfo {
                        name: "OutputOpt".to_string(),
                        documentation: "Selects which of the wind vector outputs to store.".to_string(),
                    },
                ],
            }),

            "battery" => Some(FunctionSignature {
                name: "Battery".to_string(),
                documentation: "Measures the voltage of the battery powering the datalogger."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "Dest".to_string(),
                    documentation: "Destination variable that stores the measured battery voltage, in volts.".to_string(),
                }],
            }),

            "paneltemp" => Some(FunctionSignature {
                name: "PanelTemp".to_string(),
                documentation:
                    "Measures the temperature of the datalogger wiring panel in degrees Celsius."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable to store the panel temperature reading.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter (e.g. 60 or 50 Hz for AC noise).".to_string(),
                    },
                ],
            }),

            "brhalf" => Some(FunctionSignature {
                name: "BrHalf".to_string(),
                documentation:
                    "Applies an excitation voltage to a half bridge and measures the single-ended voltage output."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the measurement result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Input voltage range for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "Excitation channel used to excite the bridge."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "Number of sensors to excite per excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "Excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Reverse excitation polarity and take a second measurement to cancel offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                ],
            }),

            "brfull" => Some(FunctionSignature {
                name: "BrFull".to_string(),
                documentation:
                    "Applies an excitation voltage to a full bridge and measures the differential voltage output."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the measurement result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Input voltage range for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "Differential channel number for the measurement."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "Excitation channel used to excite the bridge."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "Number of sensors to excite per excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "Excitation voltage, in millivolts, applied to the bridge.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Reverse excitation polarity and take a second measurement to cancel offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Reverse the differential input polarity and take a second measurement to cancel datalogger offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                ],
            }),

            "therm107" => Some(FunctionSignature {
                name: "Therm107".to_string(),
                documentation: "Measures temperature using a 107 thermistor.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number for the measurement."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Excite".to_string(),
                        documentation: "Excitation channel used to apply voltage excitation to the thermistor.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "Integ".to_string(),
                        documentation: "AC noise rejection frequency (60 or 50 Hz) or sinc filter fN1, depending on datalogger model.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "therm108" => Some(FunctionSignature {
                name: "Therm108".to_string(),
                documentation: "Measures temperature using a 108 thermistor.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number for the measurement."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Excite".to_string(),
                        documentation: "Excitation channel used to apply voltage excitation to the thermistor.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "Integ".to_string(),
                        documentation: "AC noise rejection frequency (60 or 50 Hz) or sinc filter fN1, depending on datalogger model.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "therm109" => Some(FunctionSignature {
                name: "Therm109".to_string(),
                documentation: "Measures temperature using a 109 thermistor.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number for the measurement."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Excite".to_string(),
                        documentation: "Excitation channel used to apply voltage excitation to the thermistor.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "Integ".to_string(),
                        documentation: "AC noise rejection frequency (60 or 50 Hz) or sinc filter fN1, depending on datalogger model.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "periodavg" => Some(FunctionSignature {
                name: "PeriodAvg".to_string(),
                documentation:
                    "Measures the period or frequency of a signal on a single-ended channel."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the result(s)."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Chan".to_string(),
                        documentation: "Single-ended channel number for the first measurement."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Output format: 0 = period (microseconds), 1 = frequency (Hz).".to_string(),
                    },
                    ParameterInfo {
                        name: "Cycles".to_string(),
                        documentation: "Number of signal cycles to average per scan."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Timeout".to_string(),
                        documentation: "Maximum time, in milliseconds, to wait for the specified Cycles to be measured.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                ],
            }),

            "portset" => Some(FunctionSignature {
                name: "PortSet".to_string(),
                documentation: "Sets a control port to a high or low state.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "Control port to set (e.g. C1-C8).".to_string(),
                    },
                    ParameterInfo {
                        name: "State".to_string(),
                        documentation: "Output level: 0 sets the port low, non-zero sets it high.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Execution mode: run in measurement or processing sequence, optionally attempting pipeline mode.".to_string(),
                    },
                ],
            }),

            "pulseport" => Some(FunctionSignature {
                name: "PulsePort".to_string(),
                documentation:
                    "Toggles a port, delays, toggles it back, and delays again to generate a clocking pulse."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "Port to toggle to generate the clocking pulse."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Delay".to_string(),
                        documentation: "Delay, in microseconds, after each toggle.".to_string(),
                    },
                ],
            }),

            "excitev" => Some(FunctionSignature {
                name: "ExciteV".to_string(),
                documentation:
                    "Sets an excitation channel output to a specified voltage for a specified duration."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "Excitation channel to apply the voltage to."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "Excitation voltage, in millivolts, to apply."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Delay".to_string(),
                        documentation: "Delay, in microseconds, before excitation turns off and the next instruction runs (0 = leave excitation on).".to_string(),
                    },
                ],
            }),

            "brhalf3w" => Some(FunctionSignature {
                name: "BrHalf3W".to_string(),
                documentation:
                    "Applies an excitation voltage and measures a 3-wire half bridge to calculate the resistance ratio."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the measurement result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Input voltage range for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "Excitation channel used to excite the bridge."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "Number of sensors to excite per excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "Excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Reverse excitation polarity and take a second measurement to cancel offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                ],
            }),

            "brhalf4w" => Some(FunctionSignature {
                name: "BrHalf4W".to_string(),
                documentation:
                    "Applies an excitation voltage and makes two differential voltage measurements to measure a 4-wire half bridge."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the measurement result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range1".to_string(),
                        documentation: "Input voltage range for the first differential voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range2".to_string(),
                        documentation: "Input voltage range for the second differential voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "Differential channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "Excitation channel used to excite the bridge."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "Number of sensors to excite per excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "Excitation voltage, in millivolts, applied to the bridge.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Reverse excitation polarity and take a second measurement to cancel offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Reverse the differential input polarity and take a second measurement to cancel datalogger offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                    ParameterInfo {
                        name: "ReturnV1".to_string(),
                        documentation: "If non-zero, also returns the excitation voltage measurement V1 (requires a 2-element Dest array).".to_string(),
                    },
                ],
            }),

            "brfull6w" => Some(FunctionSignature {
                name: "BrFull6W".to_string(),
                documentation:
                    "Applies an excitation voltage and makes two differential voltage measurements to measure a 6-wire full bridge."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the measurement result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range1".to_string(),
                        documentation: "Input voltage range for the first differential voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range2".to_string(),
                        documentation: "Input voltage range for the second differential voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "Differential channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "Excitation channel used to excite the bridge."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "Number of sensors to excite per excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "Excitation voltage, in millivolts, applied to the bridge.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Reverse excitation polarity and take a second measurement to cancel offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Reverse the differential input polarity and take a second measurement to cancel datalogger offsets (True/False).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                    ParameterInfo {
                        name: "ReturnV1".to_string(),
                        documentation: "If non-zero, also returns the excitation voltage measurement V1 (requires a 2-element Dest array).".to_string(),
                    },
                ],
            }),

            "tcse" => Some(FunctionSignature {
                name: "TCSE".to_string(),
                documentation:
                    "Measures a thermocouple on a single-ended channel and converts the reading to degrees Celsius."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array to store the measurement result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "Input voltage range.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "Single-ended channel number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCType".to_string(),
                        documentation: "Thermocouple type (TypeE, TypeT, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "TRef".to_string(),
                        documentation: "Reference temperature source.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasOff".to_string(),
                        documentation: "Offset handling: 0 = use background-calibration offset, 1 = measure the offset each scan.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "Delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "Lowest frequency notched out by the sinc filter."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "Multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "Offset added after scaling by Mult.".to_string(),
                    },
                ],
            }),

            "csat3" => Some(FunctionSignature {
                name: "CSAT3".to_string(),
                documentation: "Controls and retrieves wind and sonic temperature data from a CSAT3 3D sonic anemometer via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable or array (5 elements: Ux, Uy, Uz, speed of sound or temperature, and a diagnostic word) to store the CSAT3 result(s); must be an array if Reps is greater than 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of CSAT3 units to measure; their SDM addresses must be sequential.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "SDM address of the CSAT3 (0 through 14; address 15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "Command".to_string(),
                        documentation: "Selects the measurement trigger: triggers a new measurement and gets data, or retrieves data after a prior group trigger without triggering a new measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Sets the CSAT3's execution parameter, specifying the measurement frequency the CSAT3 should expect from the datalogger.".to_string(),
                    },
                ],
            }),

            "csat3b" => Some(FunctionSignature {
                name: "CSAT3B".to_string(),
                documentation: "Controls and retrieves wind and sonic temperature data from a CSAT3B 3D sonic anemometer via SDM or CPI.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Destination".to_string(),
                        documentation: "Float variable or array (at least 5 elements) to store the wind components, sonic temperature, and diagnostic word returned by the anemometer.".to_string(),
                    },
                    ParameterInfo {
                        name: "Bus".to_string(),
                        documentation: "Constant selecting the communication bus: 0 for SDM, 1 for CPI.".to_string(),
                    },
                    ParameterInfo {
                        name: "Address".to_string(),
                        documentation: "Constant identifying the CSAT3B's address on the bus (0 to 14 for SDM; 1 to 120 for CPI).".to_string(),
                    },
                    ParameterInfo {
                        name: "OperatingMode".to_string(),
                        documentation: "Constant controlling the trigger source (datalogger-triggered or self-triggered) and the bandwidth filter applied to the output.".to_string(),
                    },
                ],
            }),

            "csat3bmonitor" => Some(FunctionSignature {
                name: "CSAT3BMonitor".to_string(),
                documentation: "Retrieves enclosure temperature, relative humidity, and inclination diagnostics from a CSAT3B.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Destination".to_string(),
                        documentation: "Float variable or array to store the enclosure temperature, relative humidity, and inclination values returned by the anemometer.".to_string(),
                    },
                    ParameterInfo {
                        name: "Bus".to_string(),
                        documentation: "Constant selecting the communication bus: 0 for SDM, 1 for CPI.".to_string(),
                    },
                    ParameterInfo {
                        name: "Address".to_string(),
                        documentation: "Constant identifying the CSAT3B's address on the bus (0 to 14 for SDM; 1 to 120 for CPI).".to_string(),
                    },
                ],
            }),

            "ec100" => Some(FunctionSignature {
                name: "EC100".to_string(),
                documentation: "Retrieves measurement data from an EC100-based gas analyzer (EC150, EC155, IRGASON) via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Input variable array to store the data returned by the analyzer; its length depends on the selected EC100Cmd.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "SDM address of the analyzer (0 through 14; address 15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "EC100Cmd".to_string(),
                        documentation: "Requests which data set to retrieve from the analyzer; results are returned in Dest.".to_string(),
                    },
                ],
            }),

            "ec100configure" => Some(FunctionSignature {
                name: "EC100Configure".to_string(),
                documentation: "Reads or writes configuration settings on an EC100-based gas analyzer via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Result".to_string(),
                        documentation: "Variable receiving the success/failure code: 0 for a successful read, or for a write that matched the existing value; 1 for a write that changed the value; NAN if the setting was not acknowledged.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "SDM address of the analyzer to configure (0 through 14; address 15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "ConfigCmd".to_string(),
                        documentation: "Selects which setting to get or set.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestSource".to_string(),
                        documentation: "Variable holding the setting value being read or written; setting it to 2718 sends the save-settings command.".to_string(),
                    },
                ],
            }),

            "li7200" => Some(FunctionSignature {
                name: "LI7200".to_string(),
                documentation: "Measures CO2 and H2O concentration from an LI-7200 closed-path gas analyzer via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Input variable array to store the data returned by each LI-7200; its length depends on the number of repetitions and the selected command.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of LI-7200 units to communicate with; their SDM addresses must be sequential.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "SDM address of the LI-7200 (0 through 14; address 15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "LI7200Cmd".to_string(),
                        documentation: "Requests which data set to retrieve from the analyzer; results are returned in Dest.".to_string(),
                    },
                ],
            }),

            "li7700" => Some(FunctionSignature {
                name: "LI7700".to_string(),
                documentation: "Measures methane concentration from an LI-7700 open-path gas analyzer via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Input variable array to store the data returned by each LI-7700; its length depends on the number of repetitions and the selected command.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "Number of LI-7700 units to communicate with; their SDM addresses must be sequential.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "SDM address of the LI-7700 (0 through 14; address 15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "LI7700Cmd".to_string(),
                        documentation: "Requests which data set to retrieve from the analyzer; results are returned in Dest.".to_string(),
                    },
                ],
            }),

            "cdm_acpower" => Some(FunctionSignature {
                name: "CDM_ACPower".to_string(),
                documentation: "Measures real AC power and power-quality parameters via a CDM module in single-phase, split-phase, or three-phase configurations.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestAC".to_string(),
                        documentation: "A variable or array to store the measurement results; the number of values returned depends on ConfigAC.".to_string(),
                    },
                    ParameterInfo {
                        name: "ConfigAC".to_string(),
                        documentation: "The measurement configuration: 1 single-phase, 2 split-phase, 3 three-phase 'Y'.".to_string(),
                    },
                    ParameterInfo {
                        name: "LineFrq".to_string(),
                        documentation: "The expected line frequency in Hz: 60, 50, or a value from 2 to 20 for variable-frequency (wild AC) power.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanV".to_string(),
                        documentation: "The single-ended channel for the voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MultV".to_string(),
                        documentation: "The potential-transformer multiplier, expressed as input volts per output millivolts.".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxVrms".to_string(),
                        documentation: "The expected maximum RMS voltage at the potential transformer's primary side.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanI".to_string(),
                        documentation: "The single-ended channel for the current measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MultI".to_string(),
                        documentation: "The current-transformer multiplier, expressed as input amps per output millivolts.".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxIrms".to_string(),
                        documentation: "The expected maximum RMS current at the current transformer's primary side.".to_string(),
                    },
                    ParameterInfo {
                        name: "RepsI".to_string(),
                        documentation: "The number of current measurements to make on consecutive channels; used only for single-phase configurations.".to_string(),
                    },
                ],
            }),

            "cdm_battery" => Some(FunctionSignature {
                name: "CDM_Battery".to_string(),
                documentation: "Reads and returns a CDM module's own power-supply voltage.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable to store the CDM module's power-supply voltage.".to_string(),
                    },
                ],
            }),

            "cdm_brfull" => Some(FunctionSignature {
                name: "CDM_BrFull".to_string(),
                documentation: "Makes a 4-wire full-bridge measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the sensor signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel used to excite the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of sensors to excite with the same excitation terminal before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "The excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_brfull6w" => Some(FunctionSignature {
                name: "CDM_BrFull6W".to_string(),
                documentation: "Makes a 6-wire full-bridge measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results (1000*V2/V1).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range1".to_string(),
                        documentation: "The expected input voltage range for the first channel measured.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range2".to_string(),
                        documentation: "The expected input voltage range for the second channel measured.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel used to excite the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of sensors to excite with the same excitation terminal before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "The excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "ReturnV1".to_string(),
                        documentation: "Optional; if non-zero, also returns V1 as a second element of Dest (which must then be a two-element or larger array).".to_string(),
                    },
                ],
            }),

            "cdm_brhalf" => Some(FunctionSignature {
                name: "CDM_BrHalf".to_string(),
                documentation: "Makes a single-ended half-bridge measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the sensor signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel used to excite the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of sensors to excite with the same excitation terminal before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "The excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_brhalf3w" => Some(FunctionSignature {
                name: "CDM_BrHalf3W".to_string(),
                documentation: "Makes a 3-wire half-bridge measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the sensor signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel used to excite the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of sensors to excite with the same excitation terminal before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "The excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_brhalf4w" => Some(FunctionSignature {
                name: "CDM_BrHalf4W".to_string(),
                documentation: "Makes a 4-wire half-bridge measurement via a CDM module, commonly used with PRTCalc for RTDs.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results (V2/V1).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range1".to_string(),
                        documentation: "The expected input voltage range for the first channel measured.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range2".to_string(),
                        documentation: "The expected input voltage range for the second channel measured.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel used to excite the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of sensors to excite with the same excitation terminal before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "The excitation voltage, in millivolts, applied to the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "ReturnV1".to_string(),
                        documentation: "Optional; if non-zero, also returns V1 as a second element of Dest (which must then be a two-element or larger array).".to_string(),
                    },
                ],
            }),

            "cdm_currentdiff" => Some(FunctionSignature {
                name: "CDM_CurrentDiff".to_string(),
                documentation: "Makes a differential current-loop measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results, in milliamps.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected current range of the input from the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_delay" => Some(FunctionSignature {
                name: "CDM_Delay".to_string(),
                documentation: "Delays a CDM module's measurement or processing task sequence for a specified time.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Whether the delay affects the measurement task sequence (0) or the processing task sequence (1).".to_string(),
                    },
                    ParameterInfo {
                        name: "Delay".to_string(),
                        documentation: "The time to delay, at 10-microsecond resolution; units are set by Units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "The time units for Delay: 0 usec, 1 msec, 2 sec, 3 min, 4 hr, 5 day.".to_string(),
                    },
                ],
            }),

            "cdm_excitei" => Some(FunctionSignature {
                name: "CDM_ExciteI".to_string(),
                documentation: "Applies a current excitation to an excitation channel on a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "IxChan".to_string(),
                        documentation: "The excitation channel to apply the current to.".to_string(),
                    },
                    ParameterInfo {
                        name: "IxuA".to_string(),
                        documentation: "The current excitation, in microamps, to apply; the allowable range is ±2500 µA.".to_string(),
                    },
                    ParameterInfo {
                        name: "Delay".to_string(),
                        documentation: "The time, in microseconds, before the excitation is turned off; 0 holds it until scan end or the next excitation set.".to_string(),
                    },
                ],
            }),

            "cdm_excitev" => Some(FunctionSignature {
                name: "CDM_ExciteV".to_string(),
                documentation: "Applies a voltage excitation to an excitation channel on a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel to apply the voltage to.".to_string(),
                    },
                    ParameterInfo {
                        name: "ExmV".to_string(),
                        documentation: "The excitation voltage, in millivolts, to apply; the allowable range is ±5000 mV.".to_string(),
                    },
                    ParameterInfo {
                        name: "Delay".to_string(),
                        documentation: "The time, in microseconds, before the excitation is turned off; 0 holds it until scan end, the next excitation set, or an interrupting measurement.".to_string(),
                    },
                ],
            }),

            "cdm_muxselect" => Some(FunctionSignature {
                name: "CDM_MuxSelect".to_string(),
                documentation: "Wakes and clocks an AM16/32A or AM16/32B multiplexer to a starting channel via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "ClkPort".to_string(),
                        documentation: "The switched-5V port used to clock the multiplexer.".to_string(),
                    },
                    ParameterInfo {
                        name: "ResetPort".to_string(),
                        documentation: "The switched-5V port used to wake up and reset the multiplexer; must be unique per multiplexer.".to_string(),
                    },
                    ParameterInfo {
                        name: "ClockPW".to_string(),
                        documentation: "The clock period, in milliseconds, used to advance the multiplexer.".to_string(),
                    },
                    ParameterInfo {
                        name: "MuxChan".to_string(),
                        documentation: "The first measurement channel on the multiplexer.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode".to_string(),
                        documentation: "The clocking mode: 0 for AM16/32A clocking, 1 for AM16/32B clocking.".to_string(),
                    },
                ],
            }),

            "cdm_paneltemp" => Some(FunctionSignature {
                name: "CDM_PanelTemp".to_string(),
                documentation: "Reads a CDM wiring-panel thermistor, for use as a thermocouple reference temperature.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the panel temperature, in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive thermistors.".to_string(),
                    },
                    ParameterInfo {
                        name: "ThermChan".to_string(),
                        documentation: "The wiring-panel thermistor bank to read; choose the one closest to the analog channel used for the thermocouple measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                ],
            }),

            "cdm_periodavg" => Some(FunctionSignature {
                name: "CDM_PeriodAvg".to_string(),
                documentation: "Measures the period or frequency of a signal on a CDM single-ended channel.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Gain".to_string(),
                        documentation: "The input gain code selecting the expected peak-to-peak signal range before the zero-crossing comparator.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Threshold".to_string(),
                        documentation: "The comparator trigger threshold, in millivolts, for signals not centered on 0 V.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Whether to return the period (0) or the frequency (1) of the signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "Cycles".to_string(),
                        documentation: "The number of signal cycles to average each scan.".to_string(),
                    },
                    ParameterInfo {
                        name: "Timeout".to_string(),
                        documentation: "The maximum time, in milliseconds, to wait for Cycles to complete before storing an overrange value.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_pulseport" => Some(FunctionSignature {
                name: "CDM_PulsePort".to_string(),
                documentation: "Toggles a CDM switched-5V digital port, delays, and toggles it again to generate a clock signal.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The switched-5V port to toggle.".to_string(),
                    },
                    ParameterInfo {
                        name: "Delay".to_string(),
                        documentation: "The time, in microseconds, to delay after each toggle.".to_string(),
                    },
                ],
            }),

            "cdm_resistance" => Some(FunctionSignature {
                name: "CDM_Resistance".to_string(),
                documentation: "Measures resistance via current excitation on a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the measured signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "IexChan".to_string(),
                        documentation: "The excitation channel supplying the known current.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of series-connected sensors to excite with the same excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "EXuA".to_string(),
                        documentation: "The excitation current, in microamps; the allowable range is ±2500 µA.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasCurrent".to_string(),
                        documentation: "Optional; if 1, also returns the measured excitation current as the last element of Dest.".to_string(),
                    },
                ],
            }),

            "cdm_resistance3w" => Some(FunctionSignature {
                name: "CDM_Resistance3W".to_string(),
                documentation: "Measures resistance via current excitation using a 3-wire connection on a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the measured signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the voltage measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "IexChan".to_string(),
                        documentation: "The excitation channel supplying the known current.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPEx".to_string(),
                        documentation: "The number of series-connected sensors to excite with the same excitation channel before advancing to the next.".to_string(),
                    },
                    ParameterInfo {
                        name: "EXuA".to_string(),
                        documentation: "The excitation current, in microamps; the allowable range is ±2500 µA.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevEx".to_string(),
                        documentation: "Whether to reverse the excitation and make a second measurement, canceling excitation-related offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasCurrent".to_string(),
                        documentation: "Optional; if 1, also returns the measured excitation current as the last element of Dest.".to_string(),
                    },
                ],
            }),

            "cdm_sw12" => Some(FunctionSignature {
                name: "CDM_SW12".to_string(),
                documentation: "Sets a CDM switched-12V output port high or low to power external peripherals.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The switched-12V port to use (1=S12-1, 2=S12-2).".to_string(),
                    },
                    ParameterInfo {
                        name: "State".to_string(),
                        documentation: "Whether to set the port high (non-zero) or low (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "SWOption".to_string(),
                        documentation: "Optional run-mode selector determining whether the instruction runs in the measurement or processing task sequence.".to_string(),
                    },
                ],
            }),

            "cdm_sw5" => Some(FunctionSignature {
                name: "CDM_SW5".to_string(),
                documentation: "Sets a CDM switched-5V output port high or low to power external peripherals.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The switched-5V port to use.".to_string(),
                    },
                    ParameterInfo {
                        name: "State".to_string(),
                        documentation: "Whether to set the port high (non-zero) or low (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "SWOption".to_string(),
                        documentation: "Optional run-mode selector determining whether the instruction runs in the measurement or processing task sequence.".to_string(),
                    },
                ],
            }),

            "cdm_swpower" => Some(FunctionSignature {
                name: "CDM_SWPower".to_string(),
                documentation: "Sets the ganged switched-12V and switched-5V power output on a VOLT408 isolation module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "State".to_string(),
                        documentation: "Whether to set the switched 12V/5V outputs high (non-zero) or low (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "SWOption".to_string(),
                        documentation: "Optional run-mode selector determining whether the instruction runs in the measurement or processing task sequence.".to_string(),
                    },
                ],
            }),

            "cdm_tccomp" => Some(FunctionSignature {
                name: "CDM_TCComp".to_string(),
                documentation: "Makes a differential thermocouple measurement with automatic cold-junction compensation via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the thermocouple measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCType".to_string(),
                        documentation: "The thermocouple type being measured (TypeT, TypeE, TypeK, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "FilterEnable".to_string(),
                        documentation: "Whether to enable simultaneous 50/60 Hz notch filtering (1) or run fast, unfiltered measurements (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "The temperature units of the result: 0 Celsius, 1 Fahrenheit, 2 Kelvin.".to_string(),
                    },
                ],
            }),

            "cdm_tcdiff" => Some(FunctionSignature {
                name: "CDM_TCDiff".to_string(),
                documentation: "Makes a differential thermocouple measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the temperature result(s), in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the thermocouple signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the thermocouple measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCType".to_string(),
                        documentation: "The thermocouple type being measured (TypeT, TypeE, TypeK, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "TRef".to_string(),
                        documentation: "The variable holding the reference (cold-junction) temperature, in degrees Celsius, e.g. from CDM_PanelTemp.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "cdm_tcse" => Some(FunctionSignature {
                name: "CDM_TCSE".to_string(),
                documentation: "Makes a single-ended thermocouple measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the temperature result(s), in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the thermocouple signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the thermocouple measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCType".to_string(),
                        documentation: "The thermocouple type being measured (TypeT, TypeE, TypeK, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "TRef".to_string(),
                        documentation: "The variable holding the reference (cold-junction) temperature, in degrees Celsius, e.g. from CDM_PanelTemp.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasOff".to_string(),
                        documentation: "Whether to measure and subtract the ground offset voltage before the measurement (1) or reuse the background calibration offset (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "cdm_therm107" => Some(FunctionSignature {
                name: "CDM_Therm107".to_string(),
                documentation: "Measures a 107 thermistor probe via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Excite".to_string(),
                        documentation: "The excitation channel used to apply voltage excitation to the thermistor.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "Integ".to_string(),
                        documentation: "The AC noise rejection frequency (60 or 50 Hz) or notch filter fN1, depending on the module.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "cdm_therm108" => Some(FunctionSignature {
                name: "CDM_Therm108".to_string(),
                documentation: "Measures a 108 thermistor probe via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Excite".to_string(),
                        documentation: "The excitation channel used to apply voltage excitation to the thermistor.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "Integ".to_string(),
                        documentation: "The AC noise rejection frequency (60 or 50 Hz) or notch filter fN1, depending on the module.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "cdm_therm109" => Some(FunctionSignature {
                name: "CDM_Therm109".to_string(),
                documentation: "Measures a 109 thermistor probe via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the temperature result(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Excite".to_string(),
                        documentation: "The excitation channel used to apply voltage excitation to the thermistor.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, allowed for the signal to settle before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "Integ".to_string(),
                        documentation: "The AC noise rejection frequency (60 or 50 Hz) or notch filter fN1, depending on the module.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the result (1 = Celsius, 1.8 = Fahrenheit).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset added after scaling by Mult (0 = Celsius, 32 = Fahrenheit).".to_string(),
                    },
                ],
            }),

            "cdm_voltdiff" => Some(FunctionSignature {
                name: "CDM_VoltDiff".to_string(),
                documentation: "Makes a differential voltage measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results, in millivolts.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the sensor signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "DiffChan".to_string(),
                        documentation: "The differential channel pair for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "Whether to reverse the differential inputs and make a second measurement, canceling measurement-circuitry offset errors.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_voltse" => Some(FunctionSignature {
                name: "CDM_VoltSE".to_string(),
                documentation: "Makes a single-ended voltage measurement via a CDM module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CDM module used by the instruction; a read-only value reported by the device itself.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The CPI bus address configured on the CDM module; must be a constant, valid range 1 through 120.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array to store the measurement results, in millivolts.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; measurements are made on consecutive channels.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The expected input voltage range of the sensor signal.".to_string(),
                    },
                    ParameterInfo {
                        name: "SEChan".to_string(),
                        documentation: "The single-ended channel for the measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasOff".to_string(),
                        documentation: "Whether to measure and subtract the ground offset voltage before the measurement (1) or reuse the background calibration offset (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The delay, in microseconds, after setting up the measurement and before measuring.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The first notch filter frequency used to reject noise.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw result to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw result to engineering units.".to_string(),
                    },
                ],
            }),

            "cdm_vw300config" => Some(FunctionSignature {
                name: "CDM_VW300Config".to_string(),
                documentation: "Sends configuration settings to a CDM-VW300 vibrating-wire spectrum analyzer; must precede BeginProg.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "DeviceType".to_string(),
                        documentation: "Identifies whether the analyzer is a CDM-VW300 (0) or a CDM-VW305 (1).".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The address of the CDM-VW300 on the CPI network (1 through 120); set with DVWTool, not by this parameter.".to_string(),
                    },
                    ParameterInfo {
                        name: "SysOption".to_string(),
                        documentation: "Determines whether a numeric value or NaN is stored on a warning flag, and whether diagnostic lights are active (0, 1, 10, or 11).".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanEnable".to_string(),
                        documentation: "An array activating each CDM-VW300 input channel (0 or 1 per channel).".to_string(),
                    },
                    ParameterInfo {
                        name: "ResonAmp".to_string(),
                        documentation: "The target resonant excitation amplitude the CDM-VW300 maintains, in volts (0.001 to 0.010 V).".to_string(),
                    },
                    ParameterInfo {
                        name: "LowFreq".to_string(),
                        documentation: "The lower boundary of the valid frequency range, in Hz (290 to 6000 Hz).".to_string(),
                    },
                    ParameterInfo {
                        name: "HighFreq".to_string(),
                        documentation: "The upper boundary of the valid frequency range, in Hz (290 to 6000 Hz).".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanOptions".to_string(),
                        documentation: "The output units/format for the channel (e.g. Hz or Hz²).".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "SteinA".to_string(),
                        documentation: "The Steinhart-Hart coefficient A used to convert the embedded thermistor reading to temperature.".to_string(),
                    },
                    ParameterInfo {
                        name: "SteinB".to_string(),
                        documentation: "The Steinhart-Hart coefficient B used to convert the embedded thermistor reading to temperature.".to_string(),
                    },
                    ParameterInfo {
                        name: "SteinC".to_string(),
                        documentation: "The Steinhart-Hart coefficient C used to convert the embedded thermistor reading to temperature.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF_MeanBins".to_string(),
                        documentation: "The number of mean-value bins in the rainflow histogram.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF_AmpBins".to_string(),
                        documentation: "The number of amplitude bins in the rainflow histogram.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF_LowLim".to_string(),
                        documentation: "The low limit of the rainflow histogram.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF_HighLim".to_string(),
                        documentation: "The high limit of the rainflow histogram.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF_Hyst".to_string(),
                        documentation: "The minimum amplitude change (hysteresis) counted in the rainflow histogram.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF_Form".to_string(),
                        documentation: "The output form/units of the rainflow histogram.".to_string(),
                    },
                ],
            }),

            "cdm_vw300dynamic" => Some(FunctionSignature {
                name: "CDM_VW300Dynamic".to_string(),
                documentation: "Captures the dynamic resonant frequency output of a CDM-VW300 vibrating-wire spectrum analyzer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The address of the CDM-VW300 on the CPI network.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestFreq".to_string(),
                        documentation: "A variable or array to store the dynamic resonant frequency output.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestDiag".to_string(),
                        documentation: "A variable or array to store the diagnostic code (excitation strength and amplitude/frequency warning flags).".to_string(),
                    },
                ],
            }),

            "cdm_vw300rainflow" => Some(FunctionSignature {
                name: "CDM_VW300RainFlow".to_string(),
                documentation: "Captures rainflow-histogram data from a CDM-VW300 vibrating-wire spectrum analyzer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The address of the CDM-VW300 on the CPI network.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF1".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF2".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 2.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF3".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 3.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF4".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 4.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF5".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 5.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF6".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 6.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF7".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 7.".to_string(),
                    },
                    ParameterInfo {
                        name: "RF8".to_string(),
                        documentation: "Destination for the rainflow histogram data of channel 8.".to_string(),
                    },
                ],
            }),

            "cdm_vw300static" => Some(FunctionSignature {
                name: "CDM_VW300Static".to_string(),
                documentation: "Captures the static resonant frequency, thermistor temperature, and frequency standard deviation from a CDM-VW300 vibrating-wire spectrum analyzer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The address of the CDM-VW300 on the CPI network.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestFreq".to_string(),
                        documentation: "A variable or array to store the static (1 Hz) resonant frequency output.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestTherm".to_string(),
                        documentation: "A variable or array to store the embedded thermistor/RTD reading (temperature or resistance).".to_string(),
                    },
                    ParameterInfo {
                        name: "DestStdDev".to_string(),
                        documentation: "A variable or array to store the standard deviation of the dynamic output over the latest one-second interval.".to_string(),
                    },
                ],
            }),

            "calibrate" => Some(FunctionSignature {
                name: "Calibrate".to_string(),
                documentation: "Forces calibration of all analog channels under program control to compensate for temperature-related measurement errors.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "An optional array (minimum 60 elements) to store the calibration coefficients.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "An optional constant: 0 calibrates only the ranges used in the program; non-zero calibrates all ranges.".to_string(),
                    },
                ],
            }),

            "fieldcal" => Some(FunctionSignature {
                name: "FieldCal".to_string(),
                documentation: "Sets up the datalogger to perform calibration of one or more variables in an array.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Function".to_string(),
                        documentation: "The calibration type: 0=Zero, 1=Offset, 2=Two Point Mult+Offset, 3=Two Point Mult Only, 4=Zero Basis.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasureVar".to_string(),
                        documentation: "The variable or array being calibrated.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "How many array elements to calibrate; must equal 1 or the array size.".to_string(),
                    },
                    ParameterInfo {
                        name: "MultVar".to_string(),
                        documentation: "The variable or array that receives the computed multiplier value(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "OffsetVar".to_string(),
                        documentation: "The variable or array that receives the computed offset value(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode".to_string(),
                        documentation: "A status/trigger variable indicating the current calibration state (-6 to 6).".to_string(),
                    },
                    ParameterInfo {
                        name: "KnownVar".to_string(),
                        documentation: "The variable holding the reference set-point value(s) used for calibration.".to_string(),
                    },
                    ParameterInfo {
                        name: "Index".to_string(),
                        documentation: "Which array element to calibrate when Reps=1; must be initialized to a non-zero value.".to_string(),
                    },
                    ParameterInfo {
                        name: "Avg".to_string(),
                        documentation: "The number of points to average during calibration.".to_string(),
                    },
                ],
            }),

            "fieldcalstrain" => Some(FunctionSignature {
                name: "FieldCalStrain".to_string(),
                documentation: "Sets up the datalogger to perform a zero or shunt calibration for a strain measurement.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Function".to_string(),
                        documentation: "The calibration type: 10 for zero calibration, 13/33/43 for shunt-calibration variants.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasureVar".to_string(),
                        documentation: "The variable or array holding the StrainCalc results being calibrated.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "How many values to calibrate; must equal 1 or the full array size.".to_string(),
                    },
                    ParameterInfo {
                        name: "GFAdj".to_string(),
                        documentation: "The adjusted gauge factor(s); 0 for a zero calibration.".to_string(),
                    },
                    ParameterInfo {
                        name: "ZeromV/V".to_string(),
                        documentation: "The zero-offset value(s); 0 for a shunt calibration.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode".to_string(),
                        documentation: "A variable indicating the current calibration state (-1 to 6).".to_string(),
                    },
                    ParameterInfo {
                        name: "KnownRS".to_string(),
                        documentation: "The shunt resistance value(s) used for a shunt calibration.".to_string(),
                    },
                    ParameterInfo {
                        name: "Index".to_string(),
                        documentation: "The array element index to calibrate when Reps=1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Avg".to_string(),
                        documentation: "The number of points to average during calibration.".to_string(),
                    },
                    ParameterInfo {
                        name: "GFRaw".to_string(),
                        documentation: "The raw, manufacturer-supplied gauge factor(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "uStrainDest".to_string(),
                        documentation: "The micro-strain result variable; 0 for a shunt calibration.".to_string(),
                    },
                ],
            }),

            "loadfieldcal" => Some(FunctionSignature {
                name: "LoadFieldCal".to_string(),
                documentation: "Loads values from the FieldCal file into datalogger variables, returning True if successful.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "CheckSig".to_string(),
                    documentation: "An optional Boolean controlling whether the program's signature must match the one stored with the calibration file.".to_string(),
                }],
            }),

            "samplefieldcal" => Some(FunctionSignature {
                name: "SampleFieldCal".to_string(),
                documentation: "Stores the values in the FieldCal file to a data table; used inside a DataTable/EndTable declaration.".to_string(),
                parameters: vec![],
            }),

            "acpower" => Some(FunctionSignature {
                name: "ACPower".to_string(),
                documentation: "Measures real AC power and power-quality parameters for single-phase, split-phase, or three-phase Y systems.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "DestAC".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "ConfigAC".to_string(),
                        documentation: "The measurement configuration: 1=single-phase, 2=split-phase, 3=three-phase Y.".to_string(),
                    },
                    ParameterInfo {
                        name: "LineFrq".to_string(),
                        documentation: "The line frequency in Hz (50, 60, or a value 2-20 for a variable frequency).".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanV".to_string(),
                        documentation: "The starting channel for the voltage measurement(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "MultV".to_string(),
                        documentation: "The potential transformer multiplier (input volts per output millivolt).".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxVrms".to_string(),
                        documentation: "The expected maximum RMS voltage at the transformer primary.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanI".to_string(),
                        documentation: "The starting channel for the current measurement(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "MultI".to_string(),
                        documentation: "The current transformer multiplier (input amps per output millivolt).".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxIrms".to_string(),
                        documentation: "The expected maximum RMS current at the transformer primary.".to_string(),
                    },
                    ParameterInfo {
                        name: "RepsI".to_string(),
                        documentation: "The number of current measurements to take (single-phase configuration only).".to_string(),
                    },
                ],
            }),

            "am25t" => Some(FunctionSignature {
                name: "AM25T".to_string(),
                documentation: "Controls and measures the AM25T thermocouple multiplexer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; 0 measures the reference PRT only.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The voltage measurement range code.".to_string(),
                    },
                    ParameterInfo {
                        name: "AM25TChan".to_string(),
                        documentation: "The starting input channel number on the AM25T.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChanAnlg".to_string(),
                        documentation: "The differential analog terminal connected to the AM25T's common output.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCType".to_string(),
                        documentation: "The thermocouple type identifier.".to_string(),
                    },
                    ParameterInfo {
                        name: "TRef".to_string(),
                        documentation: "A variable holding the reference temperature, in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "ClkPort".to_string(),
                        documentation: "The control port used to clock the AM25T (C1 through C8).".to_string(),
                    },
                    ParameterInfo {
                        name: "ResPort".to_string(),
                        documentation: "The control port used to reset the AM25T (C1 through C8).".to_string(),
                    },
                    ParameterInfo {
                        name: "ExChan".to_string(),
                        documentation: "The excitation channel used for the reference PRT measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "RevDiff".to_string(),
                        documentation: "A Boolean selecting whether the differential measurement polarity is reversed.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The signal settling duration, in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The filter notch frequency, in Hz.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                ],
            }),

            "avw200" => Some(FunctionSignature {
                name: "AVW200".to_string(),
                documentation: "Reads vibrating-wire sensors via an AVW200 spectrum analyzer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Result".to_string(),
                        documentation: "A variable that receives the communication result/status code.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communications port used to reach the AVW200.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address used to route through an intermediate datalogger, if any.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The target AVW200's PakBus network address (1 through 4094).".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "AVWChan".to_string(),
                        documentation: "The AVW200 channel (1 or 2) that the sensor is connected to.".to_string(),
                    },
                    ParameterInfo {
                        name: "MuxChannel".to_string(),
                        documentation: "The starting multiplexer channel (1 through 32), or 1 if no multiplexer is used.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of measurements to take on the multiplexer.".to_string(),
                    },
                    ParameterInfo {
                        name: "BeginFreq".to_string(),
                        documentation: "The starting sweep frequency, in Hz (minimum 100).".to_string(),
                    },
                    ParameterInfo {
                        name: "EndFreq".to_string(),
                        documentation: "The ending sweep frequency, in Hz (maximum 6500).".to_string(),
                    },
                    ParameterInfo {
                        name: "ExVolt".to_string(),
                        documentation: "The excitation voltage: 1 for 5V, 2 for 12V peak-to-peak.".to_string(),
                    },
                    ParameterInfo {
                        name: "Therm50_60Hz".to_string(),
                        documentation: "The thermistor integration setting (0, 1, or 2).".to_string(),
                    },
                    ParameterInfo {
                        name: "Multiplier".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "AmpThreshold".to_string(),
                        documentation: "An optional minimum signal amplitude threshold, in millivolts.".to_string(),
                    },
                ],
            }),

            "cs616" => Some(FunctionSignature {
                name: "CS616".to_string(),
                documentation: "Enables and measures a CS616/CS625 water content reflectometer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "CS616Chan".to_string(),
                        documentation: "The single-ended terminal number for the first measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "CS616Port".to_string(),
                        documentation: "The control port used to enable the sensor(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasPerPort".to_string(),
                        documentation: "The number of control terminals used per sensor measurement.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw output period to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw output period to engineering units.".to_string(),
                    },
                ],
            }),

            "cs7500" => Some(FunctionSignature {
                name: "CS7500".to_string(),
                documentation: "Communicates with a LI-7500(A) gas analyzer via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of gas analyzers to communicate with.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The device's SDM address (0 through 14; 15 is reserved).".to_string(),
                    },
                    ParameterInfo {
                        name: "CS7500Cmd".to_string(),
                        documentation: "A code specifying which sensor data to retrieve.".to_string(),
                    },
                ],
            }),

            "currentse" => Some(FunctionSignature {
                name: "CurrentSE".to_string(),
                documentation: "Measures single-ended current via the datalogger's internal shunt resistor.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions.".to_string(),
                    },
                    ParameterInfo {
                        name: "Range".to_string(),
                        documentation: "The input voltage range, in millivolts.".to_string(),
                    },
                    ParameterInfo {
                        name: "RGChan".to_string(),
                        documentation: "The internal shunt channel to use: RG1 or RG2.".to_string(),
                    },
                    ParameterInfo {
                        name: "MeasOff".to_string(),
                        documentation: "Whether to also measure the ground offset voltage.".to_string(),
                    },
                    ParameterInfo {
                        name: "SettlingTime".to_string(),
                        documentation: "The signal settling duration, in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "fN1".to_string(),
                        documentation: "The lowest frequency eliminated by the input filter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the result.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the result.".to_string(),
                    },
                ],
            }),

            "hydraprobe" => Some(FunctionSignature {
                name: "HydraProbe".to_string(),
                documentation: "Converts raw voltages from a Stevens Hydra Probe sensor into soil measurements.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "An 11-element array that receives the processed soil measurements.".to_string(),
                    },
                    ParameterInfo {
                        name: "SourceVolts".to_string(),
                        documentation: "A 4-element array holding the raw voltage readings from the sensor.".to_string(),
                    },
                    ParameterInfo {
                        name: "ProbeType".to_string(),
                        documentation: "The probe version: 0=Standard, 1=Type A.".to_string(),
                    },
                    ParameterInfo {
                        name: "SoilType".to_string(),
                        documentation: "The soil classification: 1=sand, 2=silt, 3=clay, 4=loam.".to_string(),
                    },
                    ParameterInfo {
                        name: "Multiplier".to_string(),
                        documentation: "A scaling factor used in the temperature conversion.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An adjustment value used in the temperature conversion.".to_string(),
                    },
                ],
            }),

            "tdr100" => Some(FunctionSignature {
                name: "TDR100".to_string(),
                documentation: "Measures time-domain-reflectometry probes via a TDR100 device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The device's SDM address (0 through 14).".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "The output type: 0=La/L ratio, 1=waveform, 2=waveform+derivative, 3=conductivity.".to_string(),
                    },
                    ParameterInfo {
                        name: "MuxOrProbeSelect".to_string(),
                        documentation: "The multiplexer channel and probe selection, in ABCR format.".to_string(),
                    },
                    ParameterInfo {
                        name: "WaveAvg".to_string(),
                        documentation: "The number of reflections to average (1 through 128).".to_string(),
                    },
                    ParameterInfo {
                        name: "Vp".to_string(),
                        documentation: "The propagation velocity; set to 1.0 for soil measurements.".to_string(),
                    },
                    ParameterInfo {
                        name: "Points".to_string(),
                        documentation: "The number of waveform values to collect (20 through 2048).".to_string(),
                    },
                    ParameterInfo {
                        name: "CableLength".to_string(),
                        documentation: "The cable length, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "WindowLength".to_string(),
                        documentation: "The waveform collection window length, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "ProbeLength".to_string(),
                        documentation: "The exposed probe rod length, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "ProbeOffset".to_string(),
                        documentation: "A correction value for epoxy-encapsulated probe rods.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                ],
            }),

            "tdr200" => Some(FunctionSignature {
                name: "TDR200".to_string(),
                documentation: "Measures time-domain-reflectometry probes via a TDR200 device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The device's SDM address (0 through 14).".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "The output type: 0=La/L ratio, 1=waveform, 2=waveform+derivative, 3=conductivity.".to_string(),
                    },
                    ParameterInfo {
                        name: "MuxOrProbeSelect".to_string(),
                        documentation: "The multiplexer channel and probe selection, in ABCR format.".to_string(),
                    },
                    ParameterInfo {
                        name: "WaveAvg".to_string(),
                        documentation: "The number of reflections to average (up to 128).".to_string(),
                    },
                    ParameterInfo {
                        name: "Vp".to_string(),
                        documentation: "The propagation velocity; typically 1.0.".to_string(),
                    },
                    ParameterInfo {
                        name: "Points".to_string(),
                        documentation: "The number of waveform data points to collect (20 through 10,112).".to_string(),
                    },
                    ParameterInfo {
                        name: "CableLength".to_string(),
                        documentation: "The probe cable length, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "WindowLength".to_string(),
                        documentation: "The waveform collection window length, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "ProbeLength".to_string(),
                        documentation: "The exposed probe rod length, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "ProbeOffset".to_string(),
                        documentation: "A correction value for epoxy-encapsulated probe rods.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "NoiseRejectionFreq".to_string(),
                        documentation: "The noise-rejection filter frequency: 0, 50, or 60 Hz.".to_string(),
                    },
                    ParameterInfo {
                        name: "TDRFilterLevel".to_string(),
                        documentation: "The weighted-averaging filter level (0 through 10).".to_string(),
                    },
                    ParameterInfo {
                        name: "TDRLaa".to_string(),
                        documentation: "The algorithm used for probe-length detection (0, 1, or 2).".to_string(),
                    },
                ],
            }),

            "tga" => Some(FunctionSignature {
                name: "TGA".to_string(),
                documentation: "Measures a TGA100A/TGA200/TGA200A trace gas analyzer via SDM.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The array in which the measurement results are stored; its length depends on DataList and ScanMode.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The device's SDM address (0 through 14; 15 is reserved).".to_string(),
                    },
                    ParameterInfo {
                        name: "DataList".to_string(),
                        documentation: "Which data set to retrieve: 1=concentration/status, up to 5=all data.".to_string(),
                    },
                    ParameterInfo {
                        name: "ScanMode".to_string(),
                        documentation: "The number of scan-specific values to retrieve, matching the TGA's number of ramps (1 through 3).".to_string(),
                    },
                ],
            }),

            "quadrature" => Some(FunctionSignature {
                name: "Quadrature".to_string(),
                documentation: "Measures a shaft quadrature encoder to determine displacement and rotational direction.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A 4-element array storing the accumulator, net direction, and up/down counts.".to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The digital port pair used: C1, C3, C5, or C7.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "The counting mode: 0=X1, 1=X2, 2=X4.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A scaling factor applied to the measurement results.".to_string(),
                    },
                ],
            }),

            "sw12" => Some(FunctionSignature {
                name: "SW12".to_string(),
                documentation: "Enables or disables a switched-12V output channel to power external peripherals.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "SWChan".to_string(),
                        documentation: "The switched-12V port to control: SW12_1, SW12_2, or SW12_CSIO.".to_string(),
                    },
                    ParameterInfo {
                        name: "State".to_string(),
                        documentation: "Whether the 12V supply is enabled (non-zero) or disabled (0).".to_string(),
                    },
                    ParameterInfo {
                        name: "SW12Option".to_string(),
                        documentation: "An optional value controlling the execution context (measurement or processing task).".to_string(),
                    },
                ],
            }),

            "etsz" => Some(FunctionSignature {
                name: "ETsz".to_string(),
                documentation: "Calculates the ASCE standardized reference evapotranspiration.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Temp".to_string(),
                        documentation: "A variable holding air temperature, in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "RH".to_string(),
                        documentation: "A variable holding relative humidity, in percent.".to_string(),
                    },
                    ParameterInfo {
                        name: "uZ".to_string(),
                        documentation: "A variable holding wind speed, in meters per second.".to_string(),
                    },
                    ParameterInfo {
                        name: "Rs".to_string(),
                        documentation: "Solar radiation, in megajoules per square meter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Longitude".to_string(),
                        documentation: "The station's longitude, in decimal degrees west.".to_string(),
                    },
                    ParameterInfo {
                        name: "Latitude".to_string(),
                        documentation: "The station's latitude, in decimal degrees (±90).".to_string(),
                    },
                    ParameterInfo {
                        name: "Altitude".to_string(),
                        documentation: "The station's elevation, in meters above sea level.".to_string(),
                    },
                    ParameterInfo {
                        name: "Zw".to_string(),
                        documentation: "The wind sensor's height, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "Sz".to_string(),
                        documentation: "The reference crop type: 0 for short reference, 1 for tall reference.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "The output storage data type (e.g. IEEE4, FP2, IEEE8).".to_string(),
                    },
                    ParameterInfo {
                        name: "DisableVar".to_string(),
                        documentation: "An optional condition that excludes the measurement from output when true.".to_string(),
                    },
                ],
            }),

            "solarposition" => Some(FunctionSignature {
                name: "SolarPosition".to_string(),
                documentation: "Calculates solar azimuth, elevation, hour angle, declination, and air mass.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A 5-element array that receives azimuth, elevation, hour angle, declination, and air mass.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeArray".to_string(),
                        documentation: "A 9-element array containing date/time components from the RealTime instruction.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOffset".to_string(),
                        documentation: "The local UTC offset, in seconds; overridden by the datalogger's UTC Offset setting if enabled.".to_string(),
                    },
                    ParameterInfo {
                        name: "Latitude".to_string(),
                        documentation: "The station's latitude, in decimal degrees (±90; positive is Northern Hemisphere).".to_string(),
                    },
                    ParameterInfo {
                        name: "Longitude".to_string(),
                        documentation: "The station's longitude, in decimal degrees east of the Greenwich meridian.".to_string(),
                    },
                    ParameterInfo {
                        name: "Altitude".to_string(),
                        documentation: "The station's elevation above sea level, in meters.".to_string(),
                    },
                    ParameterInfo {
                        name: "Pressure".to_string(),
                        documentation: "The annual average barometric pressure, in millibars; -1 estimates it from temperature.".to_string(),
                    },
                    ParameterInfo {
                        name: "AirTemp".to_string(),
                        documentation: "Air temperature, in degrees Celsius, used in the pressure/position calculations.".to_string(),
                    },
                ],
            }),

            "wetdrybulb" => Some(FunctionSignature {
                name: "WetDryBulb".to_string(),
                documentation: "Computes vapor pressure from wet-bulb and dry-bulb temperatures and barometric pressure.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array that receives the computed vapor pressure, in kPa.".to_string(),
                    },
                    ParameterInfo {
                        name: "DryTemp".to_string(),
                        documentation: "A variable holding the ambient (dry-bulb) air temperature, in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "WetTemp".to_string(),
                        documentation: "A variable holding the wet-bulb temperature, in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "Pressure".to_string(),
                        documentation: "The air pressure, in kilopascals.".to_string(),
                    },
                ],
            }),

            "muxselect" => Some(FunctionSignature {
                name: "MuxSelect".to_string(),
                documentation: "Selects a channel on an AM16/32A or AM16/32B multiplexer and readies it for measurement.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ClkPort".to_string(),
                        documentation: "The control port used to clock/advance the multiplexer channel.".to_string(),
                    },
                    ParameterInfo {
                        name: "ResetPort".to_string(),
                        documentation: "The control port used to wake up and reset the multiplexer.".to_string(),
                    },
                    ParameterInfo {
                        name: "ClockPW".to_string(),
                        documentation: "The clock pulse width, in milliseconds, controlling how fast the multiplexer advances.".to_string(),
                    },
                    ParameterInfo {
                        name: "MuxChan".to_string(),
                        documentation: "The first measurement channel to select.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode".to_string(),
                        documentation: "The clocking type: 0 for AM16/32A, 1 for AM16/32B.".to_string(),
                    },
                ],
            }),

            "pulsecountreset" => Some(FunctionSignature {
                name: "PulseCountReset".to_string(),
                documentation: "Resets the pulse counter and running-average values associated with pulse count measurements.".to_string(),
                parameters: vec![],
            }),

            "prt" => Some(FunctionSignature {
                name: "PRT".to_string(),
                documentation: "Converts RTD resistance measurements to temperature using the DIN 43760 standard.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the temperature result is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of repetitions; if greater than 1, Dest must be an array.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "A variable containing the RTD resistance ratio (RS/RO).".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "An optional multiplier to convert the result to a different unit (defaults to 1, for Celsius).".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An optional offset to convert the result to a different unit (defaults to 0, for Celsius).".to_string(),
                    },
                ],
            }),

            "prtcalc" => Some(FunctionSignature {
                name: "PRTCalc".to_string(),
                documentation: "Converts RTD resistance measurements to temperature using the Callendar-Van Dusen equation.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the temperature result is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of times the calculation repeats on consecutive Source elements; defaults to 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "A variable containing the RTD resistance ratio (RS/RO).".to_string(),
                    },
                    ParameterInfo {
                        name: "PRTType".to_string(),
                        documentation: "The sensor type code (0 through 6), specifying the RTD standard and alpha value.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "An optional multiplier to convert the output to a different unit.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An optional offset to convert the output to a different unit.".to_string(),
                    },
                ],
            }),

            "moveprecise" => Some(FunctionSignature {
                name: "MovePrecise".to_string(),
                documentation: "Transfers a value into a variable as a high-precision (56-bit mantissa) number.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "PrecisionVariable".to_string(),
                        documentation: "The variable that receives the high-precision value.".to_string(),
                    },
                    ParameterInfo {
                        name: "X".to_string(),
                        documentation: "The value to move into PrecisionVariable.".to_string(),
                    },
                ],
            }),

            "pwr" => Some(FunctionSignature {
                name: "PWR".to_string(),
                documentation: "Raises X to the power of Y and returns a floating-point result.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "X".to_string(),
                        documentation: "The base value.".to_string(),
                    },
                    ParameterInfo {
                        name: "Y".to_string(),
                        documentation: "The exponent applied to the base.".to_string(),
                    },
                ],
            }),

            "ctype" => Some(FunctionSignature {
                name: "CType".to_string(),
                documentation: "Converts an expression to a specified data type (Float, IEEE4, Long, String, or Double).".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Expression".to_string(),
                        documentation: "The value or string to convert.".to_string(),
                    },
                    ParameterInfo {
                        name: "Type".to_string(),
                        documentation: "The target data type: Float, IEEE4, Long, String, or Double.".to_string(),
                    },
                ],
            }),

            "serialinchk" => Some(FunctionSignature {
                name: "SerialInChk".to_string(),
                documentation: "Returns the number of characters currently available in the serial input buffer.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "ComPort".to_string(),
                    documentation: "The communications port to check.".to_string(),
                }],
            }),

            "setsecurity" => Some(FunctionSignature {
                name: "SetSecurity".to_string(),
                documentation: "Establishes up to three hierarchical security levels restricting access to datalogger functions.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Security1".to_string(),
                        documentation: "The highest-level security code; enables program changes and ConstTable editing when entered.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security2".to_string(),
                        documentation: "The mid-level security code; allows clock changes and DataTable field editing when entered.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security3".to_string(),
                        documentation: "The lowest-level security code; permits basic data collection when entered.".to_string(),
                    },
                ],
            }),

            "watchdogtimer" => Some(FunctionSignature {
                name: "WatchdogTimer".to_string(),
                documentation: "Enables a user-programmed watchdog timer that guards the program against lockup.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The watchdog interval; 0 disables the watchdog timer.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for Interval.".to_string(),
                    },
                ],
            }),

            "pwm" => Some(FunctionSignature {
                name: "PWM".to_string(),
                documentation: "Generates a pulse-width-modulated signal on a digital port at a specified duty cycle.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "The duty cycle, as a constant or variable from 0.0 (always off) to 1.0 (always on).".to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The digital channel to output the PWM signal on.".to_string(),
                    },
                    ParameterInfo {
                        name: "Period".to_string(),
                        documentation: "The signal period; maximum is 36.4 seconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for Period (usec, msec, or sec).".to_string(),
                    },
                ],
            }),

            "dewpoint" => Some(FunctionSignature {
                name: "DewPoint".to_string(),
                documentation: "Calculates dew point temperature from air temperature and relative humidity.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination variable to store the calculated dew point.".to_string(),
                    },
                    ParameterInfo {
                        name: "Temp".to_string(),
                        documentation: "Variable holding the dry bulb air temperature, in degrees Celsius.".to_string(),
                    },
                    ParameterInfo {
                        name: "RH".to_string(),
                        documentation: "Variable holding the relative humidity, in percent.".to_string(),
                    },
                ],
            }),

            "serialopen" => Some(FunctionSignature {
                name: "SerialOpen".to_string(),
                documentation: "Opens a serial communication port.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "COM port number or constant.".to_string(),
                    },
                    ParameterInfo {
                        name: "BaudRate".to_string(),
                        documentation: "Baud rate (e.g., 9600, 115200).".to_string(),
                    },
                    ParameterInfo {
                        name: "Format".to_string(),
                        documentation: "Data format (0=8N1, etc.).".to_string(),
                    },
                    ParameterInfo {
                        name: "TXDelay".to_string(),
                        documentation: "Transmit delay in microseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "BufferSize".to_string(),
                        documentation: "Input buffer size in bytes.".to_string(),
                    },
                ],
            }),

            "serialout" => Some(FunctionSignature {
                name: "SerialOut".to_string(),
                documentation: "Sends data to a serial port.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "COM port number.".to_string(),
                    },
                    ParameterInfo {
                        name: "OutString".to_string(),
                        documentation: "String to send.".to_string(),
                    },
                    ParameterInfo {
                        name: "WaitString".to_string(),
                        documentation: "Expected response string.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumberTries".to_string(),
                        documentation: "Number of retry attempts.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Timeout in milliseconds.".to_string(),
                    },
                ],
            }),

            "serialin" => Some(FunctionSignature {
                name: "SerialIn".to_string(),
                documentation: "Reads data from a serial port.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "Destination string variable.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "COM port number.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Timeout in milliseconds.".to_string(),
                    },
                    ParameterInfo {
                        name: "TerminationChar".to_string(),
                        documentation: "Character that ends the read.".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxNumChars".to_string(),
                        documentation: "Maximum characters to read.".to_string(),
                    },
                ],
            }),

            "serialclose" => Some(FunctionSignature {
                name: "SerialClose".to_string(),
                documentation:
                    "Closes a communications port that was previously opened by SerialOpen."
                        .to_string(),
                parameters: vec![ParameterInfo {
                    name: "ComPort".to_string(),
                    documentation: "The communications port to close.".to_string(),
                }],
            }),

            "serialinrecord" => Some(FunctionSignature {
                name: "SerialInRecord".to_string(),
                documentation:
                    "Reads incoming serial data and stores a begin/end-marker-delimited record into a destination variable."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "COMPort".to_string(),
                        documentation: "The communications port to read from.".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable in which to store the record read from the buffer.".to_string(),
                    },
                    ParameterInfo {
                        name: "BeginWord".to_string(),
                        documentation: "A one- or two-byte value marking the start of the record.".to_string(),
                    },
                    ParameterInfo {
                        name: "NBytes".to_string(),
                        documentation: "The number of bytes to store after BeginWord, or 0/negative to capture up to EndWord.".to_string(),
                    },
                    ParameterInfo {
                        name: "EndWord".to_string(),
                        documentation: "A one- or two-byte value marking the end of the record.".to_string(),
                    },
                    ParameterInfo {
                        name: "NBytesReturned".to_string(),
                        documentation: "Variable that receives the number of bytes actually read.".to_string(),
                    },
                    ParameterInfo {
                        name: "SerialInRecOption".to_string(),
                        documentation: "Selects which buffered record to return and whether NAN is stored when none is available.".to_string(),
                    },
                ],
            }),

            "serialoutblock" => Some(FunctionSignature {
                name: "SerialOutBlock".to_string(),
                documentation: "Sends binary data, including null bytes, out a serial port."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communications port to send data out of."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Expression".to_string(),
                        documentation: "The data being transmitted over the port.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumberBytes".to_string(),
                        documentation: "The number of bytes from Expression to transmit."
                            .to_string(),
                    },
                ],
            }),

            "serialflush" => Some(FunctionSignature {
                name: "SerialFlush".to_string(),
                documentation: "Clears any characters currently in the serial input buffer."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "ComPort".to_string(),
                    documentation: "The communications port whose input buffer is cleared."
                        .to_string(),
                }],
            }),

            "modbusmaster" => Some(FunctionSignature {
                name: "ModbusMaster".to_string(),
                documentation:
                    "Sets up the datalogger as a Modbus master to send or retrieve data from a Modbus slave device."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives the communication result or Modbus exception code.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communications port used for the Modbus session."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "BaudRate".to_string(),
                        documentation: "The baud rate, in bps, used for the Modbus communication.".to_string(),
                    },
                    ParameterInfo {
                        name: "ModbusAddr".to_string(),
                        documentation: "The Modbus address of the target server device."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Function".to_string(),
                        documentation: "The Modbus function code specifying which data operation to perform.".to_string(),
                    },
                    ParameterInfo {
                        name: "Variable".to_string(),
                        documentation: "The variable or array used as the source or destination for the transferred data.".to_string(),
                    },
                    ParameterInfo {
                        name: "Start".to_string(),
                        documentation: "The address of the first register to read or write."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "The number of CRBasic variables to act upon."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Tries".to_string(),
                        documentation: "The number of attempts before giving up and continuing to the next instruction.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "ModbusOption".to_string(),
                        documentation: "Optional data type and byte-order formatting for the transferred registers.".to_string(),
                    },
                ],
            }),

            "tcpopen" => Some(FunctionSignature {
                name: "TCPOpen".to_string(),
                documentation:
                    "Sets up a TCP/IP socket for communication, either as a client connection or a listening server."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "IPAddr".to_string(),
                        documentation: "The destination IP address or domain name, or an empty string to listen for connections.".to_string(),
                    },
                    ParameterInfo {
                        name: "TCPPort".to_string(),
                        documentation: "The destination port (client mode) or listening port (server mode).".to_string(),
                    },
                    ParameterInfo {
                        name: "IPBuffer".to_string(),
                        documentation: "Size of the input buffer for non-PakBus communication; 0 for PakBus.".to_string(),
                    },
                    ParameterInfo {
                        name: "IPTimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, allowed to establish or maintain the connection.".to_string(),
                    },
                    ParameterInfo {
                        name: "ConnectHandle".to_string(),
                        documentation: "Variable or array that receives the handle(s) of the resulting connection(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxConnect".to_string(),
                        documentation: "The maximum number of connections this instance can create.".to_string(),
                    },
                ],
            }),

            "tcpclose" => Some(FunctionSignature {
                name: "TCPClose".to_string(),
                documentation: "Closes a TCP/IP socket that was set up for communication."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "TCPSocket".to_string(),
                    documentation: "The socket handle returned by TCPOpen.".to_string(),
                }],
            }),

            "udpopen" => Some(FunctionSignature {
                name: "UDPOpen".to_string(),
                documentation: "Opens a port for transferring UDP packets.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "IPAddr".to_string(),
                        documentation: "The target IP address or domain name.".to_string(),
                    },
                    ParameterInfo {
                        name: "UDPPort".to_string(),
                        documentation: "The UDP port number used for communication.".to_string(),
                    },
                    ParameterInfo {
                        name: "IPBuffer".to_string(),
                        documentation: "Size of the input serial buffer; must not be 0, to avoid PakBus interference.".to_string(),
                    },
                    ParameterInfo {
                        name: "IPVersion".to_string(),
                        documentation: "Address type to listen on, IPv4 or IPv6, when IPAddr is empty.".to_string(),
                    },
                ],
            }),

            "udpsocketopen" => Some(FunctionSignature {
                name: "UDPSocketOpen".to_string(),
                documentation: "Opens a UDP socket, relating a UDP source port to an ID."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "SocketID".to_string(),
                        documentation: "Variable that receives the socket ID, or a negative error code.".to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The non-ephemeral port to bind the socket to, or 0 for an ephemeral port.".to_string(),
                    },
                    ParameterInfo {
                        name: "RecvQueueSize".to_string(),
                        documentation: "The maximum number of received messages to queue for UDPSocketRecv.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interface".to_string(),
                        documentation: "The network interface to bind the socket to.".to_string(),
                    },
                ],
            }),

            "udpsocketsend" => Some(FunctionSignature {
                name: "UDPSocketSend".to_string(),
                documentation:
                    "Sends a UDP datagram to a remote device via an opened UDP socket."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BytesSent".to_string(),
                        documentation: "Variable that receives the number of bytes sent, or a negative error code.".to_string(),
                    },
                    ParameterInfo {
                        name: "SocketID".to_string(),
                        documentation: "The socket ID returned by UDPSocketOpen.".to_string(),
                    },
                    ParameterInfo {
                        name: "IPAddr".to_string(),
                        documentation: "The IP address of the device to send the datagram to."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Port".to_string(),
                        documentation: "The port of the device to send the datagram to."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Payload".to_string(),
                        documentation: "The contents of the datagram to send.".to_string(),
                    },
                    ParameterInfo {
                        name: "PayLoadLen".to_string(),
                        documentation: "The length, in bytes, of the payload.".to_string(),
                    },
                ],
            }),

            "udpsocketrecv" => Some(FunctionSignature {
                name: "UDPSocketRecv".to_string(),
                documentation:
                    "Retrieves incoming UDP packets sent to a socket's listening port."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BytesReceived".to_string(),
                        documentation: "Variable that receives the number of bytes received, or a negative error/timeout code.".to_string(),
                    },
                    ParameterInfo {
                        name: "SocketID".to_string(),
                        documentation: "The socket ID returned by UDPSocketOpen.".to_string(),
                    },
                    ParameterInfo {
                        name: "InDatagram".to_string(),
                        documentation: "The variable in which the received datagram payload is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "InDatagramLen".to_string(),
                        documentation: "The maximum number of bytes to store in InDatagram, or 0 to use all available memory.".to_string(),
                    },
                    ParameterInfo {
                        name: "RemoteIPAdd".to_string(),
                        documentation: "The IP address of the remote device that sent the datagram.".to_string(),
                    },
                    ParameterInfo {
                        name: "RemotePort".to_string(),
                        documentation: "The port of the remote device that sent the datagram."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Timeout".to_string(),
                        documentation: "Time, in milliseconds, to wait for an incoming datagram.".to_string(),
                    },
                ],
            }),

            "udpsocketclose" => Some(FunctionSignature {
                name: "UDPSocketClose".to_string(),
                documentation: "Closes an opened UDP socket and frees its associated memory."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "SocketID".to_string(),
                    documentation: "The socket ID returned by UDPSocketOpen.".to_string(),
                }],
            }),

            "emailrelay" => Some(FunctionSignature {
                name: "EmailRelay".to_string(),
                documentation:
                    "Sends an email message to one or more addresses via a Campbell Scientific relay service."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ToAddr".to_string(),
                        documentation: "One or more recipient email addresses, comma-separated.".to_string(),
                    },
                    ParameterInfo {
                        name: "Subject".to_string(),
                        documentation: "The text of the email's Subject field.".to_string(),
                    },
                    ParameterInfo {
                        name: "Message".to_string(),
                        documentation: "The body text of the email.".to_string(),
                    },
                    ParameterInfo {
                        name: "ServerResponse".to_string(),
                        documentation: "Variable that receives the mail server's response messages.".to_string(),
                    },
                    ParameterInfo {
                        name: "Attach".to_string(),
                        documentation: "File names, data table names, or table fields to attach, or an empty string for none.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "Either the time into the interval for unsent records (if Interval > 0) or the number of records to send (if Interval = 0).".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The interval at which to send unsent table data, or 0 to let NumRecsOrTimeIntoInterval control timing.".to_string(),
                    },
                    ParameterInfo {
                        name: "IntervalUnits".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval.".to_string(),
                    },
                    ParameterInfo {
                        name: "FileOption".to_string(),
                        documentation: "The file format used for streamed or attached table data.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for the connection before failing.".to_string(),
                    },
                ],
            }),

            "emailsend" => Some(FunctionSignature {
                name: "EmailSend".to_string(),
                documentation: "Sends an email message directly via an SMTP server.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ServerAddr".to_string(),
                        documentation: "The mail server's IP address or domain name, with an optional port.".to_string(),
                    },
                    ParameterInfo {
                        name: "ToAddr".to_string(),
                        documentation: "One or more recipient email addresses, comma-separated.".to_string(),
                    },
                    ParameterInfo {
                        name: "FromAddr".to_string(),
                        documentation: "The sender's email address.".to_string(),
                    },
                    ParameterInfo {
                        name: "Subject".to_string(),
                        documentation: "The text of the email's Subject field.".to_string(),
                    },
                    ParameterInfo {
                        name: "Message".to_string(),
                        documentation: "The body text of the email.".to_string(),
                    },
                    ParameterInfo {
                        name: "Attach".to_string(),
                        documentation: "File path(s), a data table, or a table field to attach.".to_string(),
                    },
                    ParameterInfo {
                        name: "UserName".to_string(),
                        documentation: "The SMTP authentication username.".to_string(),
                    },
                    ParameterInfo {
                        name: "Password".to_string(),
                        documentation: "The SMTP authentication password.".to_string(),
                    },
                    ParameterInfo {
                        name: "ServerResponse".to_string(),
                        documentation: "Variable that receives the mail server's response messages.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "Either the time into the interval for unsent records (if Interval > 0) or the number of records to send (if Interval = 0).".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The interval at which to send unsent table data, or 0 to let NumRecsOrTimeIntoInterval control timing.".to_string(),
                    },
                    ParameterInfo {
                        name: "IntervalUnits".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval.".to_string(),
                    },
                    ParameterInfo {
                        name: "FileOption".to_string(),
                        documentation: "The file format used for streamed or attached table data (e.g. TOB1, TOA5, CSIXML, CSIJSON).".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for the connection before failing.".to_string(),
                    },
                ],
            }),

            "dialmodem" => Some(FunctionSignature {
                name: "DialModem".to_string(),
                documentation: "Dials a modem device over a communications port and checks for an expected response.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The datalogger communications port to use for dialing.".to_string(),
                    },
                    ParameterInfo {
                        name: "BaudRate".to_string(),
                        documentation: "The transmission speed, in bits per second.".to_string(),
                    },
                    ParameterInfo {
                        name: "DialString".to_string(),
                        documentation: "The telephone number and modem commands to dial.".to_string(),
                    },
                    ParameterInfo {
                        name: "ResponseString".to_string(),
                        documentation: "The modem response expected to indicate a successful connection.".to_string(),
                    },
                ],
            }),

            "dialsequence" => Some(FunctionSignature {
                name: "DialSequence".to_string(),
                documentation: "Declares a PakBus dial-out route/sequence, closed by EndDialSequence.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "PakBusAddr".to_string(),
                    documentation: "The PakBus address (1 through 4094) of the remote device being contacted.".to_string(),
                }],
            }),

            "enddialsequence" => Some(FunctionSignature {
                name: "EndDialSequence".to_string(),
                documentation: "Closes a DialSequence block, reporting whether the dial sequence succeeded.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "DialSuccess".to_string(),
                    documentation: "A variable that reports whether the dial sequence (typically via DialModem) succeeded.".to_string(),
                }],
            }),

            "modemhangup" => Some(FunctionSignature {
                name: "ModemHangup".to_string(),
                documentation: "Declares code to run when a communications port hangs up, closed by EndModemHangup.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "ComPort".to_string(),
                    documentation: "The communications port to monitor for a hangup.".to_string(),
                }],
            }),

            "smsrecv" => Some(FunctionSignature {
                name: "SMSRecv".to_string(),
                documentation: "Polls a CELL2XX cellular modem for a pending SMS message.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "PhoneNumber".to_string(),
                        documentation: "A string variable that receives the sender's phone number.".to_string(),
                    },
                    ParameterInfo {
                        name: "Message".to_string(),
                        documentation: "A string variable that receives the text content of the received SMS.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeStamp".to_string(),
                        documentation: "A string variable that receives the message's timestamp.".to_string(),
                    },
                ],
            }),

            "smssend" => Some(FunctionSignature {
                name: "SMSSend".to_string(),
                documentation: "Sends an SMS message via a CELL2XX cellular modem.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable that receives the outcome status of the send attempt.".to_string(),
                    },
                    ParameterInfo {
                        name: "Swath".to_string(),
                        documentation: "The number of SMS messages to transmit (maximum 60).".to_string(),
                    },
                    ParameterInfo {
                        name: "PhoneNumber".to_string(),
                        documentation: "The recipient's phone number, including country and area codes.".to_string(),
                    },
                    ParameterInfo {
                        name: "Message".to_string(),
                        documentation: "The text content of the SMS message.".to_string(),
                    },
                ],
            }),

            "pppopen" => Some(FunctionSignature {
                name: "PPPOpen".to_string(),
                documentation:
                    "Enables a PPP network connection through an external modem and returns its IP address."
                        .to_string(),
                parameters: vec![ParameterInfo {
                    name: "Option".to_string(),
                    documentation: "Selects which IP address version(s) to return: IPv4, IPv6, or both.".to_string(),
                }],
            }),

            "pppclose" => Some(FunctionSignature {
                name: "PPPClose".to_string(),
                documentation: "Closes an open PPP connection with a server.".to_string(),
                parameters: vec![],
            }),

            "ftpclient" => Some(FunctionSignature {
                name: "FTPClient".to_string(),
                documentation: "Manages files on a server using FTP, FTPS, or SFTP."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "IPAddress".to_string(),
                        documentation: "The IP address or domain name of the FTP server, optionally with a port.".to_string(),
                    },
                    ParameterInfo {
                        name: "User".to_string(),
                        documentation: "The user name for accessing the FTP server."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Password".to_string(),
                        documentation: "The password for accessing the FTP server.".to_string(),
                    },
                    ParameterInfo {
                        name: "LocalFileName".to_string(),
                        documentation: "The local file, or data table for streaming, to send, or the destination for a retrieved file.".to_string(),
                    },
                    ParameterInfo {
                        name: "RemoteFileName".to_string(),
                        documentation: "The path and name of the file on the remote server."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "PutGetOption".to_string(),
                        documentation: "Selects send or retrieve, active or passive mode, and the FTP/FTPS/SFTP protocol.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "Either the time into the interval for unsent records (if Interval > 0) or the number of records to send (if Interval = 0).".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The interval at which to send unsent table data, or 0 to let NumRecsOrTimeIntoInterval control timing.".to_string(),
                    },
                    ParameterInfo {
                        name: "IntervalUnits".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval.".to_string(),
                    },
                    ParameterInfo {
                        name: "FileOption".to_string(),
                        documentation: "The file format used when streaming table data to the server.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for input after connecting.".to_string(),
                    },
                ],
            }),

            "httpget" => Some(FunctionSignature {
                name: "HTTPGet".to_string(),
                documentation: "Sends a GET request to an HTTP server.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "URI".to_string(),
                        documentation: "The URI of the HTTP server to access, optionally with embedded credentials.".to_string(),
                    },
                    ParameterInfo {
                        name: "Response".to_string(),
                        documentation: "The variable or file name in which the response is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Header".to_string(),
                        documentation: "Additional HTTP header information to send, and where returned headers are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for input after connecting.".to_string(),
                    },
                ],
            }),

            "httppost" => Some(FunctionSignature {
                name: "HTTPPost".to_string(),
                documentation: "Sends files or text to a URL via an HTTP POST request."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "URI".to_string(),
                        documentation: "The URI of the HTTP server to access, optionally with embedded credentials.".to_string(),
                    },
                    ParameterInfo {
                        name: "Contents".to_string(),
                        documentation: "The data, file, or data table/field name to send in the request body.".to_string(),
                    },
                    ParameterInfo {
                        name: "Response".to_string(),
                        documentation: "The variable or file name in which the response is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Header".to_string(),
                        documentation: "Additional HTTP header information to send, and where returned headers are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "Either the time into the interval for unsent records (if Interval > 0) or the number of records to send (if Interval = 0).".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The interval at which to send unsent table data, or 0 to let NumRecsOrTimeIntoInterval control timing.".to_string(),
                    },
                    ParameterInfo {
                        name: "IntervalUnits".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval.".to_string(),
                    },
                    ParameterInfo {
                        name: "FileOption".to_string(),
                        documentation: "The file format used when streaming table data in the request.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for input after connecting.".to_string(),
                    },
                ],
            }),

            "httpput" => Some(FunctionSignature {
                name: "HTTPPut".to_string(),
                documentation: "Sends files or text to a URL via an HTTP PUT request."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "URI".to_string(),
                        documentation: "The URI of the HTTP server to access, optionally with embedded credentials.".to_string(),
                    },
                    ParameterInfo {
                        name: "Contents".to_string(),
                        documentation: "The data, file, or data table/field name to send in the request body.".to_string(),
                    },
                    ParameterInfo {
                        name: "Response".to_string(),
                        documentation: "The variable or file name in which the response is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Header".to_string(),
                        documentation: "Additional HTTP header information to send, and where returned headers are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "Either the time into the interval for unsent records (if Interval > 0) or the number of records to send (if Interval = 0).".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The interval at which to send unsent table data, or 0 to let NumRecsOrTimeIntoInterval control timing.".to_string(),
                    },
                    ParameterInfo {
                        name: "IntervalUnits".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval.".to_string(),
                    },
                    ParameterInfo {
                        name: "FileOption".to_string(),
                        documentation: "The file format used when streaming table data in the request.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for input after connecting.".to_string(),
                    },
                ],
            }),

            "gps" => Some(FunctionSignature {
                name: "GPS".to_string(),
                documentation: "Synchronizes the datalogger clock with a GPS receiver and stores its position/timing data.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "GPSArray".to_string(),
                        documentation: "The variable in which to store the fifteen values returned by the GPS (location, speed, course, satellite, and timing data).".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The control port pair to which the GPS device is attached.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOffset".to_string(),
                        documentation: "The local time offset, in seconds, from UTC; use -1 to read coordinates without adjusting the clock.".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxTimeDiff".to_string(),
                        documentation: "The maximum difference, in milliseconds, between the datalogger clock and the GPS clock tolerated before the clock is changed.".to_string(),
                    },
                    ParameterInfo {
                        name: "NMEAStrings".to_string(),
                        documentation: "String array that holds the raw NMEA sentences (GPRMC, GPGGA, and any others).".to_string(),
                    },
                ],
            }),

            "ethernetpower" => Some(FunctionSignature {
                name: "EthernetPower".to_string(),
                documentation: "Turns power to all Ethernet devices on or off.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "State".to_string(),
                    documentation: "A non-zero value turns Ethernet power on; zero turns it off.".to_string(),
                }],
            }),

            "i2copen" => Some(FunctionSignature {
                name: "I2COpen".to_string(),
                documentation: "Configures a port pair for I2C communication at a specified clock rate.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPort".to_string(),
                        documentation: "The starting port for the I2C clock/data signal pair.".to_string(),
                    },
                    ParameterInfo {
                        name: "BitRate".to_string(),
                        documentation: "The I2C clock frequency, in Hertz.".to_string(),
                    },
                ],
            }),

            "i2cread" => Some(FunctionSignature {
                name: "I2CRead".to_string(),
                documentation: "Reads bytes from an I2C peripheral device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPort".to_string(),
                        documentation: "The beginning port used for the I2C clock/data signal pair.".to_string(),
                    },
                    ParameterInfo {
                        name: "Address".to_string(),
                        documentation: "The 7-bit address of the I2C peripheral device (the read bit is appended automatically).".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable in which to store the data read from the I2C device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumBytes".to_string(),
                        documentation: "The number of bytes to read from the I2C device.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Bit field specifying the transaction's start/stop/restart conditions.".to_string(),
                    },
                ],
            }),

            "i2cwrite" => Some(FunctionSignature {
                name: "I2CWrite".to_string(),
                documentation: "Writes bytes to an I2C peripheral device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPort".to_string(),
                        documentation: "The beginning port used for the I2C clock/data signal pair.".to_string(),
                    },
                    ParameterInfo {
                        name: "Address".to_string(),
                        documentation: "The 7-bit address of the I2C peripheral device (the read bit is appended automatically).".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "The variable in which the data to be written to the I2C device is stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumBytes".to_string(),
                        documentation: "The number of bytes to write to the I2C device.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Bit field specifying the transaction's start/stop/restart conditions.".to_string(),
                    },
                ],
            }),

            "spiopen" => Some(FunctionSignature {
                name: "SPIOpen".to_string(),
                documentation: "Configures the datalogger as an SPI controller for communication with peripheral devices.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPort".to_string(),
                        documentation: "The starting port for the SPI clock, COPI, and CIPO signals (C1 or C5).".to_string(),
                    },
                    ParameterInfo {
                        name: "BitRate".to_string(),
                        documentation: "The synchronous clock frequency, in hertz.".to_string(),
                    },
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "Bit field configuring clock phase, polarity, data order, and byte length.".to_string(),
                    },
                ],
            }),

            "spiread" => Some(FunctionSignature {
                name: "SPIRead".to_string(),
                documentation: "Synchronously reads a specified number of bytes from an SPI peripheral device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPort".to_string(),
                        documentation: "The starting port for the SPI clock and data signals (C1 or C5).".to_string(),
                    },
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable in which the bytes read from the SPI device are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumBytes".to_string(),
                        documentation: "The number of bytes to clock from the peripheral device.".to_string(),
                    },
                ],
            }),

            "spiwrite" => Some(FunctionSignature {
                name: "SPIWrite".to_string(),
                documentation: "Synchronously transmits a specified number of bytes to an SPI peripheral device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPort".to_string(),
                        documentation: "The starting port for the SPI clock and data signals (C1 or C5).".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "The variable containing the bytes to transmit.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumBytes".to_string(),
                        documentation: "The number of bytes to send to the peripheral device.".to_string(),
                    },
                ],
            }),

            "acceptdatarecords" => Some(FunctionSignature {
                name: "AcceptDataRecords".to_string(),
                documentation: "Configures the datalogger to receive and store data records pushed from a remote PakBus datalogger.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the remote datalogger allowed to push data records.".to_string(),
                    },
                    ParameterInfo {
                        name: "TableNo".to_string(),
                        documentation: "The table number (position in the remote datalogger's table list) whose records will be pushed.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestTableName".to_string(),
                        documentation: "The name of the local table in which to store the received records.".to_string(),
                    },
                ],
            }),

            "broadcast" => Some(FunctionSignature {
                name: "Broadcast".to_string(),
                documentation: "Sends a broadcast message to all devices on a PakBus network.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port on which to send the broadcast.".to_string(),
                    },
                    ParameterInfo {
                        name: "Message".to_string(),
                        documentation: "The numeric code identifying which broadcast message to send (e.g. beacon, routing-table reset, goodbye, or hello request).".to_string(),
                    },
                ],
            }),

            "clockreport" => Some(FunctionSignature {
                name: "ClockReport".to_string(),
                documentation: "Sends this datalogger's clock value to a specified PakBus device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port on which to send the clock value.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination device to receive the clock value.".to_string(),
                    },
                ],
            }),

            "datagram" => Some(FunctionSignature {
                name: "DataGram".to_string(),
                documentation: "Initializes a SerialServer/DataGram application that tunnels serial traffic through a PakBus network.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port carrying the tunneled serial data.".to_string(),
                    },
                    ParameterInfo {
                        name: "BaudRate".to_string(),
                        documentation: "The baud rate of the tunneled serial data.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestAppID".to_string(),
                        documentation: "The application ID of the DataGram application at the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "SrcAppID".to_string(),
                        documentation: "The application ID of this DataGram application.".to_string(),
                    },
                ],
            }),

            "encryptexempt" => Some(FunctionSignature {
                name: "EncryptExempt".to_string(),
                documentation: "Declares a PakBus address range exempt from PakBus encryption.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginPakBusAddr".to_string(),
                        documentation: "The first PakBus address in the exempt range.".to_string(),
                    },
                    ParameterInfo {
                        name: "EndPakBusAddr".to_string(),
                        documentation: "The last PakBus address in the exempt range.".to_string(),
                    },
                ],
            }),

            "getdatarecord" => Some(FunctionSignature {
                name: "GetDataRecord".to_string(),
                documentation: "Retrieves the most recent record(s) from a table on a remote PakBus datalogger into a local table.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives 0 on success, or a positive timeout/negative error code otherwise.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the remote datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the remote datalogger holding the table.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security".to_string(),
                        documentation: "The security code required by the remote datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "Tries".to_string(),
                        documentation: "The number of attempts to make before giving up.".to_string(),
                    },
                    ParameterInfo {
                        name: "TableNo".to_string(),
                        documentation: "The table number (position in the remote datalogger's table list) to read from.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestTableName".to_string(),
                        documentation: "The name of the local table in which to store the retrieved record(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxRecords".to_string(),
                        documentation: "Optional. The maximum number of records to retrieve; defaults to the single most recent record if omitted.".to_string(),
                    },
                ],
            }),

            "getfile" => Some(FunctionSignature {
                name: "GetFile".to_string(),
                documentation: "Retrieves a file from a remote PakBus datalogger and stores it locally.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives 0 on success, or a positive timeout/negative error code otherwise.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the remote datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the remote datalogger holding the file.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security".to_string(),
                        documentation: "The security code required by the remote datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "LocalFile".to_string(),
                        documentation: "The path at which to store the file on this datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "RemoteFile".to_string(),
                        documentation: "The path of the file on the remote datalogger.".to_string(),
                    },
                ],
            }),

            "getvariables" => Some(FunctionSignature {
                name: "GetVariables".to_string(),
                documentation: "Retrieves one or more variable values from a data table on a remote PakBus device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives 0 on success, or a positive timeout/negative error code otherwise.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the remote device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the remote device holding the table.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security".to_string(),
                        documentation: "The security code required by the remote device.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "TableName".to_string(),
                        documentation: "The name of the remote table containing the field(s) to retrieve.".to_string(),
                    },
                    ParameterInfo {
                        name: "FieldName".to_string(),
                        documentation: "The field name(s) to retrieve; must be a string array when Swath is greater than 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Variable".to_string(),
                        documentation: "The local variable or array in which to store the retrieved value(s).".to_string(),
                    },
                    ParameterInfo {
                        name: "Swath".to_string(),
                        documentation: "The number of contiguous values to retrieve.".to_string(),
                    },
                ],
            }),

            "pakbusclock" => Some(FunctionSignature {
                name: "PakBusClock".to_string(),
                documentation: "Configures the datalogger to accept and synchronize its clock from time broadcasts sent by a specified PakBus device.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "PakBusAddr".to_string(),
                    documentation: "The PakBus address of the device whose clock broadcasts should be accepted.".to_string(),
                }],
            }),

            "route" => Some(FunctionSignature {
                name: "Route".to_string(),
                documentation: "Returns the neighbor address of, or the route to, a PakBus datalogger.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "PakBusAddr".to_string(),
                    documentation: "The PakBus address of the destination device to look up a route for.".to_string(),
                }],
            }),

            "routes" => Some(FunctionSignature {
                name: "Routes".to_string(),
                documentation: "Retrieves the datalogger's list of known dynamic PakBus routes into an array.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Dest".to_string(),
                    documentation: "Array to receive route data, dimensioned to (4 * number of routes) + 1: ComPort, neighbor address, destination address, and expected response time per route, terminated by -1.".to_string(),
                }],
            }),

            "senddata" => Some(FunctionSignature {
                name: "SendData".to_string(),
                documentation: "Sends the most recent record from a data table to a destination PakBus device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataTable".to_string(),
                        documentation: "The name of the table whose most recent record will be sent.".to_string(),
                    },
                    ParameterInfo {
                        name: "TableOption".to_string(),
                        documentation: "Optional. Selects which record to send: 0 or -1 for the most recent, or a specific record number.".to_string(),
                    },
                ],
            }),

            "sendfile" => Some(FunctionSignature {
                name: "SendFile".to_string(),
                documentation: "Sends a file from the datalogger to another PakBus datalogger.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives 0 on success, or a positive timeout/negative error code otherwise.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the destination datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security".to_string(),
                        documentation: "The security code required by the destination datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "LocalFile".to_string(),
                        documentation: "The path of the source file on this datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "RemoteFile".to_string(),
                        documentation: "The path at which to store the file on the destination datalogger.".to_string(),
                    },
                ],
            }),

            "sendgetvariables" => Some(FunctionSignature {
                name: "SendGetVariables".to_string(),
                documentation: "Sends and/or retrieves an array of values to/from the host datalogger during its assigned time slot.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives 0 on success, or a positive timeout/negative error code otherwise.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the host datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the host datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security".to_string(),
                        documentation: "The security code required by the host datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "SendVariable".to_string(),
                        documentation: "The local variable or array whose values will be sent to the host datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "SendSwath".to_string(),
                        documentation: "The number of contiguous values to send.".to_string(),
                    },
                    ParameterInfo {
                        name: "GetVariable".to_string(),
                        documentation: "The local variable or array in which to store the values retrieved from the host datalogger.".to_string(),
                    },
                    ParameterInfo {
                        name: "GetSwath".to_string(),
                        documentation: "The number of contiguous values to retrieve.".to_string(),
                    },
                ],
            }),

            "sendtabledef" => Some(FunctionSignature {
                name: "SendTableDef".to_string(),
                documentation: "Sends a data table's definition to a destination device on the PakBus network.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataTable".to_string(),
                        documentation: "The name of the table whose definition will be sent.".to_string(),
                    },
                ],
            }),

            "sendvariables" => Some(FunctionSignature {
                name: "SendVariables".to_string(),
                documentation: "Sends one or more variable values to a table in a destination PakBus device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "Variable that receives 0 on success, or a positive timeout/negative error code otherwise.".to_string(),
                    },
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor used to route this transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "Security".to_string(),
                        documentation: "The security code required by the destination device.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Maximum time, in 0.01-second units, to wait for a response.".to_string(),
                    },
                    ParameterInfo {
                        name: "TableName".to_string(),
                        documentation: "The name of the destination table (Public/Inlocs or Status) to write to.".to_string(),
                    },
                    ParameterInfo {
                        name: "FieldName".to_string(),
                        documentation: "The field name(s) to write; must be a string array when Swath is greater than 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Variable".to_string(),
                        documentation: "The local variable or array holding the value(s) to send.".to_string(),
                    },
                    ParameterInfo {
                        name: "Swath".to_string(),
                        documentation: "The number of contiguous values to send.".to_string(),
                    },
                ],
            }),

            "staticroute" => Some(FunctionSignature {
                name: "StaticRoute".to_string(),
                documentation: "Defines a fixed route to a PakBus datalogger, for use when dynamic routing is unavailable.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communication port used to reach the neighbor.".to_string(),
                    },
                    ParameterInfo {
                        name: "NeighborAddr".to_string(),
                        documentation: "The PakBus address of the neighbor to route through.".to_string(),
                    },
                    ParameterInfo {
                        name: "PakBusAddr".to_string(),
                        documentation: "The PakBus address of the destination device reached via this route.".to_string(),
                    },
                ],
            }),

            "timeuntiltransmit" => Some(FunctionSignature {
                name: "TimeUntilTransmit".to_string(),
                documentation: "Returns the seconds remaining until the datalogger's assigned communication time slot with its host.".to_string(),
                parameters: vec![],
            }),

            "dnp" => Some(FunctionSignature {
                name: "DNP".to_string(),
                documentation: "Configures a communications port to set up the datalogger as a DNP3 outstation device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ComPort".to_string(),
                        documentation: "The communications port used for DNP3 communication.".to_string(),
                    },
                    ParameterInfo {
                        name: "BaudRate".to_string(),
                        documentation: "The baud rate, in bps, at which data is transmitted; a negative value enables autobaud mode.".to_string(),
                    },
                    ParameterInfo {
                        name: "Confirmation".to_string(),
                        documentation: "Encodes the data-link-layer confirmation mode and timeout in the form XSSS (X = confirmation mode, SSS = timeout in seconds).".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOffset".to_string(),
                        documentation: "Optional. The local time offset, in seconds, from UTC; ignored if the datalogger's UTC Offset setting is enabled.".to_string(),
                    },
                    ParameterInfo {
                        name: "MaxTimeDiff".to_string(),
                        documentation: "Optional. The maximum time difference, in milliseconds, allowed between the datalogger and DNP3 master clocks before resynchronizing; 0 resyncs immediately, -1 disables resynchronization.".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPTLS".to_string(),
                        documentation: "Optional. Enables TLS encryption for the DNP3 connection when set to 1.".to_string(),
                    },
                ],
            }),

            "dnpupdate" => Some(FunctionSignature {
                name: "DNPUpdate".to_string(),
                documentation: "Sets up the datalogger as a DNP3 outstation and determines when it updates its arrays of DNP elements.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "DNPSlaveAddr".to_string(),
                        documentation: "The DNP3 outstation address assigned to this datalogger (valid range 1-65520).".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPMasterAddr".to_string(),
                        documentation: "The DNP3 master/client address this datalogger will respond to (valid range 1-65520).".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeOut".to_string(),
                        documentation: "Optional. Seconds to wait for confirmation of an unsolicited response before retransmitting; 0 disables unsolicited responses.".to_string(),
                    },
                    ParameterInfo {
                        name: "Retries".to_string(),
                        documentation: "Optional. The number of retransmission attempts after the initial attempt fails; 0 retries unsolicited responses (with data) indefinitely until confirmed.".to_string(),
                    },
                    ParameterInfo {
                        name: "ConnectHandle".to_string(),
                        documentation: "Optional. A variable set by TCPOpen identifying the master connection to respond to, as if the request had arrived directly.".to_string(),
                    },
                ],
            }),

            "dnpvariable" => Some(FunctionSignature {
                name: "DNPVariable".to_string(),
                documentation: "Maps a variable or array to a DNP3 object, variation, and class within the datalogger's outstation configuration.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "The public variable or array supplying data to the DNP outstation.".to_string(),
                    },
                    ParameterInfo {
                        name: "Swath".to_string(),
                        documentation: "The number of elements of Source to map.".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPObject".to_string(),
                        documentation: "The DNP3 object type (e.g. binary input, analog input, counter) that Source is mapped to.".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPVariation".to_string(),
                        documentation: "The data format variation within DNPObject's group (e.g. 32-bit analog vs. floating point).".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPClass".to_string(),
                        documentation: "The DNP3 class assigned to this data: 0 for static data, or 1, 2, or 3 for event data.".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPFlag".to_string(),
                        documentation: "The DNP3 data-quality flag for this data; 1 indicates online, 0 indicates offline.".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPEvent".to_string(),
                        documentation: "Optional. An expression that triggers an event when true; the default (0) triggers an event on any value change.".to_string(),
                    },
                    ParameterInfo {
                        name: "DNPNumEvents".to_string(),
                        documentation: "The number of historical events to retain until they're received by the master.".to_string(),
                    },
                ],
            }),

            "argosdata" => Some(FunctionSignature {
                name: "ArgosData".to_string(),
                documentation: "Specifies the data to be transmitted to the Argos satellite.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable that holds the result of the instruction: -1 (True) if the transmission is successful, 0 (False) if it fails.".to_string(),
                    },
                    ParameterInfo {
                        name: "ST20Buffer".to_string(),
                        documentation: "The number of the ST20 buffer to set up; valid entries are 0 through 6 (7 is reserved for the ST20's internal temperature).".to_string(),
                    },
                    ParameterInfo {
                        name: "DataTable".to_string(),
                        documentation: "The datalogger DataTable that holds the data to be sent to the transmitter.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecords".to_string(),
                        documentation: "The number of records from the data table to copy to the buffer of the transmitter.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataFormat".to_string(),
                        documentation: "The format for the values being transmitted: \"FP2\" for two-byte values, or a comma-separated list of bit widths.".to_string(),
                    },
                ],
            }),

            "argosdatarepeat" => Some(FunctionSignature {
                name: "ArgosDataRepeat".to_string(),
                documentation: "Sets the repeat rate for the ArgosData instruction.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable that holds the result of the instruction: -1 (True) success, 0 (False) failure, or 2 if the transmitter is disconnected or has a hardware problem.".to_string(),
                    },
                    ParameterInfo {
                        name: "RepeatRate".to_string(),
                        documentation: "The amount of time, in seconds, between each packet being sent; valid rates are 0 through 255 (negative values select PTT defaults).".to_string(),
                    },
                    ParameterInfo {
                        name: "RepeatCount".to_string(),
                        documentation: "How many times the message will be repeated; valid entries are 0 to 255 (negative values select PTT defaults).".to_string(),
                    },
                    ParameterInfo {
                        name: "BufferArray".to_string(),
                        documentation: "A Boolean variable array setting the transmitter's buffers to true (use) or false (don't use); array indices correspond to buffer numbers minus 1.".to_string(),
                    },
                ],
            }),

            "argoserror" => Some(FunctionSignature {
                name: "ArgosError".to_string(),
                documentation: "Requests and clears the current error message from the Argos transmitter.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "ErrorCodes".to_string(),
                    documentation: "A string variable that holds the returned error message from the transmitter (e.g. Bad Argument, Bad Buffer, Bad Command, No Failsafe Mode, Timeout, Transmit Failure).".to_string(),
                }],
            }),

            "argossetup" => Some(FunctionSignature {
                name: "ArgosSetup".to_string(),
                documentation: "Sets up the datalogger for transmitting data via an Argos satellite.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable that holds the result of the instruction: -1 (True) success, 0 (False) failure, or 2 if the transmitter is disconnected or has a hardware problem.".to_string(),
                    },
                    ParameterInfo {
                        name: "ST20Buffer".to_string(),
                        documentation: "The buffer number to set up; valid entries are 0 through 7.".to_string(),
                    },
                    ParameterInfo {
                        name: "DecimalID".to_string(),
                        documentation: "The decimal ID number assigned to the buffer.".to_string(),
                    },
                    ParameterInfo {
                        name: "HexadecimalID".to_string(),
                        documentation: "The hexadecimal ID number assigned to the buffer.".to_string(),
                    },
                    ParameterInfo {
                        name: "Frequency".to_string(),
                        documentation: "The frequency, in Hz, assigned to the buffer; valid entries are 401630000 to 401656000 in 2000 Hz steps, plus 401676000, 401678000, and 401680000.".to_string(),
                    },
                ],
            }),

            "argostransmit" => Some(FunctionSignature {
                name: "ArgosTransmit".to_string(),
                documentation: "Initiates a single transmission to an Argos satellite when the instruction is executed.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable that holds the result of the instruction: -1 (True) success, 0 (False) failure, or 2 if the transmitter is disconnected or has a hardware problem.".to_string(),
                    },
                    ParameterInfo {
                        name: "ST20Buffer".to_string(),
                        documentation: "The number of the ST20 buffer to set up; valid entries are 0 through 6 (7 is reserved for the ST20's internal temperature).".to_string(),
                    },
                ],
            }),

            "goesdata" => Some(FunctionSignature {
                name: "GOESData".to_string(),
                documentation: "Transmits data from a data table to a GOES satellite transmitter.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable or array that stores the result/status code of the instruction; 0 indicates success.".to_string(),
                    },
                    ParameterInfo {
                        name: "Table".to_string(),
                        documentation: "The name of the data table whose records are to be transmitted.".to_string(),
                    },
                    ParameterInfo {
                        name: "TableOption".to_string(),
                        documentation: "Which records to send: all records since the last execution, the most recent only, or a specific number.".to_string(),
                    },
                    ParameterInfo {
                        name: "BufferControl".to_string(),
                        documentation: "Selects the self-timed or random buffer and its append/overwrite behavior.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataFormat".to_string(),
                        documentation: "The transmission data format (e.g. FP2, ASCII, binary).".to_string(),
                    },
                ],
            }),

            "goesfield" => Some(FunctionSignature {
                name: "GOESField".to_string(),
                documentation: "Declares an output field to include in a GOES transmission; precedes the data-table field instruction it applies to.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "NumVals".to_string(),
                        documentation: "The number of historical time-series values of the field to output; 0 appends new output per the Fields_Scan_Order setting.".to_string(),
                    },
                    ParameterInfo {
                        name: "Decimation".to_string(),
                        documentation: "Controls output frequency of values: 1 outputs every value, 2 outputs every other value, and so on.".to_string(),
                    },
                    ParameterInfo {
                        name: "Precision".to_string(),
                        documentation: "For ASCII formats, the number of decimal places; for binary formats, a power-of-10 multiplier applied before integer conversion.".to_string(),
                    },
                    ParameterInfo {
                        name: "Width".to_string(),
                        documentation: "The number of characters in the output field (maximum 13).".to_string(),
                    },
                    ParameterInfo {
                        name: "SHEF".to_string(),
                        documentation: "A string variable specifying a SHEF PE code for the field; an empty string means no code is specified.".to_string(),
                    },
                ],
            }),

            "goesgps" => Some(FunctionSignature {
                name: "GOESGPS".to_string(),
                documentation: "Retrieves GPS data from a compatible GOES satellite transmitter and stores it in two variable arrays.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "GoesArray1".to_string(),
                        documentation: "A 6-element array holding a result code plus positioning data: time, latitude/longitude, elevation, and magnetic variation.".to_string(),
                    },
                    ParameterInfo {
                        name: "GoesArray2".to_string(),
                        documentation: "A 7-element array holding GMT date/time components: year, month, day, hour, minute, second, and microsecond.".to_string(),
                    },
                ],
            }),

            "goessetup" => Some(FunctionSignature {
                name: "GOESSetup".to_string(),
                documentation: "Configures a GOES satellite transmitter for communication with the satellite.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable or array that stores the result; 0 indicates success.".to_string(),
                    },
                    ParameterInfo {
                        name: "PlatformID".to_string(),
                        documentation: "The 8-digit hexadecimal platform identification number assigned by NESDIS.".to_string(),
                    },
                    ParameterInfo {
                        name: "MsgWindow".to_string(),
                        documentation: "The transmission window duration, in seconds (1-120).".to_string(),
                    },
                    ParameterInfo {
                        name: "STChannel".to_string(),
                        documentation: "The self-timed transmission channel number; 0 disables self-timed transmission.".to_string(),
                    },
                    ParameterInfo {
                        name: "STBaud".to_string(),
                        documentation: "The self-timed transmission baud rate: 100, 300, or 1200.".to_string(),
                    },
                    ParameterInfo {
                        name: "RChannel".to_string(),
                        documentation: "The random transmission channel number, using the same range convention as STChannel.".to_string(),
                    },
                    ParameterInfo {
                        name: "RBaud".to_string(),
                        documentation: "The random transmission baud rate: 100, 300, or 1200.".to_string(),
                    },
                    ParameterInfo {
                        name: "STInterval".to_string(),
                        documentation: "The time between self-timed transmissions, as a \"Days_Hours_Minutes_Seconds\" string.".to_string(),
                    },
                    ParameterInfo {
                        name: "STOffset".to_string(),
                        documentation: "The time after midnight of the first self-timed transmission, as an \"Hours_Minutes_Seconds\" string.".to_string(),
                    },
                    ParameterInfo {
                        name: "RInterval".to_string(),
                        documentation: "The average time between random transmissions, as an \"Hours_Minutes_Seconds\" string.".to_string(),
                    },
                ],
            }),

            "goesstatus" => Some(FunctionSignature {
                name: "GOESStatus".to_string(),
                documentation: "Requests status and diagnostic information from a GOES satellite transmitter.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable or array that receives the requested status/diagnostic data; its required size varies with StatusCommand.".to_string(),
                    },
                    ParameterInfo {
                        name: "StatusCommand".to_string(),
                        documentation: "Selects the type of information requested: 0 Read Time, 1 Status, 2 Last Message Status, 3 Transmit Random Message, 4 Read Error Register, 5 Reset Error Register, 6 Return Transmitter to Online Mode.".to_string(),
                    },
                ],
            }),

            "goestable" => Some(FunctionSignature {
                name: "GOESTable".to_string(),
                documentation: "Formats and outputs a data table's records to a TX325/TX326 GOES satellite transmitter.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Result".to_string(),
                        documentation: "A string variable holding either the formatted output data or a status/error message.".to_string(),
                    },
                    ParameterInfo {
                        name: "Comport".to_string(),
                        documentation: "The communication port used to reach the transmitter.".to_string(),
                    },
                    ParameterInfo {
                        name: "Model".to_string(),
                        documentation: "Selects the transmitter model: 0 No Connection, 2 COM9602, 3 TX325/TX326.".to_string(),
                    },
                    ParameterInfo {
                        name: "BufferControl".to_string(),
                        documentation: "Selects the self-timed buffer (0) vs. random buffer (1), or a variable expression for conditional output.".to_string(),
                    },
                    ParameterInfo {
                        name: "Fields_Scan_Order".to_string(),
                        documentation: "Controls output ordering: False outputs by record/row, True outputs by field/row.".to_string(),
                    },
                    ParameterInfo {
                        name: "Newest_First".to_string(),
                        documentation: "Controls data sequence: False outputs oldest first, True outputs newest first.".to_string(),
                    },
                    ParameterInfo {
                        name: "Format".to_string(),
                        documentation: "Selects the output data format (e.g. FP2, ASCII comma-separated, SHEF).".to_string(),
                    },
                ],
            }),

            "sdmao4" => Some(FunctionSignature {
                name: "SDMAO4".to_string(),
                documentation: "Sets the output voltage on an SDM-AO4 four-channel analog output device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "A variable or array holding the voltage(s), in millivolts, to output.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of output channels to set; values greater than 4 spill onto additional SDM-AO4 devices at sequentially higher addresses.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger).".to_string(),
                    },
                ],
            }),

            "sdmao4a" => Some(FunctionSignature {
                name: "SDMAO4A".to_string(),
                documentation: "Sets the output voltage on an SDM-AO4A four-channel analog output device, the feature-extended replacement for the SDM-AO4.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "A variable or array holding the voltage(s), in millivolts, to output.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAO4ADest".to_string(),
                        documentation: "A variable that receives the status/error code of the instruction: 240 success, 241 signature error, 242 current overload, 243 both.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAO4AStartChan".to_string(),
                        documentation: "The first output channel to set; subsequent channels set by Reps continue from there.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of output channels to set; values greater than 4 spill onto additional SDM-AO4A devices at sequentially higher addresses.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAO4AOption".to_string(),
                        documentation: "The operating mode: 0 power down, 1 5V synchronous, 2 5V sequential, 3 10V synchronous, 4 10V sequential.".to_string(),
                    },
                ],
            }),

            "sdmbeginport" => Some(FunctionSignature {
                name: "SDMBeginPort".to_string(),
                documentation: "Designates an alternate set of datalogger terminals to use as an SDM port; must precede BeginProg and any SerialOpen.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "SDMPort".to_string(),
                    documentation: "The terminal group to use as the Data/Clock/Enable lines of the SDM port, in place of the default C1/C2/C3.".to_string(),
                }],
            }),

            "sdmcan" => Some(FunctionSignature {
                name: "SDMCAN".to_string(),
                documentation: "Configures and operates the SDM-CAN interface between a CAN-bus network and the datalogger.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array that receives the measurement/status results.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "TimeQuanta".to_string(),
                        documentation: "The base timing unit for the CAN bit-rate synchronization segment.".to_string(),
                    },
                    ParameterInfo {
                        name: "TSEG1".to_string(),
                        documentation: "The CAN bit-timing segment covering propagation delay plus phase segment 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "TSEG2".to_string(),
                        documentation: "The CAN bit-timing segment covering phase segment 2, setting the bit-sample point relative to TSEG1.".to_string(),
                    },
                    ParameterInfo {
                        name: "ID".to_string(),
                        documentation: "The CAN frame identifier; a positive value selects a 29-bit extended ID, a negative value an 11-bit standard ID.".to_string(),
                    },
                    ParameterInfo {
                        name: "DataType".to_string(),
                        documentation: "A numeric code (1 through 33) selecting the SDM-CAN operation to perform.".to_string(),
                    },
                    ParameterInfo {
                        name: "StartBit".to_string(),
                        documentation: "The starting bit position within the CAN data frame; valid entries are 1 through 64, negative to count from the left.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumBits".to_string(),
                        documentation: "The number of bits per value; valid entries are 1 through 64, negative to enable interrupt notification on new data.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumVals".to_string(),
                        documentation: "The number of values to transfer per execution.".to_string(),
                    },
                    ParameterInfo {
                        name: "Multiplier".to_string(),
                        documentation: "A multiplier applied to scale the raw value to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw value to engineering units.".to_string(),
                    },
                ],
            }),

            "sdmcd16ac" => Some(FunctionSignature {
                name: "SDMCD16AC".to_string(),
                documentation: "Enables or disables the relay ports of an SDM-CD16AC relay control device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "An array of values enabling (non-zero) or disabling (zero) each relay port.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of SDM-CD16AC devices to control.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The address of the first device; valid entries are 0 through 14, incrementing per device when Reps is greater than 1.".to_string(),
                    },
                ],
            }),

            "sdmcd16mask" => Some(FunctionSignature {
                name: "SDMCD16Mask".to_string(),
                documentation: "Enables or disables specific relay ports of an SDM-CD16AC device via a bit-mask filter.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "A Long variable whose bit pattern determines which ports are enabled or disabled.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMCD16Mask".to_string(),
                        documentation: "A Long value acting as a filter, indicating which specific ports will change state.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger).".to_string(),
                    },
                ],
            }),

            "sdmcvo4" => Some(FunctionSignature {
                name: "SDMCVO4".to_string(),
                documentation: "Controls the SDM-CVO4 four-channel current/voltage output device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CVO4Source".to_string(),
                        documentation: "An array of output values per channel, in millivolts (voltage mode) or microamps (current mode).".to_string(),
                    },
                    ParameterInfo {
                        name: "CVO4Reps".to_string(),
                        documentation: "The number of channels to set; 0 powers off the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14.".to_string(),
                    },
                    ParameterInfo {
                        name: "CVO4Mode".to_string(),
                        documentation: "The output-type override: 0 voltage per jumper, 1 current per jumper, 10 voltage overriding jumper, 11 current overriding jumper.".to_string(),
                    },
                ],
            }),

            "sdmgeneric" => Some(FunctionSignature {
                name: "SDMGeneric".to_string(),
                documentation: "Sends raw commands to an SDM device with no dedicated CRBasic instruction support.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable that receives the bytes returned from the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "CmdByte".to_string(),
                        documentation: "The setup/command byte sent to the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumValuesOut".to_string(),
                        documentation: "The number of values to send to the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "A variable holding the values to send to the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumValuesIn".to_string(),
                        documentation: "The number of values expected back from the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "BytesPerValue".to_string(),
                        documentation: "The byte width per value, typically 1, 2, or 4.".to_string(),
                    },
                    ParameterInfo {
                        name: "BigEndian".to_string(),
                        documentation: "The byte order of the transfer: 0 little-endian, 1 big-endian.".to_string(),
                    },
                    ParameterInfo {
                        name: "DelayByte".to_string(),
                        documentation: "The inter-byte delay, in microseconds; a negative value applies the delay on receive instead.".to_string(),
                    },
                ],
            }),

            "sdmint8" => Some(FunctionSignature {
                name: "SDMINT8".to_string(),
                documentation: "Programs and controls the SDM-INT8 eight-channel interval timer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "An array that receives the results; 1-D per channel, or 2-D when OutputOpt selects capturing all events.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14.".to_string(),
                    },
                    ParameterInfo {
                        name: "Config8_5".to_string(),
                        documentation: "A four-digit code configuring the voltage level and edge-detection settings for channels 8 through 5.".to_string(),
                    },
                    ParameterInfo {
                        name: "Config4_1".to_string(),
                        documentation: "A four-digit code configuring the voltage level and edge-detection settings for channels 4 through 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Funct8_5".to_string(),
                        documentation: "A four-digit code selecting the timing function (period, frequency, counts, etc.) for channels 8 through 5.".to_string(),
                    },
                    ParameterInfo {
                        name: "Funct4_1".to_string(),
                        documentation: "A four-digit code selecting the timing function (period, frequency, counts, etc.) for channels 4 through 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "OutputOpt".to_string(),
                        documentation: "The output-option code selecting the averaging or event-capture mode.".to_string(),
                    },
                    ParameterInfo {
                        name: "CaptureTrig".to_string(),
                        documentation: "A variable that, when true, triggers the return of captured events (used only with the capture-all-events output option).".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                ],
            }),

            "sdmio16" => Some(FunctionSignature {
                name: "SDMIO16".to_string(),
                documentation: "Sets up and operates an SDM-IO16 16-port digital I/O expansion device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "A variable or array holding measurement results (reads) or source values (writes).".to_string(),
                    },
                    ParameterInfo {
                        name: "IO16Status".to_string(),
                        documentation: "A variable that receives the result/error code of the command: 0 success, incrementing on repeated failure.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger).".to_string(),
                    },
                    ParameterInfo {
                        name: "IO16Cmd".to_string(),
                        documentation: "A numeric command code (1 through 104) selecting the read/write operation to perform.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode16_13".to_string(),
                        documentation: "A four-digit mode code configuring ports 16 through 13.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode12_9".to_string(),
                        documentation: "A four-digit mode code configuring ports 12 through 9.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode8_5".to_string(),
                        documentation: "A four-digit mode code configuring ports 8 through 5.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode4_1".to_string(),
                        documentation: "A four-digit mode code configuring ports 4 through 1.".to_string(),
                    },
                    ParameterInfo {
                        name: "Multiplier".to_string(),
                        documentation: "A multiplier applied to scale the raw results to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw results to engineering units.".to_string(),
                    },
                ],
            }),

            "sdmsio4" => Some(FunctionSignature {
                name: "SDMSIO4".to_string(),
                documentation: "Controls and transfers data with a legacy SDM-SIO4 four-port serial I/O device.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "An array, sized Reps by ValuesPerRep, holding data retrieved from (or sourced to, on send) the device.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of sequentially-addressed SDM-SIO4 devices to poll.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The address of the first device; valid entries are 0 through 14.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mode".to_string(),
                        documentation: "The port selector: 1 through 4 for an individual port, 5 for all ports.".to_string(),
                    },
                    ParameterInfo {
                        name: "Command".to_string(),
                        documentation: "An operation code (1 through 2305) selecting the command to send (e.g. send data, set comm parameters).".to_string(),
                    },
                    ParameterInfo {
                        name: "Param1".to_string(),
                        documentation: "The first command-specific parameter; its meaning depends on Command.".to_string(),
                    },
                    ParameterInfo {
                        name: "Param2".to_string(),
                        documentation: "The second command-specific parameter; its meaning depends on Command.".to_string(),
                    },
                    ParameterInfo {
                        name: "ValuesPerRep".to_string(),
                        documentation: "The number of values sent or received per device per execution.".to_string(),
                    },
                    ParameterInfo {
                        name: "Multiplier".to_string(),
                        documentation: "A multiplier applied to scale the raw values to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw values to engineering units.".to_string(),
                    },
                ],
            }),

            "sdmspeed" => Some(FunctionSignature {
                name: "SDMSpeed".to_string(),
                documentation: "Changes the bit period the datalogger uses to clock SDM bus communication, useful for long cable runs.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "BitPeriod".to_string(),
                    documentation: "The desired bit period, in microseconds; default is 26, valid range is 9 to 2000.".to_string(),
                }],
            }),

            "sdmsw8a" => Some(FunctionSignature {
                name: "SDMSW8A".to_string(),
                documentation: "Reads channels from an SDM-SW8A eight-channel switch closure module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The variable or array in which the measurement results are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "Reps".to_string(),
                        documentation: "The number of channels to read.".to_string(),
                    },
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The address of the device (0 through 14).".to_string(),
                    },
                    ParameterInfo {
                        name: "FunctOp".to_string(),
                        documentation: "Determines the result type: 0=state, 1=duty cycle, 2=pulse count, 3=module status.".to_string(),
                    },
                    ParameterInfo {
                        name: "SW8AStartChan".to_string(),
                        documentation: "The first channel to read.".to_string(),
                    },
                    ParameterInfo {
                        name: "Mult".to_string(),
                        documentation: "A multiplier applied to scale the raw measurement to engineering units.".to_string(),
                    },
                    ParameterInfo {
                        name: "Offset".to_string(),
                        documentation: "An offset applied to scale the raw measurement to engineering units.".to_string(),
                    },
                ],
            }),

            "sdmtrigger" => Some(FunctionSignature {
                name: "SDMTrigger".to_string(),
                documentation: "Broadcasts a simultaneous \"measure now\" group trigger to all SDM devices on the bus that support group triggering.".to_string(),
                parameters: vec![],
            }),

            "sdmx50" => Some(FunctionSignature {
                name: "SDMX50".to_string(),
                documentation: "Switches an SDMX50 coaxial multiplexer to a specified channel, independent of the TDR100 instruction.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "SDMAddress".to_string(),
                        documentation: "The SDM bus address of the device; valid entries are 0 through 14 (15 is reserved for SDMTrigger), incrementing per device when multiple multiplexers are used.".to_string(),
                    },
                    ParameterInfo {
                        name: "Channel".to_string(),
                        documentation: "The multiplexer channel to activate; valid entries are 1 through 8.".to_string(),
                    },
                ],
            }),

            "cpiaddmodule" => Some(FunctionSignature {
                name: "CPIAddModule".to_string(),
                documentation: "Statically assigns a CPI-bus address to a GRANITE/CDM/VWIRE module.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CDMType".to_string(),
                        documentation: "The type of CPI module being added (e.g. a specific CDM/GRANITE/VWIRE model).".to_string(),
                    },
                    ParameterInfo {
                        name: "CDMSerialNo".to_string(),
                        documentation: "The serial number of the module, used to distinguish it from other modules of the same type.".to_string(),
                    },
                    ParameterInfo {
                        name: "CDMDeviceName".to_string(),
                        documentation: "A user-assigned name for the module.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The address to statically assign to the module on the CPI bus.".to_string(),
                    },
                ],
            }),

            "cpifilesend" => Some(FunctionSignature {
                name: "CPIFileSend".to_string(),
                documentation: "Sends an OS file to a GRANITE/CDM module over the CPI bus via memory card, USR drive, or USB.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ResultCode".to_string(),
                        documentation: "A variable that stores the transmission response code (0 = success, 1 through 5 indicate various errors).".to_string(),
                    },
                    ParameterInfo {
                        name: "FileSendProgress".to_string(),
                        documentation: "A variable that reports the file-transfer completion percentage, reaching 100 when done.".to_string(),
                    },
                    ParameterInfo {
                        name: "CPIAddress".to_string(),
                        documentation: "The configured CPI address (1 to 120) of the target module; must be a constant, not a variable.".to_string(),
                    },
                    ParameterInfo {
                        name: "OSFlag".to_string(),
                        documentation: "A variable that triggers the OS transfer when set.".to_string(),
                    },
                    ParameterInfo {
                        name: "OSVersion".to_string(),
                        documentation: "A string specifying the storage drive (CRD:, USR:, or USB:) and OS file version to send.".to_string(),
                    },
                ],
            }),

            "cpispeed" => Some(FunctionSignature {
                name: "CPISpeed".to_string(),
                documentation: "Adjusts the CPI bus bit rate, needed when the bus load or cable length requires a slower speed.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Speed".to_string(),
                    documentation: "The CPI bus speed in kbps (e.g. 1000, 500, or 250); lower speeds support longer cable runs or higher bus loads.".to_string(),
                }],
            }),

            "mqttpublishtable" => Some(FunctionSignature {
                name: "MQTTPublishTable".to_string(),
                documentation: "Publishes a data table's contents to an MQTT broker; placed inside a DataTable/EndTable declaration.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "QoS".to_string(),
                        documentation: "The MQTT Quality of Service level: 0 (at most once) or 1 (at least once).".to_string(),
                    },
                    ParameterInfo {
                        name: "NumRecsOrTimeIntoInterval".to_string(),
                        documentation: "Positive: number of records per publish; negative: a sliding window; zero: publish whenever the table is called.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The time duration between publishes when using interval-driven mode.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for Interval and NumRecsOrTimeIntoInterval (Sec, Min, Hr, or Day).".to_string(),
                    },
                    ParameterInfo {
                        name: "OutputFormat".to_string(),
                        documentation: "The published data format: 1 = CSIJSON, 2 = GeoJSON, 3 = BASICJSON.".to_string(),
                    },
                    ParameterInfo {
                        name: "Longitude".to_string(),
                        documentation: "The station's longitude; used only when OutputFormat selects GeoJSON.".to_string(),
                    },
                    ParameterInfo {
                        name: "Latitude".to_string(),
                        documentation: "The station's latitude; used only when OutputFormat selects GeoJSON.".to_string(),
                    },
                    ParameterInfo {
                        name: "Altitude".to_string(),
                        documentation: "The station's altitude above sea level, in meters; used only when OutputFormat selects GeoJSON.".to_string(),
                    },
                ],
            }),

            "mqttpublishconsttable" => Some(FunctionSignature {
                name: "MQTTPublishConstTable".to_string(),
                documentation: "Enables remote editing of ConstTable values via MQTT; placed inside a ConstTable/EndConstTable declaration.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "QoS".to_string(),
                    documentation: "The MQTT Quality of Service level: 0 (at most once) or 1 (at least once).".to_string(),
                }],
            }),

            "abs" => Some(FunctionSignature {
                name: "Abs".to_string(),
                documentation: "Returns the absolute value of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the absolute value of.".to_string(),
                }],
            }),

            "sqr" => Some(FunctionSignature {
                name: "Sqr".to_string(),
                documentation: "Returns the square root of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the square root of.".to_string(),
                }],
            }),

            "sin" => Some(FunctionSignature {
                name: "Sin".to_string(),
                documentation: "Returns the sine of an angle in radians.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Radians".to_string(),
                    documentation: "Angle in radians.".to_string(),
                }],
            }),

            "cos" => Some(FunctionSignature {
                name: "Cos".to_string(),
                documentation: "Returns the cosine of an angle in radians.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Radians".to_string(),
                    documentation: "Angle in radians.".to_string(),
                }],
            }),

            "tan" => Some(FunctionSignature {
                name: "Tan".to_string(),
                documentation: "Returns the tangent of an angle in radians.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Radians".to_string(),
                    documentation: "Angle in radians.".to_string(),
                }],
            }),

            "atn2" => Some(FunctionSignature {
                name: "Atn2".to_string(),
                documentation: "Returns the arc tangent of Y/X in radians.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Y".to_string(),
                        documentation: "Y coordinate.".to_string(),
                    },
                    ParameterInfo {
                        name: "X".to_string(),
                        documentation: "X coordinate.".to_string(),
                    },
                ],
            }),

            "round" => Some(FunctionSignature {
                name: "Round".to_string(),
                documentation: "Rounds a number to specified decimal places.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Value".to_string(),
                        documentation: "The number to round.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumDigits".to_string(),
                        documentation: "Number of decimal places.".to_string(),
                    },
                ],
            }),

            "sgn" => Some(FunctionSignature {
                name: "Sgn".to_string(),
                documentation: "Returns the sign of a number as -1, 0, or 1.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the sign of.".to_string(),
                }],
            }),

            "exp" => Some(FunctionSignature {
                name: "Exp".to_string(),
                documentation: "Returns e raised to a power.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The exponent to raise e to.".to_string(),
                }],
            }),

            "ln" => Some(FunctionSignature {
                name: "Ln".to_string(),
                documentation: "Returns the natural logarithm of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to take the natural logarithm of.".to_string(),
                }],
            }),

            "log" => Some(FunctionSignature {
                name: "Log".to_string(),
                documentation: "Returns the natural logarithm of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to take the natural logarithm of.".to_string(),
                }],
            }),

            "log10" => Some(FunctionSignature {
                name: "Log10".to_string(),
                documentation: "Returns the base-10 logarithm of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to take the base-10 logarithm of.".to_string(),
                }],
            }),

            "sinh" => Some(FunctionSignature {
                name: "Sinh".to_string(),
                documentation: "Returns the hyperbolic sine of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the hyperbolic sine of.".to_string(),
                }],
            }),

            "cosh" => Some(FunctionSignature {
                name: "Cosh".to_string(),
                documentation: "Returns the hyperbolic cosine of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the hyperbolic cosine of.".to_string(),
                }],
            }),

            "tanh" => Some(FunctionSignature {
                name: "Tanh".to_string(),
                documentation: "Returns the hyperbolic tangent of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the hyperbolic tangent of.".to_string(),
                }],
            }),

            "asin" => Some(FunctionSignature {
                name: "Asin".to_string(),
                documentation: "Returns the arc sine of a number, in radians.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the arc sine of.".to_string(),
                }],
            }),

            "acos" => Some(FunctionSignature {
                name: "Acos".to_string(),
                documentation: "Returns the arc cosine of a number, in radians.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the arc cosine of.".to_string(),
                }],
            }),

            "atn" => Some(FunctionSignature {
                name: "Atn".to_string(),
                documentation: "Returns the arc tangent of a number, in radians.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the arc tangent of.".to_string(),
                }],
            }),

            "int" => Some(FunctionSignature {
                name: "Int".to_string(),
                documentation: "Returns the integer part of a number, truncating toward negative infinity.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to truncate.".to_string(),
                }],
            }),

            "fix" => Some(FunctionSignature {
                name: "Fix".to_string(),
                documentation: "Returns the integer part of a number, truncating toward zero.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to truncate.".to_string(),
                }],
            }),

            "frac" => Some(FunctionSignature {
                name: "Frac".to_string(),
                documentation: "Returns the fractional portion of a number.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to get the fractional portion of.".to_string(),
                }],
            }),

            "rnd" => Some(FunctionSignature {
                name: "Rnd".to_string(),
                documentation: "Returns a random value between 0 (inclusive) and 1 (exclusive). Takes no parentheses.".to_string(),
                parameters: vec![],
            }),

            "randomize" => Some(FunctionSignature {
                name: "Randomize".to_string(),
                documentation: "Initializes the random-number generator used by Rnd with a new seed value.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Number".to_string(),
                    documentation: "Numeric expression used as the new seed value.".to_string(),
                }],
            }),

            "ceiling" => Some(FunctionSignature {
                name: "Ceiling".to_string(),
                documentation: "Rounds a number up to the nearest integer.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to round up.".to_string(),
                }],
            }),

            "floor" => Some(FunctionSignature {
                name: "Floor".to_string(),
                documentation: "Rounds a number down to the nearest integer.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "The number to round down.".to_string(),
                }],
            }),

            "len" => Some(FunctionSignature {
                name: "Len".to_string(),
                documentation: "Returns the length of a string.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "String".to_string(),
                    documentation: "The string to measure.".to_string(),
                }],
            }),

            "mid" => Some(FunctionSignature {
                name: "Mid".to_string(),
                documentation: "Extracts a substring from a string.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "String".to_string(),
                        documentation: "Source string.".to_string(),
                    },
                    ParameterInfo {
                        name: "Start".to_string(),
                        documentation: "Starting position (1-based).".to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "Number of characters to extract.".to_string(),
                    },
                ],
            }),

            "left" => Some(FunctionSignature {
                name: "Left".to_string(),
                documentation: "Returns leftmost characters of a string.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "String".to_string(),
                        documentation: "Source string.".to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "Number of characters to return.".to_string(),
                    },
                ],
            }),

            "right" => Some(FunctionSignature {
                name: "Right".to_string(),
                documentation: "Returns rightmost characters of a string.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "String".to_string(),
                        documentation: "Source string.".to_string(),
                    },
                    ParameterInfo {
                        name: "Length".to_string(),
                        documentation: "Number of characters to return.".to_string(),
                    },
                ],
            }),

            "instr" => Some(FunctionSignature {
                name: "InStr".to_string(),
                documentation: "Finds a substring within a string.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Start".to_string(),
                        documentation: "Starting position for search.".to_string(),
                    },
                    ParameterInfo {
                        name: "SearchString".to_string(),
                        documentation: "String to search in.".to_string(),
                    },
                    ParameterInfo {
                        name: "FilterString".to_string(),
                        documentation: "String to search for.".to_string(),
                    },
                    ParameterInfo {
                        name: "SearchOption".to_string(),
                        documentation: "Method-of-search code (0-10; add 100 to strip quotes), not a boolean case-sensitivity flag.".to_string(),
                    },
                ],
            }),

            "splitstr" => Some(FunctionSignature {
                name: "SplitStr".to_string(),
                documentation: "Splits a string by delimiter into an array.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Result".to_string(),
                        documentation: "Array to store results.".to_string(),
                    },
                    ParameterInfo {
                        name: "SearchString".to_string(),
                        documentation: "String to split.".to_string(),
                    },
                    ParameterInfo {
                        name: "FilterString".to_string(),
                        documentation: "Filter for the string(s) to return; its role (delimiter set, exact-match string, or header/footer filter) depends on SplitOption.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumSplits".to_string(),
                        documentation: "Maximum number of splits.".to_string(),
                    },
                    ParameterInfo {
                        name: "SplitOption".to_string(),
                        documentation: "Split option flags.".to_string(),
                    },
                ],
            }),

            "formatfloat" => Some(FunctionSignature {
                name: "FormatFloat".to_string(),
                documentation:
                    "Formats a floating-point value as a string using a printf-style format specifier."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Value".to_string(),
                        documentation: "The floating-point value to convert to a string."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "FormatString".to_string(),
                        documentation: "A printf-style format specifier controlling width, precision, and numeric type.".to_string(),
                    },
                ],
            }),

            "formatlong" => Some(FunctionSignature {
                name: "FormatLong".to_string(),
                documentation:
                    "Converts a Long value to a decimal, hexadecimal, octal, or binary string."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "LongVar".to_string(),
                        documentation: "The Long value to format.".to_string(),
                    },
                    ParameterInfo {
                        name: "FormatString".to_string(),
                        documentation: "A format code selecting decimal, hex, octal, or binary output and field width.".to_string(),
                    },
                ],
            }),

            "lowercase" => Some(FunctionSignature {
                name: "LowerCase".to_string(),
                documentation: "Converts a string to all lowercase characters.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "String".to_string(),
                    documentation: "The string to convert to lowercase.".to_string(),
                }],
            }),

            "uppercase" => Some(FunctionSignature {
                name: "UpperCase".to_string(),
                documentation: "Converts a string to all uppercase characters.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "String".to_string(),
                    documentation: "The string to convert to uppercase.".to_string(),
                }],
            }),

            "trim" => Some(FunctionSignature {
                name: "Trim".to_string(),
                documentation:
                    "Returns a copy of a string with leading and trailing spaces removed."
                        .to_string(),
                parameters: vec![ParameterInfo {
                    name: "String".to_string(),
                    documentation: "The string to strip of leading and trailing spaces."
                        .to_string(),
                }],
            }),

            "rtrim" => Some(FunctionSignature {
                name: "RTrim".to_string(),
                documentation: "Returns a copy of a string with trailing spaces removed."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "String".to_string(),
                    documentation: "The string to strip of trailing spaces.".to_string(),
                }],
            }),

            "ltrim" => Some(FunctionSignature {
                name: "LTrim".to_string(),
                documentation: "Returns a copy of a string with leading spaces removed."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "String".to_string(),
                    documentation: "The string to strip of leading spaces.".to_string(),
                }],
            }),

            "replace" => Some(FunctionSignature {
                name: "Replace".to_string(),
                documentation:
                    "Searches a string for a substring and replaces all occurrences with another string."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "String".to_string(),
                        documentation: "The string to search.".to_string(),
                    },
                    ParameterInfo {
                        name: "Find".to_string(),
                        documentation: "The substring to search for and replace.".to_string(),
                    },
                    ParameterInfo {
                        name: "ReplaceWith".to_string(),
                        documentation: "The string used to replace each occurrence of Find."
                            .to_string(),
                    },
                ],
            }),

            "chr" => Some(FunctionSignature {
                name: "Chr".to_string(),
                documentation: "Returns a character in the extended ASCII character set."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "Code".to_string(),
                    documentation: "The extended ASCII code, 0 to 255, of the character to return.".to_string(),
                }],
            }),

            "ascii" => Some(FunctionSignature {
                name: "ASCII".to_string(),
                documentation: "Returns the ASCII value of a character in a string.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "ASCIIString".to_string(),
                    documentation: "The string, indexed as ASCIIString(1,1,X), where X selects the character position to evaluate.".to_string(),
                }],
            }),

            "strcomp" => Some(FunctionSignature {
                name: "StrComp".to_string(),
                documentation:
                    "Compares two strings to determine if they are identical or their relative sort order."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "String1".to_string(),
                        documentation: "The first string to compare.".to_string(),
                    },
                    ParameterInfo {
                        name: "String2".to_string(),
                        documentation: "The second string to compare against String1."
                            .to_string(),
                    },
                ],
            }),

            "checksum" => Some(FunctionSignature {
                name: "CheckSum".to_string(),
                documentation: "Returns a checksum signature for the characters in a string."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "ChkSumString".to_string(),
                        documentation: "The string, or a file path, whose bytes are used to compute the checksum.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChkSumType".to_string(),
                        documentation: "The checksum algorithm to use, such as CRC16, CRC32, MD5, or SHA1.".to_string(),
                    },
                    ParameterInfo {
                        name: "CheckSumSize".to_string(),
                        documentation: "The number of bytes to include in the checksum, or 0 to use the string's length.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChkSumOption1".to_string(),
                        documentation: "Additional option required by cryptographic checksum types 25-29, typically a destination array.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChkSumOption2".to_string(),
                        documentation: "Additional option for HMAC checksum types 27-28, specifying the cryptographic key.".to_string(),
                    },
                    ParameterInfo {
                        name: "ChkSumOption3".to_string(),
                        documentation: "Additional option for HMAC checksum types 27-28, specifying the key length, or 0 to auto-detect.".to_string(),
                    },
                ],
            }),

            "hextodec" => Some(FunctionSignature {
                name: "HexToDec".to_string(),
                documentation: "Converts a hexadecimal string to its float or integer decimal value.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Expression".to_string(),
                    documentation: "The hexadecimal string to convert to a decimal value."
                        .to_string(),
                }],
            }),

            "hex" => Some(FunctionSignature {
                name: "Hex".to_string(),
                documentation: "Returns a hexadecimal string representation of a Long value."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "Expression".to_string(),
                    documentation: "The value, converted to Long, to represent in hexadecimal."
                        .to_string(),
                }],
            }),

            "sprintf" => Some(FunctionSignature {
                name: "Sprintf".to_string(),
                documentation:
                    "Writes a formatted output string, built from up to ten arguments, to a destination variable."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Dest".to_string(),
                        documentation: "The string variable that receives the formatted output."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Format".to_string(),
                        documentation: "A printf-style format string with up to ten format specifiers.".to_string(),
                    },
                    ParameterInfo {
                        name: "Argument1".to_string(),
                        documentation: "The value formatted by the first format specifier in Format.".to_string(),
                    },
                ],
            }),

            "timer" => Some(FunctionSignature {
                name: "Timer".to_string(),
                documentation: "Returns elapsed time from a timer.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "TimerNumber".to_string(),
                        documentation: "Timer number (1-4).".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for result.".to_string(),
                    },
                    ParameterInfo {
                        name: "TimerOption".to_string(),
                        documentation: "0=read, 1=read and reset.".to_string(),
                    },
                ],
            }),

            "timeintointerval" => Some(FunctionSignature {
                name: "TimeIntoInterval".to_string(),
                documentation: "Returns true when the interval boundary is crossed.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "TintoInt".to_string(),
                        documentation: "Time into interval to trigger.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "Time interval to check.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units (Sec, Min, Hr, etc.).".to_string(),
                    },
                ],
            }),

            "iftime" => Some(FunctionSignature {
                name: "IfTime".to_string(),
                documentation: "Returns true at specified time intervals.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "TintoInt".to_string(),
                        documentation: "Time into interval to trigger.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "Interval length.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units.".to_string(),
                    },
                ],
            }),

            "delay" => Some(FunctionSignature {
                name: "Delay".to_string(),
                documentation: "Pauses execution for a specified time.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Option".to_string(),
                        documentation: "0=measurement task sequence, 1=processing, 2=digital/SDM measurements.".to_string(),
                    },
                    ParameterInfo {
                        name: "Duration".to_string(),
                        documentation: "Length of delay.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units (uSec, mSec, Sec).".to_string(),
                    },
                ],
            }),

            "realtime" => Some(FunctionSignature {
                name: "RealTime".to_string(),
                documentation:
                    "Extracts the datalogger's current real-time clock values into a 9-element destination array."
                        .to_string(),
                parameters: vec![ParameterInfo {
                    name: "Dest".to_string(),
                    documentation: "Array (dimensioned to 9) that receives year, month, day, hour, minute, second, microsecond, day of week, and day of year.".to_string(),
                }],
            }),

            "setstatus" => Some(FunctionSignature {
                name: "SetStatus".to_string(),
                documentation: "Changes the value of a field in the datalogger's Status table."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FieldName".to_string(),
                        documentation: "Name of the Status table field to change, enclosed in quotes.".to_string(),
                    },
                    ParameterInfo {
                        name: "Value".to_string(),
                        documentation: "New value for the field; quoted for strings, unquoted for numeric values.".to_string(),
                    },
                ],
            }),

            "setsetting" => Some(FunctionSignature {
                name: "SetSetting".to_string(),
                documentation: "Changes the value of a field in the datalogger's Settings table."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "FieldName".to_string(),
                        documentation: "Name of the Settings table field to change, enclosed in quotes.".to_string(),
                    },
                    ParameterInfo {
                        name: "Value".to_string(),
                        documentation: "New value for the field; quoted for strings, unquoted for numeric values.".to_string(),
                    },
                ],
            }),

            "movebytes" => Some(FunctionSignature {
                name: "MoveBytes".to_string(),
                documentation:
                    "Moves binary bytes of data from one memory location to another, with optional byte-order swapping."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Destination".to_string(),
                        documentation: "Variable in which the moved bytes are stored.".to_string(),
                    },
                    ParameterInfo {
                        name: "DestOffset".to_string(),
                        documentation: "Zero-based byte offset into Destination where data is written.".to_string(),
                    },
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable holding the binary data to copy; left unchanged by the move.".to_string(),
                    },
                    ParameterInfo {
                        name: "SourceOffset".to_string(),
                        documentation: "Zero-based byte offset into Source from which data is read.".to_string(),
                    },
                    ParameterInfo {
                        name: "NumBytes".to_string(),
                        documentation: "Number of bytes to copy into Destination.".to_string(),
                    },
                    ParameterInfo {
                        name: "Transfer".to_string(),
                        documentation: "Optional byte-swap mode (0-4) for handling different device byte ordering.".to_string(),
                    },
                ],
            }),

            "arraylength" => Some(FunctionSignature {
                name: "ArrayLength".to_string(),
                documentation:
                    "Returns the total number of elements across all dimensions of an array."
                        .to_string(),
                parameters: vec![ParameterInfo {
                    name: "ArrayLenVar".to_string(),
                    documentation: "Array variable for which to return the total element count."
                        .to_string(),
                }],
            }),

            "nan" => Some(FunctionSignature {
                name: "NaN".to_string(),
                documentation:
                    "Represents the IEEE-754 Not-a-Number value used to flag an invalid measurement or processing error. Takes no parentheses."
                        .to_string(),
                parameters: vec![],
            }),

            "secssince1990" => Some(FunctionSignature {
                name: "SecsSince1990".to_string(),
                documentation: "Converts between a date/time string and the number of seconds since January 1, 1990.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Source".to_string(),
                        documentation: "Variable formatted as a String (date/time) or Long (seconds since 1990) to convert.".to_string(),
                    },
                    ParameterInfo {
                        name: "DateOption".to_string(),
                        documentation: "Constant specifying the date/time string format used by Source or returned by the function.".to_string(),
                    },
                ],
            }),

            "timeisbetween" => Some(FunctionSignature {
                name: "TimeIsBetween".to_string(),
                documentation: "Returns true when the datalogger's real-time clock falls within a specified time range.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "BeginTime".to_string(),
                        documentation: "Start of the time range; included in the range that returns true.".to_string(),
                    },
                    ParameterInfo {
                        name: "EndTime".to_string(),
                        documentation: "End of the time range; not included in the range that returns true.".to_string(),
                    },
                    ParameterInfo {
                        name: "Interval".to_string(),
                        documentation: "The repeating interval that BeginTime and EndTime are measured within.".to_string(),
                    },
                    ParameterInfo {
                        name: "Units".to_string(),
                        documentation: "Time units for BeginTime, EndTime, and Interval.".to_string(),
                    },
                ],
            }),

            "daylightsaving" => Some(FunctionSignature {
                name: "DaylightSaving".to_string(),
                documentation: "Detects a custom-rule daylight saving transition and returns the clock adjustment.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "DSTSet".to_string(),
                        documentation: "Enables (-1) or disables (0) automatic clock adjustment.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTnStart".to_string(),
                        documentation: "Which occurrence (1st through Last) of DSTDayStart begins DST.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTDayStart".to_string(),
                        documentation: "The weekday on which DST begins.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTMonthStart".to_string(),
                        documentation: "The month in which DST begins.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTnEnd".to_string(),
                        documentation: "Which occurrence (1st through Last) of DSTDayEnd ends DST.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTDayEnd".to_string(),
                        documentation: "The weekday on which DST ends.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTMonthEnd".to_string(),
                        documentation: "The month in which DST ends.".to_string(),
                    },
                    ParameterInfo {
                        name: "DSTHour".to_string(),
                        documentation: "The hour (0 through 24) at which the transition occurs.".to_string(),
                    },
                ],
            }),

            "daylightsavingus" => Some(FunctionSignature {
                name: "DaylightSavingUS".to_string(),
                documentation: "Detects a US-rule daylight saving transition and returns the clock adjustment.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "DSTSet".to_string(),
                    documentation: "Enables (-1) or disables (0) automatic clock adjustment.".to_string(),
                }],
            }),

            "instructiontimes" => Some(FunctionSignature {
                name: "InstructionTimes".to_string(),
                documentation: "Populates an array with the processing time, in microseconds, of every program line; must precede BeginProg.".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Dest".to_string(),
                    documentation: "A Long-typed array, sized to the total program line count, that receives the per-line processing times.".to_string(),
                }],
            }),

            "linenum" => Some(FunctionSignature {
                name: "LineNum".to_string(),
                documentation: "Returns the current program line number, for debugging.".to_string(),
                parameters: vec![],
            }),

            "signature" => Some(FunctionSignature {
                name: "Signature".to_string(),
                documentation: "Returns a pseudo-random signature of the program code between two Signature markers.".to_string(),
                parameters: vec![],
            }),

            "displaymenu" => Some(FunctionSignature {
                name: "DisplayMenu".to_string(),
                documentation: "Marks the beginning of a custom on-screen menu definition."
                    .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "MenuName".to_string(),
                        documentation: "Menu label shown on the datalogger display (max 20 characters).".to_string(),
                    },
                    ParameterInfo {
                        name: "AddToSystem".to_string(),
                        documentation: "Constant controlling how the menu appears relative to the system menu at power-up.".to_string(),
                    },
                    ParameterInfo {
                        name: "Cursor".to_string(),
                        documentation: "Line number (1-7) where the cursor starts when the menu is entered.".to_string(),
                    },
                ],
            }),

            "submenu" => Some(FunctionSignature {
                name: "SubMenu".to_string(),
                documentation:
                    "Marks the beginning of a nested custom menu within a DisplayMenu block."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "MenuName".to_string(),
                        documentation: "Submenu label shown on the datalogger display (max 20 characters).".to_string(),
                    },
                    ParameterInfo {
                        name: "Cursor".to_string(),
                        documentation: "Line number (1-7) where the cursor starts when the submenu is entered.".to_string(),
                    },
                ],
            }),

            "menuitem" => Some(FunctionSignature {
                name: "MenuItem".to_string(),
                documentation:
                    "Defines an editable custom-menu entry showing the name and value of a variable."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "MenuItemName".to_string(),
                        documentation: "Label shown on the custom menu for this editable entry.".to_string(),
                    },
                    ParameterInfo {
                        name: "Variable".to_string(),
                        documentation: "Program variable whose value is displayed and can be edited.".to_string(),
                    },
                ],
            }),

            "menupick" => Some(FunctionSignature {
                name: "MenuPick".to_string(),
                documentation:
                    "Creates a fixed pick-list of selectable values for the preceding MenuItem."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Item1".to_string(),
                        documentation: "First selectable constant value in the pick list.".to_string(),
                    },
                    ParameterInfo {
                        name: "Item2".to_string(),
                        documentation: "Second selectable constant value in the pick list (more items may follow).".to_string(),
                    },
                ],
            }),

            "menurecompile" => Some(FunctionSignature {
                name: "MenuRecompile".to_string(),
                documentation:
                    "Creates a custom menu item that triggers a program recompile after Constant Table edits."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "CompileString".to_string(),
                        documentation: "Label for the recompile menu item (max 11 characters, 21 as pick-list header).".to_string(),
                    },
                    ParameterInfo {
                        name: "CompileVar".to_string(),
                        documentation: "Boolean variable set to Yes/No to trigger the recompile.".to_string(),
                    },
                ],
            }),

            "displayvalue" => Some(FunctionSignature {
                name: "DisplayValue".to_string(),
                documentation:
                    "Defines a read-only custom-menu entry showing a data-table field, variable, or expression."
                        .to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "MenuItemName".to_string(),
                        documentation: "Label shown on the custom menu for this read-only entry.".to_string(),
                    },
                    ParameterInfo {
                        name: "MenuExpression".to_string(),
                        documentation: "Data-table field (Table.Field), variable, or expression to display.".to_string(),
                    },
                ],
            }),

            "displayline" => Some(FunctionSignature {
                name: "DisplayLine".to_string(),
                documentation: "Displays a single line of read-only text in a custom menu."
                    .to_string(),
                parameters: vec![ParameterInfo {
                    name: "Value".to_string(),
                    documentation: "String, variable, constant, or expression to display as the line's text.".to_string(),
                }],
            }),

            "datatable" => Some(FunctionSignature {
                name: "DataTable".to_string(),
                documentation: "Defines a data table for storing measurements.".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "TableName".to_string(),
                        documentation: "Name of the data table.".to_string(),
                    },
                    ParameterInfo {
                        name: "TriggerCondition".to_string(),
                        documentation: "Condition that triggers output (True/-1 = every scan)."
                            .to_string(),
                    },
                    ParameterInfo {
                        name: "Size".to_string(),
                        documentation: "Number of records to store (-1 = auto).".to_string(),
                    },
                ],
            }),

            _ => None,
        }
    }

    /// Converts internal signature to LSP SignatureInformation
    fn to_signature_information(sig: &FunctionSignature) -> SignatureInformation {
        let param_names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
        let label = format!("{}({})", sig.name, param_names.join(", "));

        let parameters: Vec<ParameterInformation> = sig
            .parameters
            .iter()
            .map(|p| ParameterInformation {
                label: ParameterLabel::Simple(p.name.clone()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: p.documentation.clone(),
                })),
            })
            .collect();

        SignatureInformation {
            label,
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: sig.documentation.clone(),
            })),
            parameters: Some(parameters),
            active_parameter: None,
        }
    }

    /// Counts the number of commas before the cursor position to determine active parameter
    pub fn count_parameters_before_cursor(text: &str, cursor_offset: usize) -> u32 {
        let text_before_cursor = &text[..cursor_offset.min(text.len())];

        let mut paren_depth = 0;
        let mut comma_count = 0u32;
        let mut in_string = false;
        let mut found_open_paren = false;

        for ch in text_before_cursor.chars().rev() {
            match ch {
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                ')' if !in_string => paren_depth += 1,
                '(' if !in_string => {
                    if paren_depth == 0 {
                        found_open_paren = true;
                        break;
                    }
                    paren_depth -= 1;
                }
                ',' if !in_string && paren_depth == 0 => comma_count += 1,
                _ => {}
            }
        }

        if found_open_paren { comma_count } else { 0 }
    }

    /// Extracts the function name from text at cursor position
    pub fn extract_function_name(text: &str, cursor_offset: usize) -> Option<String> {
        let text_before_cursor = &text[..cursor_offset.min(text.len())];

        let mut paren_depth = 0;
        let mut in_string = false;
        let mut paren_pos = None;

        for (i, ch) in text_before_cursor.chars().rev().enumerate() {
            match ch {
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                ')' if !in_string => paren_depth += 1,
                '(' if !in_string => {
                    if paren_depth == 0 {
                        paren_pos = Some(text_before_cursor.len() - i - 1);
                        break;
                    }
                    paren_depth -= 1;
                }
                _ => {}
            }
        }

        let paren_pos = paren_pos?;

        let before_paren = &text_before_cursor[..paren_pos];
        let name_start = before_paren
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let name = before_paren[name_start..].trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod signature_lookup {
        use super::*;

        #[test]
        fn returns_signature_for_known_function() {
            let help = SignatureProvider::get_signature_help("Scan", 0);

            assert!(help.is_some());
            let help = help.expect("help should be Some");
            assert_eq!(help.signatures.len(), 1);
            assert!(help.signatures[0].label.contains("Scan"));
        }

        #[test]
        fn returns_none_for_unknown_function() {
            let help = SignatureProvider::get_signature_help("UnknownFunc", 0);

            assert!(help.is_none());
        }

        #[test]
        fn returns_none_for_calltable_since_it_takes_no_parentheses() {
            // CallTable is a bare keyword (`CallTable TableName`), not a
            // parenthesized call -- see the parser's `parse_calltable_statement`.
            assert!(SignatureProvider::get_function_signature("CallTable").is_none());
        }

        #[test]
        fn recognized_function_names_resolve_to_a_canonical_spelling() {
            // Documented aliases (e.g. Log/Ln for the natural logarithm) are
            // real in CRBasic, so this doesn't require an exact match to the
            // queried name -- only that whatever name comes back is itself a
            // real, correctly-cased entry, catching typos/casing drift like
            // "VoltSE" vs "VoltSe" without breaking intentional aliasing.
            let unknown: Vec<String> = crbasic_parser::BUILTIN_FUNCTIONS
                .iter()
                .filter_map(|(name, _)| {
                    let signature = SignatureProvider::get_function_signature(name)?;
                    let is_known = crbasic_parser::BUILTIN_FUNCTIONS
                        .iter()
                        .any(|(known, _)| *known == signature.name);
                    (!is_known).then(|| format!("{} -> {}", name, signature.name))
                })
                .collect();

            assert!(
                unknown.is_empty(),
                "Signature resolved to a name not in BUILTIN_FUNCTIONS: {:?}",
                unknown
            );
        }

        #[test]
        fn every_canonical_builtin_function_has_a_signature() {
            let missing: Vec<&str> = crbasic_parser::BUILTIN_FUNCTIONS
                .iter()
                .map(|(name, _)| *name)
                .filter(|name| SignatureProvider::get_function_signature(name).is_none())
                .collect();

            assert!(
                missing.is_empty(),
                "BUILTIN_FUNCTIONS entries missing a signature: {:?}",
                missing
            );
        }

        #[test]
        fn is_case_insensitive() {
            let help_lower = SignatureProvider::get_signature_help("scan", 0);
            let help_upper = SignatureProvider::get_signature_help("SCAN", 0);
            let help_mixed = SignatureProvider::get_signature_help("ScAn", 0);

            assert!(help_lower.is_some());
            assert!(help_upper.is_some());
            assert!(help_mixed.is_some());
        }

        #[test]
        fn sets_active_parameter() {
            let help = SignatureProvider::get_signature_help("Scan", 2);

            assert!(help.is_some());
            let help = help.expect("help should be Some");
            assert_eq!(help.active_parameter, Some(2));
        }
    }

    mod signature_content {
        use super::*;

        #[test]
        fn scan_has_four_parameters() {
            let sig = SignatureProvider::get_function_signature("Scan");

            assert!(sig.is_some());
            let sig = sig.expect("sig should be Some");
            assert_eq!(sig.parameters.len(), 4);
        }

        #[test]
        fn signature_includes_parameter_names() {
            let sig = SignatureProvider::get_function_signature("Scan");

            assert!(sig.is_some());
            let sig = sig.expect("sig should be Some");
            let param_names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert!(param_names.contains(&"Interval"));
            assert!(param_names.contains(&"Units"));
            assert!(param_names.contains(&"BufferOption"));
            assert!(param_names.contains(&"Count"));
        }

        #[test]
        fn signature_includes_documentation() {
            let sig = SignatureProvider::get_function_signature("Scan");

            assert!(sig.is_some());
            let sig = sig.expect("sig should be Some");
            assert!(!sig.documentation.is_empty());
        }

        #[test]
        fn parameters_have_documentation() {
            let sig = SignatureProvider::get_function_signature("Scan");

            assert!(sig.is_some());
            let sig = sig.expect("sig should be Some");

            for param in &sig.parameters {
                assert!(!param.documentation.is_empty());
            }
        }
    }

    mod function_categories {
        use super::*;

        #[test]
        fn has_data_table_functions() {
            assert!(SignatureProvider::get_function_signature("Sample").is_some());
            assert!(SignatureProvider::get_function_signature("Average").is_some());
        }

        #[test]
        fn has_subscan_and_iif() {
            assert!(SignatureProvider::get_function_signature("SubScan").is_some());
            assert!(SignatureProvider::get_function_signature("IIf").is_some());
        }

        #[test]
        fn has_remaining_data_functions() {
            for name in [
                "StdDev",
                "Totalize",
                "Histogram",
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
            ] {
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for data function: {}",
                    name
                );
            }
        }

        #[test]
        fn has_measurement_functions() {
            assert!(SignatureProvider::get_function_signature("PulseCount").is_some());
            assert!(SignatureProvider::get_function_signature("VoltSe").is_some());
            assert!(SignatureProvider::get_function_signature("VoltDiff").is_some());
            assert!(SignatureProvider::get_function_signature("TCDiff").is_some());
            assert!(SignatureProvider::get_function_signature("Resistance").is_some());
            assert!(SignatureProvider::get_function_signature("SDI12Recorder").is_some());
            assert!(SignatureProvider::get_function_signature("WindVector").is_some());
        }

        #[test]
        fn tcdiff_ninth_parameter_is_the_notch_filter_frequency_not_integration() {
            // Campbell Scientific's own syntax diagram names this parameter
            // `fN1` (the sinc filter's first notch frequency), distinct from
            // the `Integ` parameter used by VoltSe/VoltDiff -- confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/tcdiff.htm
            let sig = SignatureProvider::get_function_signature("TCDiff")
                .expect("TCDiff should have a signature");

            assert_eq!(sig.parameters.len(), 11);
            assert_eq!(sig.parameters[8].name, "fN1");
        }

        #[test]
        fn voltdiff_seventh_parameter_is_the_notch_filter_frequency_not_integration() {
            // Same bug class as TCDiff above, not previously checked for
            // VoltDiff -- confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voltdiff.htm
            let sig = SignatureProvider::get_function_signature("VoltDiff")
                .expect("VoltDiff should have a signature");

            assert_eq!(sig.parameters.len(), 9);
            assert_eq!(sig.parameters[6].name, "fN1");
        }

        #[test]
        fn voltse_fifth_and_seventh_parameters_match_the_official_names() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voltse.htm:
            // the 5th parameter is `MeasOff` (not `MeasOfs`), and the 7th is
            // `fN1` (the sinc filter's first notch frequency, same bug class
            // as TCDiff/VoltDiff above, not `Integ`).
            let sig = SignatureProvider::get_function_signature("VoltSe")
                .expect("VoltSe should have a signature");

            assert_eq!(sig.parameters.len(), 9);
            assert_eq!(sig.parameters[4].name, "MeasOff");
            assert_eq!(sig.parameters[6].name, "fN1");
        }

        #[test]
        fn resistance_has_fourteen_parameters_in_documented_order() {
            let sig = SignatureProvider::get_function_signature("Resistance")
                .expect("Resistance should have a signature");
            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert_eq!(
                names,
                vec![
                    "Dest",
                    "Reps",
                    "Range",
                    "DiffChan",
                    "IexChan",
                    "MeasPEx",
                    "EXuA",
                    "RevEx",
                    "RevDiff",
                    "SettlingTime",
                    "fN1",
                    "Mult",
                    "Offset",
                    "MeasCurrent",
                ]
            );
        }

        #[test]
        fn sdi12recorder_has_eight_parameters_in_documented_order() {
            let sig = SignatureProvider::get_function_signature("SDI12Recorder")
                .expect("SDI12Recorder should have a signature");
            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert_eq!(
                names,
                vec![
                    "Dest",
                    "SDIPort",
                    "SDIAddress",
                    "SDICommand",
                    "Multiplier",
                    "Offset",
                    "FillNAN",
                    "WaitonTimeout",
                ]
            );
        }

        #[test]
        fn windvector_has_eight_parameters_in_documented_order() {
            let sig = SignatureProvider::get_function_signature("WindVector")
                .expect("WindVector should have a signature");
            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert_eq!(
                names,
                vec![
                    "Reps",
                    "Speed/East",
                    "Direction/North",
                    "DataType",
                    "DisableVar",
                    "Subinterval",
                    "SensorType",
                    "OutputOpt",
                ]
            );
        }

        #[test]
        fn has_remaining_measurement_functions() {
            for name in [
                "Battery",
                "PanelTemp",
                "BrHalf",
                "BrFull",
                "Therm107",
                "Therm108",
                "Therm109",
                "PeriodAvg",
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
            ] {
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for measurement function: {}",
                    name
                );
            }
        }

        #[test]
        fn has_communication_functions() {
            assert!(SignatureProvider::get_function_signature("SerialOpen").is_some());
            assert!(SignatureProvider::get_function_signature("SerialOut").is_some());
            assert!(SignatureProvider::get_function_signature("SerialIn").is_some());
        }

        #[test]
        fn has_remaining_communication_functions() {
            for name in [
                "SerialClose",
                "SerialInRecord",
                "SerialOutBlock",
                "SerialFlush",
                "ModbusMaster",
                "TCPOpen",
                "TCPClose",
                "UDPOpen",
                "UDPSocketOpen",
                "UDPSocketSend",
                "UDPSocketRecv",
                "UDPSocketClose",
                "EmailRelay",
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
                "SDMCVO4",
                "SDMGeneric",
                "SDMINT8",
                "SDMIO16",
                "SDMSIO4",
                "SDMSpeed",
                "SDMTrigger",
                "SDMX50",
            ] {
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for communication function: {}",
                    name
                );
            }
        }

        #[test]
        fn pppclose_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("PPPClose")
                .expect("PPPClose should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn timeuntiltransmit_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("TimeUntilTransmit")
                .expect("TimeUntilTransmit should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn sdmtrigger_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("SDMTrigger")
                .expect("SDMTrigger should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn samplefieldcal_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("SampleFieldCal")
                .expect("SampleFieldCal should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn fieldcalstrain_has_eleven_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("FieldCalStrain")
                .expect("FieldCalStrain should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(
                names,
                vec![
                    "Function",
                    "MeasureVar",
                    "Reps",
                    "GFAdj",
                    "ZeromV/V",
                    "Mode",
                    "KnownRS",
                    "Index",
                    "Avg",
                    "GFRaw",
                    "uStrainDest",
                ]
            );
        }

        #[test]
        fn cpispeed_takes_a_single_speed_parameter() {
            let sig = SignatureProvider::get_function_signature("CPISpeed")
                .expect("CPISpeed should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["Speed"]);
        }

        #[test]
        fn mqttpublishtable_has_eight_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("MQTTPublishTable")
                .expect("MQTTPublishTable should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(
                names,
                vec![
                    "QoS",
                    "NumRecsOrTimeIntoInterval",
                    "Interval",
                    "Units",
                    "OutputFormat",
                    "Longitude",
                    "Latitude",
                    "Altitude",
                ]
            );
        }

        #[test]
        fn enddialsequence_takes_a_single_dialsuccess_parameter() {
            let sig = SignatureProvider::get_function_signature("EndDialSequence")
                .expect("EndDialSequence should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["DialSuccess"]);
        }

        #[test]
        fn smssend_has_four_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("SMSSend")
                .expect("SMSSend should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["ResultCode", "Swath", "PhoneNumber", "Message"]);
        }

        #[test]
        fn sdmsw8a_has_seven_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("SDMSW8A")
                .expect("SDMSW8A should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(
                names,
                vec![
                    "Dest",
                    "Reps",
                    "SDMAddress",
                    "FunctOp",
                    "SW8AStartChan",
                    "Mult",
                    "Offset",
                ]
            );
        }

        #[test]
        fn tdr200_has_sixteen_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("TDR200")
                .expect("TDR200 should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(
                names,
                vec![
                    "Dest",
                    "SDMAddress",
                    "Option",
                    "MuxOrProbeSelect",
                    "WaveAvg",
                    "Vp",
                    "Points",
                    "CableLength",
                    "WindowLength",
                    "ProbeLength",
                    "ProbeOffset",
                    "Mult",
                    "Offset",
                    "NoiseRejectionFreq",
                    "TDRFilterLevel",
                    "TDRLaa",
                ]
            );
        }

        #[test]
        fn sw12_has_three_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("SW12")
                .expect("SW12 should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["SWChan", "State", "SW12Option"]);
        }

        #[test]
        fn pulsecountreset_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("PulseCountReset")
                .expect("PulseCountReset should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn etsz_has_eleven_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("ETsz")
                .expect("ETsz should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(
                names,
                vec![
                    "Temp",
                    "RH",
                    "uZ",
                    "Rs",
                    "Longitude",
                    "Latitude",
                    "Altitude",
                    "Zw",
                    "Sz",
                    "DataType",
                    "DisableVar",
                ]
            );
        }

        #[test]
        fn cardflush_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("CardFlush")
                .expect("CardFlush should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn signature_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("Signature")
                .expect("Signature should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn tablefile_has_eight_parameters_in_official_order() {
            let sig = SignatureProvider::get_function_signature("TableFile")
                .expect("TableFile should have a signature");

            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(
                names,
                vec![
                    "FileName",
                    "Options",
                    "MaxFiles",
                    "NumRecsOrTimeIntoInterval",
                    "Interval",
                    "Units",
                    "OutStat",
                    "LastFileName",
                ]
            );
        }

        #[test]
        fn has_math_functions() {
            assert!(SignatureProvider::get_function_signature("Abs").is_some());
            assert!(SignatureProvider::get_function_signature("Sqr").is_some());
            assert!(SignatureProvider::get_function_signature("Sin").is_some());
            assert!(SignatureProvider::get_function_signature("Round").is_some());
        }

        #[test]
        fn all_remaining_math_functions_have_a_signature() {
            for name in [
                "Sgn",
                "Exp",
                "Ln",
                "Log",
                "Log10",
                "Sinh",
                "Cosh",
                "Tanh",
                "Asin",
                "Acos",
                "Atn",
                "Int",
                "Fix",
                "Frac",
                "Rnd",
                "Randomize",
                "Ceiling",
                "Floor",
            ] {
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for math function: {}",
                    name
                );
            }
        }

        #[test]
        fn rnd_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("Rnd")
                .expect("Rnd should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn has_string_functions() {
            assert!(SignatureProvider::get_function_signature("Len").is_some());
            assert!(SignatureProvider::get_function_signature("Mid").is_some());
            assert!(SignatureProvider::get_function_signature("InStr").is_some());
            assert!(SignatureProvider::get_function_signature("SplitStr").is_some());
        }

        #[test]
        fn has_remaining_string_functions() {
            for name in [
                "FormatFloat",
                "FormatLong",
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
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for string function: {}",
                    name
                );
            }
        }

        #[test]
        fn instr_third_and_fourth_parameters_match_the_official_names() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/instr.htm:
            // the 3rd parameter is `FilterString` (the string being
            // searched for), not `SearchString` (which is the official
            // name of the 2nd parameter -- the string being searched in);
            // the 4th parameter is `SearchOption`, a multi-value method
            // code (0-10, +100 for quote-stripping), not a boolean
            // case-sensitivity flag.
            let sig = SignatureProvider::get_function_signature("InStr")
                .expect("InStr should have a signature");
            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert_eq!(
                names,
                vec!["Start", "SearchString", "FilterString", "SearchOption"]
            );
        }

        #[test]
        fn splitstr_third_parameter_is_named_filterstring_not_delimiter() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/splitstr.htm:
            // depending on SplitOption, this parameter can be a delimiter
            // set, an exact-match search string, or a header/footer filter,
            // so `Delimiter` undersells its documented role.
            let sig = SignatureProvider::get_function_signature("SplitStr")
                .expect("SplitStr should have a signature");

            assert_eq!(sig.parameters[2].name, "FilterString");
        }

        #[test]
        fn has_menu_functions() {
            for name in [
                "DisplayMenu",
                "SubMenu",
                "MenuItem",
                "MenuPick",
                "MenuRecompile",
                "DisplayValue",
                "DisplayLine",
            ] {
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for menu function: {}",
                    name
                );
            }
        }

        #[test]
        fn has_time_functions() {
            assert!(SignatureProvider::get_function_signature("Timer").is_some());
            assert!(SignatureProvider::get_function_signature("TimeIntoInterval").is_some());
            assert!(SignatureProvider::get_function_signature("Delay").is_some());
        }

        #[test]
        fn timeintointerval_has_the_same_leading_tintoint_parameter_as_iftime() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/timeintointervaliftime.htm:
            // TimeIntoInterval and IfTime are documented as the same
            // instruction ("Either keyword can be used within the
            // program"), so both take the same 3 parameters -- but the
            // `timeintointerval` arm was missing the leading `TintoInt`
            // parameter that the `iftime` arm already had correctly.
            let sig = SignatureProvider::get_function_signature("TimeIntoInterval")
                .expect("TimeIntoInterval should have a signature");
            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert_eq!(names, vec!["TintoInt", "Interval", "Units"]);
        }

        #[test]
        fn all_remaining_time_functions_have_a_signature() {
            for name in [
                "RealTime",
                "SetStatus",
                "SetSetting",
                "MoveBytes",
                "ArrayLength",
                "NaN",
                "SecsSince1990",
                "TimeIsBetween",
            ] {
                assert!(
                    SignatureProvider::get_function_signature(name).is_some(),
                    "Expected a signature for time function: {}",
                    name
                );
            }
        }

        #[test]
        fn nan_takes_no_parameters() {
            let sig = SignatureProvider::get_function_signature("NaN")
                .expect("NaN should have a signature");

            assert!(sig.parameters.is_empty());
        }

        #[test]
        fn delay_has_the_leading_option_parameter() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr6/Content/Instructions/delay3.htm:
            // `Delay(Option, Delay, Units)` -- `Option` (0/1/2, selecting
            // whether the pause affects the measurement task sequence,
            // processing, or digital/SDM measurements) is a required
            // leading parameter that was entirely missing.
            let sig = SignatureProvider::get_function_signature("Delay")
                .expect("Delay should have a signature");
            let names: Vec<&str> = sig.parameters.iter().map(|p| p.name.as_str()).collect();

            assert_eq!(names, vec!["Option", "Duration", "Units"]);
        }
    }

    mod parameter_counting {
        use super::*;

        #[test]
        fn counts_zero_for_first_parameter() {
            let text = "Scan(";
            let count = SignatureProvider::count_parameters_before_cursor(text, text.len());
            assert_eq!(count, 0);
        }

        #[test]
        fn counts_one_after_first_comma() {
            let text = "Scan(1, ";
            let count = SignatureProvider::count_parameters_before_cursor(text, text.len());
            assert_eq!(count, 1);
        }

        #[test]
        fn counts_two_after_second_comma() {
            let text = "Scan(1, Sec, ";
            let count = SignatureProvider::count_parameters_before_cursor(text, text.len());
            assert_eq!(count, 2);
        }

        #[test]
        fn ignores_commas_in_strings() {
            let text = r#"SerialOut(ComPort, "a,b,c", "#;
            let count = SignatureProvider::count_parameters_before_cursor(text, text.len());
            assert_eq!(count, 2);
        }

        #[test]
        fn handles_nested_parentheses() {
            let text = "Scan(Mid(str, 1, 2), ";
            let count = SignatureProvider::count_parameters_before_cursor(text, text.len());
            assert_eq!(count, 1);
        }
    }

    mod function_name_extraction {
        use super::*;

        #[test]
        fn extracts_simple_function_name() {
            let text = "Scan(1";
            let name = SignatureProvider::extract_function_name(text, text.len());
            assert_eq!(name, Some("Scan".to_string()));
        }

        #[test]
        fn extracts_function_name_with_spaces() {
            let text = "  Scan(1";
            let name = SignatureProvider::extract_function_name(text, text.len());
            assert_eq!(name, Some("Scan".to_string()));
        }

        #[test]
        fn extracts_nested_function_name() {
            let text = "Scan(Mid(str";
            let name = SignatureProvider::extract_function_name(text, text.len());
            assert_eq!(name, Some("Mid".to_string()));
        }

        #[test]
        fn returns_none_without_parenthesis() {
            let text = "Scan";
            let name = SignatureProvider::extract_function_name(text, text.len());
            assert!(name.is_none());
        }

        #[test]
        fn handles_function_after_operator() {
            let text = "x = Scan(1";
            let name = SignatureProvider::extract_function_name(text, text.len());
            assert_eq!(name, Some("Scan".to_string()));
        }
    }
}
