//! Abstract Syntax Tree (AST) definitions for CRBasic
//!
//! This module defines the AST node types that represent the structure of CRBasic programs.

use crate::lexer::token::Span;
use serde::{Deserialize, Serialize};

/// A complete CRBasic program
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A statement in CRBasic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// Variable declaration: Public/Dim/Const identifier [As type]
    VarDeclaration {
        keyword: String, // "Public", "Dim", "Const"
        name: String,
        type_annotation: Option<String>,
        span: Span,
    },

    /// Assignment: identifier = expression
    Assignment {
        target: String,
        value: Expression,
        span: Span,
    },

    /// If-Then-Else statement
    IfStatement {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        span: Span,
    },

    /// For-Next loop
    ForLoop {
        variable: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        body: Vec<Statement>,
        span: Span,
    },

    /// Do-Loop
    DoLoop {
        condition: Option<Expression>,
        condition_at_start: bool, // true for While, false for Until
        body: Vec<Statement>,
        span: Span,
    },

    /// Function call as statement
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
        span: Span,
    },

    /// Program structure: BeginProg/EndProg, DataTable/EndTable, etc.
    ProgramStructure {
        keyword: String, // "BeginProg", "EndProg", "DataTable", "EndTable", etc.
        span: Span,
    },
}

/// An expression in CRBasic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Integer literal
    IntegerLiteral { value: i64, span: Span },

    /// Float literal
    FloatLiteral { value: f64, span: Span },

    /// String literal
    StringLiteral { value: String, span: Span },

    /// Identifier (variable reference)
    Identifier { name: String, span: Span },

    /// Binary operation (e.g., a + b, x > 5)
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },

    /// Unary operation (e.g., -x, NOT flag)
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },

    /// Function call
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
        span: Span,
    },

    /// Array access: array[index]
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    // Logical
    And,
    Or,
    Xor,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Negate, // -
    Not,    // NOT
}

impl Program {
    /// Creates a new Program node
    pub fn new(statements: Vec<Statement>, span: Span) -> Self {
        Self { statements, span }
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
            Expression::Identifier { span, .. } => *span,
            Expression::BinaryOp { span, .. } => *span,
            Expression::UnaryOp { span, .. } => *span,
            Expression::FunctionCall { span, .. } => *span,
            Expression::ArrayAccess { span, .. } => *span,
        }
    }
}
