//! CRBasic Parser Library
//!
//! This library provides lexical analysis and parsing for the CRBasic programming language,
//! which is used in Campbell Scientific data loggers.

pub mod lexer;

pub use lexer::{Token, TokenKind};
