# CRBasic LSP Development TODO

## Phase 1: Project Setup ✅

- [x] Architecture design and ADR creation
- [x] Project structure setup (Rust/Cargo + TypeScript/Vite)
- [x] Linter/Formatter setup (Rust: clippy/rustfmt, TS: ESLint/Prettier)
- [x] Pre-commit hooks configuration
- [x] TextMate Grammar implementation (syntax highlighting)

## Phase 2: Lexer Implementation (TDD) ✅

### Core Token Types

- [x] Token type definitions (Rust)
- [x] Position and Span types

### Basic Tokenization

- [x] Empty source code → EOF token
- [x] Single-quote comments (`'`) recognition
- [x] Numeric literals (integer and float)
- [x] String literals (double-quoted)
- [x] Identifiers (variable names, function names)
- [x] Keywords (case-insensitive)
  - [x] Control flow: `If`, `Then`, `Else`, `For`, `Next`, `Do`, `Loop`, etc.
  - [x] Declarations: `Public`, `Dim`, `Const`, `Alias`, `As`
  - [x] Program structure: `BeginProg`, `EndProg`, `DataTable`, `EndTable`
  - [x] Functions: `Function`, `EndFunction`, `Sub`, `EndSub`
- [x] Operators
  - [x] Arithmetic: `+`, `-`, `*`, `/`, `^`
  - [x] Comparison: `=`, `<>`, `<`, `>`, `<=`, `>=`
  - [x] Logical: `AND`, `OR`, `NOT`, `XOR` (already as keywords)
- [x] Delimiters: `(`, `)`, `[`, `]`, `,`
- [x] Line continuation (space + `_` at end of line)
- [x] Newline handling

### Edge Cases

- [x] Comments mid-line (after code)
- [x] Empty comments
- [x] Multi-line programs with mixed tokens
- [x] Case-insensitive keyword matching (covered in keywords tests)
- [ ] Invalid UTF-8 handling (future: error handling)

## Phase 3: Parser Implementation (TDD) 🚧

### AST Design

- [x] AST node type definitions
- [x] Program structure representation

### Expression Parsing

- [x] Primary expressions
  - [x] Integer literals
  - [x] Float literals
  - [x] String literals
  - [x] Identifiers
  - [x] Parenthesized expressions
- [x] Binary operations
  - [x] Arithmetic operators (`+`, `-`, `*`, `/`, `^`)
  - [x] Comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`)
  - [x] Logical operators (`AND`, `OR`, `XOR`)
  - [x] Operator precedence (power > unary > mult/div > add/sub > comparison > logical AND > logical XOR > logical OR)
- [x] Unary operations (`-`, `NOT`)
- [x] Function calls (as expressions)
  - [x] No arguments: `TimeIntoInterval()`
  - [x] Single argument: `Sqrt(16)`
  - [x] Multiple arguments: `Scan(1, Temp_C, 0)`
  - [x] Expression arguments: `Max(1 + 2, 5)`
  - [x] Nested function calls: `Avg(Max(1, 2), 3)`
- [ ] Array access

### Statement Parsing

- [ ] Expression statements (placeholder implementation exists)
- [ ] Variable declarations (`Public`, `Dim`, `Const`)
- [ ] Assignment statements
- [ ] Function call statements

### Control Flow Structures

- [ ] `If`/`Then`/`Else`/`EndIf`
- [ ] `For`/`Next` loops
- [ ] `Do`/`Loop` structures

### Program Structure

- [ ] `BeginProg`/`EndProg` parsing
- [ ] `DataTable`/`EndTable` parsing
- [ ] Function/Subroutine definitions

### Semantic Rules

- [ ] Variable scope tracking (Public vs Dim)
- [ ] Function vs Subroutine distinction (copy-back semantics)
- [ ] Model-dependent variable name validation

## Phase 4: LSP Server Implementation 📡

### Basic LSP Features

- [ ] LSP server structure (using `tower-lsp`)
- [ ] Document synchronization
- [ ] Basic diagnostics

### Advanced LSP Features

- [ ] IntelliSense (completion)
  - [ ] Keywords
  - [ ] Built-in functions
  - [ ] User-defined variables/functions
- [ ] Signature help
  - [ ] Built-in function signatures
  - [ ] Parameter documentation
- [ ] Hover information
- [ ] Go to definition
- [ ] Find all references
- [ ] Document symbols (outline view)

### Model-Dependent Validation

- [ ] File extension → model detection (`.cr1` → CR200X, `.cr6` → CR6)
- [ ] Variable name length validation
  - [ ] CR200X: 16 chars max, 12-char truncation warning
  - [ ] CR6/GRANITE: 39 chars max, 35-char recommendation
- [ ] Duplicate field name detection (12-char truncation collision)

## Phase 5: WASM Integration 🌐

- [ ] WASM bindings (`wasm-bindgen`)
- [ ] JavaScript API exports
- [ ] WASM build configuration (`wasm-pack`)
- [ ] Memory-efficient data transfer

## Phase 6: VSCode Extension Client 🔌

- [ ] Extension activation logic
- [ ] LSP client initialization
- [ ] WASM LSP server integration
- [ ] Configuration options
- [ ] Extension commands

## Phase 7: Testing & Quality 🧪

### Unit Tests

- [x] Lexer tests (32 tests passing - coverage target: 80% line, 75% branch)
  - [x] Empty source and EOF
  - [x] Comments (single-quote, mid-line, empty)
  - [x] Numeric literals (integer, float, scientific notation)
  - [x] String literals (simple, escape sequences, escaped quotes)
  - [x] Identifiers (simple, with numbers, starting with underscore)
  - [x] Keywords (case-insensitive matching, canonical form)
  - [x] Operators (arithmetic, comparison)
  - [x] Delimiters (parentheses, brackets, comma)
  - [x] Line continuation
  - [x] Whitespace and newline handling
  - [x] Integration tests (multi-line programs)
- [x] Parser tests (35 tests passing)
  - [x] Primary expressions (literals, identifiers, parentheses)
  - [x] Binary operations (arithmetic, comparison, logical)
  - [x] Unary operations (negation, NOT)
  - [x] Operator precedence validation
  - [x] Parenthesized expressions
  - [x] Function call expressions (no args, single arg, multiple args, expression args, nested)
- [ ] LSP handler tests

### Integration Tests

- [x] End-to-end tokenization tests (4 integration tests)
- [ ] Full parsing tests with sample.cr1
- [ ] LSP feature integration tests

### E2E Tests

- [ ] VSCode extension smoke tests
- [ ] Real-world CRBasic program validation

## Phase 8: Documentation & Polish 📚

- [ ] API documentation (rustdoc)
- [ ] User guide (README updates)
- [ ] Developer guide (ARCHITECTURE.md updates)
- [ ] Example programs
- [ ] Release preparation

## Known Issues / Technical Debt 🐛

- [ ] ESLint 8 deprecation warning (upgrade to ESLint 9)
- [ ] Vite CJS API deprecation warning
- [ ] Performance optimization (large files >1000 lines)

## Future Enhancements 🚀

- [ ] Code formatting (auto-indent)
- [ ] Refactoring support (rename variable)
- [ ] Snippet library
- [ ] Datalogger-specific validation profiles
- [ ] Integration with Campbell Scientific toolchain
