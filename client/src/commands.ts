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
