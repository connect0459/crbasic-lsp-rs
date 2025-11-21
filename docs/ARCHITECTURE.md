# Architecture

## Overview

`crbasic-lsp-rs` is a Visual Studio Code extension that provides comprehensive language support for CRBasic, a programming language designed for Campbell Scientific data loggers.

The project implements syntax highlighting via TextMate Grammar and advanced language features through a Language Server Protocol (LSP) implementation.

## Technology Stack

### Core Technologies

- **Client Side**: TypeScript + Vite
- **LSP Server**: Rust + WebAssembly (WASM)
- **Build System**: Cargo workspace (Rust) + npm/Vite (TypeScript)

### Testing & Quality

- **Testing Framework**:
  - Rust: built-in test framework + `cargo-tarpaulin` for coverage
  - TypeScript: Vitest
- **Linting & Formatting**:
  - Rust: clippy + rustfmt
  - TypeScript: ESLint + Prettier
- **Pre-commit Hooks**: pre-commit framework

## Project Structure

```text
crbasic-lsp-rs/
├── crates/                      # Rust workspace
│   ├── crbasic-parser/          # Core parser logic
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lexer/           # Lexical analysis
│   │   │   ├── parser/          # Syntax analysis
│   │   │   └── ast/             # Abstract Syntax Tree
│   │   └── tests/
│   ├── crbasic-lsp/             # LSP server implementation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── server.rs        # LSP server core
│   │   │   ├── handlers/        # LSP request handlers
│   │   │   ├── diagnostics/     # Diagnostic features
│   │   │   └── completion/      # IntelliSense completion
│   │   └── tests/
│   └── crbasic-wasm/            # WASM bindings
│       ├── src/
│       └── pkg/                 # WASM build output
├── client/                      # VSCode extension client (TypeScript + Vite)
│   ├── src/
│   │   ├── extension.ts         # Extension entry point
│   │   └── client.ts            # LSP client
│   ├── syntaxes/
│   │   └── crbasic.tmLanguage.json  # TextMate Grammar
│   ├── package.json
│   ├── tsconfig.json
│   └── vite.config.ts
├── docs/
│   ├── ARCHITECTURE.md          # This file
│   ├── adrs/                    # Architecture Decision Records
│   ├── researches/              # Research documents
│   └── sample-codes/            # Sample CRBasic programs
├── Cargo.toml                   # Rust workspace configuration
├── .pre-commit-config.yaml
├── .gitignore
├── CLAUDE.md                    # Claude Code collaboration rules
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
  - Handles control flow structures (If/Then/Else, For/Next, Do/Loop)
  - Parses function/subroutine definitions
- **AST**: Type-safe representation of program structure

**Key Design Decisions**:

- Pure Rust implementation (no unsafe code)
- Designed for reusability (can be used outside LSP context)
- Comprehensive error recovery for partial/invalid programs

### 2. LSP Layer (`crbasic-lsp`)

**Responsibility**: Language Server Protocol implementation providing IDE features

**Components**:

- **Server**: LSP server lifecycle management
- **Handlers**: LSP request/response handlers
  - `textDocument/completion`: IntelliSense completions
  - `textDocument/hover`: Hover information
  - `textDocument/definition`: Go to definition
  - `textDocument/references`: Find all references
  - `textDocument/diagnostic`: Real-time diagnostics
- **Diagnostics**: Model-dependent validation
  - Variable name length validation (CR200X: 12 chars, CR6/GRANITE: 35-39 chars)
  - Duplicate field name detection (12-char truncation collision for CR200X)
  - Structure validation (BeginProg/EndProg, Function/EndFunction pairing)
- **Completion**: Context-aware code completion
  - Built-in instruction database (measurement, communication, data processing)
  - Control keywords and declaration keywords
  - Variable and function references

**Key Design Decisions**:

- Uses `tower-lsp` for LSP protocol implementation
- Stateful server maintaining symbol tables and document state
- Model detection via file extension (`.cr1`, `.cr6`, etc.)

### 3. WASM Layer (`crbasic-wasm`)

**Responsibility**: WebAssembly bindings for browser/VSCode execution

**Components**:

- WASM bindings using `wasm-bindgen`
- JavaScript/TypeScript API exports
- Memory-efficient data transfer between JS and Rust

**Key Design Decisions**:

- Minimal API surface (only expose what client needs)
- Efficient serialization for AST/diagnostic data
- Built with `wasm-pack` for npm compatibility

### 4. Client Layer (`client/`)

**Responsibility**: VSCode extension providing UI integration

**Components**:

- **Extension**: VSCode extension activation and lifecycle
- **LSP Client**: Communication with LSP server via WASM
- **TextMate Grammar**: Syntax highlighting definition

**Key Design Decisions**:

- Uses VSCode's `vscode-languageclient` library
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
LSP Client (client.ts)
  ↓
WASM LSP Server (crbasic-wasm)
  ↓
LSP Server (crbasic-lsp)
  ↓
Parser (crbasic-parser)
  ↓
AST + Diagnostics/Completions
  ↓
[Return through same path]
  ↓
VSCode UI (IntelliSense, Diagnostics, etc.)
```

## Testing Strategy

### Test Coverage Targets

Based on project characteristics (new development, medium importance, medium risk):

| Metric | Target |
| :--- | :--- |
| Line Coverage | ≥ 80% |
| Branch Coverage | ≥ 75% |
| Function Coverage | ≥ 90% |

### Testing Approach

- **TDD (Test-Driven Development)**: Write tests before implementation
- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test component interactions
- **E2E Tests**: Test full VSCode extension functionality

### Test Organization

**Rust (`crates/`)**:

- Unit tests: Same file as implementation (`#[cfg(test)] mod tests`)
- Integration tests: `tests/` directory in each crate
- Test structure: Hierarchical using `mod` and `#[test]`

**TypeScript (`client/`)**:

- Test files: `*.test.ts` (same directory as source)
- Test structure: `describe()` / `test()` hierarchies
- Framework: Vitest

## Build & Development Workflow

### Development Setup

```bash
# Install Rust dependencies
cargo build

# Install TypeScript dependencies
cd client && npm install

# Install pre-commit hooks
pre-commit install
```

### Development Commands

```bash
# Run Rust tests
cargo test

# Run Rust linter
cargo clippy

# Format Rust code
cargo fmt

# Build WASM
cd crates/crbasic-wasm && wasm-pack build

# Run TypeScript tests
cd client && npm test

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
   - CR200(X) series: 16 chars max, truncated to 12 chars in output tables
   - CR6/CR1000X/GRANITE series: 39 chars max, recommended ≤ 35 chars

2. **Scope Rules**:
   - `Public` variables are always global (even when declared in subroutines)
   - `Dim` variables are local/scratch variables

3. **Function vs. Subroutine Semantics**:
   - Functions: Parameters copied in, single return value, **no copy-back**
   - Subroutines: Parameters copied in, **copied back** on exit (reference-like behavior)

4. **Line Continuation**: Space + underscore (`_`) at end of line

5. **Comments**: Single quote (`'`) to end of line

## Extension Points

Future enhancements may include:

- [ ] Code formatting (auto-indent, structure alignment)
- [ ] Refactoring support (rename variable, extract subroutine)
- [ ] Snippet library for common measurement patterns
- [ ] Datalogger model-specific validation profiles
- [ ] Integration with Campbell Scientific toolchain (program compilation, deployment)

## References

- [Research Document](./researches/research-001-crbasic-for-vscode.md): Comprehensive CRBasic language analysis
- [ADR Index](./adrs/): Architecture decision records
- [LSP Specification](https://microsoft.github.io/language-server-protocol/): Language Server Protocol
- [TextMate Grammar Guide](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide): VSCode syntax highlighting
