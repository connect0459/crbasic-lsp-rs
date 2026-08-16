# Changelog

<!--
When cutting a new release, update THREE places in this file:

1. Rename [Unreleased] to [X.Y.Z] with today's date (above), and add a fresh empty [Unreleased] section above it.
2. Update the reference links at the very bottom of this file:
    - Change [Unreleased] to compare the new tag against HEAD.
    - Add [X.Y.Z] comparing the new tag against the previous tag.
3. After the PR is merged, push the release tag. Pull main first so HEAD is the merge commit:

    ```console
    git checkout main && git pull origin main
    git tag vX.Y.Z && git push origin vX.Y.Z
    ```

   Pushing the tag triggers `.github/workflows/release.yml`, which re-verifies the quality gates, extracts this file's `[X.Y.Z]` section, and creates the GitHub Release from it automatically. Do not run `gh release create` manually; it would create the tag/Release ahead of the workflow with hand-pasted notes instead of the CHANGELOG-derived ones.
-->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-16

### Added

- CRBasic lexer and parser (`crbasic-parser`): case-insensitive keywords, full expression/statement/control-flow grammar, and model-dependent semantic validation (variable name length limits and 12-character truncation collisions for CR200X, CR6, and GRANITE dataloggers).
- Built-in instruction database (`keywords.json`, the single source of truth for completion/hover/signature-help): grown to 382 built-in functions/instructions and 90 language keywords across measurement, communication (SDM/SDI-12/CDM/CPI/MQTT/GOES/ARGOS/dial-modem/SMS/email/wireless/voice/satellite), data/file/time management, math/physical-model, and statistics/spatial-analysis families.
- Language Server (`crbasic-lsp`, via `tower-lsp-server`): diagnostics, completion, signature help, hover, go to definition, find all references, document symbols, workspace symbols, document highlight, code actions (quick fixes), code lens, folding ranges, selection ranges, linked editing ranges, inlay hints, semantic tokens, rename (with prepare support), and call hierarchy.
- WASM bindings (`crbasic-wasm`): `tokenize`, `parse`, `analyze`, and `version` APIs wrapping `crbasic-parser`, for consumers other than this project's own VSCode extension (the extension spawns the native `crbasic-lsp` binary directly; see [ADR-004](./docs/adrs/adr-004-multi-platform-packaging.md)).
- VSCode extension (`client/`): syntax highlighting via TextMate Grammar, LSP client wiring, `crbasic.restartServer` and `crbasic.showServerOutput` commands, type-time auto-indent, and an original extension icon.
- Multi-platform packaging and Marketplace publishing: per-platform `.vsix` builds (linux/darwin/win32 × x64/arm64), a `publish` job wired to the VS Code Marketplace via `vsce publish`, and a Marketplace-facing `client/README.md`/`client/LICENSE` bundled into each package.
- Curated example programs (`docs/examples/`) and CI pipeline (`.github/workflows/ci.yml`) covering both the Rust workspace and the TypeScript client.

---

[Unreleased]: <https://github.com/connect0459/crbasic-lsp-rs/compare/v0.1.0...HEAD>
[0.1.0]: <https://github.com/connect0459/crbasic-lsp-rs/releases/tag/v0.1.0>
