# Changelog

<!--
When cutting a new release, update THREE places in this file:

1. Rename [Unreleased] to [X.Y.Z] with today's date (above).
2. Update the reference links at the very bottom of this file:
    - Change [Unreleased] to compare the new tag against HEAD.
    - Add [X.Y.Z] comparing the new tag against the previous tag.
3. After the PR is merged, create a GitHub Release (this creates the remote
   tag). Pull main first so HEAD is the merge commit, then use `--target main`
   or pass the full 40-character SHA — the GitHub API rejects abbreviated SHAs:

    ```console
    git checkout main && git pull origin main
    gh release create vX.Y.Z --title "vX.Y.Z" \
      --notes-file path/to/gh-release-draft.md \
      --target main
    ```
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
- Language Server (`crbasic-lsp`, via `tower-lsp`): diagnostics, completion,
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
