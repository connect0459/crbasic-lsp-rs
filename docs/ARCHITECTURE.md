# ARCHITECTURE

## Overview

`crbasic-lsp-rs` is a Visual Studio Code extension that adds language support for CRBasic, the programming language of Campbell Scientific data loggers.

The project implements syntax highlighting with a TextMate grammar and advanced language features with a Language Server Protocol (LSP) server.

This document describes the system's current structure and design. For what is done, in progress, or planned, see [`docs/todo.md`](./todo.md); for the reasoning behind specific decisions, see the [ADRs](./adrs/).

## 1. Project Structure

```text
crbasic-lsp-rs/
├── crates/                      # Rust workspace
│   ├── crbasic-parser/          # Core parser logic (no LSP dependencies)
│   │   ├── src/
│   │   │   ├── lib.rs           # Public API exports
│   │   │   ├── lexer.rs         # Lexer module root
│   │   │   ├── lexer/
│   │   │   │   ├── scanner.rs   # Tokenization
│   │   │   │   └── token.rs     # Token definitions
│   │   │   ├── parser.rs        # Syntax analysis
│   │   │   ├── ast.rs           # Abstract Syntax Tree
│   │   │   ├── semantic.rs      # Semantic analysis
│   │   │   ├── keywords.rs      # Keyword/instruction metadata API
│   │   │   └── keywords_generated.rs  # Generated from ../keywords.json
│   │   ├── keywords.json        # Single source of truth for CRBasic
│   │   │                        # instructions (see ADR-002)
│   │   ├── examples/
│   │   └── tests/
│   │       ├── fixtures/            # Real-world regression fixtures
│   │       ├── sample_files.rs      # Integration tests over tests/fixtures/*
│   │       ├── example_programs.rs  # Assertions on docs/examples/*
│   │       └── performance.rs       # Lexer/parser performance budgets
│   ├── crbasic-lsp/              # LSP server implementation
│   │   ├── src/
│   │   │   ├── lib.rs                 # LSP public interface
│   │   │   ├── main.rs                # Binary entry point
│   │   │   ├── backend.rs             # tower-lsp-server `LanguageServer` impl
│   │   │   ├── document.rs            # Document state management
│   │   │   ├── completion.rs          # IntelliSense completion
│   │   │   ├── hover.rs               # Hover information
│   │   │   ├── signature.rs           # Signature help
│   │   │   ├── definition.rs          # Go to definition
│   │   │   ├── references.rs          # Find all references
│   │   │   ├── symbols.rs             # Document symbols
│   │   │   ├── workspace_symbol.rs    # Workspace symbol search
│   │   │   ├── document_highlight.rs  # Document highlight
│   │   │   ├── code_action.rs         # Quick fixes
│   │   │   ├── code_lens.rs           # Code lens
│   │   │   ├── folding.rs             # Folding ranges
│   │   │   ├── selection_range.rs     # Selection ranges
│   │   │   ├── linked_editing_range.rs # Linked editing ranges
│   │   │   ├── inlay_hint.rs          # Inlay hints
│   │   │   ├── semantic_tokens.rs     # Semantic highlighting
│   │   │   ├── rename.rs              # Rename (with prepare support)
│   │   │   ├── call_hierarchy.rs      # Call hierarchy
│   │   │   └── call_sites.rs          # Shared call-site resolution helper
│   │   └── tests/
│   │       └── lsp_integration.rs     # End-to-end protocol-level tests
│   └── crbasic-wasm/              # WASM bindings for `crbasic-parser`
│       ├── src/
│       │   └── lib.rs             # WASM API (tokenize/parse/analyze)
│       └── pkg/                   # WASM build output
├── client/                        # VSCode extension client (TypeScript + Vite)
│   ├── src/
│   │   ├── extension.ts           # Extension entry point (spawns the native
│   │   │                          # crbasic-lsp binary over stdio)
│   │   └── commands.ts            # Command-handler logic (restart, show output)
│   ├── scripts/
│   │   ├── copy-server.js         # Bundles the native binary into client/server/
│   │   ├── targets.js             # Per-platform Rust target definitions
│   │   ├── package-vsix.js        # Builds one .vsix per platform target
│   │   └── place-artifacts.js     # Stages built binaries for packaging
│   ├── syntaxes/
│   │   └── crbasic.tmLanguage.json  # TextMate Grammar (generated; see
│   │                                 # scripts/generate-grammar.js)
│   ├── language-configuration.json  # Bracket/comment/indentation rules
│   ├── images/                    # Extension icon assets
│   ├── README.md                  # Marketplace-facing README (bundled into .vsix)
│   ├── LICENSE                    # Copy of the repo LICENSE (bundled into .vsix)
│   ├── package.json               # Extension manifest
│   ├── tsconfig.json
│   ├── vite.config.ts             # Vite configuration
│   ├── vitest.config.ts           # Vitest configuration
│   └── eslint.config.js           # ESLint 9 Flat Config
├── scripts/
│   └── generate-grammar.js        # Regenerates keywords_generated.rs and
│                                   # crbasic.tmLanguage.json from keywords.json
├── docs/
│   ├── ARCHITECTURE.md            # This file
│   ├── todo.md                    # Project progress tracker
│   ├── adrs/                      # Architecture Decision Records
│   ├── researches/                # Research documents
│   └── examples/                  # Curated feature-showcase example programs
├── Cargo.toml                     # Rust workspace configuration
├── justfile                       # Task runner (setup, test, coverage, verify, ...)
├── .pre-commit-config.yaml
├── .gitignore
├── CLAUDE.md                      # Project-specific Claude Code rules
├── AGENTS.md                      # Same rules, generic-agent-facing filename
└── README.md
```

## 2. High-Level System Diagram

### Syntax Highlighting Flow

```text
CRBasic File (.cr1)
  ↓
TextMate Grammar (syntaxes/crbasic.tmLanguage.json)
  ↓
VSCode Tokenization
  ↓
Syntax Highlighting
```

### LSP Feature Flow

```text
User Types in VSCode
  ↓
VSCode Extension (extension.ts)
  ↓
LSP Client (vscode-languageclient, stdio transport)
  ↓
Native LSP Server Binary (crbasic-lsp, spawned as a child process)
  ↓
Parser (crbasic-parser)
  ↓
AST + Diagnostics/Completions
  ↓
[Return through same path]
  ↓
VSCode UI (IntelliSense, Diagnostics, etc.)
```

Note: this path does not go through `crbasic-wasm`. The WASM crate is an independent wrapper around `crbasic-parser` that this client does not use (see [Core Components → WASM Layer](#wasm-layer-crbasic-wasm) below).

## 3. Core Components

### Parser Layer (`crbasic-parser`)

**Responsibility**: LSP-independent parsing of CRBasic source code

**Components**:

- **Lexer**: Tokenizes CRBasic source code
  - Handles single-quote comments (`'`)
  - Recognizes line continuation (`_`)
  - Identifies keywords, identifiers, literals
- **Parser**: Builds Abstract Syntax Tree (AST)
  - Parses program structure (declarations, data tables, execution blocks)
  - Handles control flow structures (If/Then/ElseIf/Else, For/Next, Do/Loop, While/Wend, preprocessor #If/#ElseIf/#Else/#EndIf/#IfDef/#UnDef)
  - Parses function/subroutine definitions
- **AST**: Type-safe representation of program structure
- **Semantic Analyzer**: Model-dependent validation (see Domain Constraints below)

**Key Design Decisions**:

- Pure Rust implementation (no unsafe code)
- Designed for reuse outside the LSP context
- Thorough error recovery for partial or invalid programs

**Domain Constraints** (CRBasic language rules the parser encodes):

1. **Variable Name Length Limits** (model-dependent):
   - CR200(X) series (`.cr2`): 16 chars max, truncated to 12 chars in output tables
   - CR6/CR1000/CR1000X/CR300-series/GRANITE-series: 39 chars max, recommended ≤ 35 chars
2. **Scope Rules**: `Public` variables are always global (even when declared in subroutines); `Dim` variables are local/scratch variables
3. **Function vs. Subroutine Semantics**: Functions copy parameters in and return a single value with **no copy-back**; Subroutines copy parameters in and **copy them back** on exit (reference-like behavior)
4. **Line Continuation**: Space + underscore (`_`) at end of line
5. **Comments**: Single quote (`'`) to end of line

### LSP Layer (`crbasic-lsp`)

**Responsibility**: Language Server Protocol implementation providing IDE features

**Components**:

- **Backend**: `backend.rs` implements `tower-lsp-server`'s `LanguageServer` trait; lifecycle, document sync, and capability negotiation live here
- **Handlers**: one module per LSP request/response pair, each in its own file (see [Project Structure](#1-project-structure) above): `textDocument/completion`, `hover`, `signatureHelp`, `definition`, `references`, `documentHighlight`, `documentSymbol`, `workspaceSymbol`, `codeAction`, `codeLens`, `foldingRange`, `selectionRange`, `linkedEditingRange`, `inlayHint`, `semanticTokens`, `rename` (with `prepareRename`), and `callHierarchy` (prepare/incoming/outgoing)
- **Diagnostics**: push model only. `backend.rs` analyzes the document on open/change and calls `publish_diagnostics` directly; there is no `textDocument/diagnostic` pull handler or `diagnosticProvider` capability. Validation itself is model-dependent (variable name length, 12-char truncation collisions for CR200X, structure validation for `BeginProg`/`EndProg`, `Function`/`EndFunction` pairing)
- **Completion**: Context-aware code completion (built-in instruction database, control/declaration keywords, variable and function references)

**Key Design Decisions**:

- Uses `tower-lsp-server` for LSP protocol implementation (migrated from the unmaintained `tower-lsp`; see [ADR-005](./adrs/adr-005-tower-lsp-server-migration.md))
- Stateful server that maintains symbol tables and document state
- Model detection via file extension (`.cr1`, `.cr6`, etc.)
- Depends only on `crbasic-parser`, not on `crbasic-wasm` (see [ADR-001](./adrs/adr-001-rust-wasm-lsp-architecture.md))

### WASM Layer (`crbasic-wasm`)

**Responsibility**: WebAssembly bindings for `crbasic-parser`'s tokenize/parse/analyze API, for consumers other than this project's VSCode extension (for example, browser-based tooling)

**Components**:

- WASM bindings using `wasm-bindgen`
- JavaScript/TypeScript API exports
- Memory-efficient data transfer between JS and Rust

**Key Design Decisions**:

- Minimal API surface (only exposes `crbasic-parser`'s tokenize/parse/analyze functions)
- Built with `wasm-pack` for npm compatibility
- **Not used by the shipped VSCode extension**: `crbasic-lsp` depends on `tokio`/`mio`, which are incompatible with the `wasm32` target, so the LSP server cannot run as WASM. The client instead bundles and spawns the native `crbasic-lsp` binary (see the Client Layer below and [ADR-004](./adrs/adr-004-multi-platform-packaging.md)). This crate exists only so `crbasic-parser` can be reused outside this project's client.

### Client Layer (`client/`)

**Responsibility**: VSCode extension providing UI integration

**Components**:

- **Extension**: VSCode extension activation and lifecycle
- **LSP Client**: Spawns and communicates with the native `crbasic-lsp` binary over stdio, via `vscode-languageclient`
- **TextMate Grammar**: Syntax highlighting definition

**Key Design Decisions**:

- Uses VSCode's `vscode-languageclient` library, with `TransportKind.stdio` talking to a bundled, platform-specific `crbasic-lsp` executable (not WASM; see the WASM Layer above)
- Vite for fast development and optimized builds
- TextMate grammar as the first level of syntax highlighting (before the LSP server activates)

## 4. Data Stores

None. This project has no database and no persistent server-side storage. `crbasic-lsp` keeps only in-memory, per-document state (parsed AST, symbol tables) for the lifetime of the editor session, and writes nothing to disk beyond what VSCode's own document synchronization handles.

## 5. External Integrations/APIs

None at runtime. This extension **deliberately does not integrate** with Campbell Scientific's official toolchain (CRBasic Editor, LoggerNet, Short Cut). Diagnostics come entirely from this project's own parser (`crbasic-parser`), not the official compiler. See `client/README.md`'s "Known Limitations" section.

The only external API surface is used at distribution time, not runtime: the VS Code Marketplace publishing API (see Deployment & Infrastructure below).

## 6. Deployment & Infrastructure

- **CI** (`.github/workflows/ci.yml`): two jobs mirroring `just verify`, named after their top-level directories: `crates` (fmt/clippy/test/coverage) and `client` (lint/format/type-check/test/grammar-check). They run on push to `main`, on pull requests, and manually. Each job's steps are skipped when neither its own paths nor the workflow file changed (`dorny/paths-filter`). A third job, `gate`, depends on both and fails if either did not succeed, so branch protection only needs to require that one check.
- **Release** (`.github/workflows/release.yml`, see [ADR-003](./adrs/adr-003-release-process.md)): triggered only by pushing a `v*.*.*` tag. Asserts the tag matches `Cargo.toml`/`client/package.json`, re-runs the CI quality gates, then runs `build` → `package` → `publish`.
- **Packaging** (see [ADR-004](./adrs/adr-004-multi-platform-packaging.md)): `crbasic-lsp` is built as a native binary for 6 targets (`linux-x64`, `linux-arm64`, `darwin-x64`, `darwin-arm64`, `win32-x64`, `win32-arm64`); `client/scripts/copy-server.js` bundles the matching binary into `client/server/`, producing one `.vsix` per platform.
- **Publishing**: the `publish` job runs `vsce publish --packagePath ../dist-vsix/*.vsix` for every packaged `.vsix`, authenticated with a `VSCE_PAT` repository secret (Marketplace: Manage scope). If the secret is unset, the step logs a warning and exits 0 instead of failing.
- **Versioning**: lockstep SemVer across the Cargo workspace and `client/package.json` (one version number for the whole repo). See ADR-003 for the pre-1.0 MINOR/PATCH convention and the criteria for `1.0.0`.

## 7. Security Considerations

- No network exposure: the LSP server communicates with the client only over stdio as a local child process; it never opens a network socket.
- No authentication surface: it is a single-user local process within the same trust boundary as the editor.
- The in-scope vulnerability classes (WASM sandbox escapes, parser/lexer crashes on adversarial input, resource exhaustion, LSP/extension content injection) and the reporting process are documented in [`SECURITY.md`](../SECURITY.md). Use GitHub's private vulnerability reporting rather than a public issue.

## 8. Development & Testing Environment

- **Setup and commands**: [`CONTRIBUTING.md`](../CONTRIBUTING.md) is the single source for setup steps, the `just` command reference, and the pull request process. They are not duplicated here, so the two docs cannot drift apart.
- **Testing frameworks**: Rust's built-in test framework (unit tests colocated with implementation via `#[cfg(test)] mod tests`, integration tests under each crate's `tests/` directory) and Vitest for the TypeScript client (`*.test.ts`, colocated with source).
- **Coverage tooling**: `cargo-llvm-cov` for Rust, Vitest's built-in coverage for TypeScript. Targets and the Red → Green → Refactor policy are in `CONTRIBUTING.md`'s "Testing guidelines".
- **Linting & Formatting**: Rust via clippy + rustfmt; TypeScript via ESLint + Prettier; enforced locally via `pre-commit` hooks and in CI.

## 9. Future Considerations/Roadmap

- [ ] Whole-document code formatting (`textDocument/formatting`); type-time auto-indent is already implemented via `client/language-configuration.json`
- [ ] Further refactoring support (extract subroutine); rename is already implemented
- [ ] Integration with Campbell Scientific toolchain (program compilation, deployment)

For granular, actively tracked progress (what's done, in progress, or blocked), see [`docs/todo.md`](./todo.md), the maintained status ledger. This section is a stable list of directions.

## 10. Project Identification

- **Name**: crbasic-lsp-rs
- **Repository**: <https://github.com/connect0459/crbasic-lsp-rs>
- **License**: [MIT](../LICENSE)
- **Contact**: GitHub Issues for questions/bugs; GitHub private vulnerability reporting for security issues (see [`SECURITY.md`](../SECURITY.md))
- **Document last reviewed**: 2026-08-16

## 11. Glossary/Acronyms

- **LSP**: Language Server Protocol -- the client/server protocol this project implements to provide IDE features independent of any particular editor.
- **AST**: Abstract Syntax Tree -- `crbasic-parser`'s structured representation of a parsed CRBasic program.
- **WASM**: WebAssembly -- the target `crbasic-wasm` compiles `crbasic-parser` to; not used by this project's own VSCode extension (see [Core Components → WASM Layer](#wasm-layer-crbasic-wasm)).
- **ADR**: Architecture Decision Record; see [`docs/adrs/`](./adrs/).
- **`Public` / `Dim`**: CRBasic variable declaration keywords; `Public` variables are always global and monitorable, `Dim` variables are local/scratch.
- **Copy-back**: Subroutine parameter behavior where the caller's argument is updated with the subroutine's final parameter value on exit (unlike Functions, which never copy back).
- **`BeginProg`/`EndProg`, `DataTable`/`EndTable`**: CRBasic's top-level program and output-table block delimiters.
- **TextMate grammar**: The JSON-based tokenization ruleset (`crbasic.tmLanguage.json`) that VSCode uses for syntax highlighting before any LSP server is involved.
- **Datalogger models covered**: CR200(X), CR1000, CR1000X, CR3000, CR5000, CR6, CR800, CR9000(X), CR300, and GRANITE-series -- each with its own variable-name-length validation profile (see Domain Constraints under Core Components → Parser Layer).

## References

- [Research Document](./researches/research-001-crbasic-for-vscode.md): Comprehensive CRBasic language analysis
- [ADR Index](./adrs/): Architecture decision records
- [LSP Specification](https://microsoft.github.io/language-server-protocol/): Language Server Protocol
- [TextMate Grammar Guide](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide): VSCode syntax highlighting
