//! CRBasic Parser Library
//!
//! This library provides lexical analysis and parsing for the CRBasic programming language,
//! which is used in Campbell Scientific data loggers.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod semantic;

pub use lexer::{Token, TokenKind};
pub use parser::{ParseError, Parser};
pub use semantic::{
    DataloggerModel, SemanticAnalyzer, SemanticError, SemanticErrorKind, ValidationProfile,
    VariableScope,
};
