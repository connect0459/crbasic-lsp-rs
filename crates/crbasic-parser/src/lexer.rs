//! Lexical analysis module for CRBasic
//!
//! This module provides tokenization of CRBasic source code.

pub mod scanner;
pub mod token;

pub use scanner::Scanner;
pub use token::{Token, TokenKind};
