# Changelog

<!--
When cutting a new release, update THREE places in this file:

1. Rename [Unreleased] to [X.Y.Z] with today's date (above), and add a fresh
   empty [Unreleased] section above it.
2. Update the reference links at the very bottom of this file:
    - Change [Unreleased] to compare the new tag against HEAD.
    - Add [X.Y.Z] comparing the new tag against the previous tag.
3. After the PR is merged, push the release tag. Pull main first so HEAD is
   the merge commit:

    ```console
    git checkout main && git pull origin main
    git tag vX.Y.Z && git push origin vX.Y.Z
    ```

   Pushing the tag triggers `.github/workflows/release.yml`, which
   re-verifies the quality gates, extracts this file's `[X.Y.Z]` section, and
   creates the GitHub Release from it automatically. Do not run
   `gh release create` manually; it would create the tag/Release ahead of the
   workflow with hand-pasted notes instead of the CHANGELOG-derived ones.
-->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- CRBasic lexer and parser (`crbasic-parser`): case-insensitive keywords,
  full expression/statement/control-flow grammar, and model-dependent
  semantic validation (variable name length limits and 12-character
  truncation collisions for CR200X, CR6, and GRANITE dataloggers).
- Language Server (`crbasic-lsp`, via `tower-lsp-server`): diagnostics, completion,
  signature help, hover, go to definition, find all references, document
  symbols, and rename.
- WASM bindings (`crbasic-wasm`): `tokenize`, `parse`, `analyze`, and
  `version` APIs consumed by the VSCode extension.
- VSCode extension (`client/`): syntax highlighting via TextMate Grammar,
  LSP client wiring, `crbasic.restartServer` and `crbasic.showServerOutput`
  commands, and type-time auto-indent.
- Curated example programs (`docs/examples/`) and CI pipeline
  (`.github/workflows/ci.yml`) covering both the Rust workspace and the
  TypeScript client.

---

[Unreleased]: <https://github.com/connect0459/crbasic-lsp-rs/commits/main>
