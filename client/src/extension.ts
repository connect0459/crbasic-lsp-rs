/**
 * CRBasic LSP VSCode Extension Entry Point
 *
 * This extension provides language support for CRBasic, the programming language
 * used in Campbell Scientific data loggers.
 */

import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { restartServer, showServerOutput } from "./commands";

let client: LanguageClient | undefined;

/**
 * Gets the path to the language server executable
 *
 * @param context - The extension context
 * @returns The path to the server executable
 */
function getServerPath(context: vscode.ExtensionContext): string {
  // Check for custom server path in settings
  const config = vscode.workspace.getConfiguration("crbasic");
  const customPath = config.get<string>("server.path");

  if (customPath) {
    return customPath;
  }

  // Default: bundled server in extension's server directory
  const serverName = process.platform === "win32" ? "crbasic-lsp.exe" : "crbasic-lsp";
  return path.join(context.extensionPath, "server", serverName);
}

/**
 * Creates the language client
 *
 * @param context - The extension context
 * @returns The configured language client
 */
function createLanguageClient(context: vscode.ExtensionContext): LanguageClient {
  const serverPath = getServerPath(context);

  // Server options: run the LSP server binary
  const serverOptions: ServerOptions = {
    run: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
  };

  // Client options: specify which documents to sync
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "crbasic" }],
    synchronize: {
      // Watch for changes to CRBasic files
      fileEvents: vscode.workspace.createFileSystemWatcher(
        "**/*.{cr1,cr1x,cr2,cr3,cr5,cr6,cr8,cr9,cr9x,c9x,cr300,crb,dld}"
      ),
    },
  };

  return new LanguageClient("crbasic-lsp", "CRBasic Language Server", serverOptions, clientOptions);
}

/**
 * Reports a command result to the user via VSCode's message API
 *
 * @param result - The outcome returned by a command handler
 */
function reportCommandResult(result: { ok: boolean; message?: string }): void {
  if (!result.message) {
    return;
  }

  if (result.ok) {
    void vscode.window.showInformationMessage(result.message);
  } else {
    void vscode.window.showWarningMessage(result.message);
  }
}

/**
 * Registers the extension's commands (Command Palette entries)
 *
 * @param context - The extension context provided by VSCode
 */
function registerCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("crbasic.restartServer", async () => {
      reportCommandResult(await restartServer(client));
    }),
    vscode.commands.registerCommand("crbasic.showServerOutput", () => {
      reportCommandResult(showServerOutput(client));
    })
  );
}

/**
 * Extension activation function
 *
 * Called when the extension is activated (when a CRBasic file is opened).
 *
 * @param context - The extension context provided by VSCode
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  console.log("CRBasic LSP extension is activating...");

  registerCommands(context);

  try {
    // Create and start the language client
    client = createLanguageClient(context);

    // Start the client (this also starts the server)
    await client.start();

    console.log("CRBasic LSP extension is now active");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Failed to start CRBasic Language Server: ${message}`);

    // Show error message to user
    void vscode.window.showErrorMessage(
      `CRBasic Language Server failed to start: ${message}. ` +
        "Please check that the server binary is installed correctly."
    );
  }
}

/**
 * Extension deactivation function
 *
 * Called when the extension is deactivated.
 */
export async function deactivate(): Promise<void> {
  console.log("CRBasic LSP extension is deactivating...");

  if (client) {
    await client.stop();
    client = undefined;
  }

  console.log("CRBasic LSP extension is now deactivated");
}
