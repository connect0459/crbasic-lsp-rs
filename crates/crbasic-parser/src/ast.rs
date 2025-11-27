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
    /// Power operator (^)
    Power,
    /// Modulo operator (MOD)
    Modulo,

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

    /// Logical AND operator
    And,
    /// Logical OR operator
    Or,
    /// Logical XOR operator
    Xor,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Negation operator (-)
    Negate,
    /// Logical NOT operator
    Not,
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
        }
    }
}
