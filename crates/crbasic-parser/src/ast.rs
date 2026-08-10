//! Abstract Syntax Tree (AST) definitions for CRBasic
//!
//! This module defines the AST node types that represent the structure of CRBasic programs.

use crate::lexer::token::Span;
use serde::{Deserialize, Serialize};

/// A complete CRBasic program
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// The top-level statements in the program
    pub statements: Vec<Statement>,
    /// The source code span of the entire program
    pub span: Span,
}

/// Target for assignment statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignmentTarget {
    /// Simple identifier: x = 5
    Identifier {
        /// The variable name
        name: String,
        /// The source code span
        span: Span,
    },

    /// Array element access: `Data[0] = 5` or `Matrix[1][2] = 10`
    ArrayElement {
        /// The array variable name
        array: String,
        /// The index expressions (one per dimension)
        indices: Vec<Expression>,
        /// The source code span
        span: Span,
    },
}

/// A statement in CRBasic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// Variable declaration: Public/Dim/Const identifier[(dimensions)] [As type] [= initializer]
    VarDeclaration {
        /// The declaration keyword: "Public", "Dim", or "Const"
        keyword: String,
        /// The variable name
        name: String,
        /// Array dimensions if this is an array (e.g., `Data(100)` or `Matrix(10, 20)`)
        array_dimensions: Option<Vec<Expression>>,
        /// Optional type annotation (e.g., "Float", "String")
        type_annotation: Option<String>,
        /// Optional fixed size for a `String` type annotation
        /// (e.g., the `30` in `As String * 30`)
        type_size: Option<Expression>,
        /// Optional initializer expression (required for Const, optional for Public/Dim)
        initializer: Option<Expression>,
        /// The source code span
        span: Span,
    },

    /// Assignment: `identifier = expression` or `array[index] = expression`
    Assignment {
        /// The target of the assignment (identifier or array element)
        target: AssignmentTarget,
        /// The value expression
        value: Expression,
        /// The source code span
        span: Span,
    },

    /// If-Then-Else statement
    IfStatement {
        /// The condition expression
        condition: Expression,
        /// Statements to execute if condition is true
        then_branch: Vec<Statement>,
        /// Optional else branch statements
        else_branch: Option<Vec<Statement>>,
        /// The source code span
        span: Span,
    },

    /// Preprocessor conditional block: `#If`/`#IfDef` ... [`#ElseIf` ...]
    /// ... [`#Else` ...] `#EndIf`
    ///
    /// Parsed structurally only -- the condition is never evaluated, the
    /// same way a runtime `IfStatement`'s condition is never evaluated.
    /// Kept as its own variant rather than reusing `IfStatement` because it
    /// is a genuinely different construct (compile-time conditional
    /// compilation, not a runtime branch), and unlike `If`, its `Then`
    /// keyword is optional.
    PreprocessorConditional {
        /// Which directive introduced this block: `"If"` or `"IfDef"`
        directive: String,
        /// The condition expression for `#If`, or the `Const` name being
        /// checked for `#IfDef`
        condition: Expression,
        /// Statements included when the condition holds
        then_branch: Vec<Statement>,
        /// Optional statements for `#Else`, or a single nested
        /// `PreprocessorConditional` for a chained `#ElseIf`
        else_branch: Option<Vec<Statement>>,
        /// The source code span
        span: Span,
    },

    /// For-Next loop
    ForLoop {
        /// The loop variable name
        variable: String,
        /// Starting value expression
        start: Expression,
        /// Ending value expression
        end: Expression,
        /// Optional step value expression
        step: Option<Expression>,
        /// Loop body statements
        body: Vec<Statement>,
        /// The source code span
        span: Span,
    },

    /// Do-Loop
    DoLoop {
        /// Optional loop condition
        condition: Option<Expression>,
        /// Whether condition is at start (While) or end (Until)
        condition_at_start: bool,
        /// Loop body statements
        body: Vec<Statement>,
        /// The source code span
        span: Span,
    },

    /// `Select Case` multi-way branch statement
    SelectCase {
        /// The expression each `Case` clause's conditions are tested against
        test_expression: Expression,
        /// The `Case` clauses, in source order
        cases: Vec<CaseClause>,
        /// Optional `Case Else` body
        else_branch: Option<Vec<Statement>>,
        /// The source code span
        span: Span,
    },

    /// Function call as statement
    FunctionCall {
        /// The function name
        name: String,
        /// The argument expressions
        arguments: Vec<Expression>,
        /// The source code span
        span: Span,
    },

    /// Expression statement (for testing and future extensions)
    /// In CRBasic, most expressions are not valid as statements,
    /// but this allows for more flexible parsing during development
    Expression {
        /// The expression
        expression: Expression,
        /// The source code span
        span: Span,
    },

    /// Program structure: BeginProg/EndProg, DataTable/EndTable, etc.
    ProgramStructure {
        /// The structure keyword (e.g., "BeginProg", "EndProg", "DataTable", "EndTable")
        keyword: String,
        /// Optional arguments (e.g., for DataTable: table_name, autoallocate, size)
        arguments: Option<Vec<Expression>>,
        /// The source code span
        span: Span,
    },

    /// Function definition
    FunctionDefinition {
        /// The function name
        name: String,
        /// The parameter names
        parameters: Vec<String>,
        /// The function body statements
        body: Vec<Statement>,
        /// The source code span
        span: Span,
    },

    /// Subroutine definition
    SubroutineDefinition {
        /// The subroutine name
        name: String,
        /// The parameter names
        parameters: Vec<String>,
        /// The subroutine body statements
        body: Vec<Statement>,
        /// The source code span
        span: Span,
    },

    /// `Alias` statement: gives one or more alternate names to a variable.
    /// Syntax: `Alias VariableName = AliasName [, AliasName...]`
    ///
    /// Either side may use CRBasic's parenthesized subscript form
    /// (`TCTemp(1)`, `Array()`), which this project's grammar already
    /// represents as an ordinary `Expression::FunctionCall` -- so `variable`
    /// and each entry in `names` are plain expressions rather than a new
    /// indexed-name type.
    Alias {
        /// The variable being aliased
        variable: Expression,
        /// One or more alias names, in source order
        names: Vec<Expression>,
        /// The source code span
        span: Span,
    },

    /// `Units` statement: assigns an engineering-units label to a variable.
    /// Syntax: `Units VariableName = UnitLabel` or `Units VariableName() = UnitLabel`
    Units {
        /// The variable being labeled
        variable: Expression,
        /// The unit label
        unit: Expression,
        /// The source code span
        span: Span,
    },

    /// `ReadOnly` statement: marks one or more previously declared `Public`
    /// variables as visible for monitoring but not externally editable.
    /// Syntax: `ReadOnly VariableName [, VariableName...]`
    ///
    /// See `Alias`'s doc comment for why entries are plain expressions
    /// rather than a new indexed-name type.
    ReadOnly {
        /// The variables marked read-only, in source order
        variables: Vec<Expression>,
        /// The source code span
        span: Span,
    },

    /// `StructureType`/`EndStructureType` block: defines a reusable data
    /// structure. Instances are declared via `Public`/`Dim ... As
    /// StructureTypeName` (an ordinary `VarDeclaration` with a
    /// `type_annotation` naming the structure type) and members are read
    /// via dot notation (`StructureName.MemberName`, see
    /// `Expression::MemberAccess`).
    /// See <https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/structuretype.htm>
    StructureType {
        /// The structure type's name
        name: String,
        /// The member declarations and modifiers, in source order
        members: Vec<StructureMember>,
        /// The source code span
        span: Span,
    },
}

/// A single member declaration or modifier inside a `StructureType` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureMember {
    /// A bare `Name [(size)] As Type [* length]` member declaration -- no
    /// `Public`/`Dim`/`Const` prefix, since a `StructureType` block defines
    /// a type's shape, not a variable instance.
    Declaration {
        /// The member name
        name: String,
        /// Array dimensions if this member is an array (e.g. `NMEASentences(2)`)
        array_dimensions: Option<Vec<Expression>>,
        /// The member's type annotation (e.g. "Float", "String")
        type_annotation: String,
        /// Optional fixed size for a `String` type annotation
        /// (e.g. the `110` in `As String * 110`)
        type_size: Option<Expression>,
        /// The source code span
        span: Span,
    },
    /// A nested `Units`/`ReadOnly` modifier applying to one of this
    /// structure's members, parsed via the same statement grammar used at
    /// the top level.
    Modifier(Statement),
}

/// A single `Case` clause within a `Select Case` statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseClause {
    /// The comma-separated conditions in this clause's `ExpressionList`
    pub conditions: Vec<CaseCondition>,
    /// The statements to execute when this clause matches
    pub body: Vec<Statement>,
    /// The source code span
    pub span: Span,
}

/// A single condition within a `Case` clause's `ExpressionList`, per
/// Campbell Scientific's `Select Case` syntax
/// (<https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/selectcase.htm>)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaseCondition {
    /// A plain value tested for equality against the `Select Case` test
    /// expression (e.g. `Case 99`)
    Value(Expression),
    /// An inclusive range (e.g. `Case 1 To 20`)
    Range(Expression, Expression),
    /// `Is comparison-operator Expression` (e.g. `Case Is > 99`), tested
    /// against the `Select Case` test expression
    Compare {
        /// The comparison operator (always one of the six comparison
        /// variants of `BinaryOperator`)
        operator: BinaryOperator,
        /// The expression to compare the test expression against
        expression: Expression,
    },
    /// `And`/`Or`-combined conditions, for chained `Is` forms
    /// (e.g. `Case Is >= 0 And Is <= 11.25`)
    Logical {
        /// The logical operator (always `And` or `Or`)
        operator: BinaryOperator,
        /// The left-hand condition
        left: Box<CaseCondition>,
        /// The right-hand condition
        right: Box<CaseCondition>,
    },
}

/// An expression in CRBasic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Integer literal
    IntegerLiteral {
        /// The integer value
        value: i64,
        /// The source code span
        span: Span,
    },

    /// Float literal
    FloatLiteral {
        /// The floating-point value
        value: f64,
        /// The source code span
        span: Span,
    },

    /// String literal
    StringLiteral {
        /// The string value
        value: String,
        /// The source code span
        span: Span,
    },

    /// Boolean literal (True/False)
    BooleanLiteral {
        /// The boolean value
        value: bool,
        /// The source code span
        span: Span,
    },

    /// Identifier (variable reference)
    Identifier {
        /// The variable name
        name: String,
        /// The source code span
        span: Span,
    },

    /// Binary operation (e.g., a + b, x > 5)
    BinaryOp {
        /// The left operand
        left: Box<Expression>,
        /// The binary operator
        operator: BinaryOperator,
        /// The right operand
        right: Box<Expression>,
        /// The source code span
        span: Span,
    },

    /// Unary operation (e.g., -x, NOT flag)
    UnaryOp {
        /// The unary operator
        operator: UnaryOperator,
        /// The operand
        operand: Box<Expression>,
        /// The source code span
        span: Span,
    },

    /// Function call
    FunctionCall {
        /// The function name
        name: String,
        /// The argument expressions
        arguments: Vec<Expression>,
        /// The source code span
        span: Span,
    },

    /// Array access: `array[index]`
    ArrayAccess {
        /// The array expression
        array: Box<Expression>,
        /// The index expression
        index: Box<Expression>,
        /// The source code span
        span: Span,
    },

    /// Member access on a `StructureType` instance: `object.member`
    /// (e.g. `CS215.Temp`, `CS215(1).Temp`).
    /// See <https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/structuretype.htm>
    MemberAccess {
        /// The structure instance expression (identifier or indexed element)
        object: Box<Expression>,
        /// The member name being accessed
        member: String,
        /// The source code span
        span: Span,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// Addition operator (+)
    Add,
    /// Subtraction operator (-)
    Subtract,
    /// Multiplication operator (*)
    Multiply,
    /// Division operator (/)
    Divide,
    /// Integer division operator (\)
    IntegerDivide,
    /// Power operator (^)
    Power,
    /// Modulo operator (MOD)
    Modulo,
    /// String concatenation operator (&)
    Concatenate,

    /// Equality operator (=)
    Equal,
    /// Inequality operator (<>)
    NotEqual,
    /// Less than operator (<)
    LessThan,
    /// Greater than operator (>)
    GreaterThan,
    /// Less than or equal operator (<=)
    LessThanOrEqual,
    /// Greater than or equal operator (>=)
    GreaterThanOrEqual,

    /// Bit-shift left operator (<<)
    LeftShift,
    /// Bit-shift right operator (>>)
    RightShift,

    /// Logical AND operator
    And,
    /// Logical OR operator
    Or,
    /// Logical XOR operator
    Xor,
    /// Logical implication operator (IMP)
    Implication,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Negation operator (-)
    Negate,
    /// Logical NOT operator
    Not,
    /// Address-of pointer operator (@)
    AddressOf,
    /// Dereference pointer operator (!)
    Dereference,
}

impl Program {
    /// Creates a new Program node
    pub fn new(statements: Vec<Statement>, span: Span) -> Self {
        Self { statements, span }
    }
}

impl AssignmentTarget {
    /// Gets the span of this assignment target
    pub fn span(&self) -> Span {
        match self {
            AssignmentTarget::Identifier { span, .. } => *span,
            AssignmentTarget::ArrayElement { span, .. } => *span,
        }
    }
}

impl Expression {
    /// Creates an integer literal expression
    pub fn integer(value: i64, span: Span) -> Self {
        Expression::IntegerLiteral { value, span }
    }

    /// Creates a float literal expression
    pub fn float(value: f64, span: Span) -> Self {
        Expression::FloatLiteral { value, span }
    }

    /// Creates a string literal expression
    pub fn string(value: String, span: Span) -> Self {
        Expression::StringLiteral { value, span }
    }

    /// Creates a boolean literal expression
    pub fn boolean(value: bool, span: Span) -> Self {
        Expression::BooleanLiteral { value, span }
    }

    /// Creates an identifier expression
    pub fn identifier(name: String, span: Span) -> Self {
        Expression::Identifier { name, span }
    }

    /// Gets the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expression::IntegerLiteral { span, .. } => *span,
            Expression::FloatLiteral { span, .. } => *span,
            Expression::StringLiteral { span, .. } => *span,
            Expression::BooleanLiteral { span, .. } => *span,
            Expression::Identifier { span, .. } => *span,
            Expression::BinaryOp { span, .. } => *span,
            Expression::UnaryOp { span, .. } => *span,
            Expression::FunctionCall { span, .. } => *span,
            Expression::ArrayAccess { span, .. } => *span,
            Expression::MemberAccess { span, .. } => *span,
        }
    }
}
