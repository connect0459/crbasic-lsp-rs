/**
 * Tests for the pure command-handler logic in commands.ts.
 *
 * These are decoupled from the `vscode` module (which only exists inside
 * the Extension Host, not this test runner) so they can be exercised
 * directly instead of only as smoke tests.
 */

import { describe, test, expect, vi } from "vitest";
import { restartServer, showServerOutput, ServerConnection } from "./commands";

function createConnection(): ServerConnection {
  return {
    restart: vi.fn().mockResolvedValue(undefined),
    outputChannel: { show: vi.fn() },
  };
}

describe("restartServer", () => {
  test("restarts the connection and reports success", async () => {
    const connection = createConnection();

    const result = await restartServer(connection);

    expect(connection.restart).toHaveBeenCalledOnce();
    expect(result.ok).toBe(true);
  });

  test("reports failure without restarting when no connection is running", async () => {
    const result = await restartServer(undefined);

    expect(result.ok).toBe(false);
    expect(result.message).toContain("not running");
  });
});

describe("showServerOutput", () => {
  test("reveals the output channel and reports success", () => {
    const connection = createConnection();

    const result = showServerOutput(connection);

    expect(connection.outputChannel.show).toHaveBeenCalledOnce();
    expect(result.ok).toBe(true);
  });

  test("reports failure when no connection is running", () => {
    const result = showServerOutput(undefined);

    expect(result.ok).toBe(false);
    expect(result.message).toContain("not running");
  });
});
