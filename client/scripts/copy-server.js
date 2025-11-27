/**
 * Script to copy the crbasic-lsp server binary to the extension's server directory
 *
 * This script is used during the build process to bundle the language server
 * with the VSCode extension.
 */

const fs = require("fs");
const path = require("path");

const isWindows = process.platform === "win32";
const serverName = isWindows ? "crbasic-lsp.exe" : "crbasic-lsp";

// Source: Cargo release build output
const sourcePath = path.join(__dirname, "..", "..", "target", "release", serverName);

// Destination: extension's server directory
const destDir = path.join(__dirname, "..", "server");
const destPath = path.join(destDir, serverName);

// Check if source exists
if (!fs.existsSync(sourcePath)) {
  console.error(`Error: Server binary not found at ${sourcePath}`);
  console.error("Please build the server first with: npm run build-server");
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
