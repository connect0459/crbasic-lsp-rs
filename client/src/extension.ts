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
import {
  restartServer,
  showServerOutput,
  parseShowReferencesArguments,
  LspLocation,
  LspPosition,
} from "./commands";

let client: LanguageClient | undefined;

/**
 * Converts an LSP-shaped position into a real `vscode.Position` instance
 *
 * @param position - The plain JSON position received over LSP
 * @returns The equivalent `vscode.Position`
 */
function toVscodePosition(position: LspPosition): vscode.Position {
  return new vscode.Position(position.line, position.character);
}

/**
 * Converts an LSP-shaped location into a real `vscode.Location` instance
 *
 * @param location - The plain JSON location received over LSP
 * @returns The equivalent `vscode.Location`
 */
function toVscodeLocation(location: LspLocation): vscode.Location {
  return new vscode.Location(
    vscode.Uri.parse(location.uri),
    new vscode.Range(toVscodePosition(location.range.start), toVscodePosition(location.range.end))
  );
}

/**
 * Gets the path to the language server executable
 *
 * @param context - The extension context
 * @returns The path to the server executable
 */
function getServerPath(context: vscode.ExtensionContext): string {
  const config = vscode.workspace.getConfiguration("crbasic");
  const customPath = config.get<string>("server.path");

  if (customPath) {
    return customPath;
  }

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

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "crbasic" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher(
        "**/*.{cr1,cr1x,cr2,cr3,cr5,cr6,cr8,cr9,cr9x,c9x,cr300,crb,dld}"
      ),
    },
    middleware: {
      // The server's "N references" code lens targets the built-in
      // `editor.action.showReferences` command, whose arguments arrive over
      // LSP as plain JSON. VS Code validates that command's arguments as
      // real `vscode.Uri`/`Position`/`Location` instances, so they must be
      // reconstructed here before the lens is clickable.
      provideCodeLenses: async (document, token, next) => {
        const lenses = await next(document, token);
        if (!lenses) {
          return lenses;
        }

        for (const lens of lenses) {
          const args = parseShowReferencesArguments(lens.command);
          if (args && lens.command) {
            lens.command.arguments = [
              vscode.Uri.parse(args.uri),
              toVscodePosition(args.position),
              args.locations.map(toVscodeLocation),
            ];
          }
        }

        return lenses;
      },
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
    client = createLanguageClient(context);

    // Start the client (this also starts the server)
    await client.start();

    console.log("CRBasic LSP extension is now active");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Failed to start CRBasic Language Server: ${message}`);

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
