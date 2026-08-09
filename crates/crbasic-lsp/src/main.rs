//! CRBasic LSP Server Entry Point
//!
//! This binary starts the CRBasic Language Server using stdio transport.

use crbasic_lsp::CRBasicLanguageServer;
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() {
    // Set up stderr for logging (stdout is used for LSP communication)
    eprintln!("Starting CRBasic Language Server...");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(CRBasicLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    eprintln!("CRBasic Language Server stopped.");
}
