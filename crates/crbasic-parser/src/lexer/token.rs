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
///
/// Literal/identifier/keyword/comment text borrows directly from the source
/// string (`&'a str`) to avoid a per-token allocation; only `String` (the
/// escape-resolved value of a string literal) must own its data, since
/// escape processing can produce text that no longer matches any source
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TokenKind<'a> {
    /// Integer literal (e.g., 42, 1000)
    Integer(&'a str),
    /// Float literal (e.g., 3.14, 1.0e-5)
    Float(&'a str),
    /// String literal (e.g., "Hello")
    String(String),

    /// Identifier (variable names, function names)
    Identifier(&'a str),
    /// Keyword (e.g., If, For, Public)
    Keyword(&'a str),

    /// Single-line comment starting with '
    Comment(&'a str),

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
    /// String concatenation operator (&)
    Ampersand,

    /// Compound add-assign (+=)
    PlusEqual,
    /// Compound subtract-assign (-=)
    MinusEqual,
    /// Compound multiply-assign (*=)
    StarEqual,
    /// Compound divide-assign (/=)
    SlashEqual,
    /// Compound power-assign (^=)
    CaretEqual,
    /// Compound concatenate-assign (&=)
    AmpersandEqual,
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
    /// Statement separator (:), allowing multiple statements on one line
    Colon,

    /// Line continuation (space + underscore at end of line)
    LineContinuation,
    /// Newline
    Newline,
    /// End of file
    Eof,
}

/// A token with its kind, lexeme, and position
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token<'a> {
    /// The kind of token (e.g., Integer, Keyword, Operator)
    pub kind: TokenKind<'a>,
    /// The original text from source code
    pub lexeme: &'a str,
    /// The source code location of this token
    pub span: Span,
}

impl<'a> Token<'a> {
    /// Creates a new token
    pub fn new(kind: TokenKind<'a>, lexeme: &'a str, span: Span) -> Self {
        Self { kind, lexeme, span }
    }
}
