# ADR-003: Release Process (Versioning, Changelog, Automation)

**Status**: Accepted
**Date**: 2026-08-06
**Decision Makers**: Project Team
**Tags**: #release #versioning #changelog #ci-cd

## Context and Problem Statement

The project has reached `0.1.0` across every published unit (the Cargo workspace and `client/package.json`), the CI pipeline (`ci.yml`) is in place, but no git tag has ever been pushed and there is no changelog. "Cutting a release" is currently undefined: there is no rule for what the version number means, no record of what changed between versions, and no automated way to turn a tagged commit into a GitHub Release.

These three concerns are addressed together because they are one process, not three independent ones: a release is *bump the version* → *record what changed* → *tag it* → *publish the record*. Splitting them into separate ADRs would hide that dependency.

## Decision Drivers

- **Multi-component repo**: three Cargo crates (`crbasic-parser`, `crbasic-lsp`, `crbasic-wasm`) plus the `client/` VSCode extension ship together — the client only has value bundled with the server it wraps, so they cannot version usefully in isolation.
- **Pre-1.0 ambiguity**: SemVer's own spec is silent on what MINOR/PATCH mean before `1.0.0`; without a stated convention, bumps become arbitrary.
- **OSS consumers**: contributors and future users need a documented, predictable way to find out what changed and when `1.0.0` is warranted.
- **Low release frequency**: releases are infrequent, deliberate events, so the automation can afford to be thorough (re-run quality gates) rather than fast.
- **Known packaging gap**: `client/scripts/copy-server.js` bundles whatever OS's `crbasic-lsp` binary was built locally; there is no per-platform (`win32-x64`, `darwin-arm64`, `linux-x64`, ...) packaging yet. Any release automation built now must not imply a working multi-platform artifact that does not exist.

## Considered Options

This ADR bundles three coupled decisions (see Context); each is a separate options comparison.

### Decision A: Versioning scheme

#### Option A1: Independent per-package versioning

**Pros**:

- Matches how a pure library ecosystem would do it — each crate advertises its own compatibility guarantees.

**Cons**:

- The client extension has no meaning without a specific server version bundled inside it; independent versioning would require a compatibility matrix for what is really a 1:1:1:1 relationship, which is pure overhead here.

#### Option A2: Lockstep versioning (Selected)

**Pros**:

- One version number, set once in `Cargo.toml`'s `[workspace.package] version` (already the source for all three crates via `version.workspace = true`) and mirrored in `client/package.json`.
- A single number always answers "what version is this extension + server pair", with no compatibility matrix to maintain.

**Cons**:

- A change scoped to a single crate still bumps the version of every crate in the workspace.

### Decision B: Changelog format

#### Option B1: Freeform commit log

**Pros**:

- No extra file to maintain; `git log` is always accurate by construction.

**Cons**:

- Commit messages answer "why", not "what shipped in version X for a user"; conventional commit prefixes (`feat`, `fix`, ...) are a developer-facing taxonomy, not release notes.

#### Option B2: Keep a Changelog (Selected)

**Pros**:

- A human-curated `CHANGELOG.md` with an `## [Unreleased]` section that accumulates entries as work lands, retitled to `## [X.Y.Z] - YYYY-MM-DD` at release time.
- Machine-parseable enough to drive release notes automatically (see Decision C), human-readable enough to be the actual release notes.

**Cons**:

- Requires discipline: every user-facing PR must remember to add an entry.

### Decision C: Release automation

#### Option C1: Manual release notes

**Pros**:

- No workflow to write or maintain.

**Cons**:

- Whoever cuts the release copies the changelog section into the GitHub Release UI by hand — error-prone (easy to tag a version whose `Cargo.toml`/`package.json` don't actually match) and it is invisible whether the tagged commit still passes the quality gates.

#### Option C2: Tag-triggered workflow (Selected)

**Pros**:

- Pushing a `vX.Y.Z` tag runs `.github/workflows/release.yml`, which (1) asserts the tag matches both `Cargo.toml` and `client/package.json`, (2) re-runs the same fmt/clippy/test and lint/format/type-check/test gates as `ci.yml`, (3) extracts that version's `CHANGELOG.md` section, and (4) creates the GitHub Release with those notes.

**Cons**:

- Duplicates `ci.yml`'s quality-gate steps rather than sharing a reusable workflow (see Consequences).
- Deliberately does **not** build or attach a `.vsix`, and does **not** run `vsce publish` — both require the multi-platform packaging gap (see Context) to be closed first (tracked in `docs/todo.md`).

## Decision Outcome

**Chosen options**: Option A2 (lockstep SemVer), Option B2 (Keep a Changelog), and Option C2 (tag-triggered release workflow).

1. **Versioning**: Lockstep SemVer. Pre-`1.0.0`, a breaking change (removing or renaming a public API surface — WASM bindings, LSP capabilities exposed, `client` configuration keys/commands) bumps MINOR; anything else (new non-breaking feature, fix, docs, tooling) bumps PATCH. `1.0.0` is warranted once: the extension is published to the VSCode Marketplace, the multi-platform packaging gap is resolved, and the WASM/LSP public API is considered stable enough to commit to backward compatibility.
2. **Changelog**: `CHANGELOG.md` at the repo root, Keep a Changelog format. Every user-facing change gets an entry under `## [Unreleased]` in the same commit/PR that makes the change (not retroactively). Immediately before tagging a release, `[Unreleased]` is retitled to `[X.Y.Z] - YYYY-MM-DD` and a fresh empty `[Unreleased]` section is added above it.
3. **Automation**: `.github/workflows/release.yml`, triggered only by pushing a `v*.*.*` tag (a deliberate human action — no auto-tagging from pushes to `main`), creates a GitHub Release with changelog-derived notes after re-verifying the quality gates and the version/tag consistency.

## Consequences

### Positive

- ✅ One version number to reason about across the whole repo.
- ✅ Release notes are always derived from a human-curated record, not regenerated from raw commit history.
- ✅ A tag can never produce a Release whose `Cargo.toml`/`package.json` version disagrees with the tag, or whose quality gates were failing.

### Negative

- ⚠️ `CHANGELOG.md`'s `[Unreleased]` section requires discipline — a PR that forgets an entry leaves a gap in the release notes (mitigated by reviewing it during PR review, same as any other doc requirement).
- ⚠️ Re-running the quality gates inside `release.yml` duplicates `ci.yml` rather than sharing a reusable workflow, accepted here to avoid touching the already-working CI file for an infrequent event.

### Neutral

- 🔹 No artifact (`.vsix`, published Marketplace listing) is produced by this workflow yet — releases are notes-only until the multi-platform packaging gap is closed.

## Validation

We will validate this decision by:

1. Pushing the first real tag (e.g. the current `0.1.0`) and confirming `release.yml` produces a GitHub Release whose notes match `CHANGELOG.md`'s corresponding section.
2. Deliberately pushing a mismatched tag (e.g. `v9.9.9` against a `0.1.0` `Cargo.toml`) in a throwaway branch/fork to confirm the workflow fails fast instead of creating an incorrect Release.

## Affected Files

### Initial Implementation (2026-08-06)

- `CHANGELOG.md` (new): Keep a Changelog record, starting at `[Unreleased]`.
- `.github/workflows/release.yml` (new): tag-triggered release automation.
- `docs/todo.md`: Phase 8 "Release preparation" items checked off; new technical-debt item added for the multi-platform packaging gap.

### Marketplace publishing closes the packaging gap (2026-08-15)

Decision C's "Cons" and the "Neutral" consequence above both state that `release.yml` does not build/attach a `.vsix` or run `vsce publish`. That is now stale: [ADR-004](./adr-004-multi-platform-packaging.md) closed the multi-platform packaging gap this ADR named as a blocker, and `release.yml` has since grown `build` (6-platform matrix), `package`, and `publish` jobs. The `publish` job runs `vsce publish --packagePath ../dist-vsix/*.vsix`, gated on a `VSCE_PAT` secret that is now registered — so the next `v*.*.*` tag push will publish to the Marketplace for real, not just create a notes-only GitHub Release. See `docs/todo.md`'s Phase 6 "Extension packaging and publishing" entry for the account-setup history.

## Related Decisions

- [ADR-001](./adr-001-rust-wasm-lsp-architecture.md): Rust + WASM LSP architecture — the multi-platform packaging gap this ADR works around stems from the client bundling a native `crbasic-lsp` binary (`client/scripts/copy-server.js`) rather than loading `crbasic-wasm` directly in the extension host, which is a drift from that ADR's "no external binary dependencies" rationale worth revisiting separately.

## References

- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- [Semantic Versioning 2.0.0](https://semver.org/)
- [VS Code: Platform-specific extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension#platformspecific-extensions)
