/**
 * Script to copy the crbasic-lsp server binary to the extension's server directory
 *
 * This script is used during the build process to bundle the language server
 * with the VSCode extension. Without --target, it copies the binary that
 * matches the current machine (the local single-platform dev flow). With
 * --target <vscodeTarget>, it copies the cross-compiled binary for that
 * platform instead (used by the multi-platform release packaging flow, see
 * package-vsix.js).
 */

const fs = require("fs");
const path = require("path");
const { resolveTarget, binaryName } = require("./targets");

const targetFlagIndex = process.argv.indexOf("--target");
const vscodeTarget = targetFlagIndex !== -1 ? process.argv[targetFlagIndex + 1] : undefined;

const isWindows = vscodeTarget ? vscodeTarget.startsWith("win32") : process.platform === "win32";
const serverName = vscodeTarget
  ? binaryName(vscodeTarget)
  : isWindows
    ? "crbasic-lsp.exe"
    : "crbasic-lsp";

// Source: Cargo release build output. Cross-compiled builds land under
// target/<rustTriple>/release/ instead of target/release/.
const sourcePath = vscodeTarget
  ? path.join(
      __dirname,
      "..",
      "..",
      "target",
      resolveTarget(vscodeTarget).rustTriple,
      "release",
      serverName
    )
  : path.join(__dirname, "..", "..", "target", "release", serverName);

// Destination: extension's server directory
const destDir = path.join(__dirname, "..", "server");
const destPath = path.join(destDir, serverName);

// Check if source exists
if (!fs.existsSync(sourcePath)) {
  console.error(`Error: Server binary not found at ${sourcePath}`);
  console.error(
    vscodeTarget
      ? `Please build it first with: cargo build --release --target ${resolveTarget(vscodeTarget).rustTriple} -p crbasic-lsp --bin crbasic-lsp`
      : "Please build the server first with: npm run build-server"
  );
  process.exit(1);
}

// Ensure destination directory exists
if (!fs.existsSync(destDir)) {
  fs.mkdirSync(destDir, { recursive: true });
}

// Copy the binary
try {
  fs.copyFileSync(sourcePath, destPath);

  // Make executable on Unix-like systems
  if (!isWindows) {
    fs.chmodSync(destPath, 0o755);
  }

  console.log(`Successfully copied server binary to ${destPath}`);
} catch (error) {
  console.error(`Error copying server binary: ${error.message}`);
  process.exit(1);
}
