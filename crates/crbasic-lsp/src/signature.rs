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
        fn has_communication_functions() {
            assert!(SignatureProvider::get_function_signature("SerialOpen").is_some());
            assert!(SignatureProvider::get_function_signature("SerialOut").is_some());
            assert!(SignatureProvider::get_function_signature("SerialIn").is_some());
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
                "Sgn", "Exp", "Ln", "Log", "Log10", "Sinh", "Cosh", "Tanh", "Asin", "Acos", "Atn",
                "Int", "Fix", "Frac", "Rnd", "Ceiling", "Floor",
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
