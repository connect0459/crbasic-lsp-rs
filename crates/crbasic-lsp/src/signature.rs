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
