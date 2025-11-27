//! LSP Server for CRBasic
//!
//! This crate provides a Language Server Protocol implementation for CRBasic,
//! enabling IDE features like diagnostics, completion, and go-to-definition.

pub mod backend;
pub mod completion;
pub mod document;
pub mod hover;
pub mod signature;
pub mod symbols;

pub use backend::CRBasicLanguageServer;
pub use document::DocumentManager;
