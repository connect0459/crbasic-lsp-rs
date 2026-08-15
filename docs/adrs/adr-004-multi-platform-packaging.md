# ADR-004: Multi-Platform VSIX Packaging

**Status**: Accepted
**Date**: 2026-08-06
**Decision Makers**: Project Team
**Tags**: #packaging #vsce #ci-cd #cross-compilation

## Context and Problem Statement

`client/scripts/copy-server.js` bundles whatever OS's `crbasic-lsp` binary
happens to exist at `target/release/` on the build machine, and
`client/src/extension.ts`'s `getServerPath` only branches on win32 vs.
non-win32. There is no per-platform packaging: a `.vsix` built today only
runs on the OS that built it, and would silently fail (wrong binary, or a
binary for the wrong CPU architecture) on every other platform. This was
discovered while auditing release readiness for
[ADR-003](./adr-003-release-process.md) and logged in `docs/todo.md` as
blocking Phase 6's `vsce package` work.

The question: **how should the extension be packaged so that a user on any
supported desktop platform/architecture gets a working `crbasic-lsp`
binary?**

## Decision Drivers

- **Correctness**: a user on a different OS/arch than whichever machine
  built the `.vsix` must not receive a non-executable or wrong-architecture
  server binary.
- **No system-library blockers**: `crbasic-lsp`'s dependency tree
  (`tower-lsp`, `tokio`, `serde`) has no `openssl`/`native-tls` or other
  system-library dependency (checked `Cargo.lock`), so cross-compilation
  isn't blocked by missing native deps on any target.
- **CI cost**: minimize the number of distinct runners/toolchains needed to
  cover the target list.
- **Bounded coverage**: cover the realistic desktop install base (x64 and
  arm64 across Windows/macOS/Linux) without unbounded target sprawl
  (musl/Alpine, 32-bit, or ARM32 Linux are explicitly out of scope for now).

## Considered Options

### Option 1: Single universal `.vsix` bundling every platform's binary

**Pros**:

- One package, no dependency on VS Code's target-specific mechanism.

**Cons**:

- Ships all 6 native binaries to every user, even though exactly one is
  ever used per install.
- Still requires `extension.ts` to select the right binary by
  `process.platform` + CPU arch at runtime — more runtime logic, more that
  can silently go wrong, instead of relying on VS Code's own platform
  selection.

### Option 2: Load `crbasic-wasm` directly in the extension host

**Pros**:

- The direct realization of [ADR-001](./adr-001-rust-wasm-lsp-architecture.md)'s
  original "single WASM binary works everywhere" rationale — no native
  binary, no per-platform packaging question at all.

**Cons**:

- `crbasic-wasm` today only exposes one-shot `tokenize`/`parse`/`analyze`
  functions, not a running LSP server speaking `tower-lsp`'s stdio
  transport. Wiring an in-process, WASM-backed LSP server into the
  extension host (no child-process boundary, a different transport layer
  entirely) is a substantially larger undertaking than fixing packaging —
  out of scope for unblocking a release, not a packaging decision.

### Option 3: VS Code platform-specific `.vsix` packages (Selected)

**Pros**:

- The mechanism VS Code's own Marketplace and client are designed for
  exactly this situation: each user's VS Code downloads only the `.vsix`
  matching their platform/arch — no runtime binary-selection logic needed,
  no other platforms' dead weight bundled.
- Zero changes needed to `crbasic-lsp` or to `extension.ts`'s existing
  win32-vs-not check, since each package only ever contains its own
  platform's binary.
- Works with the existing `tower-lsp`/stdio architecture unchanged.

**Cons**:

- 6 separate builds per release instead of 1 (more CI time/jobs).
- Introduces cross-compilation for the two `-arm64` legs.

## Decision Outcome

**Chosen option**: **Option 3: VS Code platform-specific `.vsix` packages**

### Rationale

1. Directly fixes the defect without a disproportionate rewrite — Option 2
   is the architecturally "more correct" long-term answer to ADR-001's
   original intent, but is out of scope for a packaging fix and remains
   open as a future direction (see Related Decisions).
2. VS Code's target mechanism is the standard, Marketplace-supported
   solution for extensions bundling native binaries.
3. Ships against the current, working `tower-lsp` architecture today,
   with no dependency on WASM-in-extension-host maturity.

### Implementation Strategy

Six targets, three native runners (each building both its architectures):

| VS Code target | Rust triple | Runner |
| - | - | - |
| `linux-x64` | `x86_64-unknown-linux-gnu` | `ubuntu-latest` (native) |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | `ubuntu-latest` (cross via `gcc-aarch64-linux-gnu`) |
| `darwin-x64` | `x86_64-apple-darwin` | `macos-latest` (cross) |
| `darwin-arm64` | `aarch64-apple-darwin` | `macos-latest` (native) |
| `win32-x64` | `x86_64-pc-windows-msvc` | `windows-latest` (native) |
| `win32-arm64` | `aarch64-pc-windows-msvc` | `windows-latest` (MSVC cross) |

`client/scripts/targets.js` is the single source of truth for this mapping,
consumed by `copy-server.js --target <t>` (places the right binary),
`package-vsix.js <t>|all` (runs `vsce package --target <t>` per target),
and `place-artifacts.js` (relocates the release workflow's downloaded
per-target build artifacts into the `target/<triple>/release/` layout
`copy-server.js` expects).

## Consequences

### Positive

- ✅ Users on any of the 6 covered platform/arch combinations get a
  correctly-executing, correctly-architected `crbasic-lsp` binary.
- ✅ No runtime platform-detection code added to `extension.ts`.
- ✅ `client/.vscodeignore`, added as part of this work, trims dev-only
  files (`src/**`, `scripts/**`, build configs) from every package — worth
  doing now that this bloat is multiplied across 6 files instead of 1.

### Negative

- ⚠️ Roughly 6x the CI build time/resource for a release compared to a
  single-target build.
- ⚠️ `linux-arm64` and `win32-arm64` cross-compilation are unverified
  against real GitHub Actions runners as of this ADR (see Validation).

### Neutral

- 🔹 ADR-001's "single binary works everywhere" rationale remains
  unrealized by this decision — this ADR is a correctness fix within the
  current native-binary architecture, not a resolution of that drift.
  Option 2 above remains open for a future ADR if that's ever revisited.
- 🔹 Alpine/musl, 32-bit, and ARM32 (`armhf`) Linux targets are explicitly
  out of scope; adding one later is one new `targets.js` entry plus one new
  `build` matrix leg.

## Validation

We will validate this decision by:

1. Already done this session: `x86_64-apple-darwin` and
   `aarch64-apple-darwin` both cross-compile from this
   `aarch64-apple-darwin` host, and `npm run package -- <target>` produces
   a `.vsix` whose `server/` entry is confirmed (via `file`) to be the
   correct architecture for each.
2. **Not yet done**: a real GitHub Actions run of `release.yml`'s `build`
   and `package` jobs across all 6 targets, particularly the
   `linux-arm64` and `win32-arm64` cross-compilation legs. `release.yml`'s
   `workflow_dispatch` trigger exists specifically so this can be run and
   inspected (the resulting `.vsix` files are uploaded as a workflow
   artifact) without cutting a real release. This must be exercised at
   least once before the first real tagged release relies on it.

## Affected Files

### Initial Implementation (2026-08-06)

- `client/scripts/targets.js` (new): VS Code target ↔ Rust triple mapping.
- `client/scripts/copy-server.js`: added `--target <vscodeTarget>` support.
- `client/scripts/package-vsix.js` (new): per-target `vsce package` driver.
- `client/scripts/place-artifacts.js` (new): relocates downloaded CI
  artifacts into the cargo `target/<triple>/release/` layout.
- `client/package.json`: `@vscode/vsce` devDependency, new `package`
  script, `vscode.prepublish` no longer runs `copy-server` directly.
- `client/.vscodeignore` (new): excludes dev-only files from the package.
- `.github/workflows/release.yml`: split into `verify`/`build`/`package`
  jobs; added the 6-target build matrix and `workflow_dispatch` dry run.
- `docs/todo.md`: multi-platform packaging technical debt checked off.

## Related Decisions

- [ADR-001](./adr-001-rust-wasm-lsp-architecture.md): the native-binary
  drift from its "single WASM binary works everywhere" rationale that this
  ADR works within (via Option 3) rather than resolves (Option 2, above,
  remains the eventual full resolution if ever revisited).
- [ADR-003](./adr-003-release-process.md): originally logged this gap as a
  blocker for release automation attaching a `.vsix`; this ADR implements
  the deferred work.

## References

- [VS Code: Platform-specific extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension#platformspecific-extensions)
- [Rust Platform Support](https://doc.rust-lang.org/rustc/platform-support.html)
