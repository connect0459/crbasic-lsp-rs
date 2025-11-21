/**
 * CRBasic LSP VSCode Extension Entry Point
 *
 * This extension provides language support for CRBasic, the programming language
 * used in Campbell Scientific data loggers.
 */

import * as vscode from "vscode";

/**
 * Extension activation function
 *
 * Called when the extension is activated (when a CRBasic file is opened).
 *
 * @param context - The extension context provided by VSCode
 */
export function activate(context: vscode.ExtensionContext): void {
  console.log("CRBasic LSP extension is now active");

  // TODO: Initialize LSP client here
  // For now, only TextMate Grammar syntax highlighting is active

  // Register a simple command for testing
  const disposable = vscode.commands.registerCommand("crbasic-lsp.helloWorld", () => {
    void vscode.window.showInformationMessage("Hello from CRBasic LSP!");
  });

  context.subscriptions.push(disposable);
}

/**
 * Extension deactivation function
 *
 * Called when the extension is deactivated.
 */
export function deactivate(): void {
  console.log("CRBasic LSP extension is now deactivated");
}
