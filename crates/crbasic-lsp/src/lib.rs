//! LSP Server for CRBasic
//!
//! This crate provides a Language Server Protocol implementation for CRBasic,
//! enabling IDE features like diagnostics, completion, and go-to-definition.

pub mod backend;
pub mod call_sites;
pub mod code_action;
pub mod code_lens;
pub mod completion;
pub mod definition;
pub mod document;
pub mod document_highlight;
pub mod folding;
pub mod hover;
pub mod inlay_hint;
pub mod references;
pub mod rename;
pub mod semantic_tokens;
pub mod signature;
pub mod symbols;
pub mod workspace_symbol;

pub use backend::CRBasicLanguageServer;
pub use document::DocumentManager;
