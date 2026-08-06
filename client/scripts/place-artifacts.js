/**
 * Moves the per-target server binaries downloaded from the release
 * workflow's `build` job artifacts into the cargo `target/<triple>/release/`
 * layout that copy-server.js (and therefore package-vsix.js) expects.
 *
 * Usage: node scripts/place-artifacts.js <artifactsDir> <cargoTargetDir>
 */

const fs = require("fs");
const path = require("path");
const { TARGETS, binaryName } = require("./targets");

const [artifactsDir, cargoTargetDir] = process.argv.slice(2);
if (!artifactsDir || !cargoTargetDir) {
  throw new Error("Usage: node scripts/place-artifacts.js <artifactsDir> <cargoTargetDir>");
}

for (const { vscodeTarget, rustTriple } of TARGETS) {
  const name = binaryName(vscodeTarget);
  const destDir = path.join(cargoTargetDir, rustTriple, "release");
  fs.mkdirSync(destDir, { recursive: true });
  fs.copyFileSync(
    path.join(artifactsDir, `server-${vscodeTarget}`, name),
    path.join(destDir, name)
  );
}
