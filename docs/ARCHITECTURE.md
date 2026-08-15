# ARCHITECTURE

## Overview

`crbasic-lsp-rs` is a Visual Studio Code extension that provides comprehensive language support for CRBasic, a programming language designed for Campbell Scientific data loggers.

The project implements syntax highlighting via TextMate Grammar and advanced language features through a Language Server Protocol (LSP) implementation.

## Technology Stack

### Core Technologies

- **Client Side**: TypeScript + Vite
- **LSP Server**: Rust, distributed as a native per-platform binary (see
  [ADR-004](./adrs/adr-004-multi-platform-packaging.md))
- **Parser Core**: Rust, also exposed as a standalone WebAssembly package
  (`crbasic-wasm`) for non-VSCode consumers; the shipped extension does not
  use it (see [Architecture Layers](#3-wasm-layer-crbasic-wasm) below)
- **Build System**: Cargo workspace (Rust) + npm/Vite (TypeScript)

### Testing & Quality

- **Testing Framework**:
  - Rust: built-in test framework + `cargo-llvm-cov` for coverage
  - TypeScript: Vitest
- **Linting & Formatting**:
  - Rust: clippy + rustfmt
  - TypeScript: ESLint + Prettier
- **Pre-commit Hooks**: pre-commit framework

## Project Structure

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
│   │       ├── sample_files.rs      # Integration tests over real sample programs
│   │       ├── example_programs.rs  # Assertions on docs/examples/*
│   │       └── performance.rs       # Lexer/parser performance budgets
│   ├── crbasic-lsp/             # LSP server implementation
│   │   ├── src/
│   │   │   ├── lib.rs                 # LSP public interface
│   │   │   ├── main.rs                # Binary entry point
│   │   │   ├── backend.rs             # tower-lsp-server `LanguageServer` impl
│   │   │   ├── document.rs            # Document state management
│   │   │   ├── completion.rs          # IntelliSense completion
│   │   │   ├── hover.rs                # Hover information
│   │   │   ├── signature.rs            # Signature help
│   │   │   ├── definition.rs           # Go to definition
│   │   │   ├── references.rs           # Find all references
│   │   │   ├── symbols.rs              # Document symbols
│   │   │   ├── workspace_symbol.rs     # Workspace symbol search
│   │   │   ├── document_highlight.rs   # Document highlight
│   │   │   ├── code_action.rs          # Quick fixes
│   │   │   ├── code_lens.rs            # Code lens
│   │   │   ├── folding.rs              # Folding ranges
│   │   │   ├── selection_range.rs      # Selection ranges
│   │   │   ├── linked_editing_range.rs # Linked editing ranges
│   │   │   ├── inlay_hint.rs           # Inlay hints
│   │   │   ├── semantic_tokens.rs      # Semantic highlighting
│   │   │   ├── rename.rs               # Rename (with prepare support)
│   │   │   ├── call_hierarchy.rs       # Call hierarchy
│   │   │   └── call_sites.rs           # Shared call-site resolution helper
│   │   └── tests/
│   │       └── lsp_integration.rs      # End-to-end protocol-level tests
│   └── crbasic-wasm/            # WASM bindings for `crbasic-parser`
│       ├── src/
│       │   └── lib.rs           # WASM API (tokenize/parse/analyze)
│       └── pkg/                 # WASM build output
├── client/                      # VSCode extension client (TypeScript + Vite)
│   ├── src/
│   │   ├── extension.ts         # Extension entry point (spawns the native
│   │   │                        # crbasic-lsp binary over stdio)
│   │   └── commands.ts          # Command-handler logic (restart, show output)
│   ├── scripts/
│   │   ├── copy-server.js       # Bundles the native binary into client/server/
│   │   ├── targets.js           # Per-platform Rust target definitions
│   │   ├── package-vsix.js      # Builds one .vsix per platform target
│   │   └── place-artifacts.js   # Stages built binaries for packaging
│   ├── syntaxes/
│   │   └── crbasic.tmLanguage.json  # TextMate Grammar (generated; see
│   │                                 # scripts/generate-grammar.js)
│   ├── language-configuration.json  # Bracket/comment/indentation rules
│   ├── images/                  # Extension icon assets
│   ├── README.md                # Marketplace-facing README (bundled into .vsix)
│   ├── LICENSE                  # Copy of the repo LICENSE (bundled into .vsix)
│   ├── package.json             # Extension manifest
│   ├── tsconfig.json
│   ├── vite.config.ts           # Vite configuration
│   ├── vitest.config.ts         # Vitest configuration
│   └── eslint.config.js         # ESLint 9 Flat Config
├── scripts/
│   └── generate-grammar.js      # Regenerates keywords_generated.rs and
│                                 # crbasic.tmLanguage.json from keywords.json
├── docs/
│   ├── ARCHITECTURE.md          # This file
│   ├── todo.md                  # Project progress tracker
│   ├── adrs/                    # Architecture Decision Records
│   ├── researches/              # Research documents
│   ├── examples/                # Curated feature-showcase example programs
│   └── sample-codes/            # Sample CRBasic programs (11 datalogger models)
├── Cargo.toml                   # Rust workspace configuration
├── justfile                     # Task runner (setup, test, coverage, verify, ...)
├── .pre-commit-config.yaml
├── .gitignore
├── CLAUDE.md                    # Project-specific Claude Code rules
├── AGENTS.md                    # Same rules, generic-agent-facing filename
└── README.md
```

## Architecture Layers

### 1. Parser Layer (`crbasic-parser`)

**Responsibility**: Language-agnostic parsing of CRBasic source code

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

**Key Design Decisions**:

- Pure Rust implementation (no unsafe code)
- Designed for reusability (can be used outside LSP context)
- Comprehensive error recovery for partial/invalid programs

### 2. LSP Layer (`crbasic-lsp`)

**Responsibility**: Language Server Protocol implementation providing IDE features

**Components**:

- **Backend**: `backend.rs` implements `tower-lsp-server`'s `LanguageServer`
  trait; lifecycle, document sync, and capability negotiation live here
- **Handlers**: one module per LSP request/response pair, each with its own
  file (see [Project Structure](#project-structure) above) --
  `textDocument/completion`, `hover`, `signatureHelp`, `definition`,
  `references`, `documentHighlight`, `documentSymbol`, `workspaceSymbol`,
  `codeAction`, `codeLens`, `foldingRange`, `selectionRange`,
  `linkedEditingRange`, `inlayHint`, `semanticTokens`, `rename` (with
  `prepareRename`), `callHierarchy` (prepare/incoming/outgoing), and
  `textDocument/diagnostic`
- **Diagnostics**: Model-dependent validation
  - Variable name length validation (CR200X: 12 chars, CR6/GRANITE: 35-39 chars)
  - Duplicate field name detection (12-char truncation collision for CR200X)
  - Structure validation (BeginProg/EndProg, Function/EndFunction pairing)
- **Completion**: Context-aware code completion
  - Built-in instruction database (measurement, communication, data processing)
  - Control keywords and declaration keywords
  - Variable and function references

**Key Design Decisions**:

- Uses `tower-lsp-server` for LSP protocol implementation (migrated from the
  unmaintained `tower-lsp`; see
  [ADR-005](./adrs/adr-005-tower-lsp-server-migration.md))
- Stateful server maintaining symbol tables and document state
- Model detection via file extension (`.cr1`, `.cr6`, etc.)
- Depends only on `crbasic-parser`, not on `crbasic-wasm` (see
  [ADR-001](./adrs/adr-001-rust-wasm-lsp-architecture.md))

### 3. WASM Layer (`crbasic-wasm`)

**Responsibility**: WebAssembly bindings for `crbasic-parser`'s tokenize/
parse/analyze API, for consumers other than this project's own VSCode
extension (e.g. browser-based tooling)

**Components**:

- WASM bindings using `wasm-bindgen`
- JavaScript/TypeScript API exports
- Memory-efficient data transfer between JS and Rust

**Key Design Decisions**:

- Minimal API surface (only exposes `crbasic-parser`'s tokenize/parse/
  analyze functions)
- Efficient serialization for AST/diagnostic data
- Built with `wasm-pack` for npm compatibility
- **Not used by the shipped VSCode extension**: `crbasic-lsp` depends on
  `tokio`/`mio`, which are incompatible with the `wasm32` target, so the LSP
  server itself cannot run as WASM. The client instead bundles and spawns
  `crbasic-lsp`'s native binary directly (see the Client Layer below and
  [ADR-004](./adrs/adr-004-multi-platform-packaging.md)); this crate exists
  purely to let `crbasic-parser` be reused outside this project's own client.

### 4. Client Layer (`client/`)

**Responsibility**: VSCode extension providing UI integration

**Components**:

- **Extension**: VSCode extension activation and lifecycle
- **LSP Client**: Spawns and communicates with the native `crbasic-lsp`
  binary over stdio, via `vscode-languageclient`
- **TextMate Grammar**: Syntax highlighting definition

**Key Design Decisions**:

- Uses VSCode's `vscode-languageclient` library, with `TransportKind.stdio`
  talking to a bundled, platform-specific `crbasic-lsp` executable (not
  WASM -- see the WASM Layer above)
- Vite for fast development and optimized builds
- TextMate Grammar as first-level syntax highlighting (before LSP activation)

## Data Flow

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

Note: this path does not go through `crbasic-wasm` -- the WASM crate is an
independent, unused-by-this-client wrapper around `crbasic-parser` only (see
[Architecture Layers → WASM Layer](#3-wasm-layer-crbasic-wasm) above).

## Testing Strategy

### Test Coverage Targets

Based on project characteristics (new development, medium importance, medium risk):

| Metric | Target | Current Status |
| :--- | :--- | :--- |
| Line Coverage | ≥ 80% | ✅ Achieved |
| Branch Coverage | ≥ 75% | ✅ Achieved |
| Function Coverage | ≥ 90% | ✅ Achieved |

### Current Test Status

**Total Tests**: 1,286 passing | 0 ignored | 0 failed (plus 3 rustdoc tests)

**Test Breakdown** (counts drift as instruction coverage grows; re-run
`cargo test --workspace` and `cd client && npm run test.run` for current
figures rather than trusting this table long-term):

- **`crbasic-parser` unit tests** (308 tests): Lexer, parser, AST, and
  semantic-analysis coverage, colocated with the implementation
- **`crbasic-parser` integration tests** (34 tests): `sample_files.rs` (27,
  real-world CRBasic programs across all 11 supported datalogger models),
  `example_programs.rs` (3, `docs/examples/*` assertions), `performance.rs`
  (4, lexer/parser performance budgets)
- **`crbasic-lsp` unit tests** (665 tests): One module per LSP feature
  (completion, hover, signature help, definitions, references, symbols,
  workspace symbols, document highlight, code actions, code lens, folding,
  selection ranges, linked editing ranges, inlay hints, semantic tokens,
  rename, call hierarchy, diagnostics)
- **`crbasic-lsp` integration tests** (36 tests): `lsp_integration.rs`,
  end-to-end protocol-level requests against the real backend
- **`crbasic-wasm` unit tests** (19 tests): Tokenize, parse, analyze APIs
- **Client tests** (224 tests, Vitest): command handlers, extension
  activation, TextMate grammar, and language configuration

### Testing Approach

- **TDD (Test-Driven Development)**: All features implemented with tests-first approach
- **Unit Tests**: Component-level testing with comprehensive edge case coverage
- **Integration Tests**: Full sample file parsing (11 datalogger models)
- **Regression Tests**: All parser limitations resolved and tested

### Test Organization

**Rust (`crates/`)**:

- Unit tests: Same file as implementation (`#[cfg(test)] mod tests`)
- Integration tests: `tests/` directory in each crate
- Test structure: Hierarchical using `mod` and `#[test]`
- Test naming: English descriptions of behavior (evergreen tests)

**TypeScript (`client/`)**:

- Test files: `*.test.ts` (same directory as source)
- Test structure: `describe()` / `test()` hierarchies
- Framework: Vitest
- 224 tests across 4 files (`commands.test.ts`, `extension.test.ts`,
  `syntax-highlighting.test.ts`, `language-configuration.test.ts`)

## Build & Development Workflow

### Development Setup

```bash
# Equivalent to the three steps below
just setup

# Install Rust dependencies
cargo build

# Install TypeScript dependencies
cd client && npm install

# Install pre-commit hooks
pre-commit install
```

### Development Commands

```bash
# Run the same checks as CI: fmt, clippy, tests, coverage, grammar
# generation, client lint/format/test
just verify

# Run Rust tests
cargo test

# Run Rust linter
cargo clippy

# Format Rust code
cargo fmt

# Build WASM
cd crates/crbasic-wasm && wasm-pack build

# Run TypeScript tests
cd client && npm run test.run

# Run TypeScript linter
cd client && npm run lint

# Format TypeScript code
cd client && npm run format

# Run extension in development
cd client && npm run dev
```

### Pre-commit Validation

All commits are validated by pre-commit hooks:

- Rust: `cargo fmt --check`, `cargo clippy`
- TypeScript: ESLint, Prettier
- General: File size checks, trailing whitespace removal

## Key Technical Constraints

### CRBasic Language Constraints

1. **Variable Name Length Limits** (model-dependent):
   - CR200(X) series (`.cr2`): 16 chars max, truncated to 12 chars in output tables
   - CR6/CR1000/CR1000X/CR300-series/GRANITE-series: 39 chars max, recommended ≤ 35 chars

2. **Scope Rules**:
   - `Public` variables are always global (even when declared in subroutines)
   - `Dim` variables are local/scratch variables

3. **Function vs. Subroutine Semantics**:
   - Functions: Parameters copied in, single return value, **no copy-back**
   - Subroutines: Parameters copied in, **copied back** on exit (reference-like behavior)

4. **Line Continuation**: Space + underscore (`_`) at end of line

5. **Comments**: Single quote (`'`) to end of line

## Implementation Status

### Completed Features ✅

#### Phase 1-7: Core Implementation (Complete)

- ✅ Lexer: Full tokenization with keyword recognition
- ✅ Parser: Complete AST construction with all CRBasic constructs
- ✅ Semantic Analysis: Model-dependent validation
- ✅ LSP Features: All major LSP capabilities implemented
  - Document synchronization
  - Real-time diagnostics
  - IntelliSense (code completion)
  - Signature help
  - Hover information
  - Go to definition
  - Find all references
  - Document symbols and workspace symbols
  - Document highlight
  - Code actions (quick fixes) and code lens
  - Folding ranges and selection ranges
  - Linked editing ranges
  - Inlay hints
  - Semantic tokens
  - Rename (with prepare support)
  - Call hierarchy (prepare, incoming, outgoing)
- ✅ WASM Integration: Full JavaScript API for `crbasic-parser`, for
  consumers other than the shipped VSCode extension (see
  [WASM Layer](#3-wasm-layer-crbasic-wasm) above)
- ✅ VSCode Extension: Client integration with a bundled, native
  per-platform `crbasic-lsp` binary (see
  [ADR-004](./adrs/adr-004-multi-platform-packaging.md))

#### Phase 8: Documentation & Polish (In Progress)

- ✅ API Documentation: Complete rustdoc for all public APIs
- ✅ User Guide: README updated with current status
- ✅ Developer Guide: ARCHITECTURE updated with implementation details
- ✅ Build Warnings: ESLint 9 upgrade, Vite CJS API warnings resolved

### Parser Capabilities

**Fully Supported**:

- ✅ All CRBasic keywords and operators
- ✅ Variable declarations (Public, Dim, Const) with comma-separated syntax
- ✅ Control flow (If/Then/ElseIf/Else, For/Next, Do/Loop, While/Wend); preprocessor directives (#If/#ElseIf/#Else/#EndIf/#IfDef/#UnDef, parsed structurally, not evaluated)
- ✅ Function and Subroutine definitions
- ✅ Array access and multi-dimensional arrays
- ✅ Binary and unary operations with correct precedence
- ✅ Function calls with nested arguments
- ✅ Program structure (BeginProg/EndProg, DataTable/EndTable, NextScan)
- ✅ Boolean literals (True/False)
- ✅ Line continuation and comments
- ✅ Tab-indented statements

**Model-Specific Validation**:

- ✅ CR200X: 16-char max, 12-char truncation warnings, collision detection
- ✅ CR6 (also covers CR1000/CR1000X/CR300-series/GRANITE-series): 39-char
  max, 35-char recommendations
- ✅ File extension detection across CR200(X), CR1000(X), CR3000, CR5000,
  CR6, CR800, CR9000(X), CR300 program files

## Extension Points

Future enhancements may include:

- [ ] Code formatting (auto-indent, structure alignment)
- [ ] Further refactoring support (extract subroutine) -- rename is already
  implemented, see [Completed Features](#completed-features-) above
- [ ] Snippet library for common measurement patterns
- [ ] Advanced datalogger-specific validation profiles
- [ ] Integration with Campbell Scientific toolchain (program compilation, deployment)
- [ ] Performance optimization for large files (>1000 lines)

## References

- [Research Document](./researches/research-001-crbasic-for-vscode.md): Comprehensive CRBasic language analysis
- [ADR Index](./adrs/): Architecture decision records
- [LSP Specification](https://microsoft.github.io/language-server-protocol/): Language Server Protocol
- [TextMate Grammar Guide](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide): VSCode syntax highlighting
