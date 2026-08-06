/**
 * Packages the VSCode extension into one .vsix per platform target.
 *
 * Usage:
 *   node scripts/package-vsix.js <vscodeTarget>|all [--out <dir>]
 *
 * For each requested target, copies that platform's cross-compiled
 * crbasic-lsp binary into server/ (see copy-server.js --target) and then
 * runs `vsce package --target <vscodeTarget>`.
 */

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const { TARGETS, resolveTarget } = require("./targets");

const CLIENT_DIR = path.join(__dirname, "..");

function parseArgs(argv) {
  const [targetArg, ...rest] = argv;
  if (!targetArg) {
    throw new Error("Usage: node scripts/package-vsix.js <vscodeTarget>|all [--out <dir>]");
  }
  const outFlagIndex = rest.indexOf("--out");
  const outDir =
    outFlagIndex !== -1 ? rest[outFlagIndex + 1] : path.join(CLIENT_DIR, "..", "dist-vsix");
  return { targetArg, outDir };
}

const NPX_COMMAND = process.platform === "win32" ? "npx.cmd" : "npx";

function packageTarget(vscodeTarget, outDir) {
  resolveTarget(vscodeTarget); // throws on an unknown target

  console.log(`Packaging ${vscodeTarget}...`);
  execFileSync("node", [path.join(__dirname, "copy-server.js"), "--target", vscodeTarget], {
    stdio: "inherit",
  });
  execFileSync(
    NPX_COMMAND,
    ["--no-install", "vsce", "package", "--target", vscodeTarget, "-o", outDir],
    { stdio: "inherit", cwd: CLIENT_DIR }
  );
}

const { targetArg, outDir } = parseArgs(process.argv.slice(2));
fs.mkdirSync(outDir, { recursive: true });

const vscodeTargets = targetArg === "all" ? TARGETS.map((t) => t.vscodeTarget) : [targetArg];
for (const vscodeTarget of vscodeTargets) {
  packageTarget(vscodeTarget, outDir);
}
