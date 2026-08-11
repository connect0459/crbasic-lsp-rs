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
                "SubScan",
                "SubScan(${1:SubInterval}, ${2:Units}, ${3:Count})",
                "Begins a nested sub-scan for faster measurement or multiplexer control.",
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
                "VoltSe(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:MeasOff}, ${6:SettlingTime}, ${7:fN1}, ${8:Mult}, ${9:Offset})",
                "Measures single-ended voltage.",
            ),
            Self::create_function_completion(
                "VoltDiff",
                "VoltDiff(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:RevDiff}, ${6:SettlingTime}, ${7:fN1}, ${8:Mult}, ${9:Offset})",
                "Measures differential voltage.",
            ),
            Self::create_function_completion(
                "Therm107",
                "Therm107(${1:Dest}, ${2:Reps}, ${3:SEChan}, ${4:Excite}, ${5:SettlingTime}, ${6:Integ}, ${7:Mult}, ${8:Offset})",
                "Measures temperature using a 107 thermistor.",
            ),
            Self::create_function_completion(
                "Therm108",
                "Therm108(${1:Dest}, ${2:Reps}, ${3:SEChan}, ${4:Excite}, ${5:SettlingTime}, ${6:Integ}, ${7:Mult}, ${8:Offset})",
                "Measures temperature using a 108 thermistor.",
            ),
            Self::create_function_completion(
                "Therm109",
                "Therm109(${1:Dest}, ${2:Reps}, ${3:SEChan}, ${4:Excite}, ${5:SettlingTime}, ${6:Integ}, ${7:Mult}, ${8:Offset})",
                "Measures temperature using a 109 thermistor.",
            ),
            Self::create_function_completion(
                "TCDiff",
                "TCDiff(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:TCType}, ${6:TRef}, ${7:RevDiff}, ${8:SettlingTime}, ${9:fN1}, ${10:Mult}, ${11:Offset})",
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
                "InStr(${1:Start}, ${2:SearchString}, ${3:FilterString}, ${4:SearchOption})",
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
                "SplitStr(${1:Result}, ${2:SearchString}, ${3:FilterString}, ${4:NumSplits}, ${5:SplitOption})",
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
                "TimeIntoInterval(${1:TintoInt}, ${2:Interval}, ${3:Units})",
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
                "Delay(${1:Option}, ${2:Duration}, ${3:Units})",
                "Pauses execution for a specified time.",
            ),
            Self::create_function_completion(
                "Battery",
                "Battery(${1:Dest})",
                "Measures the voltage of the battery powering the datalogger.",
            ),
            Self::create_function_completion(
                "PanelTemp",
                "PanelTemp(${1:Dest}, ${2:fN1})",
                "Measures the temperature of the datalogger wiring panel in degrees Celsius.",
            ),
            Self::create_function_completion(
                "BrHalf",
                "BrHalf(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:ExChan}, ${6:MeasPEx}, ${7:ExmV}, ${8:RevEx}, ${9:SettlingTime}, ${10:fN1}, ${11:Mult}, ${12:Offset})",
                "Applies an excitation voltage to a half bridge and measures the single-ended voltage output.",
            ),
            Self::create_function_completion(
                "BrFull",
                "BrFull(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:ExChan}, ${6:MeasPEx}, ${7:ExmV}, ${8:RevEx}, ${9:RevDiff}, ${10:SettlingTime}, ${11:fN1}, ${12:Mult}, ${13:Offset})",
                "Applies an excitation voltage to a full bridge and measures the differential voltage output.",
            ),
            Self::create_function_completion(
                "Resistance",
                "Resistance(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:IexChan}, ${6:MeasPEx}, ${7:EXuA}, ${8:RevEx}, ${9:RevDiff}, ${10:SettlingTime}, ${11:fN1}, ${12:Mult}, ${13:Offset}, ${14:MeasCurrent})",
                "Applies a known excitation current and measures the resistance of a bridge or resistive circuit.",
            ),
            Self::create_function_completion(
                "PeriodAvg",
                "PeriodAvg(${1:Dest}, ${2:Reps}, ${3:Chan}, ${4:Option}, ${5:Cycles}, ${6:Timeout}, ${7:Mult}, ${8:Offset})",
                "Measures the period or frequency of a signal on a single-ended channel.",
            ),
            Self::create_function_completion(
                "PortSet",
                "PortSet(${1:Port}, ${2:State}, ${3:Option})",
                "Sets a control port to a high or low state.",
            ),
            Self::create_function_completion(
                "PulsePort",
                "PulsePort(${1:Port}, ${2:Delay})",
                "Toggles a port, delays, toggles it back, and delays again to generate a clocking pulse.",
            ),
            Self::create_function_completion(
                "ExciteV",
                "ExciteV(${1:ExChan}, ${2:ExmV}, ${3:Delay})",
                "Sets an excitation channel output to a specified voltage for a specified duration.",
            ),
            Self::create_function_completion(
                "BrHalf3W",
                "BrHalf3W(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:ExChan}, ${6:MeasPEx}, ${7:ExmV}, ${8:RevEx}, ${9:SettlingTime}, ${10:fN1}, ${11:Mult}, ${12:Offset})",
                "Applies an excitation voltage and measures a 3-wire half bridge to calculate the resistance ratio.",
            ),
            Self::create_function_completion(
                "BrHalf4W",
                "BrHalf4W(${1:Dest}, ${2:Reps}, ${3:Range1}, ${4:Range2}, ${5:DiffChan}, ${6:ExChan}, ${7:MeasPEx}, ${8:ExmV}, ${9:RevEx}, ${10:RevDiff}, ${11:SettlingTime}, ${12:fN1}, ${13:Mult}, ${14:Offset}, ${15:ReturnV1})",
                "Applies an excitation voltage and makes two differential voltage measurements to measure a 4-wire half bridge.",
            ),
            Self::create_function_completion(
                "BrFull6W",
                "BrFull6W(${1:Dest}, ${2:Reps}, ${3:Range1}, ${4:Range2}, ${5:DiffChan}, ${6:ExChan}, ${7:MeasPEx}, ${8:ExmV}, ${9:RevEx}, ${10:RevDiff}, ${11:SettlingTime}, ${12:fN1}, ${13:Mult}, ${14:Offset}, ${15:ReturnV1})",
                "Applies an excitation voltage and makes two differential voltage measurements to measure a 6-wire full bridge.",
            ),
            Self::create_function_completion(
                "TCSE",
                "TCSE(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:TCType}, ${6:TRef}, ${7:MeasOff}, ${8:SettlingTime}, ${9:fN1}, ${10:Mult}, ${11:Offset})",
                "Measures a thermocouple on a single-ended channel and converts the reading to degrees Celsius.",
            ),
            Self::create_function_completion(
                "SerialInRecord",
                "SerialInRecord(${1:COMPort}, ${2:Dest}, ${3:BeginWord}, ${4:NBytes}, ${5:EndWord}, ${6:NBytesReturned}, ${7:SerialInRecOption})",
                "Reads and parses incoming serial data using begin/end markers.",
            ),
            Self::create_function_completion(
                "SerialOutBlock",
                "SerialOutBlock(${1:ComPort}, ${2:Expression}, ${3:NumberBytes})",
                "Sends binary data out a serial port.",
            ),
            Self::create_function_completion(
                "SerialFlush",
                "SerialFlush(${1:ComPort})",
                "Clears any characters in the serial input buffer.",
            ),
            Self::create_function_completion(
                "ModbusMaster",
                "ModbusMaster(${1:ResultCode}, ${2:ComPort}, ${3:BaudRate}, ${4:ModbusAddr}, ${5:Function}, ${6:Variable}, ${7:Start}, ${8:Length}, ${9:Tries}, ${10:TimeOut}, ${11:ModbusOption})",
                "Sets up the datalogger as a Modbus client to send or retrieve data from a Modbus server.",
            ),
            Self::create_function_completion(
                "SDI12Recorder",
                "SDI12Recorder(${1:Dest}, ${2:SDIPort}, ${3:SDIAddress}, ${4:SDICommand}, ${5:Multiplier}, ${6:Offset}, ${7:FillNAN}, ${8:WaitonTimeout})",
                "Retrieves measurement results from an SDI-12 sensor.",
            ),
            Self::create_function_completion(
                "TCPOpen",
                "TCPOpen(${1:IPAddr}, ${2:TCPPort}, ${3:IPBuffer}, ${4:IPTimeOut}, ${5:ConnectHandle}, ${6:MaxConnect})",
                "Sets up a TCP/IP socket for communication.",
            ),
            Self::create_function_completion(
                "TCPClose",
                "TCPClose(${1:TCPSocket})",
                "Closes a TCP/IP socket that was set up for communication.",
            ),
            Self::create_function_completion(
                "UDPOpen",
                "UDPOpen(${1:IPAddr}, ${2:UDPPort}, ${3:IPBuffer}, ${4:IPVersion})",
                "Opens a port for transferring UDP packets.",
            ),
            Self::create_function_completion(
                "UDPSocketOpen",
                "UDPSocketOpen(${1:SocketID}, ${2:Port}, ${3:RecvQueueSize}, ${4:Interface})",
                "Opens a UDP socket, relating a UDP source port to an ID.",
            ),
            Self::create_function_completion(
                "UDPSocketSend",
                "UDPSocketSend(${1:BytesSent}, ${2:SocketID}, ${3:IPAddr}, ${4:Port}, ${5:Payload}, ${6:PayLoadLen})",
                "Sends a UDP datagram to a remote device via an opened UDP socket.",
            ),
            Self::create_function_completion(
                "UDPSocketRecv",
                "UDPSocketRecv(${1:BytesReceived}, ${2:SocketID}, ${3:InDatagram}, ${4:InDatagramLen}, ${5:RemoteIPAdd}, ${6:RemotePort}, ${7:Timeout})",
                "Retrieves incoming UDP packets sent to a socket's listening port.",
            ),
            Self::create_function_completion(
                "UDPSocketClose",
                "UDPSocketClose(${1:SocketID})",
                "Closes an opened UDP socket and frees its associated memory.",
            ),
            Self::create_function_completion(
                "EmailRelay",
                "EmailRelay(${1:ToAddr}, ${2:Subject}, ${3:Message}, ${4:ServerResponse}, ${5:Attach}, ${6:NumRecsOrTimeIntoInterval}, ${7:Interval}, ${8:IntervalUnits}, ${9:FileOption}, ${10:TimeOut})",
                "Sends an email message to one or more addresses via a Campbell Scientific relay service.",
            ),
            Self::create_function_completion(
                "PPPOpen",
                "PPPOpen(${1:Option})",
                "Enables a PPP network connection through an external modem and returns its IP address.",
            ),
            Self::create_function_completion(
                "PPPClose",
                "PPPClose()",
                "Closes an open PPP connection with a server.",
            ),
            Self::create_function_completion(
                "FTPClient",
                "FTPClient(${1:IPAddress}, ${2:User}, ${3:Password}, ${4:LocalFileName}, ${5:RemoteFileName}, ${6:PutGetOption}, ${7:NumRecsOrTimeIntoInterval}, ${8:Interval}, ${9:IntervalUnits}, ${10:FileOption}, ${11:TimeOut})",
                "Manages files on a server using FTP, FTPS, or SFTP.",
            ),
            Self::create_function_completion(
                "HTTPGet",
                "HTTPGet(${1:URI}, ${2:Response}, ${3:Header}, ${4:TimeOut})",
                "Sends a GET request to an HTTP server.",
            ),
            Self::create_function_completion(
                "HTTPPost",
                "HTTPPost(${1:URI}, ${2:Contents}, ${3:Response}, ${4:Header}, ${5:NumRecsOrTimeIntoInterval}, ${6:Interval}, ${7:IntervalUnits}, ${8:FileOption}, ${9:TimeOut})",
                "Sends files or text to a URL via an HTTP POST request.",
            ),
            Self::create_function_completion(
                "HTTPPut",
                "HTTPPut(${1:URI}, ${2:Contents}, ${3:Response}, ${4:Header}, ${5:NumRecsOrTimeIntoInterval}, ${6:Interval}, ${7:IntervalUnits}, ${8:FileOption}, ${9:TimeOut})",
                "Sends files or text to a URL via an HTTP PUT request.",
            ),
            Self::create_function_completion(
                "WindVector",
                "WindVector(${1:Reps}, ${2:SpeedOrEast}, ${3:DirectionOrNorth}, ${4:DataType}, ${5:DisableVar}, ${6:Subinterval}, ${7:SensorType}, ${8:OutputOpt})",
                "Calculates and stores the mean wind speed, wind vector magnitude, and direction statistics.",
            ),
            Self::create_function_completion(
                "Histogram",
                "Histogram(${1:BinSelect}, ${2:DataType}, ${3:DisableVar}, ${4:Bins}, ${5:Form}, ${6:WtVal}, ${7:LoLim}, ${8:UpLim})",
                "Stores a frequency distribution of input data across a set of bins.",
            ),
            Self::create_function_completion(
                "FieldNames",
                "FieldNames(${1:FieldNameDescriptionList})",
                "Overrides the default field names for the preceding output-processing instruction.",
            ),
            Self::create_function_completion(
                "CardOut",
                "CardOut(${1:StopRing}, ${2:Size})",
                "Creates a new data table that is stored on a memory card.",
            ),
            Self::create_function_completion(
                "NewFile",
                "NewFile(${1:NewFileVar}, ${2:FileName}, ${3:NewFileName})",
                "Determines whether a monitored file has been newly written since this instruction last ran.",
            ),
            Self::create_function_completion(
                "FileManage",
                "FileManage(${1:DeviceFileName}, ${2:Attribute})",
                "Performs a management operation, such as delete, hide, run, or format, on a file or device.",
            ),
            Self::create_function_completion(
                "FileOpen",
                "FileOpen(${1:FileName}, ${2:Mode}, ${3:SeekPoint})",
                "Opens a file for reading or writing and returns a file handle.",
            ),
            Self::create_function_completion(
                "FileClose",
                "FileClose(${1:FileHandle})",
                "Closes a file previously opened with FileOpen.",
            ),
            Self::create_function_completion(
                "FileRead",
                "FileRead(${1:FileHandle}, ${2:Destination}, ${3:Length})",
                "Reads data from an open file into a variable or array.",
            ),
            Self::create_function_completion(
                "FileWrite",
                "FileWrite(${1:FileHandle}, ${2:Source}, ${3:Length})",
                "Writes data from a variable or array to an open file.",
            ),
            Self::create_function_completion(
                "FileCopy",
                "FileCopy(${1:FromFileName}, ${2:ToFileName})",
                "Copies a file from one drive on the datalogger to another.",
            ),
            Self::create_function_completion(
                "FileRename",
                "FileRename(${1:OldFileName}, ${2:NewFileName})",
                "Renames a file stored on the datalogger.",
            ),
            Self::create_function_completion(
                "FileSize",
                "FileSize(${1:FileHandle})",
                "Returns the size, in bytes, of a specified file.",
            ),
            Self::create_function_completion(
                "FileTime",
                "FileTime(${1:FileHandle})",
                "Returns the last-modified timestamp of a specified file.",
            ),
            Self::create_function_completion(
                "FileList",
                "FileList(${1:Device}, ${2:Dest})",
                "Writes the list of file names on a device into a destination array.",
            ),
            Self::create_function_completion(
                "DataInterval",
                "DataInterval(${1:TintoInt}, ${2:Interval}, ${3:Units}, ${4:Lapses})",
                "Sets the real-time-clock-based interval on which a data table's records are generated.",
            ),
            Self::create_function_completion(
                "FormatLong",
                "FormatLong(${1:LongVar}, ${2:FormatString})",
                "Converts a Long value to a decimal, hexadecimal, or octal string.",
            ),
            Self::create_function_completion(
                "Chr",
                "Chr(${1:Code})",
                "Returns a character in the extended ASCII character set.",
            ),
            Self::create_function_completion(
                "ASCII",
                "ASCII(${1:ASCIIString})",
                "Returns the ASCII value of a character in a string.",
            ),
            Self::create_function_completion(
                "StrComp",
                "StrComp(${1:String1}, ${2:String2})",
                "Compares two strings to determine if they are identical or their sort order.",
            ),
            Self::create_function_completion(
                "CheckSum",
                "CheckSum(${1:ChkSumString}, ${2:ChkSumType}, ${3:CheckSumSize}, ${4:ChkSumOption1}, ${5:ChkSumOption2}, ${6:ChkSumOption3})",
                "Returns a checksum signature for the characters in a string.",
            ),
            Self::create_function_completion(
                "HexToDec",
                "HexToDec(${1:Expression})",
                "Converts a hexadecimal string to a float or integer.",
            ),
            Self::create_function_completion(
                "Hex",
                "Hex(${1:Expression})",
                "Returns a hexadecimal string representation of a Long value.",
            ),
            Self::create_function_completion(
                "Sprintf",
                "Sprintf(${1:Dest}, ${2:Format}, ${3:Argument1})",
                "Writes a formatted output string to a destination variable.",
            ),
            Self::create_function_completion(
                "Sgn",
                "Sgn(${1:Value})",
                "Returns the sign of a number as -1, 0, or 1.",
            ),
            Self::create_function_completion(
                "Ln",
                "Ln(${1:Value})",
                "Returns the natural logarithm.",
            ),
            Self::create_function_completion(
                "Sinh",
                "Sinh(${1:Value})",
                "Returns the hyperbolic sine.",
            ),
            Self::create_function_completion(
                "Cosh",
                "Cosh(${1:Value})",
                "Returns the hyperbolic cosine.",
            ),
            Self::create_function_completion(
                "Tanh",
                "Tanh(${1:Value})",
                "Returns the hyperbolic tangent.",
            ),
            Self::create_function_completion(
                "Frac",
                "Frac(${1:Value})",
                "Returns the fractional portion of a number.",
            ),
            Self::create_function_completion(
                "Rnd",
                "Rnd",
                "Returns a random value between 0 (inclusive) and 1 (exclusive).",
            ),
            Self::create_function_completion(
                "Ceiling",
                "Ceiling(${1:Value})",
                "Rounds a number up to the nearest integer.",
            ),
            Self::create_function_completion(
                "Floor",
                "Floor(${1:Value})",
                "Rounds a number down to the nearest integer.",
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
                "Scan(${1:Interval}, ${2:Sec}, ${3:0}, ${4:0})\n\t$0\n\tCallTable ${5:TableName}\nNextScan",
                "Scan/NextScan loop with a CallTable call",
            ),
            Self::create_pattern_snippet(
                "SlowSequenceLoop",
                "SlowSequence\n\tScan(${1:Interval}, ${2:Sec}, ${3:0}, ${4:0})\n\t\t$0\n\t\tCallTable ${5:TableName}\n\tNextScan\nEndSequence",
                "Low-priority scan sequence for non-time-critical measurements",
            ),
            Self::create_pattern_snippet(
                "DataTableSample",
                "DataTable(${1:TableName}, ${2:True}, ${3:-1})\n\tSample(${4:1}, ${5:SourceVariable}, ${6:FP2})\n\t$0\nEndTable",
                "DataTable definition with a Sample output field",
            ),
            Self::create_pattern_snippet(
                "NewProgram",
                "Const ${1:ScanIntervalSec} = ${2:5}\nPublic ${3:VarName} As ${4:Float}\n\nDataTable(${5:TableName},True,-1)\n\tSample(1,${3:VarName},FP2)\nEndTable\n\nBeginProg\n\tScan(${1:ScanIntervalSec},Sec,0,0)\n\t\t$0\n\t\tCallTable ${5:TableName}\n\tNextScan\nEndProg",
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
        items.extend(Self::output_processing_data_type_completions());

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
    /// (see `output_processing_data_type_completions`) that's only valid as
    /// a `Sample()`/`Average()`-style instruction argument.
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

    /// Returns completion items for the data types valid only as a
    /// `Sample()`/`Average()`-style instruction argument (final data
    /// storage), e.g. `Sample(1, SourceVariable, FP2)`.
    ///
    /// Per Campbell Scientific's own "Data Types" documentation, this set
    /// overlaps `data_type_completions`'s six on `Long`/`UINT1`/`Boolean`/
    /// `String` -- those are omitted here to avoid duplicate completion
    /// items, since `get_all_completions` offers both lists unconditionally
    /// (this project does not do position-sensitive completion filtering,
    /// consistent with `data_type_completions`).
    ///
    /// Not part of `LANGUAGE_KEYWORDS` for the same reason as
    /// `data_type_completions`: these already parse as plain identifiers.
    fn output_processing_data_type_completions() -> Vec<CompletionItem> {
        vec![
            Self::create_keyword_completion(
                "FP2",
                "Campbell Scientific proprietary format; 3 or 4 significant digits (2 bytes)",
            ),
            Self::create_keyword_completion(
                "IEEE4",
                "Single-precision floating point number (4 bytes); same precision as Float",
            ),
            Self::create_keyword_completion(
                "IEEE8",
                "Double-precision floating point number (8 bytes); same precision as Double",
            ),
            Self::create_keyword_completion("UINT2", "16-bit unsigned integer"),
            Self::create_keyword_completion("UINT4", "32-bit unsigned integer"),
            Self::create_keyword_completion(
                "Bool8",
                "Array of eight 1-bit Boolean values packed into 1 byte",
            ),
            Self::create_keyword_completion("NSEC", "Nanosecond-resolution time stamp (8 bytes)"),
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
            Self::create_keyword_completion("Until", "Loop condition (inverse of While)"),
            Self::create_keyword_completion("ExitFor", "Exit For loop immediately"),
            Self::create_keyword_completion("ExitDo", "Exit Do loop immediately"),
            Self::create_keyword_completion(
                "DebugBreak",
                "Suspends execution at this line when running under the CRBasic debugger",
            ),
            Self::create_keyword_completion("Restart", "Stops and restarts the running program"),
            Self::create_keyword_snippet(
                "Select Case",
                "Select Case ${1:expression}\n\tCase ${2:value}\n\t\t$0\n\tCase Else\n\t\t\nEndSelect",
                "Multi-way branch statement",
            ),
            Self::create_keyword_completion("Select", "Starts a Select Case block"),
            Self::create_keyword_completion("Case", "Branch in Select statement"),
            Self::create_keyword_completion("Is", "Comparison operator in a Case clause"),
            Self::create_keyword_completion("EndSelect", "Terminates Select block"),
            Self::create_keyword_completion("EndMenu", "Terminates a DisplayMenu block"),
            Self::create_keyword_completion("EndSubMenu", "Terminates a SubMenu block"),
            Self::create_keyword_completion("NextScan", "Terminates a Scan loop"),
            Self::create_keyword_completion(
                "ContinueScan",
                "Jumps to NextScan, skipping the rest of the scan body",
            ),
            Self::create_keyword_completion("NextSubScan", "Terminates a SubScan block"),
            Self::create_keyword_completion(
                "WaitTriggerSequence",
                "Marks a resume-point inside a SlowSequence, waiting for the trigger condition",
            ),
            Self::create_keyword_completion(
                "ExitScan",
                "Exits the entire Scan loop immediately, regardless of Count",
            ),
            Self::create_keyword_completion("Return", "Returns a value from a Function"),
            Self::create_keyword_completion("ExitFunction", "Exit Function immediately"),
            Self::create_keyword_completion(
                "Exit",
                "Used with Sub to exit a Subroutine (Exit Sub)",
            ),
            Self::create_keyword_snippet(
                "Call",
                "Call ${1:SubName}(${2:arguments})",
                "Invokes a subroutine",
            ),
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
            Self::create_keyword_completion(
                "ReadOnly",
                "Mark a Public variable as visible but not externally editable",
            ),
            Self::create_keyword_completion(
                "Optional",
                "Mark a Function/Sub parameter as optional",
            ),
            Self::create_keyword_snippet(
                "Include",
                "Include \"${1:cpu:Filename.crb}\"",
                "Pull in an external CRBasic source file",
            ),
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
            Self::create_keyword_completion(
                "SequentialMode",
                "Forces the datalogger to run in sequential execution mode; placed before BeginProg",
            ),
            Self::create_keyword_completion(
                "PipeLineMode",
                "Forces the datalogger to run in pipeline execution mode; placed before BeginProg",
            ),
            Self::create_keyword_completion(
                "PreserveVariables",
                "Retains Dim/Public variable values in memory across a power loss; placed before BeginProg",
            ),
            Self::create_keyword_completion(
                "AngleDegrees",
                "Switches trig functions to use degrees instead of radians; placed before BeginProg",
            ),
            Self::create_keyword_snippet(
                "ApplyAndRestartSequence",
                "ApplyAndRestartSequence\n\t$0\nEndApplyAndRestartSequence",
                "Runs when the ConstTable it follows has its ApplyAndRestart setting externally set (e.g. via SetSetting), typically to validate the new constant values",
            ),
            Self::create_keyword_completion(
                "EndApplyAndRestartSequence",
                "Terminates an ApplyAndRestartSequence block",
            ),
            Self::create_keyword_snippet(
                "ShutDownBegin",
                "ShutDownBegin\n\t$0\nShutDownEnd",
                "Runs cleanup code when the program stops normally; placed before BeginProg",
            ),
            Self::create_keyword_completion("ShutDownEnd", "Terminates a ShutDownBegin block"),
            Self::create_keyword_completion(
                "ESSInitialize",
                "Initializes the NTCIP Environmental Sensor Station SNMP agent; placed directly after BeginProg",
            ),
            Self::create_keyword_snippet(
                "ESSVariables",
                "ESSVariables ${1:Public}",
                "Auto-declares the standard set of NTCIP Environmental Sensor Station variables, used with ESSInitialize",
            ),
            Self::create_keyword_completion("WebPageEnd", "Terminates a WebPageBegin block"),
            Self::create_keyword_completion("EndModemHangup", "Terminates a ModemHangup block"),
            Self::create_keyword_snippet(
                "VoiceBeg",
                "VoiceBeg\n\t$0\nEndVoice",
                "Begins a block of voice-modem response code",
            ),
            Self::create_keyword_completion("EndVoice", "Terminates a VoiceBeg block"),
            Self::create_keyword_snippet(
                "DataTable",
                "DataTable(${1:TableName}, ${2:TriggerCondition}, ${3:Size})\n\t$0\nEndTable",
                "Define data storage table",
            ),
            Self::create_keyword_completion("EndTable", "Terminates DataTable block"),
            Self::create_keyword_completion(
                "TableHide",
                "Suppresses the display and data collection of this DataTable",
            ),
            Self::create_keyword_completion(
                "OpenInterval",
                "Includes all measurements since the last data storage, spanning missed output intervals",
            ),
            Self::create_keyword_completion(
                "FillStop",
                "Stops data storage once this DataTable reaches its configured size, instead of overwriting the oldest records",
            ),
            Self::create_keyword_snippet(
                "CallTable",
                "CallTable ${1:TableName}",
                "Invoke a previously declared DataTable, storing a record if its trigger condition fires",
            ),
            Self::create_keyword_snippet(
                "ConstTable",
                "ConstTable(${1:TableName}, ${2:Hidden})\n\tConst $0\nEndConstTable",
                "Define a block of editable constants",
            ),
            Self::create_keyword_completion("EndConstTable", "Terminates ConstTable block"),
            Self::create_keyword_snippet(
                "SlowSequence",
                "SlowSequence\n\t$0\nEndSequence",
                "Begins a slow sequence scan block",
            ),
            Self::create_keyword_completion("EndSequence", "Ends a slow sequence scan block"),
            Self::create_keyword_snippet(
                "StructureType",
                "StructureType ${1:TypeName}\n\t$0\nEndStructureType",
                "Define a reusable data structure",
            ),
            Self::create_keyword_completion("EndStructureType", "Terminates StructureType block"),
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
            Self::create_keyword_completion("IMP", "Logical implication operator"),
            Self::create_keyword_completion("EQV", "Logical equivalence operator"),
            Self::create_keyword_completion("INTDV", "Integer division operator (synonym for \\)"),
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
            assert!(labels.contains(&"CallTable"));
            assert!(labels.contains(&"ConstTable"));
            assert!(labels.contains(&"EndConstTable"));
            assert!(labels.contains(&"StructureType"));
            assert!(labels.contains(&"EndStructureType"));
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

        mod documentation_accuracy {
            use super::*;

            fn completion_for(label: &str) -> CompletionItem {
                CompletionProvider::get_keyword_completions()
                    .into_iter()
                    .find(|c| c.label == label)
                    .unwrap_or_else(|| panic!("expected a completion item for: {}", label))
            }

            #[test]
            fn consttable_snippet_names_its_second_parameter_hidden_not_enabled() {
                let insert_text = completion_for("ConstTable").insert_text.unwrap();

                assert!(
                    !insert_text.contains("Enabled") && insert_text.contains("Hidden"),
                    "the official syntax \
                     (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/consttableendconsttable.htm) \
                     names the second parameter Hidden (1 = visible only at highest security \
                     level, 0/omitted = standard visible table), not Enabled: {}",
                    insert_text
                );
            }

            #[test]
            fn applyandrestartsequence_detail_does_not_claim_placement_before_consttable() {
                let detail = completion_for("ApplyAndRestartSequence").detail.unwrap();

                assert!(
                    !detail.contains("before ConstTable"),
                    "the official example \
                     (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/applyandrestartsequence.htm) \
                     declares ApplyAndRestartSequence after the ConstTable it applies to, not \
                     before: {}",
                    detail
                );
            }

            #[test]
            fn exitscan_detail_is_distinct_from_continuescan_iteration_skip() {
                let exit_scan_detail = completion_for("ExitScan").detail.unwrap();
                let continue_scan_detail = completion_for("ContinueScan").detail.unwrap();

                assert_ne!(
                    exit_scan_detail, continue_scan_detail,
                    "ExitScan breaks out of the entire Scan loop regardless of Count \
                     (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/scannextscan.htm), \
                     a different, stronger effect than ContinueScan's skip-to-next-iteration -- \
                     the two details must not read as describing the same behavior"
                );
                assert!(
                    exit_scan_detail.to_lowercase().contains("loop"),
                    "ExitScan's detail should describe leaving the Scan loop itself, not just \
                     \"the current scan\": {}",
                    exit_scan_detail
                );
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

    mod output_processing_data_type_completions {
        use super::*;

        #[test]
        fn includes_every_type_valid_only_in_output_processing_instructions() {
            let completions = CompletionProvider::output_processing_data_type_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            // Per Campbell Scientific's own "Data Types" documentation, these
            // are valid as a Sample()/Average()-style instruction argument
            // but not after `As` in a Public/Dim declaration -- the inverse
            // of `data_type_completions`'s six.
            for expected in ["FP2", "IEEE4", "IEEE8", "UINT2", "UINT4", "Bool8", "NSEC"] {
                assert!(
                    labels.contains(&expected),
                    "Missing output-processing data type completion: {}",
                    expected
                );
            }
        }

        #[test]
        fn does_not_duplicate_types_already_valid_after_as() {
            let completions = CompletionProvider::output_processing_data_type_completions();
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            for already_covered in ["Float", "Double", "Long", "Boolean", "String", "UINT1"] {
                assert!(
                    !labels.contains(&already_covered),
                    "{} is already offered by data_type_completions",
                    already_covered
                );
            }
        }

        #[test]
        fn types_have_correct_kind() {
            let completions = CompletionProvider::output_processing_data_type_completions();

            for completion in &completions {
                assert_eq!(completion.kind, Some(CompletionItemKind::KEYWORD));
            }
        }

        #[test]
        fn is_included_in_all_completions() {
            let completions = CompletionProvider::get_all_completions(None);
            let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

            assert!(labels.contains(&"FP2"));
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
        fn tcdiff_ninth_placeholder_is_the_notch_filter_frequency_not_integration() {
            // Campbell Scientific's own syntax diagram names this parameter
            // `fN1`, distinct from the `Integ` parameter used by
            // VoltSe/VoltDiff -- confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/tcdiff.htm
            let completions = CompletionProvider::get_builtin_function_completions();
            let tcdiff = completions.iter().find(|c| c.label == "TCDiff").unwrap();

            assert!(tcdiff.insert_text.as_ref().unwrap().contains("${9:fN1}"));
        }

        #[test]
        fn voltdiff_seventh_placeholder_is_the_notch_filter_frequency_not_integration() {
            // Same bug class as TCDiff above, not previously checked for
            // VoltDiff -- confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voltdiff.htm
            let completions = CompletionProvider::get_builtin_function_completions();
            let voltdiff = completions.iter().find(|c| c.label == "VoltDiff").unwrap();

            assert!(voltdiff.insert_text.as_ref().unwrap().contains("${7:fN1}"));
        }

        #[test]
        fn voltse_fifth_and_seventh_placeholders_match_the_official_names() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voltse.htm:
            // the 5th parameter is `MeasOff` (not `MeasOfs`), and the 7th is
            // `fN1` (same bug class as TCDiff/VoltDiff above, not `Integ`).
            let completions = CompletionProvider::get_builtin_function_completions();
            let voltse = completions.iter().find(|c| c.label == "VoltSe").unwrap();
            let snippet = voltse.insert_text.as_ref().unwrap();

            assert!(snippet.contains("${5:MeasOff}"));
            assert!(snippet.contains("${7:fN1}"));
        }

        #[test]
        fn therm10x_functions_include_settling_time_and_integration_parameters() {
            // Therm107/108/109's official 8-parameter syntax
            // (Dest, Reps, SEChan, Excite, SettlingTime, Integ, Mult, Offset)
            // was missing SettlingTime/Integ entirely, truncated to 6
            // parameters -- confirmed via Campbell Scientific's Model
            // 107/108/109 manuals (s.campbellsci.com/documents/us/manuals/107.pdf)
            // and the CR6 Measurement and Control System manual.
            let completions = CompletionProvider::get_builtin_function_completions();

            for name in ["Therm107", "Therm108", "Therm109"] {
                let completion = completions.iter().find(|c| c.label == name).unwrap();
                let snippet = completion.insert_text.as_ref().unwrap();

                assert!(
                    snippet.contains("${5:SettlingTime}"),
                    "{name} snippet missing SettlingTime: {snippet}"
                );
                assert!(
                    snippet.contains("${6:Integ}"),
                    "{name} snippet missing Integ: {snippet}"
                );
                assert!(
                    snippet.contains("${7:Mult}"),
                    "{name} snippet's Mult should be the 7th parameter: {snippet}"
                );
                assert!(
                    snippet.contains("${8:Offset}"),
                    "{name} snippet's Offset should be the 8th parameter: {snippet}"
                );
            }
        }

        #[test]
        fn instr_third_and_fourth_placeholders_match_the_official_names() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/instr.htm:
            // parameter order is (Start, SearchString, FilterString,
            // SearchOption) -- the codebase had `String`/`SearchString`
            // shifted one slot early and mislabeled the 4th as a boolean
            // `CaseSensitive`.
            let completions = CompletionProvider::get_builtin_function_completions();
            let instr = completions.iter().find(|c| c.label == "InStr").unwrap();
            let snippet = instr.insert_text.as_ref().unwrap();

            assert!(snippet.contains("${2:SearchString}"));
            assert!(snippet.contains("${3:FilterString}"));
            assert!(snippet.contains("${4:SearchOption}"));
        }

        #[test]
        fn splitstr_third_placeholder_is_named_filterstring_not_delimiter() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/splitstr.htm
            let completions = CompletionProvider::get_builtin_function_completions();
            let splitstr = completions.iter().find(|c| c.label == "SplitStr").unwrap();

            assert!(
                splitstr
                    .insert_text
                    .as_ref()
                    .unwrap()
                    .contains("${3:FilterString}")
            );
        }

        #[test]
        fn timeintointerval_snippet_includes_the_leading_tintoint_parameter() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/timeintointervaliftime.htm:
            // TimeIntoInterval and IfTime are the same instruction and take
            // the same 3 parameters, but this snippet was missing the
            // leading `TintoInt` parameter.
            let completions = CompletionProvider::get_builtin_function_completions();
            let tii = completions
                .iter()
                .find(|c| c.label == "TimeIntoInterval")
                .unwrap();

            assert_eq!(
                tii.insert_text.as_deref(),
                Some("TimeIntoInterval(${1:TintoInt}, ${2:Interval}, ${3:Units})")
            );
        }

        #[test]
        fn delay_snippet_includes_the_leading_option_parameter() {
            // Confirmed at
            // https://help.campbellsci.com/crbasic/cr6/Content/Instructions/delay3.htm:
            // `Delay(Option, Delay, Units)` -- `Option` was entirely
            // missing from this snippet.
            let completions = CompletionProvider::get_builtin_function_completions();
            let delay = completions.iter().find(|c| c.label == "Delay").unwrap();

            assert_eq!(
                delay.insert_text.as_deref(),
                Some("Delay(${1:Option}, ${2:Duration}, ${3:Units})")
            );
        }

        #[test]
        fn builtin_functions_have_documentation() {
            let completions = CompletionProvider::get_builtin_function_completions();
            let scan = completions.iter().find(|c| c.label == "Scan").unwrap();

            assert!(scan.documentation.is_some());
        }

        fn insert_text_for<'a>(completions: &'a [CompletionItem], label: &str) -> &'a str {
            completions
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("no completion found for {label}"))
                .insert_text
                .as_deref()
                .unwrap_or_else(|| panic!("{label} completion has no insert_text"))
        }

        #[test]
        fn battery_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "Battery"),
                "Battery(${1:Dest})"
            );
        }

        #[test]
        fn paneltemp_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "PanelTemp"),
                "PanelTemp(${1:Dest}, ${2:fN1})"
            );
        }

        #[test]
        fn brhalf_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "BrHalf"),
                "BrHalf(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:ExChan}, \
                 ${6:MeasPEx}, ${7:ExmV}, ${8:RevEx}, ${9:SettlingTime}, ${10:fN1}, \
                 ${11:Mult}, ${12:Offset})"
            );
        }

        #[test]
        fn brfull_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "BrFull"),
                "BrFull(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:ExChan}, \
                 ${6:MeasPEx}, ${7:ExmV}, ${8:RevEx}, ${9:RevDiff}, ${10:SettlingTime}, \
                 ${11:fN1}, ${12:Mult}, ${13:Offset})"
            );
        }

        #[test]
        fn resistance_snippet_includes_the_optional_trailing_meascurrent_parameter() {
            // Confirmed at https://help.campbellsci.com/crbasic/cr6/Content/Instructions/resistance.htm:
            // `MeasCurrent` is documented as an optional trailing parameter,
            // not a separate overload.
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "Resistance"),
                "Resistance(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:DiffChan}, ${5:IexChan}, \
                 ${6:MeasPEx}, ${7:EXuA}, ${8:RevEx}, ${9:RevDiff}, ${10:SettlingTime}, \
                 ${11:fN1}, ${12:Mult}, ${13:Offset}, ${14:MeasCurrent})"
            );
        }

        #[test]
        fn periodavg_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "PeriodAvg"),
                "PeriodAvg(${1:Dest}, ${2:Reps}, ${3:Chan}, ${4:Option}, ${5:Cycles}, \
                 ${6:Timeout}, ${7:Mult}, ${8:Offset})"
            );
        }

        #[test]
        fn portset_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "PortSet"),
                "PortSet(${1:Port}, ${2:State}, ${3:Option})"
            );
        }

        #[test]
        fn pulseport_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "PulsePort"),
                "PulsePort(${1:Port}, ${2:Delay})"
            );
        }

        #[test]
        fn excitev_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "ExciteV"),
                "ExciteV(${1:ExChan}, ${2:ExmV}, ${3:Delay})"
            );
        }

        #[test]
        fn brhalf3w_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "BrHalf3W"),
                "BrHalf3W(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:ExChan}, \
                 ${6:MeasPEx}, ${7:ExmV}, ${8:RevEx}, ${9:SettlingTime}, ${10:fN1}, \
                 ${11:Mult}, ${12:Offset})"
            );
        }

        #[test]
        fn brhalf4w_snippet_includes_the_optional_trailing_returnv1_parameter() {
            // Confirmed at https://help.campbellsci.com/crbasic/cr6/Content/Instructions/brhalf4w.htm:
            // `ReturnV1` is documented as an optional trailing parameter.
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "BrHalf4W"),
                "BrHalf4W(${1:Dest}, ${2:Reps}, ${3:Range1}, ${4:Range2}, ${5:DiffChan}, \
                 ${6:ExChan}, ${7:MeasPEx}, ${8:ExmV}, ${9:RevEx}, ${10:RevDiff}, \
                 ${11:SettlingTime}, ${12:fN1}, ${13:Mult}, ${14:Offset}, ${15:ReturnV1})"
            );
        }

        #[test]
        fn brfull6w_snippet_includes_the_optional_trailing_returnv1_parameter() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "BrFull6W"),
                "BrFull6W(${1:Dest}, ${2:Reps}, ${3:Range1}, ${4:Range2}, ${5:DiffChan}, \
                 ${6:ExChan}, ${7:MeasPEx}, ${8:ExmV}, ${9:RevEx}, ${10:RevDiff}, \
                 ${11:SettlingTime}, ${12:fN1}, ${13:Mult}, ${14:Offset}, ${15:ReturnV1})"
            );
        }

        #[test]
        fn tcse_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "TCSE"),
                "TCSE(${1:Dest}, ${2:Reps}, ${3:Range}, ${4:SEChan}, ${5:TCType}, ${6:TRef}, \
                 ${7:MeasOff}, ${8:SettlingTime}, ${9:fN1}, ${10:Mult}, ${11:Offset})"
            );
        }

        #[test]
        fn serialinrecord_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "SerialInRecord"),
                "SerialInRecord(${1:COMPort}, ${2:Dest}, ${3:BeginWord}, ${4:NBytes}, \
                 ${5:EndWord}, ${6:NBytesReturned}, ${7:SerialInRecOption})"
            );
        }

        #[test]
        fn serialoutblock_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "SerialOutBlock"),
                "SerialOutBlock(${1:ComPort}, ${2:Expression}, ${3:NumberBytes})"
            );
        }

        #[test]
        fn serialflush_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "SerialFlush"),
                "SerialFlush(${1:ComPort})"
            );
        }

        #[test]
        fn modbusmaster_snippet_matches_the_current_modbusclient_signature() {
            // Campbell Scientific renamed this instruction ModbusClient;
            // ModbusMaster still compiles for backward compatibility with the
            // same parameter list. Confirmed at
            // https://help.campbellsci.com/crbasic/cr6/Content/Instructions/modbusclient.htm
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "ModbusMaster"),
                "ModbusMaster(${1:ResultCode}, ${2:ComPort}, ${3:BaudRate}, ${4:ModbusAddr}, \
                 ${5:Function}, ${6:Variable}, ${7:Start}, ${8:Length}, ${9:Tries}, \
                 ${10:TimeOut}, ${11:ModbusOption})"
            );
        }

        #[test]
        fn sdi12recorder_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "SDI12Recorder"),
                "SDI12Recorder(${1:Dest}, ${2:SDIPort}, ${3:SDIAddress}, ${4:SDICommand}, \
                 ${5:Multiplier}, ${6:Offset}, ${7:FillNAN}, ${8:WaitonTimeout})"
            );
        }

        #[test]
        fn tcpopen_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "TCPOpen"),
                "TCPOpen(${1:IPAddr}, ${2:TCPPort}, ${3:IPBuffer}, ${4:IPTimeOut}, \
                 ${5:ConnectHandle}, ${6:MaxConnect})"
            );
        }

        #[test]
        fn tcpclose_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "TCPClose"),
                "TCPClose(${1:TCPSocket})"
            );
        }

        #[test]
        fn udpopen_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "UDPOpen"),
                "UDPOpen(${1:IPAddr}, ${2:UDPPort}, ${3:IPBuffer}, ${4:IPVersion})"
            );
        }

        #[test]
        fn udpsocketopen_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "UDPSocketOpen"),
                "UDPSocketOpen(${1:SocketID}, ${2:Port}, ${3:RecvQueueSize}, ${4:Interface})"
            );
        }

        #[test]
        fn udpsocketsend_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "UDPSocketSend"),
                "UDPSocketSend(${1:BytesSent}, ${2:SocketID}, ${3:IPAddr}, ${4:Port}, \
                 ${5:Payload}, ${6:PayLoadLen})"
            );
        }

        #[test]
        fn udpsocketrecv_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "UDPSocketRecv"),
                "UDPSocketRecv(${1:BytesReceived}, ${2:SocketID}, ${3:InDatagram}, \
                 ${4:InDatagramLen}, ${5:RemoteIPAdd}, ${6:RemotePort}, ${7:Timeout})"
            );
        }

        #[test]
        fn udpsocketclose_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "UDPSocketClose"),
                "UDPSocketClose(${1:SocketID})"
            );
        }

        #[test]
        fn emailrelay_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "EmailRelay"),
                "EmailRelay(${1:ToAddr}, ${2:Subject}, ${3:Message}, ${4:ServerResponse}, \
                 ${5:Attach}, ${6:NumRecsOrTimeIntoInterval}, ${7:Interval}, \
                 ${8:IntervalUnits}, ${9:FileOption}, ${10:TimeOut})"
            );
        }

        #[test]
        fn pppopen_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "PPPOpen"),
                "PPPOpen(${1:Option})"
            );
        }

        #[test]
        fn pppclose_snippet_has_no_parameters() {
            // Confirmed at https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/pppclose.htm:
            // the syntax diagram shows `variable = PPPClose` with no arguments.
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "PPPClose"), "PPPClose()");
        }

        #[test]
        fn ftpclient_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FTPClient"),
                "FTPClient(${1:IPAddress}, ${2:User}, ${3:Password}, ${4:LocalFileName}, \
                 ${5:RemoteFileName}, ${6:PutGetOption}, ${7:NumRecsOrTimeIntoInterval}, \
                 ${8:Interval}, ${9:IntervalUnits}, ${10:FileOption}, ${11:TimeOut})"
            );
        }

        #[test]
        fn httpget_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "HTTPGet"),
                "HTTPGet(${1:URI}, ${2:Response}, ${3:Header}, ${4:TimeOut})"
            );
        }

        #[test]
        fn httppost_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "HTTPPost"),
                "HTTPPost(${1:URI}, ${2:Contents}, ${3:Response}, ${4:Header}, \
                 ${5:NumRecsOrTimeIntoInterval}, ${6:Interval}, ${7:IntervalUnits}, \
                 ${8:FileOption}, ${9:TimeOut})"
            );
        }

        #[test]
        fn httpput_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "HTTPPut"),
                "HTTPPut(${1:URI}, ${2:Contents}, ${3:Response}, ${4:Header}, \
                 ${5:NumRecsOrTimeIntoInterval}, ${6:Interval}, ${7:IntervalUnits}, \
                 ${8:FileOption}, ${9:TimeOut})"
            );
        }

        #[test]
        fn windvector_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "WindVector"),
                "WindVector(${1:Reps}, ${2:SpeedOrEast}, ${3:DirectionOrNorth}, \
                 ${4:DataType}, ${5:DisableVar}, ${6:Subinterval}, ${7:SensorType}, \
                 ${8:OutputOpt})"
            );
        }

        #[test]
        fn histogram_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "Histogram"),
                "Histogram(${1:BinSelect}, ${2:DataType}, ${3:DisableVar}, ${4:Bins}, \
                 ${5:Form}, ${6:WtVal}, ${7:LoLim}, ${8:UpLim})"
            );
        }

        #[test]
        fn fieldnames_snippet_takes_a_single_comma_separated_string_parameter() {
            // Confirmed at https://help.campbellsci.com/crbasic/cr6/Content/Instructions/fieldnames.htm:
            // FieldNames takes exactly one quoted, comma-separated string
            // ("Fieldname1:Description1,Fieldname2:Description2..."), not a
            // multi-parameter list.
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FieldNames"),
                "FieldNames(${1:FieldNameDescriptionList})"
            );
        }

        #[test]
        fn cardout_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "CardOut"),
                "CardOut(${1:StopRing}, ${2:Size})"
            );
        }

        #[test]
        fn newfile_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "NewFile"),
                "NewFile(${1:NewFileVar}, ${2:FileName}, ${3:NewFileName})"
            );
        }

        #[test]
        fn filemanage_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileManage"),
                "FileManage(${1:DeviceFileName}, ${2:Attribute})"
            );
        }

        #[test]
        fn fileopen_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileOpen"),
                "FileOpen(${1:FileName}, ${2:Mode}, ${3:SeekPoint})"
            );
        }

        #[test]
        fn fileclose_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileClose"),
                "FileClose(${1:FileHandle})"
            );
        }

        #[test]
        fn fileread_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileRead"),
                "FileRead(${1:FileHandle}, ${2:Destination}, ${3:Length})"
            );
        }

        #[test]
        fn filewrite_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileWrite"),
                "FileWrite(${1:FileHandle}, ${2:Source}, ${3:Length})"
            );
        }

        #[test]
        fn filecopy_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileCopy"),
                "FileCopy(${1:FromFileName}, ${2:ToFileName})"
            );
        }

        #[test]
        fn filerename_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileRename"),
                "FileRename(${1:OldFileName}, ${2:NewFileName})"
            );
        }

        #[test]
        fn filesize_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileSize"),
                "FileSize(${1:FileHandle})"
            );
        }

        #[test]
        fn filetime_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileTime"),
                "FileTime(${1:FileHandle})"
            );
        }

        #[test]
        fn filelist_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FileList"),
                "FileList(${1:Device}, ${2:Dest})"
            );
        }

        #[test]
        fn datainterval_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "DataInterval"),
                "DataInterval(${1:TintoInt}, ${2:Interval}, ${3:Units}, ${4:Lapses})"
            );
        }

        #[test]
        fn formatlong_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "FormatLong"),
                "FormatLong(${1:LongVar}, ${2:FormatString})"
            );
        }

        #[test]
        fn chr_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Chr"), "Chr(${1:Code})");
        }

        #[test]
        fn ascii_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "ASCII"),
                "ASCII(${1:ASCIIString})"
            );
        }

        #[test]
        fn strcomp_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "StrComp"),
                "StrComp(${1:String1}, ${2:String2})"
            );
        }

        #[test]
        fn checksum_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "CheckSum"),
                "CheckSum(${1:ChkSumString}, ${2:ChkSumType}, ${3:CheckSumSize}, \
                 ${4:ChkSumOption1}, ${5:ChkSumOption2}, ${6:ChkSumOption3})"
            );
        }

        #[test]
        fn hextodec_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "HexToDec"),
                "HexToDec(${1:Expression})"
            );
        }

        #[test]
        fn hex_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Hex"), "Hex(${1:Expression})");
        }

        #[test]
        fn sprintf_snippet_matches_official_signature() {
            // Confirmed at https://help.campbellsci.com/crbasic/cr6/Content/Instructions/sprintf.htm:
            // Sprintf takes 1-10 variadic value arguments after the format
            // string; only the first is placeholder'd here, matching how a
            // call site is typically extended by hand.
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "Sprintf"),
                "Sprintf(${1:Dest}, ${2:Format}, ${3:Argument1})"
            );
        }

        #[test]
        fn sgn_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Sgn"), "Sgn(${1:Value})");
        }

        #[test]
        fn ln_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Ln"), "Ln(${1:Value})");
        }

        #[test]
        fn sinh_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Sinh"), "Sinh(${1:Value})");
        }

        #[test]
        fn cosh_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Cosh"), "Cosh(${1:Value})");
        }

        #[test]
        fn tanh_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Tanh"), "Tanh(${1:Value})");
        }

        #[test]
        fn frac_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Frac"), "Frac(${1:Value})");
        }

        #[test]
        fn rnd_snippet_takes_no_parentheses() {
            // Confirmed at https://help.campbellsci.com/crbasic/cr6/Content/Instructions/rnd.htm:
            // the documented syntax is `variable = RND` with no arguments
            // and no parentheses at all -- seeding is a separate `Randomize`
            // instruction, not a parameter to `RND`.
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Rnd"), "Rnd");
        }

        #[test]
        fn ceiling_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(
                insert_text_for(&completions, "Ceiling"),
                "Ceiling(${1:Value})"
            );
        }

        #[test]
        fn floor_snippet_matches_official_signature() {
            let completions = CompletionProvider::get_builtin_function_completions();

            assert_eq!(insert_text_for(&completions, "Floor"), "Floor(${1:Value})");
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
                        type_size: None,
                        initializer: None,
                        span: dummy_span(),
                    },
                    Statement::VarDeclaration {
                        keyword: "Const".to_string(),
                        name: "PI".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
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
                    type_size: None,
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
