/**
 * Single source of truth mapping VS Code's platform-specific extension
 * target names to the Rust target triple that produces the matching native
 * `crbasic-lsp` binary.
 *
 * See: https://code.visualstudio.com/api/working-with-extensions/publishing-extension#platformspecific-extensions
 */

const TARGETS = [
  { vscodeTarget: "linux-x64", rustTriple: "x86_64-unknown-linux-gnu" },
  { vscodeTarget: "linux-arm64", rustTriple: "aarch64-unknown-linux-gnu" },
  { vscodeTarget: "darwin-x64", rustTriple: "x86_64-apple-darwin" },
  { vscodeTarget: "darwin-arm64", rustTriple: "aarch64-apple-darwin" },
  { vscodeTarget: "win32-x64", rustTriple: "x86_64-pc-windows-msvc" },
  { vscodeTarget: "win32-arm64", rustTriple: "aarch64-pc-windows-msvc" },
];

/**
 * Looks up the target entry for a VS Code target name.
 *
 * @param {string} vscodeTarget - e.g. "darwin-arm64"
 * @returns {{ vscodeTarget: string, rustTriple: string }}
 * @throws {Error} if the target is not one of TARGETS
 */
function resolveTarget(vscodeTarget) {
  const entry = TARGETS.find((t) => t.vscodeTarget === vscodeTarget);
  if (!entry) {
    const supported = TARGETS.map((t) => t.vscodeTarget).join(", ");
    throw new Error(`Unknown VS Code target "${vscodeTarget}". Supported targets: ${supported}`);
  }
  return entry;
}

/**
 * Returns the `crbasic-lsp` binary filename for a VS Code target,
 * accounting for the Windows ".exe" suffix.
 *
 * @param {string} vscodeTarget - e.g. "win32-x64"
 * @returns {string}
 */
function binaryName(vscodeTarget) {
  return vscodeTarget.startsWith("win32") ? "crbasic-lsp.exe" : "crbasic-lsp";
}

module.exports = { TARGETS, resolveTarget, binaryName };
