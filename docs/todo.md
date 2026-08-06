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
- [x] Multibyte UTF-8 characters in comments/strings ✅ Resolved
  - `Scanner` mixed byte-offset (`advance`) and char-index (`peek`/`peek_next`)
    semantics for its `current` cursor; any multibyte UTF-8 character (e.g.
    `°`, `µ`, non-ASCII text in a comment or string literal) desynced the two,
    causing `scan_comment_text`/`scan_string` to spin forever instead of
    terminating (confirmed via manual repro before the fix)
  - Fixed by making `peek`/`peek_next` byte-offset aware
    (`source[current..].chars().next()`), matching `advance`
  - As a side effect, removed the `O(n)` `chars().nth()` re-scan on every
    lookahead call

## Phase 3: Parser Implementation (TDD) 🚧

### AST Design

- [x] AST node type definitions
- [x] Program structure representation

### Expression Parsing

- [x] Primary expressions
  - [x] Integer literals
  - [x] Float literals
  - [x] String literals
  - [x] Boolean literals (True/False)
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
- [x] Array access
  - [x] Simple array access: `Data[0]`
  - [x] Variable index: `Temp_C[i]`
  - [x] Expression index: `Data[i + 1]`
  - [x] Multi-dimensional: `Matrix[1][2]`

### Statement Parsing

- [x] Expression statements
  - [x] Function calls converted to FunctionCall statements
  - [x] Other expressions wrapped in Expression statements (for testing flexibility)
- [x] Variable declarations (`Public`, `Dim`, `Const`)
  - [x] Public declaration without type (`Public Temp_C`)
  - [x] Public declaration with type annotation (`Public Temp_C As Float`)
  - [x] Dim declaration (`Dim i`)
  - [x] Const declaration with initializer (`Const PI = 3.14`)
  - [x] Array declarations (`Public Data(100)`, `Dim Matrix(10, 20)`)
- [x] Assignment statements
  - [x] Simple assignment (`x = 5`)
  - [x] Assignment with expressions (`x = 1 + 2`)
  - [x] Assignment to array elements (`Data[0] = 5`, `Matrix[1][2] = 100`)
- [x] Function call statements
  - [x] Function calls with no arguments (`TimeIntoInterval()`)
  - [x] Function calls with arguments (`Scan(1, Temp_C, 0)`)

### Control Flow Structures

- [x] `If`/`Then`/`Else`/`EndIf`
  - [x] Simple If-Then-EndIf
  - [x] If-Then-Else-EndIf
- [x] `For`/`Next` loops
  - [x] Simple For loop without Step
  - [x] For loop with Step
  - [x] For loop with expressions
- [x] `Do`/`Loop` structures
  - [x] Do While loop (condition at start)
  - [x] Do Loop While (condition at end)
  - [x] Infinite loop (no condition)

### Program Structure

- [x] `BeginProg`/`EndProg` parsing
  - [x] BeginProg statement
  - [x] EndProg statement
  - [x] Complete BeginProg/EndProg program
- [x] `DataTable`/`EndTable` parsing
  - [x] DataTable with arguments
  - [x] EndTable statement
  - [x] Complete DataTable/EndTable structure
- [x] Function/Subroutine definitions
  - [x] Function without parameters
  - [x] Function with parameters
  - [x] Subroutine without parameters
  - [x] Subroutine with parameters

### Semantic Rules

- [x] Variable scope tracking (Public vs Dim)
- [x] Function vs Subroutine distinction (copy-back semantics)
- [x] Model-dependent variable name validation
  - [x] CR200X: 16 char max, 12-char truncation collision detection
  - [x] CR6/GRANITE: 39 char max, 35 char recommended
  - [x] File extension-based model detection
  - [x] Rules centralized in a `ValidationProfile` per model (see Future
    Enhancements → Datalogger-specific validation profiles)

## Phase 4: LSP Server Implementation 📡

### Basic LSP Features

- [x] LSP server structure (using `tower-lsp`)
  - [x] Backend implementation with LanguageServer trait
  - [x] Document manager for state tracking
  - [x] Full text document synchronization
- [x] Document synchronization
  - [x] did_open notification handling
  - [x] did_change notification handling (full sync)
  - [x] did_close notification handling
- [x] Basic diagnostics
  - [x] Integration with semantic analyzer
  - [x] Real-time error/warning publishing
  - [x] Position mapping (1-indexed parser → 0-indexed LSP)

### Advanced LSP Features

- [x] IntelliSense (completion)
  - [x] Keywords with snippets (control flow, declarations, program structure)
  - [x] Built-in functions (Scan, measurement, math, string, time functions)
  - [x] User-defined variables/functions (extracted from AST)
  - [x] Multi-statement pattern snippets (`ScanLoop`, `SlowSequenceLoop`,
    `DataTableSample`, `NewProgram`)
- [x] Signature help
  - [x] Built-in function signatures (Scan, measurement, math, string, time, etc.)
  - [x] Parameter documentation with descriptions
  - [x] Active parameter detection (comma counting)
  - [x] Nested function support
- [x] Hover information
  - [x] Keyword descriptions with syntax examples
  - [x] Position-based token lookup
  - [x] LSP hover handler integration
- [x] Go to definition
  - [x] Symbol definition extraction from AST (variables, functions, subroutines)
  - [x] Position-based identifier lookup
  - [x] Definition location resolution
  - [x] LSP goto_definition handler integration
- [x] Find all references
  - [x] Symbol occurrence search in token stream
  - [x] Multiple reference location collection
  - [x] LSP references handler integration
- [x] Document symbols (outline view)
  - [x] Program structure (BeginProg, EndProg, DataTable, EndTable)
  - [x] Function definitions with parameters
  - [x] Subroutine definitions with parameters
  - [x] Variable declarations (Public, Dim, Const)
  - [x] Nested symbols in functions/subroutines
  - [x] Symbol kind classification (Function, Method, Variable, Constant, Namespace)
- [x] Rename symbol
  - [x] `textDocument/prepareRename` (identifier range lookup and validation)
  - [x] `textDocument/rename` (workspace edit renaming all occurrences)
  - [x] New name identifier syntax validation (`ResponseError` on invalid input)

### Model-Dependent Validation

- [x] File extension → model detection (`.cr1` → CR200X, `.cr6` → CR6)
- [x] Variable name length validation
  - [x] CR200X: 16 chars max, 12-char truncation warning
  - [x] CR6/GRANITE: 39 chars max, 35-char recommendation
- [x] Duplicate field name detection (12-char truncation collision)

## Phase 5: WASM Integration ✅

- [x] WASM bindings (`wasm-bindgen`)
  - [x] Tokenize API: `tokenize(source)` → JSON tokens
  - [x] Parse API: `parse(source)` → JSON AST with error handling
  - [x] Analyze API: `analyze(source, file_path)` → JSON diagnostics
  - [x] Version API: `version()` → package version
- [x] JavaScript API exports
  - [x] TypeScript definitions auto-generated
  - [x] Web target build support
- [x] WASM build configuration (`wasm-pack`)
  - [x] wasm32-unknown-unknown target
  - [x] Release optimizations (opt-level "s", LTO)
- [x] WASM tests (18 tests passing)
  - [x] Tokenize API tests
  - [x] Parse API tests
  - [x] Analyze API tests
  - [x] Model detection tests
  - [x] Version API tests

## Phase 6: VSCode Extension Client 🔌

- [x] Extension activation logic
  - [x] activate() with LanguageClient initialization
  - [x] deactivate() with client cleanup
  - [x] Error handling with user notifications
- [x] LSP client initialization
  - [x] vscode-languageclient integration
  - [x] stdio transport configuration
  - [x] Document selector for CRBasic files
- [x] Native LSP server binary
  - [x] crbasic-lsp binary build (main.rs)
  - [x] Server bundling script (copy-server.js)
  - [x] Cross-platform support (Windows/Unix)
- [x] Configuration options
  - [x] crbasic.server.path setting for custom server path
- [x] Extension commands ✅ Resolved
  - Added `crbasic.restartServer` ("CRBasic: Restart Language Server")
    and `crbasic.showServerOutput` ("CRBasic: Show Server Output") to
    `contributes.commands` in `client/package.json`
  - Command-handler decision logic lives in `client/src/commands.ts`,
    decoupled from the `vscode` module (which only exists inside the
    Extension Host) via a small `ServerConnection` interface -- so it can
    be unit tested directly instead of only as a smoke test. `extension.ts`
    wires these pure functions to `vscode.commands.registerCommand` and
    the real language client
  - 4 unit tests (`client/src/commands.test.ts`, Red→Green)
- [ ] Extension packaging and publishing (future)
  - [ ] Extension icon (image asset + `icon` field in `client/package.json`)
  - [ ] `vsce package` script/config to produce a `.vsix` locally
    - Blocked on the multi-platform packaging gap (see Known Issues /
      Technical Debt → Packaging Gap)
  - [ ] VS Code Marketplace publisher account and Personal Access Token
  - [ ] Publish workflow (GitHub Actions `vsce publish` on release tag)

## Phase 7: Testing & Quality 🧪

### Unit Tests

- [x] Lexer tests (35 tests passing - coverage target: 80% line, 75% branch)
  - [x] Empty source and EOF
  - [x] Comments (single-quote, mid-line, empty)
  - [x] Numeric literals (integer, float, scientific notation)
  - [x] String literals (simple, escape sequences, escaped quotes)
  - [x] Identifiers (simple, with numbers, starting with underscore)
  - [x] Keywords (case-insensitive matching, canonical form)
  - [x] Boolean literals (True/False, case-insensitive)
  - [x] Operators (arithmetic, comparison)
  - [x] Delimiters (parentheses, brackets, comma)
  - [x] Line continuation
  - [x] Whitespace and newline handling
  - [x] Integration tests (multi-line programs)
- [x] Parser tests (131 tests passing)
  - [x] Primary expressions (literals, identifiers, parentheses)
  - [x] Binary operations (arithmetic, comparison, logical)
  - [x] Unary operations (negation, NOT)
  - [x] Operator precedence validation
  - [x] Parenthesized expressions
  - [x] Function call expressions (no args, single arg, multiple args, expression args, nested)
  - [x] Array access expressions (simple, variable index, expression index, multi-dimensional)
  - [x] Assignment statements (simple, with expressions, array elements)
  - [x] Variable declarations (Public, Dim, Const with type annotations and initializers)
  - [x] Function call statements (as statements, not just expressions)
  - [x] Program structure (BeginProg, EndProg, DataTable with arguments, EndTable)
  - [x] Control flow structures (If-Then-Else-EndIf, For-Next loops, Do-Loop structures)
  - [x] Function/Subroutine definitions (Function, Sub with parameters)
- [x] Semantic analyzer tests (23 tests passing)
  - [x] Datalogger model detection (file extension → model mapping, 14 extension tests)
  - [x] Validation profile thresholds (table-driven across all 4 models:
    `model_name`, `max_variable_length`, `recommended_variable_length`,
    `truncation_length`)
  - [x] Variable scope tracking (Public = Global, Dim = Local)
  - [x] Variable name length validation (CR200X: 16 max, CR6: 39 max)
  - [x] Recommended length warnings (CR200X: 12, CR6: 35)
  - [x] Truncation collision detection (CR200X 12-char truncation)
- [x] LSP handler tests (114 unit tests + 17 integration tests = 131 tests passing)
  - [x] Document manager (open, update, close operations)
  - [x] Document analysis and caching
  - [x] Model detection from file URI
  - [x] Diagnostic conversion (semantic errors → LSP diagnostics)
  - [x] Position mapping (parser → LSP coordinates)
  - [x] Document symbols extraction (9 tests)
    - [x] Program structure symbols (BeginProg, DataTable, etc.)
    - [x] Function and Subroutine symbols
    - [x] Variable declaration symbols
    - [x] Nested symbols in function bodies
    - [x] Symbol kind classification
  - [x] Hover information (15 tests)
    - [x] Keyword hover descriptions
    - [x] Position-based token lookup
    - [x] Half-open interval span handling
  - [x] Completion / IntelliSense (26 tests)
    - [x] Keyword completions with snippets
    - [x] Built-in function completions
    - [x] User-defined variable/function completions
    - [x] Multi-statement pattern snippets (linked tabstops across
      declaration and usage, e.g. DataTable name shared with CallTable)
    - [x] Completion item kinds and formatting
  - [x] Signature help (24 tests)
    - [x] Function signature lookup
    - [x] Parameter documentation
    - [x] Active parameter counting
    - [x] Function name extraction
  - [x] Go to Definition (12 tests)
    - [x] Symbol definition extraction (variables, functions, subroutines)
    - [x] Identifier position lookup
    - [x] Definition location resolution
    - [x] Span to Range conversion
  - [x] Find All References (8 tests)
    - [x] Single and multiple reference finding
    - [x] Identifier filtering
    - [x] Location generation with correct URIs and ranges
  - [x] Rename Symbol (6 tests)
    - [x] New name identifier syntax validation
    - [x] Identifier range lookup for `prepareRename`
    - [x] Workspace edit construction renaming every occurrence
- [x] WASM binding tests (18 tests passing)
  - [x] Tokenize API (returns JSON array)
  - [x] Parse API (success/error handling)
  - [x] Analyze API (model-specific diagnostics)
  - [x] Model detection from file path
  - [x] Version API

### Integration Tests

- [x] End-to-end tokenization tests (4 integration tests)
- [x] Sample file integration tests (27 tests in `tests/sample_files.rs`) ✅ All passing
  - [x] Tokenization tests (10 tests - all passing)
  - [x] Parsing tests (10 tests - all passing)
  - [x] AST structure tests (2 evergreen tests - all passing, 6 non-evergreen tests removed)
  - [x] Semantic analysis tests (4 tests - all passing)
  - [x] Real-world validation test (1 comprehensive test - all passing)
- [x] Example program tests (3 tests in `tests/example_programs.rs`) ✅ All passing
  - [x] Getting-started example has zero diagnostics
  - [x] Scope-and-copyback example has zero diagnostics
  - [x] CR200X pitfalls example reproduces the documented errors/warnings
- [x] LSP feature integration tests (17 tests in `tests/lsp_integration.rs`) ✅ All passing
  - [x] Document synchronization (open, change, close)
  - [x] Diagnostics publishing (valid program, invalid syntax, CR200X truncation warnings)
  - [x] Completion / IntelliSense (keywords, user-defined variables, pattern snippets)
  - [x] Hover information (keyword descriptions)
  - [x] Signature help (built-in function signatures)
  - [x] Go to definition (variable declaration lookup)
  - [x] Find references (all variable references)
  - [x] Document symbols (program structure extraction)
  - [x] Rename symbol (rename all occurrences, reject invalid names,
    `prepareRename` range lookup)

### E2E Tests

- [x] VSCode extension smoke tests (5 tests in `client/src/extension.test.ts`) ✅ All passing
  - [x] Extension module structure (activate/deactivate functions)
  - [x] Server path resolution logic (Windows/Unix platforms)
  - [x] Extension configuration (file extensions)
  - [x] Module loadability
- [x] Real-world CRBasic program validation (1 comprehensive test in `tests/sample_files.rs`) ✅ All passing
  - [x] Complete validation pipeline (tokenize → parse → analyze) for all 10 sample files
  - [x] Model-specific semantic analysis (CR200X, CR6, GRANITE)
  - [x] Zero semantic errors verification for real-world programs

## Phase 8: Documentation & Polish 📚

- [x] API documentation (rustdoc) ✅ Complete
  - Added comprehensive rustdoc comments to all public APIs
  - Documented all struct fields, enum variants, and functions
  - Verified with `cargo doc -D missing_docs -D warnings`
  - Generated documentation available at `target/doc/`
- [x] User guide (README updates) ✅ Complete
  - Simplified Development Status section to focus on overview only
  - Fixed GitHub repository URLs (connect0459/crbasic-lsp-rs)
  - Moved detailed progress tracking to docs/todo.md
  - Maintained focus on project overview, usage, and license
- [x] Developer guide (ARCHITECTURE.md updates) ✅ Complete
  - Updated Project Structure with actual file organization
  - Added Implementation Status section with completed features
  - Updated Testing Strategy with current test results (146 tests)
  - Added Parser Capabilities section
  - Updated Extension Points with future enhancements
- [x] Example programs ✅ Resolved
  - Added `docs/examples/` with 3 curated, heavily-commented programs
    distinct from `docs/sample-codes/` (real-world parser regression
    fixtures): `01-getting-started.CR6` (basic program shape),
    `02-scope-and-copyback.CR6` (Public-inside-Sub global scope,
    Function vs Sub parameter copy-back), and
    `03-cr200x-length-pitfalls.CR1` (deliberately triggers the max-length,
    recommended-length, and 12-char truncation-collision diagnostics)
  - Added `docs/examples/README.md` explaining what each file demonstrates
    and which LSP feature to try (hover, go-to-definition, find
    references, completion snippets)
  - Added `crates/crbasic-parser/tests/example_programs.rs` (3 tests) so
    the examples' documented behavior stays verified against the real
    analyzer instead of just asserted in prose
  - Linked from the main `README.md`'s Documentation section and Project
    Structure tree
- [x] CI/CD pipeline (GitHub Actions) ✅ Resolved
  - Added `.github/workflows/ci.yml` with two jobs, mirroring `just verify`:
    `rust` (`cargo fmt --all --check`, `cargo clippy --all-targets
    --all-features -- -D warnings`, `cargo test --workspace`) and `client`
    (`npm run lint`, `npm run format:check`, `npm run type-check`,
    `npm test -- --run`)
  - Triggers on push to `main`, on every pull request, and manually via
    `workflow_dispatch`; `dorny/paths-filter` skips each job's checks when
    the push/PR touches neither that job's paths nor the workflow file
    itself, so a client-only change doesn't pay for a Rust toolchain setup
    and vice versa
  - All third-party actions pinned by commit SHA (with the version as a
    trailing comment) rather than by mutable tag, following the same
    convention used in the user's other repositories
  - Verified locally: `cargo test --workspace` passes (327 lib/integration
    tests; the 2 `crbasic-parser` doctests fail locally only because this
    machine's global `~/.cargo/config.toml` injects a `-lpython3.11`
    linker flag for an unrelated project -- not reproducible on a clean CI
    runner), `client`'s lint/format:check/type-check/test (44 tests) all
    pass, and `actionlint` reports no issues on the workflow file itself
- [x] Release preparation ✅ Resolved
  - [x] Versioning policy from `0.1.0` (SemVer pre-1.0 conventions, when to
    cut `1.0.0`)
    - Documented in `docs/adrs/adr-003-release-process.md`: lockstep SemVer
      across the Cargo workspace and `client/package.json`; pre-1.0 breaking
      changes bump MINOR, everything else bumps PATCH; `1.0.0` requires
      Marketplace publication, the multi-platform packaging gap (see new
      technical-debt item below) closed, and a stable WASM/LSP public API
  - [x] GitHub Releases workflow (tag → release notes → artifact upload)
    - Added `.github/workflows/release.yml`, triggered only by pushing a
      `v*.*.*` tag: asserts the tag matches `Cargo.toml` and
      `client/package.json`, re-runs the same fmt/clippy/test and
      lint/format/type-check/test gates as `ci.yml`, extracts that version's
      `CHANGELOG.md` section, and creates a GitHub Release with those notes
      via `softprops/action-gh-release`
    - Deliberately does not build/attach a `.vsix` or run `vsce publish` --
      both require the multi-platform packaging gap below to be solved
      first; `actionlint` reports no issues on the new workflow file
  - [x] `CHANGELOG.md` (format and update cadence)
    - Added at repo root in Keep a Changelog format, starting with a single
      `[Unreleased]` section (no tag has ever been pushed yet); cadence is
      documented in ADR-003: every user-facing change adds an entry under
      `[Unreleased]` in the same PR, retitled to `[X.Y.Z] - YYYY-MM-DD`
      immediately before tagging

## Known Issues / Technical Debt 🐛

### Parser Limitations (discovered during integration testing)

- [x] Multiple variable declarations on single line (`Public PTemp, Batt_volt`) ✅ Resolved
- [x] Boolean literals as function arguments (`False`, `True`) ✅ Resolved
- [x] NextScan keyword not recognized (lexer) ✅ Resolved
- [x] NextScan as a statement (parser support) ✅ Resolved
- [x] Tab-indented statements handling ✅ Resolved (lexer already skips tabs correctly)

### Packaging Gap (discovered while designing the release workflow)

- [ ] Multi-platform `.vsix` packaging
  - `client/scripts/copy-server.js` copies whatever OS's `crbasic-lsp`
    binary happens to exist at `target/release/` on the build machine, and
    `client/src/extension.ts`'s `getServerPath` only branches on win32 vs.
    non-win32 -- there is no per-platform (`win32-x64`, `darwin-arm64`,
    `linux-x64`, ...) packaging. A `.vsix` built today only works on the OS
    that built it
  - This is a drift from
    [ADR-001](./adrs/adr-001-rust-wasm-lsp-architecture.md)'s stated
    rationale ("no external binary dependencies... single WASM binary works
    everywhere"): the client ended up bundling a native binary instead of
    loading `crbasic-wasm` directly in the extension host
  - Blocks the remaining Phase 6 "Extension packaging and publishing" items
    (`vsce package` needs a `--target <platform>` build matrix, one native
    binary compiled per target, before it can produce a working artifact)

### Build Warnings

- [x] ESLint 8 deprecation warning (upgrade to ESLint 9) ✅ Resolved
  - Upgraded from ESLint 8.56.0 to 9.17.0
  - Migrated from `.eslintrc.json` to `eslint.config.mjs` (Flat Config)
  - Updated `typescript-eslint` to 8.18.2
- [x] Vite CJS API deprecation warning ✅ Resolved
  - Renamed `vite.config.ts` to `vite.config.mts` for ESM support
- [x] Performance optimization (large files >1000 lines) ✅ Resolved
  - Measured against a synthetic 1000-line/24,448-char program: Lexer was
    1.03–1.59ms against a <1ms target (Parser 0.36–0.59ms and Diagnostics
    1.5–2.2ms were already well within their <10ms/<50ms targets)
  - Root cause: `Scanner`/`Token`/`TokenKind` allocated a new `String` for
    every token's lexeme, plus a `to_lowercase()` temporary and a clone for
    every identifier/keyword lookup against a per-`Scanner::new`-rebuilt
    `HashMap<String, String>`
  - Fixed by giving `Token`/`TokenKind` a lifetime so they borrow `&str`
    slices directly from the source instead of allocating, and replacing the
    keyword `HashMap` with a static table checked via `eq_ignore_ascii_case`
    (no allocation, no per-`Scanner::new` rebuild cost). As a side effect,
    `crbasic-lsp` no longer clones the whole document text before tokenizing
    on every hover/definition/references/symbols/diagnostics request, and
    `crbasic-wasm` no longer copies the whole source string on every call
  - Result: Lexer now 0.73–0.83ms (release build), clearing the <1ms target
  - Added `crates/crbasic-parser/tests/performance.rs` as a permanent
    regression guard (budgets relaxed under `cfg!(debug_assertions)` since
    `cargo test` runs in debug mode by default; run
    `cargo test --release --test performance` to check the real numbers)

## Future Enhancements 🚀

- [x] Code formatting (auto-indent) ✅ Resolved (type-time only)
  - Added `indentationRules` to `client/language-configuration.json`, so
    VSCode's built-in editor indents/dedents as you type -- no LSP
    `textDocument/formatting` provider was added (scoped out; revisit
    separately if whole-document "Format Document" is wanted)
  - `increaseIndentPattern` covers block openers (`BeginProg`,
    `DataTable(`, `Sub`, `Function`, `For`, `Do`, `SlowSequence`,
    `Select Case`) plus the block-if form (`If ... Then` at end of line,
    distinct from the one-line `If ... Then stmt` form, which does not
    indent); `Else`/`ElseIf`/`Case` both dedent their own line and indent
    the next, matching how `else`/`case` behave in curly-brace languages
  - `decreaseIndentPattern` covers the matching closers (`EndProg`,
    `EndTable`, `EndSub`, `EndFunction`, `EndSequence`, `EndSelect`,
    `EndIf`, `Next`, `Loop`) plus `Else`/`ElseIf`/`Case`; `\b` word
    boundaries keep `Next` from matching `NextScan` (a real, separate
    CRBasic keyword)
  - Both patterns use VSCode's `{pattern, flags}` object form with
    `flags: "i"` for case-insensitive matching, since CRBasic keywords
    are case-insensitive
  - 35 Vitest cases (`client/src/language-configuration.test.ts`,
    table-driven) exercise the compiled regexes the same way VSCode does
    (`pattern.test(singleLine)`), covering every block keyword plus the
    `NextScan`/one-line-`If` false-positive risks
- [x] Refactoring support (rename variable) ✅ Resolved
  - Added `RenameProvider` (`crates/crbasic-lsp/src/rename.rs`) handling
    `textDocument/rename` and `textDocument/prepareRename`
  - `prepare_rename` returns the range of the identifier under the cursor;
    `get_rename_edit` builds a `WorkspaceEdit` renaming every occurrence of
    that identifier in the token stream
  - Rejects rename requests whose `new_name` is not a valid CRBasic
    identifier (ASCII letter/underscore start, alphanumeric/underscore body),
    returning a `ResponseError` per the LSP spec
  - 6 unit tests (`crates/crbasic-lsp/src/rename.rs`) + 3 integration tests
    (`crates/crbasic-lsp/tests/lsp_integration.rs`)
- [x] Snippet library ✅ Resolved
  - Added `CompletionProvider::get_pattern_snippet_completions`
    (`crates/crbasic-lsp/src/completion.rs`), extending the existing
    single-keyword snippets with multi-statement CRBasic idioms:
    `ScanLoop` (Scan/NextScan + CallTable), `SlowSequenceLoop`
    (SlowSequence/EndSequence wrapping a Scan loop), `DataTableSample`
    (DataTable + Sample field), and `NewProgram` (a full starter skeleton:
    Const/Public declarations, DataTable, BeginProg/Scan/EndProg)
  - `NewProgram` links tabstops across declaration and usage (e.g. the
    table name typed once fills both the `DataTable` and `CallTable`
    placeholders), demonstrating linked-placeholder snippet editing
  - Delivered via the LSP `textDocument/completion` response
    (`CompletionItemKind::SNIPPET`), keeping snippets editor-agnostic
    rather than VSCode-only (unlike a `contributes.snippets` JSON file)
  - 3 unit tests (`crates/crbasic-lsp/src/completion.rs`) + 1 integration
    test (`crates/crbasic-lsp/tests/lsp_integration.rs`)
- [x] Datalogger-specific validation profiles ✅ Resolved
  - Added `ValidationProfile` (`crates/crbasic-parser/src/semantic.rs`):
    a single struct holding every per-model rule (`model_name`,
    `max_variable_length`, `recommended_variable_length`,
    `recommended_length_reason`, `truncation_length`)
  - `DataloggerModel::profile()` is now the sole source of truth; adding a
    new model means adding one match arm here instead of editing four
    scattered ones
  - Removed the now-redundant `max_variable_length()`,
    `recommended_variable_length()`, `truncation_length()`, and the
    private `SemanticAnalyzer::model_name()` helper — nothing outside
    `semantic.rs` called them, and `analyze_variable_declaration` /
    `check_truncation_collisions` read the profile directly
  - Consolidated 6 single-fact tests into 1 table-driven test
    (`each_model_has_its_documented_validation_thresholds`) covering all
    4 models, avoiding duplicate coverage of the same profile data through
    two call paths
  - `ValidationProfile` re-exported from `crbasic-parser`'s crate root
- [ ] Integration with Campbell Scientific toolchain ⏸️ Deferred
  - Checked `docs/researches/research-001-crbasic-for-vscode.md`: it
    documents the language spec (from CS's public help pages) but no
    public API/CLI for the official toolchain (CRBasic Editor, LoggerNet,
    Short Cut)
  - Concrete scope depends on access to proprietary CS software (e.g. a
    local CRBasic Editor compiler binary to shell out to for real
    compilation diagnostics, or LoggerNet-specific naming conventions like
    `Flag()` arrays) that isn't available in this environment to build
    against or test
  - Revisit once a concrete integration target (specific CS tool +
    interface) is identified
