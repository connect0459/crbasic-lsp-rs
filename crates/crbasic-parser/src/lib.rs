//! CRBasic Parser Library
//!
//! This library provides lexical analysis and parsing for the CRBasic programming language,
//! which is used in Campbell Scientific data loggers.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use lexer::{Token, TokenKind};
pub use parser::{ParseError, Parser};
