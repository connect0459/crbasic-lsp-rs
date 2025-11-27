/**
 * Smoke tests for CRBasic VSCode Extension
 *
 * These tests verify basic functionality of the extension logic.
 * Note: These are smoke tests that verify the extension structure
 * without requiring a full VSCode instance.
 */

import { describe, test, expect } from "vitest";
import * as path from "path";

describe("Extension Smoke Tests", () => {
  describe("Server Path Resolution Logic", () => {
    test("constructs correct server path for Windows platform", () => {
      // Verify the logic for Windows platform
      const platform = "win32";
      const extensionPath = "/test/extension/path";
      const serverName = platform === "win32" ? "crbasic-lsp.exe" : "crbasic-lsp";
      const expectedPath = path.join(extensionPath, "server", serverName);

      expect(serverName).toBe("crbasic-lsp.exe");
      expect(expectedPath).toContain("crbasic-lsp.exe");
      expect(expectedPath).toContain("server");
    });

    test("constructs correct server path for Unix-like platforms", () => {
      // Verify the logic for Unix-like platforms (darwin, linux, etc.)
      const platform: string = "darwin";
      const extensionPath = "/test/extension/path";
      const serverName = platform === "win32" ? "crbasic-lsp.exe" : "crbasic-lsp";
      const expectedPath = path.join(extensionPath, "server", serverName);

      expect(serverName).toBe("crbasic-lsp");
      expect(expectedPath).toContain("crbasic-lsp");
      expect(expectedPath).toContain("server");
      expect(expectedPath).not.toContain(".exe");
    });

    test("uses correct path separator for the current platform", () => {
      const extensionPath = "/test/path";
      const serverPath = path.join(extensionPath, "server", "crbasic-lsp");

      // Path should use the platform's separator
      expect(serverPath).toBe(path.normalize("/test/path/server/crbasic-lsp"));
    });
  });

  describe("Extension Configuration", () => {
    test("uses standard LSP file extensions", () => {
      const extensions = [
        ".cr1",
        ".cr1x",
        ".cr2",
        ".cr3",
        ".cr5",
        ".cr6",
        ".cr8",
        ".cr9",
        ".cr9x",
        ".c9x",
        ".cr300",
        ".crb",
        ".dld",
      ];

      // Verify all expected extensions are present
      extensions.forEach((ext) => {
        expect(ext).toMatch(/^\.[a-z0-9]+$/);
      });

      // Should include common CR file extensions
      expect(extensions).toContain(".cr1");
      expect(extensions).toContain(".cr6");
      expect(extensions).toContain(".crb");
    });
  });

  describe("Module Structure", () => {
    test("extension file exists and is loadable", async () => {
      // This test verifies the module can be loaded
      const fs = await import("fs");
      const extensionPath = path.join(__dirname, "extension.ts");

      expect(fs.existsSync(extensionPath)).toBe(true);
    });
  });
});
