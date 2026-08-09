//! LSP Server for CRBasic
//!
//! This crate provides a Language Server Protocol implementation for CRBasic,
//! enabling IDE features like diagnostics, completion, and go-to-definition.

pub mod backend;
pub mod code_action;
pub mod completion;
pub mod definition;
pub mod document;
pub mod document_highlight;
pub mod hover;
pub mod references;
pub mod rename;
pub mod semantic_tokens;
pub mod signature;
pub mod symbols;

pub use backend::CRBasicLanguageServer;
pub use document::DocumentManager;
