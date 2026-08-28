/**
 * Tests for the pure command-handler logic in commands.ts.
 *
 * These are decoupled from the `vscode` module (which only exists inside
 * the Extension Host, not this test runner) so they can be exercised
 * directly instead of only as smoke tests.
 */

import { describe, test, expect, vi } from "vitest";
import {
  restartServer,
  showServerOutput,
  parseShowReferencesArguments,
  SHOW_REFERENCES_COMMAND,
  ServerConnection,
} from "./commands";

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

describe("parseShowReferencesArguments", () => {
  function validRawCommand(): { command: string; arguments: unknown[] } {
    return {
      command: SHOW_REFERENCES_COMMAND,
      arguments: [
        "file:///test.cr6",
        { line: 3, character: 7 },
        [
          {
            uri: "file:///test.cr6",
            range: {
              start: { line: 4, character: 0 },
              end: { line: 4, character: 6 },
            },
          },
        ],
      ],
    };
  }

  test("parses the URI, position, and locations from a valid showReferences command", () => {
    const result = parseShowReferencesArguments(validRawCommand());

    expect(result).toEqual({
      uri: "file:///test.cr6",
      position: { line: 3, character: 7 },
      locations: [
        {
          uri: "file:///test.cr6",
          range: { start: { line: 4, character: 0 }, end: { line: 4, character: 6 } },
        },
      ],
    });
  });

  test("returns undefined when the command is undefined", () => {
    expect(parseShowReferencesArguments(undefined)).toBeUndefined();
  });

  test("returns undefined for a command other than editor.action.showReferences", () => {
    const command = { ...validRawCommand(), command: "crbasic.someOtherCommand" };

    expect(parseShowReferencesArguments(command)).toBeUndefined();
  });

  test("returns undefined when arguments are missing", () => {
    const command = { command: SHOW_REFERENCES_COMMAND };

    expect(parseShowReferencesArguments(command)).toBeUndefined();
  });

  test("returns undefined when the position argument is malformed", () => {
    const command = validRawCommand();
    command.arguments[1] = { line: 3 };

    expect(parseShowReferencesArguments(command)).toBeUndefined();
  });

  test("returns undefined when a location in the array is malformed", () => {
    const command = validRawCommand();
    command.arguments[2] = [{ uri: "file:///test.cr6" }];

    expect(parseShowReferencesArguments(command)).toBeUndefined();
  });

  test("returns an empty locations array for an unused symbol", () => {
    const command = validRawCommand();
    command.arguments[2] = [];

    const result = parseShowReferencesArguments(command);

    expect(result?.locations).toEqual([]);
  });
});
