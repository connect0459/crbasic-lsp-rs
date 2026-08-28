/**
 * Pure command-handler logic for the CRBasic extension commands.
 *
 * Kept free of the `vscode` module (which only exists inside the Extension
 * Host) so it can be unit tested directly; `extension.ts` wires these
 * functions to real `vscode.commands.registerCommand` calls and the real
 * language client.
 */

/** Minimal shape of a language client needed to restart it or show its output. */
export interface ServerConnection {
  restart: () => Promise<void>;
  outputChannel: { show: () => void };
}

/** Result of a command that requires a running server connection. */
export interface CommandResult {
  ok: boolean;
  message?: string;
}

const NOT_RUNNING_MESSAGE = "CRBasic Language Server is not running.";

/**
 * Restarts the language server connection.
 *
 * @param connection - The active server connection, or undefined if the
 *   server has not started (e.g. activation failed).
 * @returns The outcome, with a user-facing message on either path.
 */
export async function restartServer(
  connection: ServerConnection | undefined
): Promise<CommandResult> {
  if (!connection) {
    return { ok: false, message: NOT_RUNNING_MESSAGE };
  }

  await connection.restart();
  return { ok: true, message: "CRBasic Language Server restarted." };
}

/**
 * Reveals the language server's output channel.
 *
 * @param connection - The active server connection, or undefined if the
 *   server has not started.
 * @returns The outcome, with a user-facing message when it fails.
 */
export function showServerOutput(connection: ServerConnection | undefined): CommandResult {
  if (!connection) {
    return { ok: false, message: NOT_RUNNING_MESSAGE };
  }

  connection.outputChannel.show();
  return { ok: true };
}

/** LSP-shaped position: zero-based line/character, as sent over the wire. */
export interface LspPosition {
  line: number;
  character: number;
}

/** LSP-shaped range: a start/end pair of {@link LspPosition}. */
export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

/** LSP-shaped location: a document URI paired with a range within it. */
export interface LspLocation {
  uri: string;
  range: LspRange;
}

/** Parsed, still vscode-free arguments for the `editor.action.showReferences` command. */
export interface ShowReferencesArguments {
  uri: string;
  position: LspPosition;
  locations: LspLocation[];
}

/**
 * The built-in VS Code command the CRBasic language server's "N references"
 * code lens targets. Its arguments arrive over LSP as plain JSON, but VS
 * Code validates them as real `vscode.Uri`/`Position`/`Location` instances,
 * so callers must reconstruct them (see `parseShowReferencesArguments`).
 */
export const SHOW_REFERENCES_COMMAND = "editor.action.showReferences";

function isLspPosition(value: unknown): value is LspPosition {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as LspPosition;
  return typeof candidate.line === "number" && typeof candidate.character === "number";
}

function isLspRange(value: unknown): value is LspRange {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as LspRange;
  return isLspPosition(candidate.start) && isLspPosition(candidate.end);
}

function isLspLocation(value: unknown): value is LspLocation {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as LspLocation;
  return typeof candidate.uri === "string" && isLspRange(candidate.range);
}

/**
 * Extracts and validates the `editor.action.showReferences` arguments the
 * server attaches to a code lens command.
 *
 * The server sends plain JSON (a URI string, a position, and a location
 * array); this only validates their shape without constructing any `vscode`
 * types, so it stays testable outside the Extension Host.
 *
 * @param command - The code lens's resolved command, if any.
 * @returns The parsed arguments, or `undefined` if the command isn't a
 *   `showReferences` invocation or its arguments don't match the expected shape.
 */
export function parseShowReferencesArguments(
  command: { command: string; arguments?: unknown[] } | undefined
): ShowReferencesArguments | undefined {
  if (!command || command.command !== SHOW_REFERENCES_COMMAND || !command.arguments) {
    return undefined;
  }

  const [uri, position, locations] = command.arguments;
  if (
    typeof uri !== "string" ||
    !isLspPosition(position) ||
    !Array.isArray(locations) ||
    !locations.every(isLspLocation)
  ) {
    return undefined;
  }

  return { uri, position, locations };
}
