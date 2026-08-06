# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/) under
the pre-`1.0.0` conventions recorded in
[ADR-003](./docs/adrs/adr-003-release-process.md).

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

[Unreleased]: https://github.com/connect0459/crbasic-lsp-rs/commits/main
