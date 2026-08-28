/**
 * Builds the crbasic-lsp server for this machine, packages a .vsix for it,
 * and installs that .vsix into the local `code` CLI — a one-command path
 * for trying a change in a real VS Code instance instead of an Extension
 * Development Host.
 *
 * Usage:
 *   node scripts/install-local.js
 */

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const { resolveTarget } = require("./targets");

const REPO_ROOT = path.join(__dirname, "..", "..");
const OUT_DIR = path.join(REPO_ROOT, "dist-vsix-local");

const PLATFORM_NAMES = { darwin: "darwin", linux: "linux", win32: "win32" };

function localVscodeTarget() {
  const platform = PLATFORM_NAMES[process.platform];
  const arch = process.arch;
  if (!platform || (arch !== "x64" && arch !== "arm64")) {
    throw new Error(`Unsupported local platform/arch: ${process.platform}/${process.arch}`);
  }
  const vscodeTarget = `${platform}-${arch}`;
  resolveTarget(vscodeTarget); // throws if this platform/arch isn't a packaged target
  return vscodeTarget;
}

const vscodeTarget = localVscodeTarget();
const { rustTriple } = resolveTarget(vscodeTarget);

console.log(`Building crbasic-lsp for ${rustTriple}...`);
execFileSync(
  "cargo",
  ["build", "--release", "--target", rustTriple, "-p", "crbasic-lsp", "--bin", "crbasic-lsp"],
  { stdio: "inherit", cwd: REPO_ROOT }
);

fs.rmSync(OUT_DIR, { recursive: true, force: true });
fs.mkdirSync(OUT_DIR, { recursive: true });

console.log(`Packaging ${vscodeTarget}...`);
execFileSync(
  "node",
  [path.join(__dirname, "package-vsix.js"), vscodeTarget, "--out", OUT_DIR],
  { stdio: "inherit" }
);

const [vsixFile] = fs.readdirSync(OUT_DIR).filter((name) => name.endsWith(".vsix"));
if (!vsixFile) {
  throw new Error(`No .vsix was produced in ${OUT_DIR}`);
}
const vsixPath = path.join(OUT_DIR, vsixFile);

console.log(`Installing ${vsixFile}...`);
execFileSync("code", ["--install-extension", vsixPath], { stdio: "inherit" });
