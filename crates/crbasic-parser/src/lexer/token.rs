//! Token definitions for CRBasic lexer
//!
//! This module defines the token types and structures used in lexical analysis.

use serde::{Deserialize, Serialize};

/// Represents a token's position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Position {
    /// Creates a new position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Represents a span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Starting position
    pub start: Position,
    /// Ending position
    pub end: Position,
}

impl Span {
    /// Creates a new span
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// Token kinds in CRBasic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    // Literals
    /// Integer literal (e.g., 42, 1000)
    Integer(String),
    /// Float literal (e.g., 3.14, 1.0e-5)
    Float(String),
    /// String literal (e.g., "Hello")
    String(String),

    // Identifiers and Keywords
    /// Identifier (variable names, function names)
    Identifier(String),
    /// Keyword (e.g., If, For, Public)
    Keyword(String),

    // Comments
    /// Single-line comment starting with '
    Comment(String),

    // Operators
    /// Addition operator (+)
    Plus,
    /// Subtraction operator (-)
    Minus,
    /// Multiplication operator (*)
    Star,
    /// Division operator (/)
    Slash,
    /// Power operator (^)
    Caret,
    /// Assignment/Equality operator (=)
    Equal,
    /// Less than (<)
    LessThan,
    /// Greater than (>)
    GreaterThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,
    /// Not equal (<>)
    NotEqual,

    // Delimiters
    /// Left parenthesis (
    LeftParen,
    /// Right parenthesis )
    RightParen,
    /// Left bracket [
    LeftBracket,
    /// Right bracket ]
    RightBracket,
    /// Comma ,
    Comma,

    // Special
    /// Line continuation (space + underscore at end of line)
    LineContinuation,
    /// Newline
    Newline,
    /// End of file
    Eof,
}

/// A token with its kind, lexeme, and position
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The kind of token (e.g., Integer, Keyword, Operator)
    pub kind: TokenKind,
    /// The original text from source code
    pub lexeme: String,
    /// The source code location of this token
    pub span: Span,
}

impl Token {
    /// Creates a new token
    pub fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
        Self { kind, lexeme, span }
    }
}
