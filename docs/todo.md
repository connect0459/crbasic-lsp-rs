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
  - [x] Simple array access: `Data(0)`
  - [x] Variable index: `Temp_C(i)`
  - [x] Expression index: `Data(i + 1)`
  - [x] Multi-dimensional: `Matrix(1, 2)`
  - Originally implemented with a fabricated `Data[0]` bracket syntax; see
    Round 12 (Reference Implementation & Official Docs Comparison) below
    for the real-syntax fix

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
  - [x] Assignment to array elements (`Data(0) = 5`, `Matrix(1, 2) = 100`)
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
  - [x] Extension icon (image asset + `icon` field in `client/package.json`) ✅ Resolved
    - Original artwork only: an external sensor wired into a chassis with a
      terminal block, a status LED, and a mini live-readout trace, on a flat
      dark-navy badge with a teal accent -- no Campbell Scientific logo,
      trademark, or brand color, since this extension is unofficial and
      third-party
    - Confirmed via `campbellsci.com/copyright` that using their trademarked
      logo/name would fall under a use "that implies your own endorsement or
      ownership, rather than Campbell Scientific's"; precedent from other
      unofficial vendor-language extensions (`vscode_abap_remote_fs`,
      `vscode-rpgle`) confirmed the norm is original artwork, not the
      vendor's logo
    - Source vector kept at `client/images/icon.svg`; `vsce` rejects SVG as
      the `icon` field itself ("SVGs cannot be used as icons"), so
      `client/images/icon.png` (256x256, meeting the Retina requirement) is
      what `package.json`'s `icon` field points to
    - Verified with `npx vsce package`: both files bundle correctly under
      `images/` in the packaged `.vsix`
  - [x] `vsce package` script/config to produce a `.vsix` locally ✅ Resolved
    - See Known Issues / Technical Debt → Packaging Gap for details
      ([ADR-004](./adrs/adr-004-multi-platform-packaging.md))
  - [ ] VS Code Marketplace publisher account and Personal Access Token
  - [x] Publish workflow (GitHub Actions `vsce publish` on release tag) ✅ Resolved
    - Added a `publish` job to `.github/workflows/release.yml`, running after
      `package` and gated on `github.event_name == 'push'` (skipped entirely
      on `workflow_dispatch` dry runs)
    - Downloads the `vsix-packages` artifact and runs `vsce publish
      --packagePath ../dist-vsix/*.vsix`, relying on `vsce` inferring each
      package's target from its own manifest rather than repackaging
    - Reads the Marketplace token from a `VSCE_PAT` secret (`vsce publish -p`
      defaults to this env var); if the secret is unset, the step logs
      `::warning::` and exits 0 instead of failing, since the publisher
      account and PAT above don't exist yet -- flipping it on later needs no
      workflow change
    - `actionlint` reports no issues on the updated workflow file

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

### Parser Limitations (discovered while designing preprocessor directive support, 2026-08-09)

- [x] `ElseIf` not implemented in `If`/`EndIf` parsing (bug) ✅ Resolved
  - `parse_if_statement` (`crates/crbasic-parser/src/parser.rs`) only
    stopped its `then_branch` loop at `Else` or `EndIf`; there was no
    `"ElseIf"` case anywhere in the parser. `ElseIf` was registered in
    `keywords.json` and had completion/hover/language-configuration
    coverage, so it read as supported, but a real
    `If ... ElseIf ... EndIf` program failed to parse. Confirmed via repro
    before fixing: `Unexpected token: Keyword("ElseIf")`.
  - Same bug class as the resolved `Mod` gap above (advertised via
    completion/hover, silently broken in the parser) but likely higher
    real-world impact, since `If`/`ElseIf`/`EndIf` chains are far more
    common in CRBasic programs than the `Mod` operator.
  - Found while researching preprocessor directive support below: `#If`'s
    real semantics (confirmed against Campbell Scientific's own docs, see
    below) require the same `#ElseIf` chaining logic, so this needed
    deciding first rather than duplicating an `ElseIf`-chaining
    implementation independently for the `#`-prefixed form.
  - Fixed by factoring the shared `condition Then statements` parsing into
    a new `parse_if_clause` helper, called recursively so each `ElseIf`
    desugars into a nested `IfStatement` held in `else_branch` -- only the
    outermost `If` ever consumes the chain's single closing `EndIf`. Every
    downstream consumer (folding, semantic tokens, definitions, call
    sites) already walked `IfStatement`'s branches generically, so none
    needed changes
  - 3 new tests (`parses_if_elseif_endif`, `parses_if_elseif_else_endif`,
    `parses_multiple_chained_elseif_branches`) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate passes

### Reference Implementation Comparison (2026-08-09)

Found while comparing this project against a couple of external,
TextMate-only CRBasic syntax-highlighting extensions (one maintained by a
Campbell Scientific employee), consulted locally as reference material but
not part of this repository. Both gaps below were confirmed by actually
parsing a minimal repro, not just by reading the reference grammars.

- [x] `Mod` operator not implemented in the parser (bug) ✅ Resolved
  - `Mod` was registered in `keywords.json` under the `logical` category,
    so it appeared in completion and hover as if it were usable, but the
    parser's binary-operator grammar had no case for it at all -- `x = 10
    Mod 3` failed with `Unexpected token: Keyword("MOD")`. Confirmed via a
    throwaway `cargo run --example` repro (not committed) before fixing.
  - Both reference extensions list `MOD` as an operator alongside
    `AND`/`OR`/`XOR`, corroborating that this is a real CRBasic operator,
    not a keyword that should be removed.
  - Fixed in `parse_multiplicative` (`crates/crbasic-parser/src/parser.rs`):
    `Mod` is now matched at the same precedence as `*`/`/`, the common
    convention across BASIC dialects; `BinaryOperator::Modulo` already
    existed in the AST but was never constructed anywhere until this fix.
  - 2 new tests (`parses_modulo`,
    `modulo_has_same_precedence_as_multiplication_and_division`) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate passes.
- [x] `While ... Wend` loop construct not supported ✅ Resolved
  - `While` was only recognized as part of `Do While ... Loop`; the
    standalone `While <condition> ... Wend` loop (a distinct CRBasic
    construct) was unparseable, and `Wend` wasn't even a recognized
    keyword. Confirmed via repro before fixing:
    `Unexpected token: Keyword("While")`.
  - Both reference extensions independently list `While`/`Wend` as a loop
    construct; `docs/researches/research-001-crbasic-for-vscode.md` §5.2
    enumerates For/Next, Do/While, Do/Until, and Loop but omits While/Wend
    entirely -- an oversight in the original research, not a deliberate
    exclusion.
  - Parses to the same `Statement::DoLoop` AST shape as
    `Do While ... Loop` (`condition_at_start: true`) rather than a new
    variant, since CRBasic documents them as equivalent constructs --
    folding, semantic tokens, definitions, and call-sites all already
    handle `DoLoop` generically, so none needed changes
  - Added `Wend` to `keywords.json` and regenerated the grammar files;
    added matching completion/hover coverage (required to keep the
    existing `every_language_keyword_has_a_completion_item` /
    `every_language_keyword_has_hover_info` completeness tests green)
  - `client/language-configuration.json`'s `indentationRules` and
    `folding.markers` updated to recognize `While`/`Wend` the same way
    `Do`/`Loop` already are, with matching Vitest cases added
  - 2 new parser tests (`parses_while_wend_loop_with_condition`,
    `while_loop_requires_wend_to_close`) added Red-first; full
    `build`/`test`/`clippy`/`fmt` (Rust) and `lint`/`format:check`/`test`
    (client) gates pass

Not flagged as gaps (verified during the same comparison):

- `.dld` file extension: already registered in `client/package.json` and
  `keywords.json`; `DataloggerModel::from_extension` correctly returns
  `Unknown` for it since `.dld` is a generic, model-agnostic program file
  per `research-001` -- applying model-specific validation to it would be
  incorrect, not a missing feature.
- Both reference extensions' language-configuration files set
  `lineComment: "//"`, which is wrong for CRBasic (real comments use `'`);
  this project's `client/language-configuration.json` already uses `'`
  correctly and should not be changed to match.
- `Eqv`/`Imp`/`IntDv` operators appear in one reference extension's
  operator list, but with no corroboration from Campbell Scientific's own
  docs and a strong resemblance to a copy-pasted VB6 operator list -- not
  acted on without independent verification.
- One reference extension's file-import/PC400-launch commands are thin,
  Windows-only wrappers (text paste, shelling out to a hardcoded
  `PC400.exe` path) -- not worth porting.

### Reference Implementation & Official Docs Comparison, Round 2 (2026-08-09)

Found while re-surveying the two reference extensions and Campbell
Scientific's own docs (help.campbellsci.com) for gaps not caught by the
first comparison round above. Each finding below was verified against
help.campbellsci.com directly, not just the reference grammars.

- [x] `Select`/`Case`/`EndSelect`/`ExitFor`/`ExitDo` advertised via
  completion/hover but unparseable (bug) ✅ Resolved
  - Same "advertised via completion/hover, silently broken in the
    parser" bug class as the already-resolved `Mod` and `ElseIf` gaps
    -- a third, unfixed instance. Confirmed via repro before fixing
    (e.g. `Unexpected token: Keyword("Select")`).
  - `Select Case`'s real grammar (verified against
    [help.campbellsci.com's Select Case page](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/selectcase.htm))
    supports a comma-separated `ExpressionList` per `Case` clause:
    plain values, `Expression To Expression` ranges, and `Is
    comparison-operator Expression` comparisons, optionally chained
    with `And`/`Or` (e.g. `Case Is >= 0 And Is <= 11.25`, from the
    docs' own wind-direction example). There is no `ExitSelect`
    statement in the real language (see below).
  - New `Statement::SelectCase`, `CaseClause`, and `CaseCondition` AST
    types (`crates/crbasic-parser/src/ast.rs`) capture this without a
    hacky implicit-operand expression node --
    `CaseCondition::Compare`/`Logical` store the operator and
    right-hand expression directly. Every downstream consumer that
    already walks `If`/`For`/`Do` bodies (call site collection,
    semantic tokens, definitions, semantic analysis, folding) gained a
    matching `SelectCase` arm; `symbols.rs`/`completion.rs`'s
    user-defined-symbol extraction deliberately did not, since neither
    already recurses into `If`/`For`/`Do` bodies either
  - 7 new parser tests (`control_flow_select_case`) + 2 new parser
    tests (`control_flow_exit_statements`) added Red-first
- [x] `Return`, `ExitFunction`, and `Exit Sub` entirely unsupported ✅ Resolved
  - Real, documented CRBasic constructs
    ([Function/EndFunction](https://help.campbellsci.com/crbasic/landing/Content/Instructions/functionendfunction.htm),
    [Sub/Exit Sub/EndSub](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/subexitsubendsub.htm))
    with zero keyword, parser, completion, or hover coverage at all --
    not previously tracked anywhere in this file.
  - `Return(expression)` is parsed with its documented required
    parentheses. `ExitFunction` is one word, but Campbell Scientific's
    own syntax diagram spells the `Sub` equivalent as two separate
    keyword tokens (`Exit Sub`), confirmed via two independent fetches
    of the official page -- `Exit` alone is a parse error unless
    followed by `Sub`, matching the real, asymmetric grammar
  - 5 new parser tests (`control_flow_return_and_exits`) added
    Red-first; 1 new end-to-end test (`semantic.rs`) confirms a
    program combining Select Case, ExitFor, Return, ExitFunction, and
    Exit Sub together produces zero semantic errors
- [x] `ExitSelect`, `Continue`, `Break`, `GoTo` are fabricated -- not
  real CRBasic keywords ✅ Resolved
  - Present in `keywords.json`/`completion.rs`/`hover.rs` with real
    completion/hover text, but corroborated by neither reference
    grammar nor Campbell Scientific's own docs. The real CRBasic
    exit/jump vocabulary is `ExitFor`/`ExitDo`/`ExitScan`/
    `ContinueScan`/`ExitFunction`/`Exit Sub`, with no generic
    `Continue`/`Break`/`GoTo`, and `Select Case` has no exit statement
    of its own at all
  - Advertising nonexistent syntax to users of an open-source tool is
    misleading, so removed rather than "fixed" -- confirmed with the
    user before removing, since this changes previously-offered
    completion candidates
- [x] `Next`'s optional trailing counter list (`Next [counter [,
  counter]...]`, e.g. `Next i`) silently corrupted the surrounding
  statement list (bug) ✅ Resolved
  - Found as a side effect of writing an `ExitFor` test: `Next i`
    leaked `i` into the enclosing statement list as a bogus
    `Expression::Identifier` statement, since `parse_for_loop` only
    ever consumed the bare `Next` keyword. Every pre-existing `For`
    loop test used bare `Next`, so this went undetected despite `Next
    <counter>` being a far more common idiom in real CRBasic than the
    bare form
  - Fixed by optionally consuming a comma-separated identifier list
    after `Next`, purely cosmetically (not cross-checked against the
    loop variable, matching how CRBasic itself does not require the
    name to match)
  - 2 new regression tests (`parses_next_with_counter_variable`,
    `parses_next_with_comma_separated_counter_list`) added Red-first
- [x] `client/language-configuration.json`'s `folding.markers` missing
  `\b` word boundaries (bug) ✅ Resolved
  - Found while writing a folding test for the `Select Case` addition
    below: `indentationRules`' patterns already added `\b` word
    boundaries specifically to keep `Next` from matching `NextScan`
    (see the `While`/`Wend` entry above), but the same fix was never
    applied to `folding.markers` -- "NextScan" matched the `end`
    marker's bare `Next` alternative, and by the same flaw a line like
    `DoWork(x)` or `SubTotal = 1` would falsely match the `start`
    marker's bare `Do`/`Sub` alternatives. No tests existed for
    `folding.markers` before this, so the bug was previously invisible
  - Fixed by adding `\b` to every alternative in both patterns; also
    added `Select Case`/`EndSelect` to `folding.markers` (previously
    only in `indentationRules`), consistent with how `Case` stays out
    of both marker patterns for the same reason `Else`/`ElseIf` do --
    they're branches, not their own foldable region
  - New `folding.markers` describe block
    (`client/src/language-configuration.test.ts`) covering every
    start/end alternative plus false-positive guards for the exit/
    return keywords and the word-boundary bug class
- [x] Single-line `If condition Then statement[: statement...]` (no
  `EndIf`) is unparseable ✅ Resolved
  - Found while writing an end-to-end test for the constructs above;
    confirmed real and confirmed via repro:
    `If x = 5 Then y = 1` fails with `Expected 'EndIf' to close If
    statement`, since `parse_if_clause` unconditionally requires a
    closing `EndIf`/`Else`/`ElseIf`
  - Confirmed via Campbell Scientific's own
    [If...Then...Else page](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/ifthenelse.htm):
    the single-line form's `EndIf` is implied at the end of the line,
    and the docs' own example
    (`If A > 10 Then A = A + 1 : B = B + A : C = C + B {EndIf}`) shows
    it combined with a second, compounding gap -- `:` as a
    statement separator on one line -- which this lexer/parser had no
    support for either (`:` wasn't even a recognized token)
  - Re-verified against [CRBasic Program Structure](https://help.campbellsci.com/crbasic/cr1000x/Content/Info/crbasicprogramstructure.htm)
    that `:` is a **general** multi-statement-per-line separator, not
    an `If`-only special case -- so the fix targets the shared
    statement-list loop, not `If` specifically
  - Added a `Colon` token to the lexer; `skip_whitespace_and_comments`
    (used between statements in the program body and every block body
    -- `For`, `Do`, `Function`, `Sub`, etc.) now skips it the same way
    it already skips `Newline`/`Comment`, so `x = 1 : y = 2` works
    uniformly everywhere a statement list is parsed, not just at the
    top level
  - `parse_if_clause` now detects the single-line form by checking
    whether a newline immediately follows `Then`; if not, it parses a
    colon-separated statement list (via new `parse_colon_separated_statements`)
    for `then_branch`, an optional same-line `Else` clause, and returns
    without expecting `EndIf`. `IfClause`'s tuple gained a `bool` flag
    so `parse_if_statement` knows whether to skip the `EndIf` check;
    `#If`/`#IfDef` preprocessor conditionals (which have no single-line
    form) got their own `PreprocessorClause` type instead of sharing
    `IfClause`, since they diverged
  - 6 new parser tests (single-line `If` with/without `Else`, with
    colon-separated statements, confirming it doesn't swallow the
    following line, plus 2 general colon-separator tests covering the
    program body and a `For` loop body) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate passes

Not flagged as gaps (verified during the same comparison):

- `Include` statement: confirmed real (pulls in an external source
  file via a device-path prefix, e.g. `Include "cpu:Sensor_PT500_Lib.crb"`)
  but remains a deferred, separate gap per the existing framing in the
  Preprocessor directive support entry above -- structural parsing
  would be low effort, but resolving/indexing the included file's
  symbols needs cross-file infrastructure this LSP explicitly doesn't
  have yet (same open-documents-only scope note as
  `workspaceSymbolProvider`/`callHierarchyProvider`)
- `keywords.json`'s `builtinFunctions` list (126 entries) is far short
  of the CS-employee-maintained grammar's ~420 unique names -- whole
  categories are absent (GOES/ARGOS satellite telemetry, DNP,
  CDM_*/SDM* peripheral modules, CSAT3/LI7200/LI7700 sensors, PakBus
  networking, custom-menu instructions). This is a content-volume
  scope decision (each needs real per-parameter completion snippets
  and hover prose authored, not a mechanical list sync), not a bug --
  flagged here for future prioritization rather than acted on now,
  consistent with the already-deferred "~35 highlighted-but-not-
  completed functions" note in the Keyword/instruction list
  unification entry above

### Reference Implementation & Official Docs Comparison, Round 3 (2026-08-09)

Found during a third comparison round, this time auditing operators
specifically (not covered by Rounds 1-2) against Campbell Scientific's
canonical [Operators](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/operators1.htm)
page, plus a keywords.json/parser.rs sweep for other constructs absent
from both reference grammars' coverage. Ordered by real-world frequency.

- [x] `&` string concatenation operator not implemented ✅ Resolved
  - No `Ampersand` token existed in the lexer. Confirmed real and
    distinct from `+` at
    [concatenation.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/concatenation.htm)
    and the master Operators page ("String" category lists both `+`
    and `&`). High frequency: string-building (table names, filenames,
    log messages) is extremely common in real CRBasic programs.
  - Added the `Ampersand` token to the lexer and
    `BinaryOperator::Concatenate` to the AST, parsed in
    `parse_additive` at the same precedence tier as `+`/`-` (both are
    grouped under the docs' "String" operator category)
  - 2 new lexer/parser tests (`recognizes_string_concatenation_operator`,
    `parses_string_concatenation`,
    `concatenation_has_same_precedence_as_addition`) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] Compound assignment operators (`+=`, `-=`, `*=`, `/=`, `^=`,
  `&=`) not implemented ✅ Resolved (`\=` deferred to the `\` entry below)
  - The scanner only ever emitted bare `Plus`/`Minus`/`Star`/`Slash`/
    `Caret`, never checking for a trailing `=`. Confirmed at
    [compoundoperators.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/compoundoperators.htm)
    and the master Operators page. Common shorthand in
    accumulator/counter code.
  - Desugars to a plain `Statement::Assignment` whose value is a
    `BinaryOp` reading the same target (`x += y` becomes `x = x + y`),
    rather than a new AST variant -- every downstream consumer already
    handles `Assignment` generically. Works for both plain identifiers
    and array elements (`Data[0] += 1`)
  - `\=` deferred to land together with the `\` integer-division entry
    below, since both need the same new `Backslash` token
  - 1 new lexer test (`recognizes_compound_assignment_operators`) + 2
    new parser tests (`desugars_compound_assignment_operators_to_assignment_with_binary_op`,
    `desugars_compound_assignment_to_array_element`) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] `\` integer-division operator not implemented ✅ Resolved
  - Not in the lexer at all (the only existing `\` handling was inside
    string-literal escapes, unrelated). Confirmed on the master
    Operators page (Arithmetic category, distinct from `/`).
  - Added `Backslash`/`BackslashEqual` tokens and
    `BinaryOperator::IntegerDivide`, parsed in `parse_multiplicative`
    at the same precedence tier as `*`/`/`; `\=` slots into the
    compound-assignment desugaring added above
  - 1 new lexer test (`recognizes_integer_division_operator`, plus
    `\=` added to the compound-assignment lexer test) + 3 new parser
    tests (`parses_integer_division`,
    `integer_division_has_same_precedence_as_multiplication_and_division`,
    plus a `\=` case added to the compound-assignment desugaring test)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate
    passes
- [x] `<<` / `>>` bit-shift operators not implemented ✅ Resolved
  - The lexer's `<`/`>` cases only checked for a trailing `=`
    (-> `<=`/`>=`), and `<` also checked for `>` (-> `<>`); a doubled
    `<<`/`>>` would lex as two separate tokens and fail to parse.
    Confirmed at
    [bitshiftoperators.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/bitshiftoperators.htm),
    used for bit-level protocol/status-word parsing on `Long`-typed
    variables.
  - Added `LeftShift`/`RightShift` tokens and matching `BinaryOperator`
    variants, parsed via a new `parse_shift` precedence tier between
    comparison and additive (binds tighter than `<`/`>`/`=`, looser
    than `+`/`-`/`&`) -- the common C-family convention, since
    Campbell Scientific's docs don't specify shift precedence relative
    to other operator categories
  - 1 new lexer test (`recognizes_bit_shift_operators`) + 3 new parser
    tests (`parses_left_shift`, `parses_right_shift`,
    `shift_binds_tighter_than_comparison_but_looser_than_addition`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate
    passes
- [x] `ConstTable`/`EndConstTable` declaration block not implemented
  ✅ Resolved
  - Not in `keywords.json`, no parser rule. Confirmed at
    [consttableendconsttable.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/consttableendconsttable.htm):
    a documented, moderately common pattern letting field techs edit
    constants and recompile without touching the constant's use
    sites.
  - Added to `keywords.json` under a new `consttable` category, wired
    into `generate-grammar.js`'s TextMate output alongside the
    existing `datatable` one; reused `DataTable`'s parenthesized
    argument parsing in `parse_program_structure` since
    `ConstTable(Name, Enabled)` takes the same shape. `Const`
    declarations inside the block parse as ordinary flat statements,
    matching how `DataTable`'s body already works
  - Added matching completion snippets, hover text, folding-range
    pairing (a new stack alongside the existing
    `BeginProg`/`DataTable` ones in `folding.rs`), and
    indentation/folding-marker regexes with Vitest coverage, so the
    new block gets the same editor support as `DataTable`/`EndTable`
  - 3 new parser tests (`parses_const_table_with_arguments`,
    `parses_end_const_table_statement`,
    `parses_complete_const_table_structure`) + 1 new folding test
    (`pairs_const_table_with_the_matching_end_const_table`) + 88
    Vitest cases (existing suite extended) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass
- [x] `Optional` parameter modifier in `Function`/`Sub` parameter
  lists not implemented ✅ Resolved
  - `parse_function_definition`/`parse_subroutine_definition` only
    accepted bare identifiers in the parameter list; `Optional` wasn't
    even a keyword. Confirmed at
    [optional.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/optional.htm).
    Lower-moderate frequency.
  - Added `Optional` to `keywords.json` (`declaration` category) and
    taught both parameter-parsing loops to skip a leading `Optional`
    keyword before the parameter name. Parameters stay a plain
    `Vec<String>` as before -- this project doesn't track
    per-parameter types or other modifiers either, so adding metadata
    just for `Optional` would be inconsistent with the existing shape
  - 2 new parser tests (`parses_function_with_an_optional_parameter`,
    `parses_subroutine_with_an_optional_parameter`) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] `@`/`!` pointer operators not implemented ✅ Resolved (operator
  forms only)
  - The master Operators page has a "Pointer" category listing both.
    Distinct from the previously-dismissed VB6-lookalike operators
    (see below) -- CRBasic genuinely added pointer support via
    `@`/`!`. Low frequency.
  - Confirmed via the linked
    [pointer-variable docs page](https://help.campbellsci.com/crbasic/cr1000x/Content/Info/pointervariable.htm):
    `@` is a prefix "address of" operator (`Ptr = @MyVariable`) and
    `!` is a prefix dereference operator (`MyVariable = !Ptr`)
  - Added `At`/`Bang` tokens and
    `UnaryOperator::AddressOf`/`Dereference`, parsed in `parse_unary`
    alongside the existing `-`/`NOT` prefix operators
  - Deliberately scoped to the operator forms only -- the docs also
    show `!` as a **suffix** on type names in pointer declarations
    (`Dim ptr as long!`) and indexed/cast dereference forms
    (`!Pointer(X)`, `!(FLOAT!)(p+1)`); those are a separate, more
    involved declaration-syntax extension not covered here
  - 1 new lexer test (`recognizes_pointer_operators`) + 2 new parser
    tests (`parses_address_of_operator`, `parses_dereference_operator`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate
    passes
- [x] `Imp` logical operator not implemented ✅ Resolved
  - Round 1 grouped `Eqv`/`Imp`/`IntDv` together as "unconfirmed,
    resembles a copy-pasted VB6 operator list, not acted on." The
    master Operators page's Logical category confirms `IMP`
    (implication) is genuinely documented CRBasic. `Eqv` and `IntDv`
    still do **not** appear anywhere on that page, so the original
    skepticism about those two stands -- only `Imp` was reconsidered
    here. Low frequency.
  - Added `IMP` to `keywords.json` and `BinaryOperator::Implication`,
    parsed via a new `parse_logical_imp` tier below the existing
    OR/XOR/AND chain -- the loosest-binding logical operator, per the
    common BASIC-family convention (Campbell Scientific's docs don't
    state precedence explicitly). Added matching completion/hover
    entries
  - 2 new parser tests (`parses_imp_operation`,
    `imp_has_lower_precedence_than_or`) added Red-first; full workspace
    `build`/`test`/`clippy`/`fmt` gate passes

Not flagged as gaps (verified during the same comparison):

- `Case Else`, `<>`/`NotEqual`, `^`/`Caret`, and all `As`-clause data
  types (`Float`/`Boolean`/`UINT1`/etc.) were checked against the
  master Operators page and are already correctly implemented.
- `Include` (see Round 2's already-deferred entry above): no new
  information surfaced this round.

### Reference Implementation & Official Docs Comparison, Round 4 (2026-08-09)

Found during a fourth comparison round, prompted by re-surveying
`.connect0459/ref-repos/` and help.campbellsci.com specifically for gaps
Rounds 1-3 missed (control-flow bugs, editor-level folding/indentation,
and a lexer categorization bug), rather than operators/keywords already
swept. Each finding verified against an official docs page plus either a
real parse repro or a source-code read, not just a reference grammar.

- [x] `Scan`/`SubScan`/`SlowSequence` blocks had zero folding or
  indentation support ✅ Resolved
  - `Scan...NextScan` is the single most common block in a CRBasic
    program (the main measurement loop), yet `client/language-configuration.json`
    and `crates/crbasic-lsp/src/folding.rs` had no handling for it at
    all -- every other block construct (`If`, `For`, `Do`, `Function`,
    `Sub`, `Select Case`, `DataTable`, `ConstTable`, `BeginProg`, `#If`,
    even the rarer `SlowSequence`/`EndSequence`) already did
  - Fixing `SlowSequence`/`EndSequence` surfaced a deeper, previously
    unnoticed bug: both were miscategorized in `keywords.json` as
    `builtinFunctions` instead of `languageKeywords`, so the lexer's
    `lookup_keyword` (which only checks `LANGUAGE_KEYWORDS`) never
    tokenized them as keywords at all -- confirmed via a throwaway
    token dump that `SlowSequence`/`EndSequence` lexed as plain
    `Identifier`s, parsing as inert expression statements invisible to
    `parse_program_structure`, completion's keyword list, and hover.
    They're bare block markers with no arguments (like `BeginProg`/
    `DataTable`), not parenthesized calls (like `Scan`/`SubScan`), so
    moved to `languageKeywords` and wired into `parse_program_structure`
  - Added `Scan\s*\(`/`SubScan\s*\(` to `increaseIndentPattern`/
    `folding.markers` start, and `NextScan\b`/`NextSubScan\b` to
    `decreaseIndentPattern`/`folding.markers` end (word-boundary-safe
    against `ContinueScan`, matching the Round 2 `\b` fix); `SlowSequence`/
    `EndSequence` added to `folding.markers` (already present in
    `indentationRules` for `SlowSequence`, but `EndSequence` was missing
    even there)
    - `folding.rs` now pairs `Scan`/`NextScan` and `SubScan`/`NextSubScan`
      the same way it already pairs `BeginProg`/`EndProg` and
      `DataTable`/`EndTable`, keyed off `Statement::FunctionCall`'s name
      for the openers since `Scan(...)`/`SubScan(...)` take parenthesized
      arguments
  - 4 new folding tests, 1 new parser test, and 21 new Vitest cases
    (indentation + folding.markers, including false-positive guards for
    `ContinueScan` and a bare `ScanValue = 5` assignment) added Red-first
- [x] `Do Until condition ... Loop` / `Do ... Loop Until condition`
  silently corrupted the statement list (bug) ✅ Resolved
  - Confirmed at [Do...Loop](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/doloop.htm):
    `Until` is a documented alternative to `While` at either position,
    but `Until` wasn't a registered keyword at all -- `parse_do_loop`
    only ever checked for `While`. Repro confirmed before fixing: `Do
    Until x > 10 ... Loop` parsed **without error** but wrongly, leaking
    `Until`/`x > 10` into the loop body as bogus statements and silently
    becoming an unconditional loop
  - Desugars to the same `DoLoop` shape as `While`, with the condition
    wrapped in a logical `Not` (`Do Until cond` behaves like `Do While
    Not cond`), rather than adding a new AST field -- every downstream
    consumer already handles `DoLoop` generically
  - 4 new parser tests added Red-first
- [x] `Call SubName(args)` silently corrupted the statement list (bug)
  ✅ Resolved
  - Confirmed at Campbell Scientific's own
    [Sub/Exit Sub/EndSub](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/subexitsubendsub.htm)
    page (`Call ConvertCtoF(TC(I), TC_F(I))`), but `Call` wasn't
    registered -- it split into a bogus `Identifier("Call")` expression
    statement followed by the real function call
  - `Call` is purely an optional prefix, so `parse_call_statement` just
    consumes the keyword and delegates to the existing call-expression
    path. A pre-existing test used `"Call"` as an arbitrary function
    name to test boolean-literal arguments (incidental to its actual
    intent); renamed to `"Invoke"` since it collided with the new keyword
  - 1 new parser test added Red-first
- [x] `Alias`/`Units` statements advertised via completion/hover but
  unparseable (bug) ✅ Resolved
  - Same "advertised via completion/hover, silently broken in the
    parser" bug class as the already-resolved `Mod`/`ElseIf`/`Select
    Case` gaps -- a fourth, unfixed instance. Confirmed real at
    [Alias](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/alias.htm)
    (including the multi-name form,
    `Alias Array = FrontRoom, BedRoom, GreatRoom(4), Laundry`) and
    [Units](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/units.htm);
    `parse_statement` had no branch for either, so both were hard parse
    errors
  - New `Statement::Alias`/`Statement::Units` AST variants. Both sides
    of each statement are parsed via `parse_primary` rather than
    `parse_expression`, since CRBasic's parenthesized subscript form
    (`TCTemp(1)`) already parses as an ordinary `Expression::FunctionCall`
    at that level, and stopping there avoids `=` being misread as the
    comparison operator it is everywhere else. `call_sites.rs` (shared by
    inlay hints and call hierarchy) deliberately skips both statements'
    operands, since they're names, not real calls, even when
    subscript-shaped
  - 5 new parser tests added Red-first
- [x] Fixed-length string declarations (`As String * N`) unparseable
  ✅ Resolved
  - Confirmed at [Dim](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/dim.htm)
    (default size 24, minimum 4; `Public` supports the same syntax as
    `Dim`). The declaration parser only ever looked for `=`/newline
    after the type identifier, so `* 30` failed with "Unexpected token:
    Star"
  - Added an optional `type_size` field to `VarDeclaration`, parsed via
    `parse_primary` for the same `=`-ambiguity reason as `Alias`/`Units`
    above. Every other `VarDeclaration` construction site across the
    workspace (test fixtures in `semantic.rs` and the LSP provider
    tests) updated with the new field
  - 3 new parser tests added Red-first
- [x] `ContinueScan` keyword never implemented ✅ Resolved
  - Confirmed at the [Scan, NextScan](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/scannextscan.htm)
    page -- and already cited in this project's own Round 2 entry (as
    part of the real exit/jump vocabulary justifying the fabricated
    `Continue`/`Break`/`GoTo` removal) but never itself registered.
    Lexed as a bare, unrecognized identifier with no completion, hover,
    or parser support
  - Parses the same way as the already-supported `ExitFor`/`ExitDo`: a
    bare keyword handled by `parse_program_structure`
  - 1 new parser test added Red-first
- [x] `SubScan`/`NextSubScan` nested-scan block entirely unregistered
  ✅ Resolved
  - Confirmed at [SubScan, NextSubScan](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/subscannextsubscan.htm)
    (nests inside the main `Scan` for faster analog measurement or
    AM16/32 multiplexer control); both reference tmLanguage grammars
    list it, but this project had no coverage at all -- it only
    "parsed" by accident, as an unrecognized identifier/function call
  - Mirrors the existing `Scan`/`NextScan` split: `SubScan` is a
    `builtinFunctions` entry (parenthesized arguments, ordinary
    `FunctionCall` parsing already handles it), `NextSubScan` is a
    `languageKeywords` entry handled by `parse_program_structure`
  - 1 new parser test added Red-first

Not flagged as gaps (verified during the same comparison):

- `Debug`/`DebugBreak`, `ArrayIndex`, `StationName`/`Status`,
  `LoggerType`, `ReadOnly`, `RunProgram`, bare `End` each appear in only
  **one** of the two reference tmLanguage files, not both -- per this
  project's own corroboration bar (established in Round 1), not
  escalated to findings without independent confirmation.
- No separate `Aliases`/`EndAliases` block construct exists in the
  official docs distinct from the single `Alias` statement -- the
  multi-name comma form is part of the same instruction, not a separate
  block.
- Multi-dimensional array declarations (`Dim arr(3,4)`) already parse
  correctly.

### Reference Implementation & Official Docs Comparison, Round 5 (2026-08-10)

Found during a fifth comparison round, prompted by re-surveying
`.connect0459/ref-repos/` (particularly `crbasic-vscode-support/src/`,
`snippets/`, and `sources/`, not closely read in Rounds 1-4) and
help.campbellsci.com for gaps the first four rounds missed. Each finding
verified against an official docs page and a real parse repro, not just a
reference grammar.

- [x] `ReadOnly Var1, Var2, ...` statement -- hard parse error (bug)
  ✅ Resolved
  - Round 4's "Not flagged" note dismissed `ReadOnly` as appearing in only
    one reference grammar (this project's own corroboration bar from
    Round 1). A fresh grep of both reference grammars' actual files this
    round found it in **both**, and it's independently confirmed real at
    [ReadOnly](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/readonly.htm)
    -- a comma-separated identifier list, typically paired right after a
    `Public` declaration to make a calculated variable visible-but-not-
    externally-editable. A common idiom, not niche.
  - Same "advertised-shape-but-unparseable" bug class as the
    already-resolved `Mod`/`Select Case`/`Alias`/`Units` gaps, except here
    it wasn't even advertised -- `ReadOnly` had zero keyword, parser,
    completion, or hover coverage at all, confirmed via repro:
    `Public Mult, Offset` / `ReadOnly Mult, Offset` failed with
    `Unexpected token: Comma`.
  - New `Statement::ReadOnly` AST variant, parsed the same way as the
    existing `Alias`/`Units` statements: a comma-separated list of
    `parse_primary`-parsed expressions (supporting both plain identifiers
    and the parenthesized-subscript form, e.g. `ReadOnly Cal(1)`)
  - 4 new parser tests added Red-first
- [x] `StructureType`/`EndStructureType` block -- entirely unregistered,
  hard parse error (bug) ✅ Resolved
  - Confirmed at
    [StructureType/EndStructureType](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/structuretype.htm):
    defines a reusable data structure (member declarations, no
    `Public`/`Dim`/`Const` prefix), instantiated via the existing
    `Public`/`Dim ... As StructureTypeName` grammar and accessed via dot
    notation (`CS215(1).Temp`). Explicitly recommended by Campbell
    Scientific to shorten programs using `AVW200()`, `GPS()`,
    `SDI12Recorder()`, and similar multi-value instructions -- a real,
    moderately common pattern in modern CR6/GRANITE programs. Confirmed
    via repro before fixing: `StructureType Foo` / `Bar As Float` /
    `EndStructureType` failed with `Unexpected token: Keyword("As")`.
  - Added a `Dot` token (previously any `.` not part of a number literal
    was silently dropped by the scanner's unknown-character fallback) and
    `Expression::MemberAccess`, parsed as a postfix in the same loop that
    already builds `FunctionCall`/`ArrayAccess` from a leading identifier
  - New `Statement::StructureType`/`StructureMember` AST types. Member
    declarations reuse the same array-dimension and fixed-length-string
    parsing as `parse_var_declaration` (`As Type [* length]`); nested
    `Units`/`ReadOnly` modifiers reuse those statements' own parsers
    unchanged rather than duplicating their grammar
  - **Deliberately scoped to reading members only**: `object.member` is
    not extended to assignment targets in this pass, mirroring how
    `Alias`/`Units` already only support the read side of their own
    parenthesized-subscript operands -- extending the existing
    identifier/array-element assignment-target fast path in
    `parse_statement` to a third, dot-chained target shape is a larger,
    separable change than the block-parsing gap this entry tracks
  - `generate-grammar.js` needed its own fix alongside this: adding a new
    `keywords.json` category (`structuretype`) without a matching
    hardcoded scope block in the codegen script silently dropped those
    keywords from the generated TextMate grammar (they still reached
    `LANGUAGE_KEYWORDS` for the parser/lexer, just not
    `crbasic.tmLanguage.json`) -- caught by manually grepping the
    generated file for the new keyword after regenerating, not by any
    existing check. Fixed by adding a `structuretype` scope block
    alongside the existing `consttable` one
  - `client/language-configuration.json`'s indentation and folding rules
    extended the same way `ConstTable`/`EndConstTable` already are, with
    matching Vitest cases added; `folding.rs` pairs `StructureType`'s own
    span directly (no stack needed, unlike `ConstTable`, since this
    block parses as one AST node spanning both keywords rather than two
    independent flat statements)
  - 1 new lexer test (`Dot` token, plus a regression test confirming
    float literals like `3.14` are unaffected), 7 new parser tests, 1 new
    folding test, and 8 new Vitest cases added Red-first
- [x] `WaitTriggerSequence` bare keyword -- silently corrupted the
  statement list (bug) ✅ Resolved
  - Confirmed at
    [TriggerSequence, WaitTriggerSequence](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/triggersequencewaittriggersequence.htm):
    a bare keyword marking a resume-point inside a `SlowSequence`/
    `Do...Loop`, a direct companion feature to `SlowSequence`/`NextScan`
    folding support added in Round 4 but missed there
  - Same silent-corruption bug class Round 4 found and fixed for
    `ContinueScan`/`Next i`: lexed as a plain identifier, parsing as an
    inert no-op expression statement inside the enclosing loop body
  - Parses the same way as the already-supported `ContinueScan`: a bare
    keyword handled by `parse_program_structure`
  - 1 new parser test added Red-first
- [x] `DebugBreak` bare keyword -- silently corrupted the statement list
  (bug) ✅ Resolved
  - Round 4's "Not flagged" note dismissed `Debug`/`DebugBreak` as
    appearing in only one reference grammar. Independently confirmed real
    this round at
    [Debug, DebugBreak](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/debugdebugbreak.htm):
    a bare, argument-less breakpoint marker placed inline in code
  - Same silent-corruption bug class as `WaitTriggerSequence` above.
    Lower real-world impact than the other three findings this round
    since it's a debug-only instruction, not something production
    programs depend on at runtime
  - Parses the same way as `ExitFor`/`ExitDo`: a bare keyword handled by
    `parse_program_structure`
  - 1 new parser test added Red-first

Not flagged as gaps (verified during the same comparison):

- `ArrayIndex`, `TableFile`, `GetRecord`, `AvgSpa`, `PulseCountReset`,
  `NewFieldNames`, `TriggerSequence`, `SemaphoreGet`/`SemaphoreRelease`,
  `RunProgram`, `LoggerType`, `WaitDigTrig` -- all function-call-shaped,
  parse correctly today via the generic `FunctionCall` grammar
  (repro-verified for `ArrayIndex`). Part of the same already-deferred
  ~126-vs-~420 `builtinFunctions` content backlog from Round 2 (each
  needs real per-parameter completion snippets/hover prose authored, not
  a parser fix); worth prioritizing these specific names first whenever
  that backlog is tackled, since they're core/common rather than
  telemetry-module-specific, but not acted on here.
- `crbasic-vscode-support/src/extension.js`: only the file-import/
  PC400-launch commands already noted (and dismissed) in Round 1; nothing
  new.
- `crbasic-vscode-support/sources/`: icon images only, no content.
- `crbasic-vscode-support/snippets/crbasic.json`: 11 basic snippets; the
  only name not already in `keywords.json` was `CallTable`, which turned
  out to already be present (`time` category) -- a stale assumption, not
  a gap.
- `Eqv`/`IntDv`/a stray `|` operator seen in one grammar's operator
  regex -- still no official-docs corroboration (Round 3's bar), looks
  like a VB6-list/regex-escaping artifact; correctly left dismissed.

### Reference Implementation & Official Docs Comparison, Round 6 (2026-08-10)

Found during a sixth comparison round, this time driven by a fresh
full-text grep of both reference grammars' keyword lists against
`keywords.json` (rather than spot-checking specific instructions), plus a
re-read of `docs/researches/`. Each finding verified against an official
help.campbellsci.com page and a real parse repro before fixing.

- [x] `ExitScan` miscategorized as a `builtinFunctions` entry instead of a
  `languageKeywords` one (bug) ✅ Resolved
  - Same "silently corrupted statement list" bug class as the
    already-resolved `ContinueScan`/`WaitTriggerSequence`/`DebugBreak`
    gaps. `ExitScan` *was* present in `keywords.json`, but filed under
    `builtinFunctions` (category `time`); since the lexer's
    `lookup_keyword` only checks `LANGUAGE_KEYWORDS`, it lexed as a plain
    `Identifier` and parsed as an inert expression statement instead of a
    real scan-exit. Confirmed real and bare (no parens) at
    [Scan, NextScan](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/scannextscan.htm),
    the same page already cited for `ContinueScan`.
  - Moved to `languageKeywords` (category `scan`, alongside `NextScan`/
    `ContinueScan`); its pre-existing `builtinFunctions`-style completion
    item replaced with a keyword-style one; added to the parser's bare
    program-structure keyword list
  - 1 new parser test (`parses_exitscan_inside_scan_loop`) added Red-first
- [x] `SequentialMode`/`PipeLineMode` bare keywords -- entirely
  unregistered, same silent-corruption bug class ✅ Resolved
  - Confirmed real, bare (no parens), placed before `BeginProg`, at
    [SequentialMode, PipeLineMode](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/sequentialmodepipeli2.htm).
    Present in both reference grammars; zero coverage anywhere in this
    project before this fix
  - Added to `languageKeywords` (category `program`, alongside
    `BeginProg`/`EndProg`), the parser's bare program-structure keyword
    list, and matching completion/hover entries
  - 2 new parser tests (`parses_sequentialmode_before_beginprog`,
    `parses_pipelinemode_before_beginprog`) added Red-first
- [x] `DisplayMenu`/`SubMenu`'s bare closing keywords `EndMenu`/
  `EndSubMenu` -- entirely unregistered, same silent-corruption bug class
  ✅ Resolved
  - Confirmed at
    [DisplayMenu, EndMenu](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/displaymenuendmenu.htm):
    a custom keypad/display menu block, closed by a bare `EndMenu`, with
    nestable `SubMenu("Name")`/bare `EndSubMenu` blocks inside. Present in
    both reference grammars' keyword lists
  - `DisplayMenu(...)`/`SubMenu(...)` themselves already parse correctly
    today as ordinary `FunctionCall` statements (parenthesized-call shape
    needs no special grammar), but `EndMenu`/`EndSubMenu` lexed as plain
    identifiers and parsed as inert expression statements, corrupting the
    block structure
  - Added `EndMenu`/`EndSubMenu` to `languageKeywords` (new `menu`
    category) and the parser's bare program-structure keyword list;
    `generate-grammar.js` gained a matching `keyword.control.menu.crbasic`
    scope block (the Round 5 `StructureType` lesson: a new category needs
    an explicit codegen scope or it silently drops from the generated
    TextMate grammar)
  - `folding.rs` pairs `DisplayMenu`/`EndMenu` and `SubMenu`/`EndSubMenu`
    the same way it already pairs `Scan`/`NextScan` and `SubScan`/
    `NextSubScan`, keyed off the `FunctionCall` statement's name for the
    openers; `client/language-configuration.json`'s indentation and
    `folding.markers` regexes extended to match, with matching Vitest
    cases
  - Deliberately did **not** register `DisplayMenu`/`SubMenu` themselves in
    `keywords.json`: they already parse correctly, and adding
    completion/hover/highlighting parity for them is separate content
    work in the same already-deferred tier as the ~126-vs-~420
    `builtinFunctions` backlog (Round 2)
  - 2 new parser tests (`parses_endmenu_closing_a_display_menu_block`,
    `parses_endsubmenu_nested_inside_display_menu`) + 2 new folding tests
    - 16 new Vitest cases (indentation + folding.markers) added Red-first

Not flagged as gaps (verified during the same comparison):

- Full re-diff of both `syntaxes/*.tmLanguage.json` files' keyword lists
  against `keywords.json` turned up nothing else uncovered; the
  `Eqv`/`IntDv`/stray-`|` operator artifacts remain uncorroborated by
  official docs, consistent with Rounds 1-3
- `docs/researches/research-001-crbasic-for-vscode.md` re-read in full:
  its line-continuation, scope, and Function/Sub copy-back-semantics
  sections are all either already implemented or LSP-doc/hover content,
  not parser gaps
- `crbasic-vscode-support/snippets/crbasic.json` (exhaustively checked in
  Round 5) and `src/extension.js` (dismissed in Round 1) had nothing new

### Reference Implementation & Official Docs Comparison, Round 7 (2026-08-10)

Found during a seventh comparison round, this time driven by a fresh sweep
for bare (no-parenthesis) declaration-section instructions and
`DataTable`-body modifiers -- a vein Rounds 1-6 hadn't specifically targeted
(they focused on control flow, operators, and scan/menu block keywords).
Each finding verified against an official help.campbellsci.com page and a
real parse repro before fixing; all six follow the same "advertised-shape
bare keyword silently corrupts the statement list" bug class as the
already-resolved `ContinueScan`/`WaitTriggerSequence`/`SequentialMode`/
`PipeLineMode`/`EndMenu`/`EndSubMenu` gaps from Rounds 4-6.

- [x] `Restart` bare keyword -- silently corrupted the statement list
  (bug) ✅ Resolved
  - Confirmed at [Scan, NextScan](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/restart.htm):
    forces the datalogger to stop and restart the running program,
    documented inside an `If ProgramRestart = True ... EndIf` fault-recovery
    idiom within a `Scan` loop. Zero prior coverage anywhere in this project
  - Parses the same way as the already-supported `DebugBreak`: a bare
    keyword handled by `parse_program_structure`
  - 1 new parser test (`parses_restart_inside_if_statement`) added
    Red-first
- [x] `PreserveVariables` bare keyword -- silently corrupted the statement
  list (bug) ✅ Resolved
  - Confirmed at
    [PreserveVariables](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/preservevariables.htm):
    placed before `BeginProg`, retains `Dim`/`Public` variable values in
    memory across a power loss -- a common production requirement for
    field dataloggers. Same declarations-section placement as the
    already-supported `SequentialMode`/`PipeLineMode`, but itself missed
  - 1 new parser test (`parses_preservevariables_before_beginprog`) added
    Red-first
- [x] `ApplyAndRestartSequence`/`EndApplyAndRestartSequence` block --
  entirely unregistered, same silent-corruption bug class ✅ Resolved
  - Confirmed at
    [ApplyAndRestartSequence, EndApplyAndRestartSequence](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/applyandrestartsequence.htm):
    a declarations-section block, placed immediately before `ConstTable`,
    that validates a `ConstTable` field before it's applied at runtime --
    the official example program pairs the two directly. Both keywords
    were completely absent from this project despite `ConstTable` itself
    (Round 3) already being supported
  - Parsed the same flat-statement way `ConstTable`/`EndConstTable` already
    are, rather than a single spanning AST node
  - Folding support added: `folding.rs` pairs the two the same way it
    already pairs `ConstTable`/`EndConstTable`, via a new stack;
    `client/language-configuration.json`'s indentation and
    `folding.markers` regexes extended to match, with matching Vitest
    cases
  - 1 new parser test
    (`parses_applyandrestartsequence_block_before_beginprog`) + 1 new
    folding test (`pairs_apply_and_restart_sequence_with_its_matching_end`)
    - 4 new Vitest cases added Red-first
- [x] `ShutDownBegin`/`ShutDownEnd` block -- entirely unregistered, same
  silent-corruption bug class ✅ Resolved
  - Confirmed at
    [ShutDownBegin, ShutDownEnd](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/shutdownbeginshutdownend.htm):
    a declarations-section block that runs cleanup code (e.g. closing a
    serial port) when the program stops normally -- common in
    serial/telemetry-heavy programs
  - Same treatment as `ApplyAndRestartSequence`/`EndApplyAndRestartSequence`
    above: flat-statement parsing, `folding.rs` stack-based pairing,
    `client/language-configuration.json` indentation/folding-marker
    coverage with matching Vitest cases
  - 1 new parser test
    (`parses_shutdownbegin_shutdownend_block_before_beginprog`) + 1 new
    folding test (`pairs_shutdownbegin_with_its_matching_shutdownend`) + 4
    new Vitest cases added Red-first
- [x] `TableHide`/`OpenInterval` `DataTable`-body keywords -- silently
  corrupted the statement list (bug) ✅ Resolved
  - Confirmed at
    [TableHide](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/tablehide.htm)
    (suppresses a table's display and data collection, placed immediately
    after the `DataTable` statement) and
    [OpenInterval](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/openinterval.htm)
    (makes time series processing include all measurements since the last
    data storage, spanning missed output intervals, instead of only the
    current one). Both are bare keywords inside a `DataTable(...) ...
    EndTable` body, which -- like `ConstTable`'s body -- parses as an
    ordinary flat statement list, so an unrecognized bare keyword there
    vanishes as an inert identifier statement the same way `ContinueScan`
    did before its Round 4 fix
  - 2 new parser tests (`parses_tablehide_inside_datatable_body`,
    `parses_openinterval_inside_datatable_body`) added Red-first

Not flagged as gaps (verified during the same comparison):

- `DialSequence(...)`/`EndDialSequence(...)` (PakBus dial-modem routing
  block): both keywords are parenthesized calls, so they already parse
  correctly today via the generic `FunctionCall` grammar -- no parser bug.
  Editor-level folding/indentation pairing is a real but low-priority gap
  (legacy dial-modem telemetry, rare in modern deployments), deferred
  rather than acted on this round
- `BeginBurstTrigger`/`EndBurstTrigger`: appears in only one reference
  grammar (tied to old CR9032-era hardware), no live official-docs page
  found -- does not clear this project's corroboration bar, consistent
  with how `Eqv`/`IntDv` were dismissed in Rounds 1/3
- `Restore`: 404s on help.campbellsci.com, only in one reference grammar --
  looks like a VB6 `DATA`-statement leftover, dismissed on the same basis
- Hex/octal/binary numeric literals: no such syntax documented on the
  master Operators page; CRBasic instead exposes `HexToDec`/`Hex` as
  string-conversion functions, consistent with the lexer's decimal-only
  `scan_number`. Correction (Round 17): this note originally named the
  companion function `DecToHex`, which turned out to be fabricated --
  `HexToDec`'s own official page names its inverse `Hex`, not `DecToHex`;
  see Round 17 below
- Array-of-`StructureType` declarations (e.g. `Dim CS215(4) As
  CS215Data`): already parse correctly -- `parse_var_declaration` parses
  array dimensions generically before the `As` type name, independent of
  whether the type name resolves to a `StructureType`
- Full re-diff of both reference grammars' keyword lists against
  `keywords.json` (250 shared names): everything else unmatched falls
  inside the already-deferred ~126-vs-~420 `builtinFunctions` content
  backlog (Round 2/5); spot-checked several likely bare-keyword
  candidates individually (`Erase`, `Randomize`, `Broadcast`,
  `ClockReport`, `DaylightSaving(US)`, `EncryptExempt`, `DataEvent`,
  `WorstCase`) to rule out parser bugs specifically -- all parenthesized,
  all already parse correctly via the generic `FunctionCall` grammar

### Reference Implementation & Official Docs Comparison, Round 8 (2026-08-10)

Found during an eighth comparison round, this time targeting angles Rounds
1-7 hadn't systematically covered: a fabrication audit of every current
`keywords.json` entry (not just ones found incidentally), a fresh
re-verification of previously-dismissed uncorroborated names, and the
`Include` statement's exact current parse behavior. Each finding verified
against an official help.campbellsci.com page fetched directly, not just a
reference grammar.

- [x] `FillStop` bare keyword inside `DataTable` body -- silently corrupted
  the statement list (bug) ✅ Resolved
  - Same bug class as Round 7's `TableHide`/`OpenInterval` fix, missed by
    that round despite being a sibling `DataTable`-body keyword. Confirmed
    real and bare (no parens) at
    [FillStop](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/fillstop.htm):
    stops data storage once the table reaches its configured size, instead
    of the default ring-memory overwrite behavior
  - Parsed the same way as `TableHide`/`OpenInterval`: a bare keyword
    handled by `parse_program_structure`
  - 1 new parser test (`parses_fillstop_inside_datatable_body`) added
    Red-first
- [x] `Include "Device:Filename"` -- silently corrupted the statement list
  (bug) ✅ Resolved
  - Round 2 characterized `Include` as a deferred *feature* ("structural
    parsing would be low effort... deferred" pending cross-file
    infrastructure), which undersold it: a repro showed it parses **without
    error** but wrongly, splitting into two bogus statements (an
    `Identifier("Include")` expression followed by the path string
    literal) -- the same silent-corruption bug class as `ContinueScan` and
    `Next`'s trailing counter list. Confirmed real at
    [Include](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/include.htm)
  - New `Statement::Include` AST variant consumes the keyword and its path
    expression as one unit. Deliberately still does not resolve, read, or
    index the referenced file -- this project has no cross-file
    infrastructure yet, matching the existing open-documents-only scope of
    `workspaceSymbolProvider`/`callHierarchyProvider`. Splits the bug
    (fixed now) from the resolution/indexing feature (still deferred)
  - 2 new parser tests (`parses_include_of_a_string_path`,
    `include_does_not_swallow_the_following_statement`) added Red-first
- [x] `Sqrt`, `LCase`, `UCase`, `TableName`, `IsNaN` -- fabricated, not real
  CRBasic ✅ Resolved (removed)
  - First systematic fabrication audit of `keywords.json`'s full 202
    entries, rather than relying on names found incidentally (the prior
    fabrication removal, `ExitSelect`/`Continue`/`Break`/`GoTo` in Round 2,
    was found that way). Each confirmed absent from official docs and both
    reference grammars:
    - `Sqrt`: not documented anywhere, and not an alias of `Sqr` --
      [Sqr's own page](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/sqr.htm)
      never mentions "Sqrt". Worst of the five: it shipped in
      `completion.rs` with a false claim ("alias for Sqr"), unlike the
      other four which were only phantom `keywords.json` entries with no
      completion/hover coverage
    - `LCase`/`UCase`: the real, documented names are
      [LowerCase](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/lowercase.htm)/[UpperCase](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/uppercase.htm)
      (both already correctly present in `keywords.json`); `lcase.htm`/
      `ucase.htm` both 404
    - `TableName`: a syntax-diagram placeholder meaning "substitute your
      own declared `DataTable`'s name" (as in
      [`TableName.FieldName`](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/tablenamefieldname.htm)),
      not a real standalone instruction -- the same placeholder-extraction
      mistake, worth watching for elsewhere in the `data` category
    - `IsNaN`: `NaN` is a real predefined comparison constant
      (`x = NaN`), but there is no such wrapper function --
      `isnan.htm` 404s and Campbell's own forum guidance describes
      checking for NaN via direct comparison, not a function call
  - Removed from `keywords.json`; `Sqrt`'s `completion.rs` entry removed;
    `signature.rs`'s `"sqr" | "sqrt"` match arm narrowed to `"sqr"` and its
    test comment (which cited the false alias claim as its example)
    corrected to cite the genuine `Log`/`Ln` alias pair instead (confirmed
    real at
    [LOG or LN](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/logorln.htm):
    "The LOG, or LN, function returns the natural logarithm of a number");
    `inlay_hint.rs`'s incidental `"Sqrt"` test fixture (unrelated to the
    finding -- just an arbitrary call name) renamed to the real `"Sqr"`

Not flagged as gaps (verified during the same comparison):

- `TriggerSequence` (companion to the already-supported bare
  `WaitTriggerSequence`, Round 5): confirmed function-shaped
  (`TriggerSequence(SequenceNum, TimeOut)`) at the same official page
  already cited for `WaitTriggerSequence` -- already parses correctly via
  the generic `FunctionCall` grammar, not a second missed bare keyword
- `Debug` (bare-keyword sibling of `DebugBreak`): confirmed function-shaped
  (`Debug(DebugSequence, HistorySize, Control, LineBreak, TraceHistory)`),
  so it was never a bare-keyword risk regardless of Round 4's
  single-reference-grammar dismissal
- `Eqv`/`IntDv`/stray `|` operator/`BeginBurstTrigger`/`EndBurstTrigger`/
  `Restore`: re-searched with fresh queries and alternate datalogger-model
  doc pages; still zero official-docs corroboration. Status unchanged from
  Rounds 1/3/7
- `Ln`: unlike the four removed fabrications above, this one is
  independently corroborated by the `cr-basic-ms-vscode` reference
  grammar's function list -- not a fabrication, and confirmed genuinely
  real (see the `Log`/`Ln` alias finding above)
- `DialSequence`/`EndDialSequence` folding (Round 7's deferred, low-priority
  item): re-confirmed still purely an editor-folding gap, not a parser bug
  -- the statement itself already parses correctly via the generic
  `FunctionCall` grammar
- Full independent re-diff of both reference grammars' keyword lists
  against `keywords.json`: beyond `FillStop` above, everything else
  unmatched resolves to either the already-deferred ~126-vs-~420
  `builtinFunctions` content backlog (function-shaped, parses fine
  generically) or names already individually dismissed in Rounds 4/5/7
  (`ArrayIndex`, `StationName`, `Status`, `LoggerType`, `RunProgram`,
  `SemaphoreGet`/`Release`, `WaitDigTrig`, `NewFieldNames`, etc.)

### Reference Implementation & Official Docs Comparison, Round 9 (2026-08-10)

Found during a ninth comparison round, this time targeting angles Rounds
1-8 hadn't systematically covered: `Dim`/`Public`/`Const` initializer
syntax specifically (rather than keyword-list diffing), and a fresh
full-text diff of both reference grammars' keyword/snippet lists against
`keywords.json`. Unlike every prior round, two of the three findings below
are the *inverse* of the dominant bug class -- a real, documented
construct that was unparseable, and over-permissive grammar that silently
accepted invalid syntax -- rather than a bare keyword being swallowed as
an inert identifier. Each finding verified against an official
help.campbellsci.com page and a real parse repro before fixing.

- [x] Brace-list array initializer (`{v1, v2, ...}`) entirely unlexed
  (bug) ✅ Resolved
  - `{`/`}` fell through the scanner's catch-all unknown-character arm in
    `crates/crbasic-parser/src/lexer/scanner.rs`, silently vanishing from
    the token stream. Confirmed real at
    [Dim](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/dim.htm)
    and [Public](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/public.htm)
    (`Public MyArray(3) = {3, 6, 9}`) and
    [Const](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/const1.htm)
    (`Const A = {1,2,3,4,5,6,7,8,9,10}`, added in OS9). Repro confirmed:
    `Public Array (2,3) = {1,2,3,4}` failed with `Expected identifier
    after variable declaration keyword or comma`, since the braces
    disappeared and the bare `1,2,3,4` was misread as a comma-separated
    *variable list* continuation. Scalar and fixed-length-string
    initializers already worked correctly -- only the brace-array form
    was broken
  - Added `LeftBrace`/`RightBrace` tokens and an
    `Expression::ArrayLiteral` AST variant, parsed by a new
    `parse_var_initializer` helper shared by both declaration-parsing
    functions (`parse_var_declaration`, `parse_single_var_with_keyword`).
    `call_sites.rs` (shared by inlay hints and call hierarchy) gained a
    matching arm walking each element the same way it already walks
    function-call arguments
  - 1 new lexer test (`recognizes_braces`) + 3 new parser tests
    (`parses_brace_list_array_initializer`,
    `parses_multi_dimensional_brace_list_array_initializer`,
    `parses_brace_list_array_initializer_on_a_second_comma_separated_variable`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate
    passes
- [x] Multi-variable `Dim`/`Public` declarations (`Dim a, b, c`) broke
  inside every nested block (bug) ✅ Resolved
  - The comma-expansion loop that turns one comma-separated declaration
    line into multiple `VarDeclaration` statements existed only in
    `Parser::parse()`'s top-level loop -- every other statement-list loop
    (`If`/`ElseIf`/`Else` branches, single-line `If`, `Select Case`,
    `For`, `Do`, `While`, `Function`, `Sub`, `#If`) had no equivalent
    handling. Confirmed at
    [Dim](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/dim.htm),
    which explicitly documents both halves of the combination that
    tripped this bug: multiple variables comma-separated on one line, and
    `Dim` variables local to their enclosing `Function`/`Sub`. Repro
    confirmed: `Dim a, b` inside a `Function`/`Sub`/`If`/`For`/`Do` body
    failed with `Unexpected token: Comma`
  - Factored the expansion into a shared `parse_statement_into` helper,
    used by every statement-list call site instead of each one calling
    `parse_statement` directly
  - 4 new parser tests (inside a `Function` body, a `Sub` body, an `If`
    block, a `For` loop, and a `Do` loop) added Red-first
- [x] `Const` illegitimately accepted comma-separated multiple constants
  on one line (over-permissive grammar) ✅ Resolved
  - The same comma-expansion loop applied uniformly to `Public`/`Dim`/
    `Const` alike, so `Const A = 1, B = 2` silently produced two valid
    `VarDeclaration` statements with no error. Campbell Scientific's own
    [Const](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/const1.htm)
    docs carry an explicit bolded NOTE: "Only one constant can be defined
    with each Const declaration. Unlike other similar languages, CRBasic
    does not allow multiple constants to be defined with one
    declaration." The inverse of the dominant bug class this project's
    comparison rounds are tuned to find via keyword-presence diffing --
    this one only surfaces from reading a docs page's fine print
  - `parse_statement_into` now returns a `ParseError` when a comma follows
    a `Const` declaration, instead of expanding into a second constant
  - 1 new parser test (`const_with_a_second_comma_separated_constant_is_a_parse_error`)
    added Red-first

Not flagged as gaps (verified during the same comparison):

- `Sample`/`Average`/`Totalize`/`Maximum`/`Minimum`/`StdDev`/`Median`/
  `WindVector`/`Covariance`/`RectPolar`/`TableFieldNames`/`SampleMaxMin`/
  `AvgRun`/`ETsz` (`DataTable`-body output-processing instructions): all
  confirmed function-shaped via official docs syntax diagrams -- already
  parse correctly via the generic `FunctionCall` grammar, part of the
  already-deferred ~126-vs-~420 `builtinFunctions` content backlog
- `Percentile`, `SampleFieldNames`: no official docs page exists for
  either name under any URL guess or search, and neither is present in
  `keywords.json` -- not real CRBasic instructions, no fabrication to
  remove

### Reference Implementation & Official Docs Comparison, Round 10 (2026-08-10)

Found during a tenth comparison round. Rounds 1-9 had already mined both
reference grammars and help.campbellsci.com to the point of diminishing
returns on grammar/keyword coverage, so this round instead audited (a)
consistency of every AST-walking LSP feature across all `Statement`
variants, and (b) a specific gap Round 9's own "Data type completions"
entry had named in passing but never turned into its own tracked item.

- [x] `Sample()`/`Average()`-style output-processing data types (`FP2`,
  `IEEE4`, `IEEE8`, `UINT2`, `UINT4`, `Bool8`, `NSEC`) had no completion or
  hover coverage ✅ Resolved
  - Round 9's "Data type completions" entry explicitly noted this set as
    "a different position this project doesn't offer type completions
    for," but that gap was never itself tracked or acted on until now.
    Re-verified against Campbell Scientific's own
    [Data Types](https://help.campbellsci.com/crbasic/cr1000x/Content/Info/datatypes.htm)
    page: `Long`/`UINT1`/`Boolean`/`String` are valid in both this set and
    the six already-covered `As`-clause types, so only the seven listed
    above are new
  - This project's completion model has no position-sensitive filtering
    anywhere (`get_all_completions` already offers the six `As`-clause
    types unconditionally, not just after `As`), so the seven new types
    were added as an `output_processing_data_type_completions()` category
    following that same precedent, wired into `get_all_completions`
  - `hover.rs`'s existing `get_data_type_description` (scoped to the six
    `As`-clause types) gained a sibling
    `get_output_processing_data_type_description`, tried second so a
    plain identifier still correctly returns `None`
  - 4 new completion tests + 1 new hover test added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate passes
- AST-consumer completeness audit (enumerated every `Statement` variant --
  including `SelectCase`, `PreprocessorConditional`, `Alias`, `Units`,
  `ReadOnly`, `Include`, `StructureType` -- against every AST-walking LSP
  feature: `definition.rs`, `semantic_tokens.rs`, `folding.rs`,
  `symbols.rs`, `code_lens.rs`, `call_hierarchy.rs`, `completion.rs`):
  no gap found. `definition.rs`/`semantic_tokens.rs` consistently recurse
  into every statement holding a nested `Vec<Statement>` body
  (`IfStatement`, `PreprocessorConditional`, `ForLoop`, `DoLoop`,
  `SelectCase`'s per-case bodies); the apparent gaps in `symbols.rs`/
  `completion.rs` (not recursing into any of those bodies at all) are the
  already-documented, deliberate Round 2 scope decision, not a new bug
- LSP position-encoding audit: the lexer's `column` counter increments per
  Rust `char` (Unicode scalar value), which matches the LSP spec's
  required UTF-16 code-unit counting for every codepoint below U+10000.
  Only astral-plane codepoints (emoji, rare scripts) would misalign by one
  column per character -- not acted on, given CRBasic source is
  effectively ASCII/Latin-1 engineering code in practice
- `Identifier()` with zero arguments nested inside another call's
  argument list (e.g. `Sample(9,Var(),String)`'s `Var()`): confirmed via
  reading `parse_primary`'s function-call-argument loop that this already
  parses fine both standalone and nested -- no special "wildcard"
  handling needed or missing
- `TypeOf`, `ClockSet`, `ComPortIsActive`, `ResetTable`, `MenuItem`,
  `MenuPick`, `DisplayValue`, `WatchdogTimer`, `SW12`, `Battery`, `Timer`,
  `TriggerSequence`: all confirmed function-shaped via docs
- `ClockChange`: docs show `Variable = ClockChange` -- a bare
  pseudo-variable used only as an expression operand (same shape as the
  already-fine `NaN`), never as its own statement; already parses
  correctly as a plain identifier
- `Sequential` (as distinct from the already-supported `SequentialMode`):
  no docs page exists; not a real separate instruction
- `LoggerType` as a standalone construct: only ever used as an ordinary
  identifier inside expressions (e.g. `#If LoggerType = "CR1000X"`);
  already handled generically, no dedicated grammar needed
- Array-of-`StructureType` combined with fixed-length strings on the same
  declaration: not a meaningful real construct -- `As StructureTypeName`
  and `As String * N` are mutually exclusive type-annotation forms
- `crbasic-vscode-support/src/`: contains only `extension.js` (already
  dismissed in Round 1); no other source files exist in either reference
  repo
- Full fresh regex extraction of every keyword/pattern identifier from
  both reference repos' `.tmLanguage.json` files and
  `crbasic-vscode-support/snippets/crbasic.json` against `keywords.json`
  (363 unmatched names): every one resolves to either the already-deferred
  `builtinFunctions` content backlog or names individually dismissed in
  this or prior rounds (`Eqv`, `IntDv`, `Restore`,
  `BeginBurstTrigger`/`EndBurstTrigger`, `ArrayIndex`, `Status`,
  `StationName`, `Debug`, `RunProgram`, etc.)

### Reference Implementation & Official Docs Comparison, Round 11 (2026-08-10)

Found during an eleventh comparison round, this time auditing angles the
first ten hadn't covered: the file-extension-to-datalogger-model mapping's
own correctness (Rounds 1-10 only ever audited extension *coverage*, never
whether the mapping itself was right), the LSP layer's outstanding provider
gaps, and string/numeric-literal edge cases. Each finding verified against
an official help.campbellsci.com page (or, for the extension mapping,
cross-referenced against multiple independent sources since no single page
enumerates every extension) and a real repro before fixing.

- [x] `.cr1`/`.cr1x` misclassified as CR200X; `.crb` misclassified as a
  GRANITE-specific extension (bug) ✅ Resolved
  - `DataloggerModel::from_extension` (`crates/crbasic-parser/src/semantic.rs`)
    grouped `.cr1`/`.cr1x` with CR200X's 16-char profile, but confirmed via
    Campbell Scientific's own
    [Program File Extension](https://help.campbellsci.com/crbasic/landing/Content/Info/Program_File_Extension.htm)
    page and independent corroboration (a CR1000 manual's reference to
    `Default.cr1`, a CR1000X getting-started guide's `default.CR1X`, and a
    Campbell forum thread instructing `Temp.CR1`→`Temp.CR1X` when upgrading
    CR1000→CR1000X): `.cr1` is CR1000's own extension and `.cr1x` is
    CR1000X's, both in the 39-char group. Only `.cr2` is CR200(X)'s
    extension. This was also the exact example `AGENTS.md`/`CLAUDE.md`
    itself documented (`.cr1` → CR200X) -- the original assumption was
    simply wrong, not merely under-audited
  - `.crb` was mapped to a dedicated `GRANITE` model, but the same official
    page shows `.crb` is valid across CR1000/CR1000X/CR6/CR300/CR350/GRANITE
    alike -- a generic extension playing the same role `.dld` already does,
    not GRANITE-specific. Since GRANITE's validation profile was otherwise
    identical to CR6's and no dedicated extension triggers it anymore, the
    `GRANITE` variant is removed and folded into `CR6` (confirmed with the
    user rather than decided unilaterally, since it changes a documented
    enum shape)
  - Real-world impact: legitimate 17-39 char variable names in actual
    CR1000/CR1000X programs were getting false `MaxLengthExceeded` errors
  - Renamed `docs/examples/03-cr200x-length-pitfalls.CR1` to `.CR2`, since
    the whole point of that example is to trigger CR200X's diagnostics --
    the old name would silently stop demonstrating them once opened in the
    actual extension
  - Updated `AGENTS.md`/`CLAUDE.md`, `README.md`, and `docs/ARCHITECTURE.md`,
    which had either repeated the same wrong assumption or (README.md's
    case) already documented the correct mapping while the code contradicted
    it
  - Test fixtures across `semantic.rs`, `document.rs` (crbasic-lsp),
    `lib.rs` (crbasic-wasm), and `sample_files.rs` updated; two existing
    sample-file regression tests (`sample-cr1000.CR1`,
    `sample-cr1000x-series.CR1X`) had been silently exercising this exact
    bug, asserting zero errors under the wrong (CR200X) model the whole
    time -- switched to CR6
- [x] `scan_string` implemented fabricated C-style backslash escapes (`\n`,
  `\t`, `\r`, `\\`, `\"`) (bug) ✅ Resolved
  - CRBasic has no backslash-escape syntax in string literals at all --
    confirmed via a Campbell Scientific user forum thread ("Double quote in
    a string") stating explicitly that `Chr(34)` concatenation is the only
    way to embed a quote, since there is no escape mechanism, not even
    VB-style doubled quotes. `scan_string`
    (`crates/crbasic-parser/src/lexer/scanner.rs`) had no basis for this
    C-family behavior -- confirmed via repro: a literal Windows path like
    `"C:\network\path"` had its `\n` silently converted to a real newline,
    corrupting the string value
  - `scan_string` now copies every character up to the closing `"`
    verbatim; a backslash has no special meaning
  - 2 lexer tests rewritten Red-first
    (`treats_backslash_as_a_literal_character`,
    `does_not_merge_adjacent_string_literals`), replacing the two tests that
    had baked in the fabricated escape behavior as intended
- [x] `&H` (hexadecimal) / `&B` (binary) integer-literal prefixes entirely
  unhandled ✅ Resolved
  - Confirmed real at help.campbellsci.com's
    [endword parameter page](https://help.campbellsci.com/crbasic/cr6/Content/parameters/endword.htm)
    (`&H80000000`), with `&B` corroborated by both reference tmLanguage
    grammars' `constant.numeric` patterns. The lexer's `&` handling only
    ever emitted `Ampersand`/`AmpersandEqual`, so `&HFF` lexed as
    string-concatenation followed by a bare `HFF` identifier
  - The lexer now recognizes `&H`/`&h`/`&B`/`&b` as literal prefixes, but
    only when a valid digit immediately follows -- otherwise `&` stays the
    concatenation operator, so `A&Bvar` (concatenation with a variable
    starting with B or H) still lexes correctly. The parser strips the
    prefix and radix-parses the digits into the literal's `i64` value
    (new `parse_integer_literal` helper, `crates/crbasic-parser/src/parser.rs`)
  - 4 new lexer tests + 2 new parser tests added Red-first
- [x] Truncation-collision diagnostics never populated `related_information`
  (a loose end noted but not acted on in the Codebase Survey Candidates
  section above) ✅ Resolved
  - `check_truncation_collisions` (`semantic.rs`) already grouped every
    colliding symbol (each with its own `declaration_span`) before emitting
    one `SemanticError` per member, but only ever kept that member's own
    name -- the other colliding symbol's location was computed and then
    discarded instead of being surfaced to the client
  - `SemanticErrorKind::TruncationCollision` gained a
    `colliding_with: Vec<(String, Span)>` field; `backend.rs` builds each
    diagnostic's `related_information` from it, letting editors jump
    directly to the other declaration causing the collision
  - 1 new semantic-analyzer test + 2 new backend tests added Red-first

Not flagged as gaps (verified during the same comparison):

- Remaining unimplemented LSP capabilities (`selection_range`,
  `linked_editing_range`, `type_definition`, `implementation`,
  `declaration`, `document_link`, `color`, `execute_command`, `moniker`,
  `inline_value`, the pull-model `diagnostic` request,
  `textDocument/formatting`): audited against tower-lsp-server's full
  capability surface and the LSP 3.17 spec. `type_definition`/
  `implementation`/`declaration`/`document_link`/`color`/`moniker`/
  `inline_value` don't semantically apply to CRBasic (no type hierarchy, no
  URLs/color literals in source, single-file server, no debug adapter).
  `execute_command` and the pull-model `diagnostic` request have no
  concrete need yet (push-based `publish_diagnostics` already works).
  `selection_range`/`linked_editing_range` are plausible future work -- AST
  spans and the existing `document_highlight`/`rename` identifier-occurrence
  logic could support them -- but not urgent enough to act on now.
  Whole-document `textDocument/formatting` was re-evaluated and confirmed
  still out of scope: the parser discards comments as trivia
  (`parser.rs`'s `skip_whitespace_and_comments`), and the AST has no
  comment-carrying nodes, so an AST-driven formatter would silently delete
  every comment on "Format Document" -- a real fix would need a second,
  token-based (not AST-based) code-generation pass, a bigger undertaking
  than the rest of this round
- Full CRBasic string/numeric literal edge-case sweep (line-continuation
  inside multi-line constructs, comment forms besides `'`, numeric type
  suffixes like VB-style `L`/`UL`): no further gaps found clearing this
  project's repro-plus-official-docs bar. `REM`-style comments don't exist
  in CRBasic; numeric type suffixes have no official-docs corroboration and
  resemble a copy-pasted C-family artifact in one reference grammar
- CR9000X's officially-documented OS-version-dependent length limit (16
  chars pre-2011-OS release, 39 chars after) can't be expressed by
  extension-based detection alone, since the file extension carries no OS
  version -- an inherent limitation of this project's detection strategy,
  not a bug to fix

### Reference Implementation & Official Docs Comparison, Round 12 (2026-08-10)

Found during a twelfth comparison round. Rounds 1-11 had exhausted
keyword/operator diffing against the two reference grammars, so this round
instead targeted fresh angles: numeric/string literal edge cases, an
AST-consumer walk for LSP-layer behavior (not language-grammar gaps), and a
full fabrication audit of a foundational syntax assumption never
re-examined since Phase 3. Each finding verified against an official
help.campbellsci.com page and a real parse repro before fixing.

- [x] Array element access used a fabricated `Data[0]` bracket syntax --
  the real CRBasic syntax is `Data(0)`, and the real syntax's *write* form
  was unparseable (bug) ✅ Resolved
  - This project's parser has used `[index]` bracket syntax for array
    access since Phase 3 (the very first parser implementation), never
    checked against an official source. Confirmed via multiple independent
    help.campbellsci.com pages
    ([Dim](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/dim.htm):
    `"DimArray3D(1,2,3) = count"`; [Arrays and Indexes into
    Arrays](https://help.campbellsci.com/crbasic/cr6/Content/Info/arraysandindexintoarrays.htm))
    plus a web search corroborating "the () pair must always be present" --
    CRBasic has no bracket array syntax at all, only `Name(index)`, the
    same syntax used for calls. Neither `docs/researches/` nor any ADR ever
    stated a rationale for the bracket choice
  - Real impact, not just a cosmetic mismatch: since `Name(args)` was
    already parsed uniformly as `Expression::FunctionCall` (accepted
    design, see Round 5's `StructureType` entry), array element *reads*
    already worked by accident through that path. But the real *write*
    form, `Data(0) = 5`, was never recognized by the assignment-target
    fast path (which only checked for `[`) -- confirmed via repro: it fell
    through to the generic expression parser and silently became an inert
    whole-statement *comparison* (`=` read as the comparison operator)
    instead of an assignment, the same silent-corruption bug class as
    `Next i`/`ContinueScan`/`Include` in earlier rounds. Real-world CRBasic
    programs assigning to array elements -- an extremely common
    operation -- were silently mishandled by this LSP
  - The same fast path also never recognized a `StructureType` member
    target (`CS215.Temp = 25`, deferred as a "read-only" scope decision in
    Round 5): confirmed via repro to hit the identical silent
    comparison-misparse, not merely "unsupported" as Round 5 believed
  - Fixed by factoring the postfix-chain parsing (call/array-index parens,
    member-access dots) shared by `parse_primary` and the
    assignment-target detector into one `parse_postfix_chain` helper, then
    deriving the assignment target from the parsed expression via a new
    `expression_to_assignment_target` instead of hand-rolling
    bracket-specific lookahead. Added `AssignmentTarget::Member` alongside
    the existing `Identifier`/`ArrayElement` variants
  - Removed the `[`/`]` lexer tokens and the now-unreachable
    `Expression::ArrayAccess` AST variant, since nothing produces or needs
    them once array reads and writes both go through the same `Name(args)`
    shape real CRBasic uses -- confirmed with the user before removing,
    since this is a foundational syntax change (matches the bar set by
    Round 11's `GRANITE` enum removal and Round 2's fabricated-keyword
    removals)
  - Blast radius was smaller than the fabrication's age suggested: bracket
    syntax appeared in exactly 10 test fixtures (all in
    `crbasic-parser`'s own lexer/parser test suite) and nowhere in
    `docs/sample-codes/`, `docs/examples/`, or any LSP-provider matching
    logic (those match on AST shape or token names, not source syntax) --
    rewritten to the real paren syntax rather than removed, since they
    remain valid coverage of array parsing
  - 4 new parser tests (`array_element_assignment_is_not_misparsed_as_a_comparison`,
    3 tests in a new `member_assignment_statements` module) + 1 rewritten
    lexer test (`tokenizes_array_element_assignment`, replacing
    `tokenizes_array_access`) added Red-first; full workspace
    `build`/`test`/`clippy`/`fmt` gate passes (llvm-cov: 92.89% line /
    98.05% function, no regression from the 80%/90% gate)
- [x] Unterminated string literals swallowed every following line up to
  the next stray `"` or EOF (bug) ✅ Resolved
  - `scan_string` had no line boundary, only `"` or EOF. Confirmed via
    repro: a forgotten closing quote (e.g. a mistyped Windows path) caused
    every subsequent source line -- including real code -- to be silently
    absorbed into the string's value up to wherever the next `"` happened
    to appear, corrupting the program far past the actual typo. No
    official docs page addresses this (malformed-input handling isn't a
    language-spec question), but CRBasic string literals are single-line
    by every example seen across all 12 rounds, so stopping at `\n` matches
    how virtually every BASIC dialect treats string literals
  - Fixed by stopping `scan_string` at `\n` the same way it already stops
    at `"` or EOF
  - 1 new lexer test
    (`unterminated_string_does_not_swallow_the_rest_of_the_file`) added
    Red-first

Not flagged as gaps (verified during the same comparison):

- Leading-dot (`.5`) and trailing-dot (`5.`) float literals are hard parse
  errors today (`scan_number` requires a digit before the dot, and a digit
  after it). One reference grammar's regex
  (`([0-9]+\.?[0-9]*)|(\.[0-9]+)`) would accept both forms, but no official
  help.campbellsci.com page was found describing numeric-literal grammar
  precisely enough to confirm or deny either form -- doesn't clear this
  project's own repro-plus-official-docs corroboration bar (the same bar
  Round 11 used to keep `Eqv`/`IntDv` dismissed). Left unaddressed rather
  than loosening the bar
- `Parser::parse()` has no error recovery: the first `ParseError` anywhere
  in a document aborts the entire parse, so `Document::analyze()` never
  sets `doc.ast`, and every AST-dependent LSP feature (document symbols,
  semantic tokens, folding, code lens, inlay hints, go-to-definition,
  find-references, rename, call hierarchy, and the user-defined-symbol
  portion of completion) goes dark for the *whole file* until that one
  syntax error is fixed -- only token-based features (keyword hover,
  TextMate highlighting) keep working. This is an LSP-layer UX question
  (many LSPs behave this way pre-1.0), not a CRBasic-language gap, and a
  real fix (statement-level synchronize/panic-mode recovery collecting
  multiple independent `ParseError`s per file) is a larger, deliberate
  design change rather than a same-class bug fix -- left unaddressed here,
  flagged for a future round to decide as its own scoped piece of work
- `&O` octal literal prefix: no official docs page found, and neither
  reference grammar's `constant.numeric` regex includes an octal
  alternative (only `&H`/`&B`) -- reconfirms Round 7's original dismissal
- `&H`/`&B` edge cases (invalid digit after the prefix, numeric overflow):
  read `parse_integer_literal` and the scanner's `&` handling -- both
  already handled correctly (invalid digit falls back to plain
  concatenation by design; overflow is a clean `ParseError`, not a panic)
- Colon-separated (`:`) multi-statement lines inside every block type
  added since Round 2's original fix (`StructureType` member lists,
  `ConstTable`, `ApplyAndRestartSequence`, `ShutDownBegin`/`End`,
  `DisplayMenu`, `DataTable` body): repro-verified across all of these --
  already works correctly everywhere via the shared
  `skip_whitespace_and_comments` helper
- `DataTable`-body sibling instructions `Resolution`, `Format`, `Trigger`,
  `WaitAll` are not documented instructions at all (`Trigger` is a
  *parameter* of `DataTable(...)` itself, not a body keyword);
  `FieldNames` is real, function-shaped, and already registered and
  parses correctly
- Full re-diff of both reference repos' keyword lists against
  `keywords.json`, including punctuation/case-mangled slip-throughs
  (`#`-prefixed, dotted names): every remaining unmatched token is
  TextMate scope-name/JSON-structural noise, not a CRBasic keyword
- `hover.rs`/`completion.rs` prose spot-checked against official Data
  Types and `INT, FIX` pages (Boolean = -1/0, Long = 32-bit signed, `Int`
  truncates toward -∞ vs. `Fix` toward 0, etc.): all accurate. A full
  line-by-line sweep of all ~90 hover descriptions and ~120 completion
  doc-strings (e.g. verifying every built-in function's documented
  parameter *order*) was time-boxed out rather than padding this round --
  flagged for a future round

### Reference Implementation & Official Docs Comparison, Round 13 (2026-08-10)

Found during a thirteenth comparison round, targeting angles not yet
systematically covered: a bare-vs-parenthesized re-audit of the full
`builtinFunctions` list (the dominant bug class across Rounds 4-8, but
never checked exhaustively in one pass), a `client/language-configuration.json`
diff against both reference repos beyond `indentationRules`/`folding.markers`
(the only two properties prior rounds ever touched), and a targeted
parameter-order spot-check of `signature.rs`/`hover.rs` (time-boxed out of
Round 12). Each finding verified against an official help.campbellsci.com
page and a real parse repro before fixing.

- [x] `CallTable` -- bare statement silently misparsed (bug) ✅ Resolved
  - `keywords.json` registered `CallTable` as a parenthesized `builtinFunctions`
    entry, but its real syntax is bare: confirmed at
    [CallTable](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/calltable.htm)
    (`Syntax: CallTable [Name]`, sole example `CallTable METDATA`, no
    parentheses anywhere on the page). Same silent-corruption bug class as
    the already-resolved `ContinueScan`/`Include`/`Data(0)=5` gaps, but on
    the single most common data-table-invocation statement in the
    language -- confirmed via repro that `CallTable Test` parsed **without
    error** but wrongly, as two dead no-op statements instead of a real
    table invocation. This project's own `docs/sample-codes/sample-cr6-series.CR6`
    already used the real bare form and was silently mishandled by it
  - Moved to `languageKeywords` (`datatable` category, alongside
    `TableHide`/`OpenInterval`/`FillStop`); new `Statement::CallTable` AST
    variant parsed the same way as `Alias`/`Units`/`ReadOnly` (bare
    keyword + `parse_primary`-parsed operand). `call_sites.rs` skips its
    operand for the same reason it already skips `Alias`/`Units`/`ReadOnly`
    (a table name, not a real call)
  - `completion.rs`'s builtin-function-style entry (and the three
    multi-statement pattern snippets that used `CallTable(${5:TableName})`)
    replaced with the real bare-keyword snippet; `signature.rs`'s
    parenthesized signature entry removed (a bare keyword has no parameter
    list to show); `hover.rs` gained a keyword-style entry
  - 3 new parser tests (`calltable_statement` module) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass
- [x] `Watch`/`Voltage` -- fabricated, not real CRBasic ✅ Resolved (removed)
  - First exhaustive bare-vs-parenthesized re-check of the full
    `builtinFunctions` list turned up two names with no official docs page
    (both 404) and no presence in either local reference grammar. `git log
    -S` traces both to the exact same pre-Round-1 hand-written
    `tmLanguage.json` line that also contained `Sqrt`/`LCase`/`UCase`/
    `TableName`/`IsNaN` -- the five fabrications Round 8's "first
    systematic fabrication audit" explicitly removed -- but these two
    survived that audit
  - Neither had any completion, hover, or signature coverage, so removal
    from `keywords.json` was the entire fix
- [x] Stale `[`/`]` bracket support in `client/language-configuration.json`
  (bug) ✅ Resolved
  - `brackets`/`autoClosingPairs`/`surroundingPairs` still listed `[`/`]`,
    but Round 12 removed CRBasic's fabricated bracket-array syntax from
    the lexer/parser entirely -- confirmed via repro that `Data[0] = 5`
    lexes today as `Identifier Integer Equal Integer` (the scanner's
    unknown-character catch-all silently drops `[`/`]`) and parses to an
    inert `0 = 5` comparison instead of an assignment or an error, the
    exact bug Round 12 fixed for `(` still live for `[`. No prior round
    had diffed these three properties against either reference repo (only
    `indentationRules`/`folding.markers` were ever compared)
  - Dropped `[`/`]` from all three arrays; added the first Vitest coverage
    for them (`client/src/language-configuration.test.ts`), since none
    existed before this fix

Not flagged as gaps (verified during the same comparison):

- Full bare-vs-parenthesized sweep of the current 119 `builtinFunctions`
  entries (regex-filtered for Mode/Begin/End/Hide/Stop/Trigger/Break/
  Reset-suffixed names, the pattern every prior bare-keyword miss shared,
  plus a fresh official-docs re-fetch for `WorstCase`, `DataEvent`,
  `EncryptExempt`, `Erase`, `Randomize`, `Broadcast`, `ClockReport`,
  `NewFieldNames`, `ArrayLength`, `MoveBytes`, `CardOut`, `NewFile`,
  `IIf`, `RealTime`, `PortSet`, `PulsePort`, `SetStatus`, `DataInterval`):
  all confirmed genuinely always-parenthesized. `CallTable` above was the
  only miscategorized entry remaining
- `client/language-configuration.json`'s `comments` (no `blockComment` --
  correct, confirmed no CRBasic block-comment syntax exists; both
  reference repos' `/* */` block-comment entries are a copy-paste
  artifact, same family as their already-dismissed `//` line-comment
  error) and `wordPattern` (undefined here and in both references; VS
  Code's built-in default already matches this project's real
  `[A-Za-z_][A-Za-z0-9_]*` identifier grammar, confirmed by reading
  `scan_identifier` directly) checked and found correct as-is
  - the `"` autoClosingPair's `notIn: ["string"]` guard is also correct;
    neither reference repo's erroneous `'`/`'` autoClosingPair (treating
    the comment marker as a quote character) was copied here, and rightly
    so
- `signature.rs` parameter-order spot-check against official syntax
  diagrams for `Scan`, `Sample`, `Average`, `Minimum`, `Maximum`,
  `DataTable`, and `SerialOpen`: all match exactly. `WindVector`/
  `TCDiff`/`Resistance`/`SDI12Recorder` have zero `signature.rs`/`hover.rs`
  coverage at all -- already part of the Round 2 Codebase Survey
  Candidates' named backlog (`Resistance` explicitly), not a new gap
- Parser error recovery (Round 12's deferred design question): no new
  official-docs-confirmed CRBasic mechanism found that categorically
  worsens the already-flagged blast radius. `Include` (this LSP's only
  real file-splitting primitive) remaining unresolved, combined with
  Campbell's own example programs conventionally being single monolithic
  files, corroborates that "whole file goes dark on one typo" is the
  common case for real deployed programs -- not a new mechanism, just
  supporting evidence that prioritizing this isn't premature. Left as a
  scoping decision for a future round, not acted on here

### Reference Implementation & Official Docs Comparison, Round 14 (2026-08-10)

Found during a fourteenth comparison round, checking angles the prior
thirteen rounds hadn't yet touched directly: both reference repos'
`snippets/`, `package.json` `contributes` blocks (commands, keybindings,
categories, language `extensions`), and `src/` implementation files (not
just their grammars/language-configuration), plus a recheck of the
Round 2/13-flagged `signature.rs`/`hover.rs` coverage gap. No new bug-class
gap was found; this round's only change is a documentation correction.

- Verified, not a gap: `crbasic-vscode-support`'s 11 snippets (`Public`,
  `Const`, `DataTable`, `ForLoop`, `IfThenElse`, `Subroutine`, `CallTable`,
  `ScanLoop`, `EndProgram`, `Variable`, `Comment`) are all single-statement
  patterns already covered (in richer, multi-statement form for several)
  by this project's existing keyword snippets and the `ScanLoop`/
  `DataTableSample` pattern snippets from the Snippet library entry above
- Verified, not a gap: both reference repos' `package.json` `contributes`
  blocks checked line-by-line. `crbasic-vscode-support`'s `commands`/
  `keybindings` (`extension.importCRB`, `extension.openPC400`) are the
  same thin Windows-only PC400 wrappers Round 1 already declined to port.
  Both repos' supported file `extensions` (`.cr1`/`.cr1x`/`.cr2`/`.cr3`/
  `.cr300`/`.cr5`/`.cr6`/`.cr8`/`.cr9`/`.cr9x`/`.dld`/`.crb`, and the
  smaller subset in `cr-basic-ms-vscode`) are already a strict subset of
  `client/package.json`'s and `DataloggerModel::from_extension`'s
  (`crates/crbasic-parser/src/semantic.rs`) coverage -- nothing missing.
  `cr-basic-ms-vscode`'s `categories: ["Formatters", "Testing"]` has no
  corresponding formatter/test-runner contribution anywhere in that
  repo -- inaccurate marketplace metadata on their side, not a feature to
  match
- Verified, not a gap: `crbasic-vscode-support/src/extension.js` (its only
  `src/` file) is entirely the same PC400/import command logic already
  covered above; `cr-basic-ms-vscode` has no `src/` directory at all
  (grammar-only extension)
- [x] `WindVector`/`TCDiff`/`Resistance`/`SDI12Recorder` missing
  `signature.rs`/`hover.rs` coverage (Round 2/13 backlog) ✅ Resolved
  - Re-verified against current code before fixing: `TCDiff`, `Resistance`,
    and `SDI12Recorder` were registered `BUILTIN_FUNCTIONS` entries
    (`crates/crbasic-parser/keywords.json`) with zero `signature.rs`/
    `hover.rs` coverage, matching the backlog description exactly.
    `WindVector`, however, wasn't registered *anywhere* in the codebase --
    a strictly bigger gap than "missing signature/hover" (no completion,
    hover, syntax highlighting, or signature help at all), not caught by
    Round 2/13 because neither round grepped for its absence, only assumed
    it was a registered name like the other three
  - Added `WindVector` to `keywords.json` (`data` category, alongside
    `Average`/`Maximum`/`Totalize` -- the other output-processing
    instructions) and regenerated `keywords_generated.rs` and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `node scripts/generate-grammar.js`
  - Added all four to `signature.rs::get_function_signature` and a new
    `hover.rs::get_measurement_function_description` (scoped to exactly
    these four, the same narrow-set pattern `get_data_type_description`
    already uses), with parameter names/order verified against each
    instruction's own syntax diagram on help.campbellsci.com (`tcdiff.htm`,
    `resistance.htm`, `sdi12recorder.htm`, `windvector.htm`)
  - Found and fixed a real bug while writing `TCDiff`'s signature: its
    existing `completion.rs` snippet labeled the 9th placeholder `Integ`,
    but Campbell Scientific's syntax diagram names it `fN1` (the sinc
    filter's first notch frequency) -- a different parameter from the
    `Integ` used by `VoltSe`/`VoltDiff`, not a renaming of the same one
  - No completion snippet was added for `WindVector` itself (only 1 of the
    now-117 `BUILTIN_FUNCTIONS` entries has a real per-parameter
    completion snippet) -- still the deliberately deferred content-volume
    decision from the Codebase Survey Candidates backlog, unaffected by
    this fix
  - 4 new `has_measurement_functions` assertions + 4 new parameter-order
    tests (`signature.rs`) + 1 new regression test
    (`tcdiff_ninth_placeholder_is_the_notch_filter_frequency_not_integration`,
    `completion.rs`) + 4 new hover tests (`hover.rs`) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate and
    `node scripts/generate-grammar.js --check` pass
- Documentation correction (no code change): the Preprocessor directive
  support entry above claimed `Include` was "entirely unsupported by this
  project" -- true when that entry was written, but Round 8's `Include`
  fix (this file, Reference Implementation & Official Docs Comparison,
  Round 8) already resolved its structural parsing. Corrected in place;
  cross-file resolution of the included file's symbols remains the actual
  open gap, matching `workspaceSymbolProvider`/`callHierarchyProvider`'s
  already-accepted open-documents-only scope boundary

### Reference Implementation & Official Docs Comparison, Round 15 (2026-08-10)

Found while turning Round 13's 7-function signature.rs spot check into a
full sweep: of the ~33 functions `signature.rs` covers, only those 7
(Scan, Sample, Average, Minimum, Maximum, DataTable, SerialOpen) plus
Round 14's 4 (WindVector, TCDiff, Resistance, SDI12Recorder) had ever been
checked against help.campbellsci.com's syntax diagrams. The remaining 22
were verified here (2 parallel research passes), turning up the same
TCDiff-style mislabeling bug in six more functions. Each finding
re-verified directly against the official docs before fixing.

- [x] `Delay`, `InStr`, `SplitStr`, `TimeIntoInterval`, `VoltDiff`,
  `VoltSe` -- six parameter-list mismatches in `signature.rs`/
  `completion.rs` (bug) ✅ Resolved
  - `Delay` was missing its required leading `Option` parameter (0/1/2,
    selecting whether the pause affects the measurement task sequence,
    processing, or digital/SDM measurements) -- confirmed at
    [delay3.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/delay3.htm).
    Only `(Duration, Units)` existed before this fix
  - `InStr`'s 3rd and 4th parameters were both wrong: the 3rd was named
    `SearchString`, colliding with the *official* name of the 2nd
    parameter (the string being searched *in*) -- the real 3rd parameter
    is `FilterString` (the string being searched *for*). The 4th was
    named/documented as a boolean `CaseSensitive (0/1)`, but official
    `SearchOption` (confirmed at
    [instr.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/instr.htm)
    and
    [searchoption.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/parameters/searchoption.htm))
    is a 0-10 method-of-search code (plus +100 for quote-stripping), not
    a simple flag -- a semantic mismatch, not just a rename
  - `SplitStr`'s 3rd parameter was named `Delimiter`, underselling its
    documented role: per
    [splitstr.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/splitstr.htm),
    `FilterString`'s actual behavior (delimiter set, exact-match string,
    or header/footer filter) depends on `SplitOption`
  - `TimeIntoInterval` was missing the leading `TintoInt` parameter that
    its documented alias `IfTime` already had correctly -- confirmed at
    [timeintointervaliftime.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/timeintointervaliftime.htm):
    "This instruction is also known as IfTime. Either keyword can be used
    within the program," with identical 3-parameter syntax
  - `VoltDiff` and `VoltSe` both used `Integ` for their 7th parameter --
    the exact same mislabeling already fixed for `TCDiff`/`Resistance` in
    prior rounds, just not applied here; the real parameter is `fN1` (the
    sinc filter's first notch frequency), confirmed at
    [voltdiff.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voltdiff.htm)
    and
    [voltse.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voltse.htm).
    `VoltSe`'s 5th parameter was also misspelled `MeasOfs` (official:
    `MeasOff`)
  - 6 new parser-side tests (`signature.rs`: one per function, checking
    full parameter-name order) + 6 new snippet tests (`completion.rs`:
    one per function, checking the corrected placeholder names) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate passes.
    No `hover.rs` coverage exists for any of these six functions, so
    nothing to fix there

Not flagged as gaps (verified during the same comparison):

- Every other unchecked function (`Abs`, `Atn2`, `Cos`, `IfTime`, `Left`,
  `Len`, `Mid`, `PulseCount`, `Right`, `Round`, `SerialIn`, `SerialOut`,
  `Sin`, `Sqr`, `Tan`, `Timer`) confirmed to match the official parameter
  names/order/count exactly (naming-only differences, e.g. `Value` vs.
  the official `number`, are not a bug per the standing precedent from
  the `TCDiff` fix -- only flagged when a *different* parameter is
  substituted or misdescribed)

### Reference Implementation & Official Docs Comparison, Round 16 (2026-08-10)

Found while extending Round 15's `signature.rs` parameter-order sweep to
`completion.rs`'s own per-parameter snippets: `completion.rs` has 52
functions with a real (non-generic) snippet, but only 33 of those overlap
with `signature.rs`'s already-fully-audited set, leaving 23 functions whose
snippets had never been checked against official syntax diagrams at all
(`signature.rs`'s Round 15 sweep covered a different subset). Audited via
two parallel research passes, matching Round 15's methodology. Each finding
verified against an official source before fixing.

- [x] `Therm107`/`Therm108`/`Therm109` -- missing `SettlingTime`/`Integ`
  parameters, truncated to 6 of 8 parameters (bug) ✅ Resolved
  - `completion.rs`'s snippets were
    `Therm10X(Dest, Reps, SEChan, Excite, Mult, Offset)` -- 6 parameters.
    The real, documented syntax is `Therm10X(Dest, Reps, SEChan, Excite,
    SettlingTime, Integ, Mult, Offset)` -- 8 parameters, confirmed via
    Campbell Scientific's Model 107/108/109 instruction manuals
    (`s.campbellsci.com/documents/us/manuals/10{7,8,9}.pdf`) and the CR6
    Measurement and Control System manual; no dedicated
    help.campbellsci.com page exists for these three (confirmed 404), so
    the PDF manuals are the authoritative source here, same as how earlier
    rounds treated PDF-only instructions
    - Same mislabeled/missing-parameter bug class as the already-resolved
    `TCDiff`/`VoltDiff`/`VoltSe` `Integ`→`fN1` fixes (Rounds 14-15), but a
    parameter *omission* rather than a rename -- silently dropping
    `SettlingTime`/`Integ` also misplaced `Mult`/`Offset` into the wrong
    argument positions for anyone using the snippet as a template
  - 1 new completion test
    (`therm10x_functions_include_settling_time_and_integration_parameters`,
    table-driven across all three functions) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate passes

Not flagged as gaps (verified during the same comparison):

- The other 22 of the 23 previously-unaudited `completion.rs`-only
  functions (`SubScan`, `Totalize`, `StdDev`, `SerialClose`, `Exp`, `Log10`,
  `Int`, `Fix`, `Asin`, `Acos`, `Atn`, `Replace`, `Trim`, `LTrim`, `RTrim`,
  `UpperCase`, `LowerCase`, `FormatFloat`, `RealTime`) all confirmed to
  match their official parameter names/order/count exactly. `RealTime`
  and `FormatFloat` got extra scrutiny (a single-parameter array-fill
  instruction and a printf-style format string, respectively, both
  confirmed genuinely correct as single/two-parameter shapes, not
  under-specified)

### Reference Implementation & Official Docs Comparison, Round 17 (2026-08-10)

Found during a seventeenth comparison round, tackling the two items Round 12
and Round 13 had explicitly deferred (a full line-by-line accuracy sweep of
`hover.rs`'s ~70 keyword descriptions and `completion.rs`'s keyword snippet
text) plus a fresh sweep for gaps the prior sixteen rounds hadn't caught,
using three parallel research passes. Each finding independently
re-verified against help.campbellsci.com before fixing (not just taken from
the research pass's own citation).

- [x] `Mod` hover text claimed "integer division" (bug) ✅ Resolved
  - `Mod`'s operands can be any number, not just integers -- confirmed at
    [mod.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/mod.htm),
    whose own worked example (`19 MOD 6.7` = `5.6`) uses a non-integer
    operand and produces a non-integer result. Reworded to describe it as
    the remainder of `A / B` with that example
- [x] `ApplyAndRestartSequence` placement/semantics wrong in both `hover.rs`
  and `completion.rs` (bug) ✅ Resolved
  - Both files claimed it's "placed before `ConstTable`" and passively
    "validates a field before it is applied at runtime". The official
    example
    ([applyandrestartsequence.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/applyandrestartsequence.htm))
    declares it *after* the `ConstTable`/`EndConstTable` block it applies
    to (it needs to reference the table's already-declared fields), and
    it's arbitrary code that runs when the table's `ApplyAndRestart`
    setting is externally set (e.g. via `SetSetting`) -- the block itself
    is what triggers the restart, not a passive gate in front of one
  - Corrected the ordering, wording, and worked example in both files
- [x] `ConstTable`'s second parameter named `Enabled` instead of `Hidden`,
  with inverted meaning (bug) ✅ Resolved
  - Confirmed at
    [consttableendconsttable.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/consttableendconsttable.htm):
    the real parameter is `Hidden` (1 = visible only at the highest
    security level, 0/omitted = standard visible table) -- a user filling
    in `Enabled = True` per the old placeholder name would have requested
    the opposite of what "Enabled" implies. Fixed in both `hover.rs`'s
    `consttable`/`applyandrestartsequence` examples and `completion.rs`'s
    `ConstTable` snippet
- [x] `FillStop` hover claimed it must be "placed immediately after the
  DataTable statement" (bug) ✅ Resolved
  - The official example
    ([fillstop.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/fillstop.htm))
    places it after `DataInterval(...)`, not immediately after `DataTable`;
    no such ordering rule exists in the docs. Reworded to "used within the
    DataTable declaration" (`completion.rs`'s `FillStop` detail had no such
    claim, so nothing to fix there)
- [x] `ExitScan` completion detail read identically to `ContinueScan`'s,
  losing the loop-exit-vs-iteration-skip distinction (bug) ✅ Resolved
  - `ExitScan` breaks out of the entire `Scan`/`NextScan` loop regardless of
    `Count`; `ContinueScan` only skips to the next iteration. Confirmed at
    [scannextscan.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/scannextscan.htm),
    which explicitly contrasts the two. Reworded `ExitScan`'s detail to name
    the loop-exit behavior explicitly
  - 4 new `hover.rs` tests + 3 new `completion.rs` tests (one per finding
    above, `documentation_accuracy` submodules) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] `Asc`, `DecToHex`, `FreqCount`, `Thermocouple`, `Variance`, `UDPClose`
  -- fabricated, not real CRBasic (bug) ✅ Resolved (removed)
  - Same fabrication bar Round 9 used for `Sqrt`/`LCase`/`UCase`/
    `TableName`/`IsNaN`: absent from both reference repos' grammars and
    404 on help.campbellsci.com's otherwise-reliable slug pattern (verified
    working for every real function confirmed below). Each has a real
    counterpart this project was missing entirely: `ASCII`
    ([ascii.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/ascii.htm)),
    `Hex`
    ([hex.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/hex.htm),
    also `HexToDec`'s own page names `Hex` as its inverse, not
    `DecToHex`), real frequency measurement via the already-present
    `PulseCount`, real thermocouple measurement via the already-present
    `TCDiff`/`TCSE`, `Moment` (deferred -- falls in the existing
    output-processing content-volume backlog), and the newer
    `UDPSocketOpen`/`UDPSocketSend`/`UDPSocketRecv`/`UDPSocketClose` family
    (confirmed at
    [udpsocketopen.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/udpsocketopen.htm);
    `UDPOpen`'s own page confirms that older API has no `UDPClose`
    counterpart at all)
  - None of the six had any `completion.rs`/`hover.rs`/`signature.rs`
    coverage, so removal was a pure `keywords.json` edit
- [x] `ASCII`, `Hex`, `Sinh`, `Cosh`, `Tanh`, `Frac`, `Sprintf`,
  `UDPSocketOpen`/`UDPSocketSend`/`UDPSocketRecv`/`UDPSocketClose` missing
  entirely ✅ Resolved
  - `Sinh`/`Cosh`/`Tanh`/`Frac` are direct siblings of the already-present
    `Sin`/`Cos`/`Tan`/`Sqr` (confirmed at
    [sinh.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/sinh.htm)
    and equivalent `cosh`/`tanh`/`frac` pages). `Sprintf` is a
    general-purpose formatted-string function (confirmed at
    [sprintf.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/sprintf.htm)),
    the same string-building use case Round 3 cited as the motivation for
    the `&` concatenation operator
  - Added to `keywords.json` (`string`/`math`/`communication` categories
    matching their siblings) and regenerated `keywords_generated.rs` and
    `client/syntaxes/crbasic.tmLanguage.json`. Per Round 2's Codebase
    Survey Candidates' standing content-volume decision, no
    `completion.rs`/`hover.rs`/`signature.rs` entries were added for these
    (matches how most of the ~117 `BUILTIN_FUNCTIONS` entries already have
    no rich snippet)
- [x] `AngleDegrees` missing entirely, and its absence hid a real parser
  bug (bug) ✅ Resolved
  - Confirmed at
    [angledegrees.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/angledegrees.htm):
    a bare declaration (no parentheses/arguments), placed before
    `BeginProg`, that switches `ATN`/`ATN2`/`ACOS`/`ASIN`/`RectPolar` to
    return degrees and `COS`/`TAN`/`SIN` to interpret their arguments as
    degrees
  - Because it wasn't registered anywhere, it lexed as a plain identifier
    and silently parsed as an inert `Statement::Expression` instead of a
    real declaration or a parse error -- the same "bare keyword becomes a
    phantom statement" bug class already fixed 8+ times in Rounds 4-7 for
    `Restart`/`PreserveVariables`/`SequentialMode`/`PipeLineMode`/etc, just
    never caught because it wasn't in either reference grammar's name list
    those rounds diffed against
  - Added to `keywords.json`'s `program` category and to the bare-keyword
    dispatch list in `parse_program_structure`'s caller
    (`crates/crbasic-parser/src/parser.rs`), alongside matching
    `hover.rs`/`completion.rs` coverage
  - 1 new parser test (`parses_angledegrees_before_beginprog`) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass

Not flagged as gaps (verified during the same comparison):

- `Optional`'s completion snippet example (`Function Scale(a, Optional b)`
  with no default value) -- `optional.htm`'s fetched content was
  self-contradictory about whether a default value is required in the
  declaration; left unflagged per this project's own standard of only
  fixing claims confirmed wrong, not claims merely unverifiable
- `TableHide`'s "immediately after the DataTable statement" claim -- unlike
  `FillStop`, its official example does place it as the very first
  statement in the block, so this one is not contradicted
- The remaining ~65 `hover.rs`/`completion.rs` keyword entries (`If`/
  `Then`/`Else`/`Do`/`Loop`/`While`/`Until`/`Select`/`Case`/`Public`/`Dim`/
  `Const`/`BeginProg`/`Function`/`Sub`/`Return`/`AND`/`OR`/`NOT`/`XOR`/`#If`/
  `#IfDef`/etc, plus every already-passing `completion.rs` snippet from
  `If` through `True`/`False`): each claim checked against official docs or
  a reference repo, no further discrepancy found
- `Randomize` and `TypeOf`: real CRBasic functions, but already explicitly
  swept and deliberately deferred to the content-volume backlog in Rounds 7
  and 10 respectively -- not a new finding
- `Case Else`, data types after `As`, `Eqv`/`Imp`/`IntDv`, `RectPolar` and
  the rest of the `Sample`/`Average` output-processing family,
  `ArrayIndex`, `Status`, `StationName`, `NaN`, `EndStructureType`: all
  explicitly checked and already resolved/dismissed in Rounds 1-16

### Reference Implementation & Official Docs Comparison, Round 18 (2026-08-10)

Found while cross-checking `keywords.json` against this project's own
`hover.rs`/`completion.rs` prose for instruction names mentioned in
description text but never independently verified to be registered
themselves -- an angle not covered by the keyword-name diffing Rounds 1-17
used. Each finding verified directly against help.campbellsci.com before
fixing.

- [x] CRBasic Custom Menus instruction family (`DisplayMenu`, `SubMenu`,
  `MenuItem`, `MenuPick`, `MenuRecompile`, `DisplayValue`, `DisplayLine`)
  and `SetSetting` entirely absent from `keywords.json` (bug) ✅ Resolved
  - Same "advertised via completion/hover, silently missing" bug class as
    the already-resolved `Mod`/`ElseIf`/`Select Case` gaps, but one level
    removed: this project's own `hover.rs` text for `EndMenu` says it
    "Terminates a `DisplayMenu` block" and for `EndSubMenu` says it
    terminates a "`SubMenu` block", and `ApplyAndRestartSequence`'s/
    `ConstTable`'s hover and completion text both cite `SetSetting` as the
    mechanism that triggers a restart -- yet none of `DisplayMenu`,
    `SubMenu`, `MenuItem`, `MenuPick`, `MenuRecompile`, `DisplayValue`,
    `DisplayLine`, or `SetSetting` were themselves present in
    `keywords.json` at all, so a reader following the hover text's own
    cross-references would find no syntax highlighting, completion, or
    hover for the very instructions being described
  - Confirmed real and independently verified via help.campbellsci.com for
    each:
    [displaymenuendmenu.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/displaymenuendmenu.htm)
    (`DisplayMenu`/`SubMenu` nest, e.g. `DisplayMenu("DataView",-1) ...
    SubMenu("PanelTemps") ... EndSubMenu ... EndMenu`),
    [menuitem.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/menuitem.htm),
    [menupick.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/menupick.htm),
    [menurecompile.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/menurecompile.htm),
    [displayvalue.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/displayvalue.htm),
    [displayline.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/displayline.htm),
    and `SetSetting` documented alongside the already-registered
    `SetStatus` on the same
    [setstatussetsetting.htm](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/setstatussetsetting.htm)
    page
  - Not a parser bug: all eight already parse correctly as ordinary
    function-call syntax (confirmed by the pre-existing `DisplayMenu`/
    `SubMenu`/`EndSubMenu`/`EndMenu` parser tests), since CRBasic's
    function-call grammar doesn't require pre-registration -- only the
    bare, parenthesis-less `EndMenu`/`EndSubMenu` closing keywords needed
    that, and both were already registered. The actual gap was purely
    `keywords.json`'s completion/highlighting/hover coverage
  - Added the seven Custom Menu instructions under a new `menu`
    `builtinFunctions` category (paralleling the existing `menu`
    `languageKeywords` category `EndMenu`/`EndSubMenu` already use) and a
    matching `support.function.menu.crbasic` TextMate section in
    `scripts/generate-grammar.js`; `SetSetting` joins its documented
    sibling `SetStatus` under the existing `time` category. Per this
    project's standing content-volume decision (Round 2), no
    `completion.rs`/`hover.rs`/`signature.rs` snippets were authored for
    any of the eight -- consistent with how most `BUILTIN_FUNCTIONS`
    entries already have no rich snippet
  - 2 new tests
    (`builtin_functions_include_custom_menu_entries`,
    `builtin_functions_include_set_setting_entry`, `keywords.rs`) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate and
    client `lint`/`format:check`/`test` gate pass;
    `node scripts/generate-grammar.js --check` confirms the regenerated
    `keywords_generated.rs`/`crbasic.tmLanguage.json` are committed

Not flagged as gaps (verified during the same comparison):

- `SetSettings` (plural) and `SetSecurity`: real, documented CRBasic
  instructions confirmed on the same official page family as
  `SetSetting`/`SetStatus`, but neither is referenced by this project's
  own hover/completion prose the way `SetSetting` was -- their absence
  from `keywords.json` falls under the already-accepted, deliberately
  deferred content-volume backlog (Round 2's ~420-vs-126 name-count
  decision), not this round's narrower "internally cross-referenced but
  unregistered" bug class

### Reference Implementation & Official Docs Comparison, Round 19 (2026-08-10)

Found during a nineteenth comparison round, this time auditing angles not yet
covered: whether every lexer/parser-supported syntax construct has matching
`client/language-configuration.json` bracket-pairing config (Rounds 1-18 only
ever diffed `indentationRules`/`folding.markers`/the `[`/`]` removal against
this file, never audited it for a *missing* addition), a TextMate-grammar
punctuation-scoping cross-check, and a fresh look for any AST expression
variant not reached by the LSP-layer call-site/hint walkers introduced across
prior rounds. Rounds 1-18 had already exhausted keyword/operator diffing
against both reference grammars (TextMate-only extensions with no semantic
layer of their own), so this round targeted editor-config completeness
instead.

- [x] `{`/`}` (brace-list array initializer syntax, added in Round 9) had no
  `brackets`/`autoClosingPairs`/`surroundingPairs` entries in
  `client/language-configuration.json` (bug) ✅ Resolved
  - Round 9 added `LeftBrace`/`RightBrace` lexer tokens and
    `Expression::ArrayLiteral` so `Public MyArray(3) = {3, 6, 9}` parses
    correctly, but the editor-level bracket-pairing config was never
    updated to match -- typing `{` didn't auto-insert its closing `}`, and
    VSCode's bracket-matching/surround-selection features didn't recognize
    the pair at all. The inverse of Round 13's "stale `[`/`]` bracket
    support" finding (an addition that should have shipped alongside a
    parser change, but didn't, rather than a removal that should have but
    didn't happen)
  - Added `["{", "}"]` to `brackets`/`surroundingPairs` and
    `{ "open": "{", "close": "}" }` to `autoClosingPairs`
  - Confirmed no matching gap in `client/syntaxes/crbasic.tmLanguage.json`:
    neither `(`/`)` nor `{`/`}` are given their own TextMate punctuation
    scope in this grammar (bracket-pair colorization and matching are
    already handled entirely by `language-configuration.json`'s `brackets`
    field), so no grammar change was needed for consistency
  - Confirmed no matching gap in the LSP layer: `call_sites.rs` already
    walks into `Expression::ArrayLiteral`'s elements (added alongside the
    Round 9 fix), so nested function calls inside a brace-list initializer
    already get inlay hints/call-hierarchy coverage
  - 3 new Vitest cases (`client/src/language-configuration.test.ts`) added
    Red-first; client `lint`/`format:check`/`test` gate passes

Not flagged as gaps (verified during the same comparison):

- Full re-audit of every `keywords.json` entry cross-referenced by another
  entry's `hover.rs`/`completion.rs` prose (Round 18's bug class): no further
  instance found beyond the Custom Menu family/`SetSetting` already fixed
- `docs/researches/` contains only the already-fully-mined `research-001`;
  `docs/sample-codes/`'s 10 fixture files are project-authored regression
  fixtures (already exercised by `tests/sample_files.rs`), not an
  independent real-world corpus, so re-running them was not expected to
  (and did not) surface new gaps
- File-extension-to-model mapping (`DataloggerModel::from_extension`)
  cross-checked once more against `client/package.json`'s full `.cr1`/
  `.cr1x`/`.cr2`/`.cr3`/`.cr5`/`.cr6`/`.cr8`/`.cr9`/`.cr9x`/`.c9x`/`.cr300`/
  `.crb`/`.dld` extension list: every extension already has a matching,
  tested `from_extension` arm; no gap since Round 11's fix

### Reference Implementation & Official Docs Comparison, Round 20 (2026-08-10)

Found during a twentieth comparison round, following up directly on Round
19's discovery that `client/language-configuration.json` can drift out of
sync with the lexer/parser without anyone noticing. This round audited the
other hand-maintained highlighting surface with the same risk --
`scripts/generate-grammar.js`'s hard-coded `operators`/`numbers`/`strings`
blocks in `crbasic.tmLanguage.json`'s codegen, which (unlike the rest of the
grammar) are **not** sourced from `keywords.json` and so never benefited
from any of Rounds 1-18's keyword-list diffing. Each finding verified by
reading the current lexer/parser source directly (the ground truth these
blocks are supposed to mirror), not by re-fetching official docs already
cited when each operator/literal was originally implemented.

- [x] `&`, `\`, `<<`, `>>`, `@`, `!`, `Imp`, and every compound assignment
  operator (`+=`, `-=`, `*=`, `/=`, `^=`, `&=`, `\=`) rendered as unstyled
  plain text; `Mod` matched only the literal uppercase spelling with no word
  boundary (bug) ✅ Resolved
  - `generate-grammar.js`'s `operators` object is hand-written, not derived
    from `keywords.json` -- confirmed by reading the codegen: `languageKeywords`
    entries categorized `logical` (`AND`, `OR`, `NOT`, `XOR`, `MOD`, `IMP`)
    are the *only* language-keyword category with no corresponding
    `kw.get("logical")` call anywhere in the script (every other category --
    `scan`, `preprocessor`, `control`, `declaration`, `program`, `datatable`,
    `consttable`, `structuretype`, `menu`, `function` -- is wired in), so
    these six keywords depend entirely on the separate hand-written block
    ever being kept current. It wasn't: every operator added in Round 3
    (`&`, `\`, `<<`/`>>`, compound assignment, `@`/`!`) and Round 3's later
    `Imp` addition shipped with zero matching grammar update. `Mod` was
    present but broken -- filed under the arithmetic pattern as a bare,
    case-sensitive, non-word-bounded `MOD` alternative instead of the
    logical pattern's `(?i)\b...\b` treatment `AND`/`OR`/`NOT`/`XOR` already
    got, so the `Mod`/`mod` spelling used in real code (and in this
    project's own hover/completion examples) never highlighted at all
  - Also fixed, found while rewriting the block: the comparison pattern's
    alternation order (`=|<>|<|>|<=|>=`) tried the single-character `<`/`>`
    alternatives before their two-character `<=`/`>=`/`<>` counterparts, so
    a real `<=`/`>=` matched as two separate single-character tokens instead
    of one -- reordered to try the longer alternatives first. Removed the
    trailing bare `=` assignment pattern as confirmed-unreachable dead code
    (the comparison pattern, listed earlier in the array, already consumes
    every `=` first)
  - New dedicated patterns added ahead of comparison/arithmetic in the
    array (so two-character operators win the position tie instead of being
    split by an earlier-listed single-character alternative):
    `keyword.operator.assignment.compound.crbasic`,
    `keyword.operator.bitwise.crbasic` (`<<`/`>>`),
    `keyword.operator.string.crbasic` (`&`),
    `keyword.operator.pointer.crbasic` (`@`/`!`); `MOD`/`IMP` moved into
    `keyword.operator.logical.crbasic` alongside `AND`/`OR`/`NOT`/`XOR`
  - New `client/src/syntax-highlighting.test.ts` (project's first Vitest
    coverage of `crbasic.tmLanguage.json`'s patterns -- no prior test file
    touched this grammar at all): loads the generated JSON directly and
    exercises each pattern as a real JS `RegExp`, stripping TextMate's
    Oniguruma-only inline `(?i)` prefix and applying it as the `i` flag
    instead (confirmed necessary: Node's `RegExp` throws `Invalid group` on
    a literal `(?i)` even on current Node). 20 new test cases added
    Red-first; client `lint`/`format:check`/`test` gate passes
- [x] `&H`/`&B` hexadecimal/binary integer literal prefixes (added in Round
  11) had no matching `numbers` pattern at all (bug) ✅ Resolved
  - Confirmed via repro-equivalent regex testing: `&HFF` matched no pattern
    in the `numbers` repository entry, so `&` fell through to (after the
    fix above) the new string-concatenation operator pattern, coloring a
    single hex/binary literal as an operator followed by a bare, unstyled
    identifier instead of one numeric constant -- a regression this round's
    own operator fix would otherwise have introduced by making the bare `&`
    pattern newly present and eager to match first
  - Added `constant.numeric.hex.crbasic` (`(?i)&h[0-9a-f]+\b`) and
    `constant.numeric.binary.crbasic` (`(?i)&b[01]+\b`) ahead of the
    existing float/integer patterns; `#numbers` is already included before
    `#operators` in the grammar's top-level pattern list (confirmed by
    reading the generated JSON), so the longer numeric match wins the
    position tie against the bare `&` operator, mirroring the lexer's own
    "prefix only counts with a digit immediately after" rule exactly
    (`A&Bvar` without a following digit must still highlight as
    concatenation, not a broken hex/binary literal)
  - 5 new test cases (`syntax-highlighting.test.ts`) added Red-first,
    including a regression case for the no-following-digit fallback
- [x] TextMate string-literal highlighting still modeled a backslash-escape
  mechanism CRBasic doesn't have, and let an unterminated string span past
  the end of its line (bug) ✅ Resolved
  - Round 11 fixed the real lexer (`scan_string`) to treat `\` as a plain
    character with no escape meaning, and Round 12 fixed it to stop at
    end-of-line for an unterminated string -- but `generate-grammar.js`'s
    `strings` block was never updated to match either fix: it still
    declared a `constant.character.escape.crbasic` sub-pattern matching
    `\\.`, and its `end` pattern was a bare `"` with no line-boundary
    fallback. A literal Windows path like `"C:\network\path"` still
    rendered with `\n`/`\p` colored as if they were escape sequences, and a
    forgotten closing quote would still visually swallow every following
    line up to the next stray `"` in the file -- the exact editor-vs-lexer
    divergence Round 19 named as this round's motivating risk, just not
    caught until actually auditing the `strings` block specifically
  - Removed the escape sub-pattern entirely; changed `end` from `"` to
    `"|$` so an unterminated string stops highlighting at its own line's
    end, matching `scan_string` exactly
  - 2 new test cases added Red-first, checking the pattern's *structure*
    (no `patterns` array; `end` contains a `$` alternative) rather than
    running a real tokenizer -- no `vscode-textmate`/Oniguruma engine is a
    dependency of this project, and adding one solely to test this one
    property would be a disproportionately large dependency for what it
    verifies
- Verified, not a gap: re-ran the same "is every `languageKeywords`/
  `builtinFunctions` category wired into the codegen" check that surfaced
  the orphaned `logical` category above, this time for every category in
  both lists. `logical` was the only orphaned `languageKeywords` category;
  every `builtinFunctions` category (`communication`, `data`, `logical`,
  `math`, `measurement`, `menu`, `scan`, `string`, `time`) already has a
  matching `fn.get(...)` call. No further orphaned category exists after
  this round's fix

### LSP-Layer Consumer Completeness Audit, Round 21 (2026-08-10)

Found during a twenty-first round, this time targeting an angle the twenty
keyword/grammar-diffing rounds above never covered: whether every named,
span-carrying `Statement` variant is registered by *every* LSP provider that
walks declarations, rather than diffing against the two reference
TextMate-only grammars again (which have no semantic layer and can't reveal
this class of gap). Checked by grepping every `Statement::` match arm across
`crates/crbasic-lsp/src/*.rs` and cross-referencing against the full list of
`Statement` variants in `ast.rs`, then verifying the one gap found with a
real parse before fixing -- consistent with every prior round's verify-before-fix
bar, just against this project's own code as ground truth instead of an
external grammar or official docs page.

- [x] `StructureType` declarations invisible to Go to Definition and the
  Outline view (bug) ✅ Resolved
  - `StructureType` (added in Round 5) is a named, span-carrying declaration
    exactly like `Function`/`Sub`/`Public`/`Dim`/`Const` -- but
    `definition.rs::extract_from_statement` and
    `symbols.rs::extract_symbol` both match on `Statement` variants with a
    catch-all `_ => {}`/`_ => None` arm, and neither had a `StructureType`
    case. Confirmed via a real repro: in `Dim CS215(4) As CS215Data`, go to
    definition on `CS215Data` returned nothing, and `CS215Data`'s own
    `StructureType CS215Data ... EndStructureType` block never appeared in
    the Outline view, even though clicking the same name in a `Function`
    call or `Public` declaration works correctly
  - Same "feature shipped, one consumer missed" bug class as the many
    already-resolved keyword-registration gaps (Rounds 1-20), just found at
    the LSP-provider layer instead of the lexer/parser/grammar layer
  - Added a `Struct` variant to `definition.rs`'s `SymbolKind` (its own
    internal enum, distinct from the LSP protocol's `SymbolKind`) and a
    `Statement::StructureType` arm inserting it into the definitions map.
    Rust's exhaustiveness checking then surfaced the one other consumer that
    matches on this enum: `call_hierarchy.rs`'s `item_for_definition`, which
    now explicitly keeps `Struct` (like `Variable`) out of the call
    hierarchy, since a type declaration isn't a callable symbol
  - Added a `Statement::StructureType` arm to `symbols.rs::extract_symbol`
    producing a `SymbolKind::STRUCT` `DocumentSymbol`, with each
    `StructureMember::Declaration` member as a `SymbolKind::FIELD` child --
    mirroring how `Function`/`Sub` already list their nested declarations.
    `StructureMember::Modifier` (a nested `Units`/`ReadOnly`) deliberately
    contributes no child, consistent with how the top-level `Units`/
    `ReadOnly` statements already don't get their own document symbol
    either
  - `code_lens.rs`'s "N references" lens and `crbasic-lsp/src/backend.rs`'s
    diagnostic wiring both needed no changes: both already iterate
    generically over every entry `DefinitionProvider::extract_definitions`
    returns, so `StructureType` declarations picked up a references lens
    for free once the definitions-map gap was closed
  - Every other `Statement::` match site was cross-checked against the full
    18-variant list the same way: `call_sites.rs` (shared by inlay hints and
    call hierarchy's outgoing-calls walk) already has all 18 variants;
    `folding.rs`/`semantic_tokens.rs`/`completion.rs`'s user-defined-symbol
    extraction/`workspace_symbol.rs` (which delegates to `symbols.rs`
    entirely rather than matching `Statement` itself) omit variants that
    genuinely don't need handling there (e.g. `Alias`/`Units`/`ReadOnly`
    have no nested body to fold or recurse into), each already consistent
    with a design decision documented in an earlier round -- no further gap
    found
  - 1 new `definition.rs` test (`extracts_structure_type_definitions`) + 1
    new `call_hierarchy.rs` test (`returns_none_for_a_structure_type`) + 1
    new `symbols.rs` test
    (`extracts_structure_type_symbol_with_member_children`) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt`/`coverage` gate and client
    `lint`/`format:check`/`test` gate pass

Not flagged as gaps (verified during the same audit):

- Cross-diffed `cr-basic-ms-vscode`'s 514-name keyword list (extracted to
  `syntaxes/cr-basic.tmLanguage.json.names.txt`) against this project's
  `keywords.json` in full: ~300 unmatched names, but every one checked
  individually (`Debug`, `Read`, `ArrayIndex`, and a further sample) is
  either parenthesized/optional-argument-shaped (already parses correctly
  via the generic `FunctionCall` grammar, e.g. `Debug`'s own syntax diagram
  is `Debug [(DebugSequence, HistorySize, Control, LineBreak,
  TraceHistory)]`) or a bare keyword already registered under a different
  match path -- falls entirely inside the already-accepted, deliberately
  deferred ~126-vs-~420 `builtinFunctions` content-volume backlog (Round 2),
  not a new bug class
  - `End` (a bare keyword in `cr-basic-ms-vscode`'s grammar): re-checked
    against `crbasic-vscode-support`'s grammar and found absent there,
    same single-grammar-only status Round 4 already dismissed it for; no
    new corroborating evidence surfaced this round
- `crbasic-lsp/src/hover.rs`, `references.rs`, `document_highlight.rs`, and
  `code_action.rs` have zero `Statement::` match arms by design -- each
  operates on the token stream or on diagnostics rather than walking the
  AST, confirmed by reading each file rather than assumed from the grep
  count alone

### Semantic Rule Coverage Audit, Round 22 (2026-08-10)

Found during a twenty-second round, this time targeting an angle the
twenty-one rounds above never covered: not keyword/grammar diffing or
AST-variant-vs-consumer wiring, but whether `semantic.rs`'s *rule content*
itself is missing a documented, compile-time-enforced CRBasic rule. Checked
by reading `semantic.rs` in full against Campbell Scientific's own docs for
each declaration keyword it already partially handles.

- [x] `Const` reassignment (`PI = 99` after `Const PI = 3.14`) never
  diagnosed as an error (bug) ✅ Resolved
  - Campbell Scientific's own
    [`Const` page](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/const1.htm)
    states outright: "Unlike variables, constants cannot be changed while
    the program is running." `semantic.rs` tracked `Public`-vs-`Dim` scope
    per declaration but discarded the `Const` keyword itself immediately
    after computing scope, and `analyze_statement` had no match arm for
    `Statement::Assignment` at all (it fell into the catch-all `_ => {}`),
    so an assignment was never checked against the symbol table at all.
    Confirmed via a real parse-and-analyze repro (a throwaway Cargo crate
    outside the repo) before fixing: `analyze()` returned `errors: []` for
    a program containing `Const PI = 3.14` followed by `PI = 99`
  - Added `is_const: bool` to `Symbol`, set from `keyword == "Const"` in
    `analyze_variable_declaration` (previously the keyword was used once,
    inline, only to compute `scope`, then dropped); added a new
    `SemanticErrorKind::ConstReassignment { variable_name, declared_at }`
    variant and a `Statement::Assignment` arm in `analyze_statement`
    calling a new `check_const_reassignment` helper. Only
    `AssignmentTarget::Identifier` is checked -- `Const` declarations are
    always scalar per the same docs page, so `ArrayElement`/`Member`
    targets can never refer to one
  - Rust's exhaustiveness checking surfaced the one consumer that matches
    on `SemanticErrorKind` outside `semantic.rs`:
    `backend.rs::code_action_data`, which now returns `(None, None)` for
    `ConstReassignment` (like the existing `TruncationCollision` arm) --
    reassigning a `Const` has no mechanical quick fix
  - 3 new `semantic.rs` tests (`reassigning_a_const_variable_is_a_semantic_error`,
    `assigning_to_a_public_variable_is_not_a_semantic_error`,
    `assigning_to_an_undeclared_identifier_is_not_flagged_here`) + 1 new
    `backend.rs` test (`omits_quick_fix_data_for_const_reassignment`) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt`/`coverage` gate
    passes (93.18% line / 97.98% function, both clear of the 80%/90%
    targets). Re-ran the full `cargo test --workspace` suite (including
    `docs/examples/`'s `01-getting-started.CR6` and
    `docs/sample-codes/sample-complex-realworld.CR6`, both of which declare
    `Const`s) to confirm no false positives against real `Const` usage that
    never gets reassigned

Not flagged as a gap (investigated during the same audit, corroboration too
weak to act on yet):

- Redeclaring a variable name (`Dim X As Float` followed by `Dim X As
  Long`, or `Const X = 1` followed by `Dim X As Long`) is also silently
  accepted -- `self.symbols.insert(...)` is a plain `HashMap::insert` that
  overwrites on a repeated name, with no prior-declaration check at all.
  Unlike the `Const`-reassignment gap above, no Campbell Scientific docs
  page was found stating outright that redeclaration is a compile error (a
  general web search surfaced only generic BASIC-family reasoning, not a
  help.campbellsci.com citation). It is suggestively corroborated by this
  project's own Preprocessor Directive Support entry above (`#UnDef`
  exists specifically to legally *un-declare* a `Const` so it can be
  redeclared, which only makes sense if redeclaring it unconditionally is
  normally illegal) but implementing it also risks reintroducing exactly
  the false-positive that entry deliberately designed around: `#If`/`#Else`
  branches are both always walked unconditionally (conditions are never
  evaluated), so the official docs' own idiom of declaring the *same*
  `Const` name differently in two mutually-exclusive branches would need
  branch-exclusivity tracking to avoid a false "duplicate declaration"
  diagnostic -- meaningfully more design work than a mechanical check,
  deferred rather than rushed

### Declaration-Keyword Semantic Rule Audit, Round 23 (2026-08-10)

Found while re-surveying `.connect0459/ref-repos/`'s two reference
extensions and help.campbellsci.com for gaps the prior 22 rounds hadn't
covered. Rounds 1-22 had already exhausted keyword/grammar diffing,
operator coverage, and LSP-provider/AST-consumer wiring, so this round
targeted the one remaining unswept angle: sibling constraints on the same
`Const`/`Dim` documentation pages that Round 22 (and Round 9, for the
comma-list and brace-initializer rules) had already partially mined but
not read to completion. All three gaps below were confirmed by reading
the actual parser/semantic code paths, not just by reading the docs.

- [x] `Const` without an initializer never diagnosed as an error (bug)
  ✅ Resolved
  - Campbell Scientific's own
    [`Const` page](https://help.campbellsci.com/crbasic/cr6/Content/Instructions/const1.htm)
    gives `Const ConstantName = Expression` as the syntax, with no
    "optional" language around `= Expression` (unlike `Public`/`Dim`,
    where an initializer genuinely is optional). `ast.rs`'s own doc
    comment on `VarDeclaration::initializer` already said "required for
    Const, optional for Public/Dim," but nothing enforced it:
    `parse_var_declaration` applied the same optional-`=` logic to all
    three keywords, so `Const PI` (no value) parsed successfully with a
    silently discarded `None` initializer. Confirmed via repro before
    fixing.
  - Fixed with a single check in `parse_var_declaration`
    (`crates/crbasic-parser/src/parser.rs`): after parsing the optional
    initializer, `keyword == "Const" && initializer.is_none()` returns a
    `ParseError` rather than a new AST variant or field, since this is
    purely a required-syntax check, the same class as "Expected identifier
    after variable declaration keyword" a few lines above
  - 1 new parser test (`const_without_an_initializer_is_a_parse_error`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate
    passes
- [x] `Const` type annotation accepts types Campbell Scientific's docs
  explicitly disallow for constants (bug) ✅ Resolved
  - The same `Const` page states outright: "Valid data types for
    constants are: Long..., Float..., Double, and String... Other data
    types return a compile error" -- narrower than the six-type set
    (`Float`/`Double`/`Long`/`Boolean`/`String`/`UINT1`) already validated
    for `Public`/`Dim`'s `As` clause (the "Data type completions" Future
    Enhancement above). `Boolean` and `UINT1` are valid there but not for
    `Const`. `semantic.rs` had zero type-annotation validation of any
    kind before this fix -- `type_annotation` was stored on `Symbol` but
    never checked against any allowed set -- so `Const Enabled As Boolean
    = True` was silently accepted.
  - Added a `CONST_ALLOWED_TYPES` constant and a check in
    `analyze_variable_declaration` (`crates/crbasic-parser/src/semantic.rs`):
    only runs when `keyword == "Const"` and a type annotation is present,
    matched case-insensitively (type annotations are lexed as plain
    identifiers, not keywords, so no case normalization happens upstream).
    New `SemanticErrorKind::InvalidConstType { variable_name, type_name }`
    variant; `backend.rs::code_action_data` gained a matching arm
    returning `(None, None)` (like `ConstReassignment`), since there's no
    single mechanically-correct type to suggest as a fix
  - Deliberately **not** fixed in this round: `completion.rs`'s
    `data_type_completions` still suggests all six types unconditionally
    after `Const X As`, including the two now-invalid ones. This is the
    same pre-existing, accepted limitation the "Data type completions"
    Future Enhancement above already noted for `Public`/`Dim` --
    `get_all_completions` has no position-sensitive filtering at all (it
    doesn't know what keyword or clause precedes the cursor), so scoping
    completions to the declaring keyword would be a new completion-model
    capability, not a mechanical fix
  - 5 new `semantic.rs` tests (`const_type_validation` module) covering
    both rejected types, all 4 documented-valid types (table-driven),
    case-insensitive matching, and a `Public`-with-`Boolean` control case
    confirming the new check doesn't leak, + 1 new `backend.rs` test
    (`omits_quick_fix_data_for_invalid_const_type`) added Red-first; full
    workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] Array declarations beyond CRBasic's dimension-count limit silently
  accepted (bug) ✅ Resolved
  - Campbell Scientific's
    [`Dim` page](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/dim.htm)
    states verbatim: "The maximum number of array dimensions allowed in a
    Dim statement is three. If you attempt to dimension a variable higher
    than three dimensional, an error occurs," plus a stricter sub-rule:
    "Strings can be dimensioned only up to 2 dimensions instead of the 3
    allowed for other data types." `parse_var_declaration`'s
    dimension-parsing loop pushed onto an unbounded `Vec` on every comma
    with no count check, and `analyze_statement`'s `VarDeclaration` match
    arm destructured `array_dimensions` with `..` and discarded it
    entirely -- confirmed by reading the full call path -- so `Dim
    Matrix(1,2,3,4)` and a 3-dimensional `Dim S(2,2,2) As String` both
    parsed and analyzed with zero diagnostics. Lowest priority of the
    three findings this round: multi-dimensional arrays beyond 2D are
    already uncommon in real CRBasic programs, and 4+-dimensional arrays
    are rarer still.
  - `analyze_variable_declaration` (`crates/crbasic-parser/src/semantic.rs`)
    gained an `array_dimension_count: Option<usize>` parameter (the call
    site passes `array_dimensions.as_ref().map(Vec::len)` rather than the
    full `Vec<Expression>`, since only the count is needed); checks
    `dimension_count > max_dimensions` where `max_dimensions` is 2 for a
    case-insensitive `String` type annotation, 3 otherwise. New
    `SemanticErrorKind::TooManyArrayDimensions { variable_name,
    dimension_count, max_dimensions }` variant; `backend.rs`'s
    `code_action_data` gained a matching `(None, None)` arm, same
    reasoning as the other two non-mechanically-fixable kinds above. This
    shared parsing/analysis path also feeds `StructureType` member
    declarations (Round 5), so the fix applies there too, though no
    dedicated `StructureType` test was added since the underlying check is
    identical
  - 5 new `semantic.rs` tests (`array_dimension_limits` module): a
    4-dimensional array error, a 3-dimensional array as the non-`String`
    boundary control case, a 3-dimensional `String` array error, a
    2-dimensional `String` array as its boundary control case, and a
    non-array declaration control case, + 1 new `backend.rs` test
    (`omits_quick_fix_data_for_too_many_array_dimensions`) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate passes

Not flagged as gaps (investigated during the same round):

- `signature.rs` built-in function parameter correctness (counts,
  names, ordering, optionality), statement-level grammar for
  `Alias`/`Units`/`CallTable`/`Route`/`Pipeline`, and any
  reference-grammar keyword still absent from `keywords.json` were all
  re-checked and found already exhausted by Rounds 13-21 -- no new
  findings along those angles.
- `ReadOnly`'s documented array/alias placement rule (`ReadOnly` must
  target the `Alias`, not the underlying `Public` variable, for arrays)
  is real per its own docs page, but the page never states what happens
  if violated (no "compile error" language) -- same weak-corroboration
  bar that caused Round 22 to defer the general variable-redeclaration
  check; not acted on for the same reason.

### Reference Implementation & Official Docs Comparison, Round 24 (2026-08-10)

Found while re-surveying `.connect0459/ref-repos/`'s two reference extensions
and help.campbellsci.com for gaps the prior 23 rounds hadn't covered.
Rounds 1-23 had already exhausted general keyword/grammar diffing, operator
coverage, control-flow constructs, preprocessor directives, declaration-
keyword semantic rules, and LSP-provider/AST-consumer wiring, so this round
targeted the one remaining unswept corner of `keywords.json`'s
`builtinFunctions` categories: PakBus/telemetry, voice-modem, and
web-server instruction families that earlier rounds bucketed wholesale
into the "content-volume backlog" without checking each name's actual
bare-vs-parenthesized grammar shape. All four findings below are new
instances of the same "advertised via a real docs page but silently
mis-lexed" bug class already fixed 10+ times in Rounds 1-8
(`ContinueScan`, `WaitTriggerSequence`, `EndMenu`/`EndSubMenu`,
`SequentialMode`, etc.), confirmed by actually parsing each repro before
fixing, not just by reading the docs.

- [x] `ESSVariables`, `WebPageEnd`, `EndModemHangup`, `VoiceBeg`/`EndVoice`
  entirely unregistered bare keywords (bug) ✅ Resolved
  - None of these appeared anywhere in `keywords.json`, so the lexer's
    keyword table never tokenized them; each lexed as a plain
    `Identifier` and the parser silently accepted it as an inert
    `Statement::Expression` -- or, for `ESSVariables Dim`, as a hard
    `ParseError`, since `Dim` (a real keyword) was then misread as the
    start of a new statement with no identifier after it.
  - `ESSVariables Public`/`ESSVariables Dim` (confirmed at
    [essvariables.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/essvariables.htm)):
    auto-declares ~1665 NTCIP Environmental Sensor Station variables for
    roadway-weather/DOT telemetry programs, with an optional `Public`/`Dim`
    modifier defaulting to `Public`. Repro before fixing: Campbell's own
    example (`ESSVariables Dim` immediately followed by an ordinary
    `Public BattV` declaration) failed with `Expected identifier after
    variable declaration keyword` -- a hard parse error that blocks
    semantic analysis for the entire file, not just a cosmetic gap.
  - `WebPageEnd` (confirmed at
    [webpagebeginwebpageend.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/webpagebeginwebpageend.htm))
    terminates a `WebPageBegin(WebPageName, WebPageCmd)` block (serves a
    custom web UI from the datalogger); `EndModemHangup` (confirmed at
    [modemhangupendmodemhangup.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/modemhangupendmodemhangup.htm))
    terminates a `ModemHangup(ComPort)` block; `VoiceBeg`/`EndVoice`
    (confirmed at
    [voicekey.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/voicekey.htm))
    bracket voice-modem response code as a bare pair, like
    `ShutDownBegin`/`ShutDownEnd`.
  - `WebPageBegin` and `ModemHangup` themselves were left unregistered
    deliberately: both already parse correctly today via the generic
    `Statement::FunctionCall` grammar (same as `DisplayMenu`, `Debug`,
    etc.), since any identifier followed by `(` parses as a function call
    regardless of `keywords.json` registration -- only the bare closers
    needed a fix. `folding.rs` already had precedent for pairing a
    non-keyword `FunctionCall` opener with a keyword `ProgramStructure`
    closer (`DisplayMenu`/`EndMenu`, `Scan`/`NextScan`), so
    `WebPageBegin`/`WebPageEnd` and `ModemHangup`/`EndModemHangup` reuse
    that exact mechanism instead of promoting the openers to keywords.
  - `ESSVariables`'s optional modifier is parsed as a new branch in
    `parse_program_structure` (`crates/crbasic-parser/src/parser.rs`)
    rather than via `parse_expression`, since `Public`/`Dim` are keyword
    tokens, not expression-parseable values; stored as an
    `Expression::Identifier` argument, mirroring how `#UnDef`/`Return`
    already attach a single argument to a `ProgramStructure` statement.
  - Added matching completion snippets/hover text (required to keep the
    existing `every_language_keyword_has_a_completion_item`/
    `every_language_keyword_has_hover_info` completeness tests green) and
    `client/language-configuration.json` indentation/folding-marker
    coverage for the two genuine block pairs, with matching Vitest cases
  - 6 new parser tests (`parses_webpageend_closing_a_webpagebegin_block`,
    `parses_endmodemhangup_closing_a_modemhangup_block`,
    `parses_voicebeg_endvoice_block`,
    `parses_bare_essvariables_with_no_modifier`,
    `parses_essvariables_with_public_modifier`,
    `essvariables_dim_modifier_no_longer_corrupts_the_surrounding_program`)
    - 3 new folding tests added Red-first; full workspace
    `build`/`test`/`clippy`/`fmt` and client `lint`/`format:check`/`test`
    gates pass

Not flagged as gaps (verified during the same round):

- `NULL` (appears only in `cr-basic-ms-vscode`'s `constant.language` list):
  every corroborating hit uses "null" informally to mean an empty string
  (`""`), never as a language keyword/literal -- doesn't clear this
  project's corroboration bar, the same VB-family-artifact class as the
  already-dismissed `Eqv`/`IntDv`.
- `EndBurstTrigger`/`BeginBurstTrigger`, `EndDialSequence` (parenthesized,
  already dismissed Rounds 7/8), and bare `End` (single-grammar-only) --
  re-confirmed still correctly dismissed, no new evidence.
- `GOESSetup`/`ArgosSetup`-family, `ModbusClient`, `DNPVariable`: checked
  each for the same suspicious "bare block closer" shape that surfaced
  the findings above; none exists -- they're ordinary parenthesized calls
  with no matching `End...` keyword.
- `Alias`/`Units` multi-pair grammar, scientific-notation numeric
  literals, and line continuation: re-verified against official docs and
  found to already match this project's implementation exactly.
- `StructureType` member-chain assignment through array indices
  (`CS215(1).Temp = 25`) and nested member chains: `AssignmentTarget::Member.object`
  is already a boxed `Expression`, so array-indexed and chained member
  targets already compose correctly -- no gap found.

### Reference Implementation & Official Docs Comparison, Round 25 (2026-08-10)

Found while re-surveying `.connect0459/ref-repos/`'s two reference extensions
and help.campbellsci.com for gaps the prior 24 rounds hadn't covered. One new
instance of the same "advertised via a real docs page but silently mis-lexed"
bug class fixed 10+ times in Rounds 1-8/24, confirmed by actually parsing a
repro before fixing, not just by reading the docs.

- [x] `ESSInitialize` entirely unregistered bare keyword (bug) ✅ Resolved
  - Confirmed at
    [essinitialize.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/essinitialize.htm):
    initializes the NTCIP Environmental Sensor Station SNMP agent, with two
    documented forms -- bare `ESSInitialize` or `ESSInitialize ("private,
    public")` (an optional SNMP read/write community string) -- and "should
    be placed directly after the `BeginProg` instruction." Present in both
    reference grammars, and this project's own `ESSVariables` hover/
    completion text (fixed in Round 24) already cross-references it by name,
    so a reader following that text's own reference would have found no
    matching coverage.
  - `ESSInitialize` was absent from `keywords.json`, so the bare form lexed
    as a plain `Identifier` and silently fell through to the
    assignment/expression path, corrupting into a dead
    `Statement::Expression { Identifier("ESSInitialize") }` no-op instead of
    a real declaration. Confirmed via a throwaway `cargo run --example`
    repro before fixing. The parenthesized form already parsed fine via the
    generic `Statement::FunctionCall` grammar, the same
    "only the bare closer needs a fix" shape as `WebPageBegin`/`ModemHangup`
    in Round 24.
  - Added to `keywords.json`'s `program` category and the bare-keyword
    dispatch list in `parse_program_structure`
    (`crates/crbasic-parser/src/parser.rs`); its optional parenthesized
    community-string argument reuses the same comma-separated
    expression-list parsing already shared by `DataTable`/`ConstTable`,
    rather than a new one-off branch
  - Added matching completion snippet/hover text (required to keep the
    existing `every_language_keyword_has_a_completion_item`/
    `every_language_keyword_has_hover_info` completeness tests green)
  - 2 new parser tests (`parses_bare_essinitialize_with_no_arguments`,
    `parses_essinitialize_with_community_string_argument`) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt` and client
    `lint`/`format:check`/`test` gates pass

Not flagged as gaps (verified during the same round):

- `ExitSub` (appears fused as one word in `cr-basic-ms-vscode`'s bare-keyword
  regex): confirmed this is the reference grammar's own bug -- its regex
  requires a single token with no internal whitespace, which never matches
  real source's two-token `Exit Sub` (already independently verified via two
  fetches of the official syntax diagram in Round 2). Not a gap here.
- `StationName`, `FileManage`, `TimedControl`: each confirmed parenthesized
  via their own docs pages, already parsing correctly via the generic
  `FunctionCall` grammar. Re-confirms Rounds 4/8's dismissal pattern.
- `Status` (used as `Status.FieldName`): ordinary `Expression::MemberAccess`
  on a plain identifier, already generically supported since Round 5's `Dot`
  token addition. Re-confirms Rounds 4/8/10's dismissal.
- `ArrayIndex`, `LoggerType`, `NewFieldNames`, `RunProgram`,
  `SemaphoreGet`/`SemaphoreRelease`, `TriggerSequence`, `WaitDigTrig`,
  `Delay`, `IfTime`, `IIF`: re-confirmed function-shaped/already-handled, no
  new evidence found.
- Full re-diff of `cr-basic-ms-vscode`'s ~514-name keyword list against
  `keywords.json` (295 unmatched names): every other unmatched name resolves
  to either the already-accepted content-volume `builtinFunctions` backlog
  (Round 2) or an individually-dismissed name from a prior round (`Eqv`,
  `IntDv`, `Restore`, `BeginBurstTrigger`/`EndBurstTrigger`,
  `EndDialSequence`, `NULL`, etc.).
- `docs/researches/research-001-crbasic-for-vscode.md` re-read in full for a
  statement-level construct not yet implemented: none found, consistent with
  Rounds 6/19's "already fully mined" conclusion.

### Diagnostic Position Accuracy Audit (2026-08-10)

Found while auditing `crbasic-lsp`'s diagnostic-publishing path for
consistency with the WASM layer, prompted by a fresh look at
`crates/crbasic-parser/src/semantic.rs` and its callers -- an angle the
eleven reference/keyword comparison rounds above hadn't covered, since it's
an LSP-layer bug rather than a parser-grammar gap.

- [x] Syntax error diagnostics always reported at a hardcoded `(0, 0)`
  position instead of the real error location (bug) ✅ Resolved
  - `crbasic_parser::ParseError` already carries a real `span: Span` with
    the exact source location, and `crbasic-wasm`'s `parse()`/`analyze()`
    already surface it correctly via `ErrorLocation { line, column }` --
    but `crbasic-lsp`'s `Document::analyze()` discarded it, collapsing the
    error to a plain `String` via `format!("Parse error: {:?}", e)`, and
    `backend.rs::analyze_and_publish_diagnostics` then hardcoded the
    diagnostic's `range` to `(0, 0)-(0, 0)` regardless of where the syntax
    error actually was. Confirmed via repro: a syntax error anywhere in a
    multi-line file (e.g. a bare `Public` with no variable name) was
    reported at the very first character of the document instead of its
    real line/column
  - No existing test caught this: `publishes_error_diagnostics_for_invalid_syntax`
    (`crbasic-lsp/tests/lsp_integration.rs`) only asserted that `did_open`
    doesn't panic, never inspecting the published diagnostic's range or
    message content
  - `Document::analyze()` now returns `Result<(), ParseError>` instead of
    `Result<(), String>`, preserving the span; a new
    `CRBasicLanguageServer::parse_error_to_diagnostic` builds the
    diagnostic's range from that span via the same `position_to_lsp`
    conversion already used for semantic-error diagnostics, and uses the
    error's own `message` directly instead of a debug-formatted string
  - 1 new `document.rs` test
    (`analyze_returns_the_parse_errors_own_source_location`) + 1 new
    `backend.rs` test
    (`converts_a_parse_error_to_a_diagnostic_at_its_own_source_location`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate
    passes

### Packaging Gap (discovered while designing the release workflow)

- [x] Multi-platform `.vsix` packaging ✅ Resolved
  - Documented in [ADR-004](./adrs/adr-004-multi-platform-packaging.md):
    adopted VS Code's platform-specific extension mechanism (one native
    `.vsix` per target) over a single universal package or an
    in-extension-host WASM rewrite
  - Added `client/scripts/targets.js` (VS Code target ↔ Rust triple
    mapping, the single source of truth), `copy-server.js --target
    <vscodeTarget>`, `package-vsix.js <target>|all` (runs `vsce package
    --target` per target), and `place-artifacts.js` (relocates CI-downloaded
    binaries into the `target/<triple>/release/` layout)
  - `.github/workflows/release.yml`'s single job split into
    `verify`/`build`/`package`; `build` is a 6-target matrix covering
    `linux-x64`, `linux-arm64`, `darwin-x64`, `darwin-arm64`, `win32-x64`,
    `win32-arm64` across 3 native runners (macOS and Windows each
    cross-compile their own second architecture; Linux's `aarch64` leg
    cross-compiles via `gcc-aarch64-linux-gnu`)
  - Added a `workflow_dispatch` trigger so the full build+package matrix can
    be run and inspected (as a workflow artifact) without cutting a real
    release
  - Added `client/.vscodeignore` to trim dev-only files from every package
    now that the bloat multiplies across 6 `.vsix` files instead of 1
  - **Not yet verified in real CI**: the `linux-arm64` and `win32-arm64`
    cross-compilation legs. Verified locally instead: both `darwin-x64` and
    `darwin-arm64` cross-compile correctly on this host, and the packaged
    `.vsix` for each was confirmed (via `file`) to contain only its own
    correct-architecture binary. Run the new `workflow_dispatch` trigger at
    least once before relying on this for a real tagged release
  - This remains a correctness fix within the current native-binary
    architecture, not a resolution of
    [ADR-001](./adrs/adr-001-rust-wasm-lsp-architecture.md)'s original
    "single WASM binary works everywhere" rationale (see ADR-004's
    Consequences)
- [x] Run `release.yml`'s `workflow_dispatch` trigger at least once to
  validate the `linux-arm64` and `win32-arm64` cross-compilation legs on
  real GitHub Actions runners, before the first real tagged release
  ✅ Resolved
  - Ran via `gh workflow run Release --ref main`:
    [run 31252687294](https://github.com/connect0459/crbasic-lsp-rs/actions/runs/31252687294)
  - `verify`, all 6 `build` matrix legs (including `linux-arm64` and
    `win32-arm64`), and `package` all completed successfully; `publish`
    correctly reported `skipped` since `workflow_dispatch` isn't
    `github.event_name == 'push'`
  - Downloaded artifacts confirm real output: one `server-<target>` binary
    artifact per target plus a `vsix-packages` artifact containing all 6
    packaged `.vsix` files (~9.4 MB total)

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
- [x] `selection_range`/`linked_editing_range` LSP providers ✅ Resolved
  - Picked up from the Round 12 comparison's "plausible future work" note
    (see Known Issues / Technical Debt above): both were named as
    unimplemented but supportable via existing AST spans and the
    `document_highlight`/`rename` identifier-occurrence lookup
  - `SelectionRangeProvider` (`crates/crbasic-lsp/src/selection_range.rs`)
    walks the AST's statement spans from the whole `Program` down to the
    innermost enclosing block (recursing into `If`/`For`/`Do`/`Function`/
    `Sub`/`Select Case` bodies, mirroring `FoldingRangeProvider`'s block
    traversal), then adds the identifier token under the cursor as the
    final, innermost step -- giving editors incremental "expand selection"
    support from a variable name out to the whole program
  - `Statement::span()` (already defined in `parser.rs`, exhaustively
    matching all 18 statement variants) is reused directly rather than
    duplicated, since it already covers every variant this traversal needs
  - `LinkedEditingRangeProvider`
    (`crates/crbasic-lsp/src/linked_editing_range.rs`) is a thin wrapper
    around `ReferencesProvider::find_identifier_at_position`/
    `find_all_references` (the same lookup `document_highlight`/`rename`
    already share), returning every occurrence of the symbol under the
    cursor plus a CRBasic identifier `word_pattern`
    (`^[A-Za-z_][A-Za-z0-9_]*$`) so clients constrain in-place edits to
    valid identifier characters
  - Both wired into `backend.rs` (`selection_range`/`linked_editing_range`
    handlers and matching `ServerCapabilities` entries); no client-side
    changes needed, since `vscode-languageclient` negotiates these
    automatically from the server's advertised capabilities
  - `type_definition`/`implementation`/`declaration`/`document_link`/
    `color`/`moniker`/`inline_value`/`execute_command`/pull-model
    `diagnostic`/`textDocument/formatting` remain out of scope per the
    Round 12 comparison's reasoning -- unchanged by this round
  - 11 new unit tests (6 in `selection_range.rs`, 5 in
    `linked_editing_range.rs`) + 4 new integration tests
    (`tests/lsp_integration.rs`) added Red-first, across 2 commits (one
    per provider); full workspace `build`/`test`/`clippy`/`fmt` gate and
    client `lint`/`format:check`/`test` gate pass (476 `crbasic-lsp` lib
    tests, up from 465)
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
- [x] Preprocessor directive support (`#If`/`#ElseIf`/`#Else`/`#EndIf`/`#IfDef`/`#UnDef`) ✅ Resolved
  - Found in the same external reference comparison as above: both
    reference grammars list these as conditional-compilation directives;
    this project's lexer/parser have no handling for `#`-prefixed tokens
    at all. Confirmed via repro: `#If 1` mis-lexes as plain `If` (the `#`
    is silently dropped), producing `Expected 'Then' after If condition`
  - Real semantics confirmed against Campbell Scientific's own
    [Conditional Compilation](https://help.campbellsci.com/crbasic/landing/Content/Info/conditionalcompilation.htm)
    docs (not just the two reference extensions, which only listed the
    directive names):
    - `#If`'s condition is either a `LoggerType` comparison (`#If
      LoggerType = GRANITE6`, against specific model names like CR1000,
      CR1000X, CR3000, CR800, CR300, CR6, CR5000, CR9000X, GRANITE6/9/10)
      or a `Const`-based boolean/equality check (`#If Add107 Then`, `#If
      Section = "Section_PT500_Settings"`) -- ordinary expression syntax,
      reusing the same `parse_expression` grammar as runtime `If`
    - `#IfDef <ConstName> [Then]` checks whether a `Const` of that name
      was already declared; `Then` appears optional in the official
      examples (unlike runtime `If`, which requires it)
    - `#UnDef <ConstName>` un-declares a `Const` so it can be redeclared,
      used together with `Include` (itself unsupported, see below) to
      stitch together library files that each define their own same-named
      constants
    - Official examples show the *same* `Const` name deliberately declared
      differently in each mutually-exclusive `#If`/`#Else` branch
  - Decided scope: parse structurally only, without evaluating conditions
    or selecting a branch -- consistent with how runtime `If` is already
    handled (both branches always walked, condition never evaluated).
    Precisely evaluating `LoggerType` isn't reliably possible without
    guessing: this project only tracks the coarse `DataloggerModel`
    grouping (CR200X/CR6/GRANITE) from file extension, not the specific
    model names `LoggerType` compares against
  - Checked whether "keep both branches unconditionally" would cause false
    duplicate-declaration diagnostics for the official `Const`-in-both-
    branches idiom above: it would not, since `semantic.rs` has no
    duplicate-declaration check at all today (`self.symbols` is a plain
    `HashMap` that silently overwrites on re-insertion) -- de-risks this
    design choice
  - No longer blocked: the `ElseIf` bug above is resolved, so `#ElseIf`
    can reuse the same recursive `parse_if_clause`-style chaining shape
    instead of needing its own implementation
  - `Include` (referenced by `#UnDef`'s real use case) was unsupported by
    this project at the time this entry was written -- separate, related
    gap, not addressed here. Its structural parsing was later resolved in
    Round 8 below; see Round 14's correction note
  - Implemented: lexer treats `#` as starting a single `#`-prefixed
    keyword token (`crates/crbasic-parser/src/lexer/scanner.rs`);
    `keywords.json` gained a new `preprocessor` category, and the codegen
    script's `alternation()` helper drops the leading `\b` it normally
    uses, since `\b` can't match on either side of a non-word character
    like `#`
  - New `Statement::PreprocessorConditional` AST variant (kept distinct
    from `IfStatement` since it's a genuinely different, compile-time
    construct with an optional `Then`); `#ElseIf` desugars into a nested
    `PreprocessorConditional` in `else_branch`, mirroring the `ElseIf` fix;
    `#UnDef` reuses `ProgramStructure`
  - Every statement-walking consumer (folding, semantic tokens, semantic
    analysis, definitions, call sites) gained a matching arm so
    declarations/calls nested inside a preprocessor block stay visible to
    those features instead of silently disappearing
  - `client/language-configuration.json`'s indentation and folding rules
    extended to recognize the new directives the same way `If`/`EndIf`
    already are, with matching Vitest cases added
  - New tests: 1 lexer test (`recognizes_preprocessor_directive_keywords`)
    - 6 parser tests (`parses_hash_if_without_then_keyword`,
    `parses_hash_if_with_then_keyword`,
    `parses_hash_if_elseif_else_endif_chain`, `parses_hash_ifdef_else_endif`,
    `hash_if_requires_hash_endif_to_close`, `parses_hash_undef`) added
    Red-first; full Rust (`build`/`test`/`clippy`/`fmt`) and client
    (`lint`/`format:check`/`test`) gates pass
  - Manually verified end-to-end (lex -> parse -> semantic analyze) with a
    combined repro exercising `#If`/`#ElseIf`/`#Else`/`#EndIf`, `#IfDef`,
    `#UnDef`, `Mod`, `While`/`Wend`, and `ElseIf` together: zero semantic
    errors, confirming these fixes compose correctly
- [x] Data type completions/hover after `As` (e.g. `Public x As <cursor>`) ✅ Resolved
  - Found in the same comparison: `Public x As IEEE4` already parses
    correctly today (the type annotation is captured as a generic
    identifier, not validated against a fixed list), so this was a
    completion/hover gap, not a parser gap
  - The reference extensions' combined type list (`Float`, `Double`,
    `Long`, `Boolean`, `String`, `FP2`, `IEEE4`, `IEEE8`, `UINT1`,
    `UINT2`, `UINT4`, `Bool8`, `NSEC`) turned out to conflate two
    different CRBasic concepts once checked against Campbell Scientific's
    own ["Data Types"](https://help.campbellsci.com/crbasic/cr1000x/Content/Info/datatypes.htm)
    documentation: only **six** (`Float` -- the default, `Double`, `Long`,
    `Boolean`, `String`, `UINT1`) are valid after `As` in a `Public`/`Dim`
    declaration; the rest (`FP2`, `IEEE4`, `IEEE8`, `UINT2`, `UINT4`,
    `Bool8`, `NSEC`, ...) are a separate output-processing type set valid
    only as a `Sample()`/`Average()`-style instruction argument, a
    different position this doesn't cover
  - Implemented as a new `data_type_completions()` category in
    `crates/crbasic-lsp/src/completion.rs`, wired into
    `get_all_completions`. Deliberately **not** added to
    `LANGUAGE_KEYWORDS`: the parser reads a type annotation as a plain
    identifier, and reclassifying these as lexer keywords would break
    that existing parsing
  - `hover.rs`'s token-based lookup (`get_hover_for_token`) now also
    checks `TokenKind::Identifier` against this same six-name set, scoped
    narrowly enough that ordinary variable identifiers still correctly
    return no hover (verified: the pre-existing
    `returns_none_for_identifier` test still passes unmodified)
  - 4 new tests (3 completion + 1 hover) added Red-first; full workspace
    `build`/`test`/`clippy`/`fmt` gate passes

### Codebase Survey Candidates (2026-08-09)

Found while auditing the codebase for further work once Phases 1-8 were
complete; none of these were previously tracked anywhere in this file.

- [x] LSP Semantic Tokens (`textDocument/semanticTokens`) ✅ Resolved
  - Added `SemanticTokensProvider` (`crates/crbasic-lsp/src/semantic_tokens.rs`):
    walks the AST to record where each `Public`/`Dim`/`Const` variable and
    `Function`/`Sub` is declared, then classifies every matching identifier
    token in the document as `variable` or `function`, with `declaration`,
    `readonly` (Const), and `global` (Public) modifiers
  - The declaring occurrence is identified by the first identifier token
    matching a symbol's name on its declaration's source line, rather than
    tracking a separate span per name -- CRBasic requires declaration
    before use, so this holds for every real program
  - Deliberately out of scope: function/subroutine parameters and `For`
    loop variables (no per-name span exists for either in the AST) --
    these fall back to the existing TextMate highlighting
  - Wired into `crates/crbasic-lsp/src/backend.rs`: `semantic_tokens_provider`
    capability plus the `textDocument/semanticTokens/full` handler
  - 12 unit tests (`semantic_tokens.rs`) + 1 integration test
    (`crates/crbasic-lsp/tests/lsp_integration.rs`)
- [x] Keyword/instruction list unification (`keywords.json` + codegen) ✅ Resolved
  - ADR-002 called for a shared source of truth to avoid duplicating
    keyword/builtin-function lists between
    `client/syntaxes/crbasic.tmLanguage.json` and
    `crates/crbasic-lsp/src/completion.rs::get_builtin_function_completions`;
    the two lists had already diverged independently (confirmed drift, see
    below) since it was never built
  - Added `crates/crbasic-parser/keywords.json` as the single source of
    truth (name + category, for both language keywords and built-in
    functions) and `scripts/generate-grammar.js`, which generates
    `crates/crbasic-parser/src/keywords_generated.rs` (re-exported as
    `crbasic_parser::{LANGUAGE_KEYWORDS, BUILTIN_FUNCTIONS}`) and the full
    `client/syntaxes/crbasic.tmLanguage.json` from it, with a `--check`
    flag wired into `just verify` and the CI `client` job
  - `crates/crbasic-parser/src/lexer/scanner.rs`'s hand-written keyword
    table is now sourced from `LANGUAGE_KEYWORDS` instead of maintaining
    its own copy -- same zero-allocation `eq_ignore_ascii_case` lookup,
    just generated instead of hand-written
  - `completion.rs`/`hover.rs` keep their hand-authored snippets/prose
    (deliberately not mechanically generated), but gained completeness
    tests against `LANGUAGE_KEYWORDS`/`BUILTIN_FUNCTIONS` (TDD red→green);
    `signature.rs` gained a matching casing-drift test. This surfaced and
    fixed real, pre-existing gaps: `Continue`/`Break`/`Is`/`GoTo`/
    `EndSelect` were missing from both completion and hover; `NextScan`
    was miscategorized as a built-in function completion instead of a
    keyword (removed the duplicate); `client/syntaxes/crbasic.tmLanguage.json`
    had phantom `Exit`/`Until` entries that don't exist in the parser, and
    listed `AND|OR|NOT|XOR|TRUE|FALSE` in two different repository groups
    (now only in the hand-written `operators` section, which also gained a
    `True`/`False` literal pattern so they don't lose highlighting)
  - `DataInterval` moved from the (incorrect) `DataTable`/`EndTable`
    keyword group into the built-in-function `data` category -- it isn't
    a scanner.rs keyword, just a plain instruction identifier
  - Deliberately **not** attempted: expanding
    `completion.rs::get_builtin_function_completions` to full parity with
    `BUILTIN_FUNCTIONS` (~35 highlighted-but-not-completed functions, e.g.
    `Battery`, `Resistance`, `Histogram`, `Chr`, `SetStatus` -- each needs
    real per-parameter snippets/docs authored, a separate content effort,
    not a list-unification one); `hover.rs`'s and `signature.rs`'s prose
    content otherwise untouched beyond the small keyword-gap fix above and
    the new drift-detection tests
- [x] Additional standard LSP providers not yet implemented ✅ Resolved
  (all sub-items below complete)
  - [x] `documentHighlightProvider` ✅ Resolved
    - Added `DocumentHighlightProvider`
      (`crates/crbasic-lsp/src/document_highlight.rs`), reusing
      `ReferencesProvider::find_identifier_at_position` and
      `find_all_references` rather than duplicating the identifier lookup
      and occurrence search -- a document highlight is the same query as
      Find All References, scoped to the open document instead of the
      whole workspace
      - Every occurrence is reported with `DocumentHighlightKind::TEXT`;
        CRBasic's AST has no per-token read/write distinction, so a finer
        `Read`/`Write` classification isn't available yet
      - Wired into `crates/crbasic-lsp/src/backend.rs`:
        `document_highlight_provider` capability plus the
        `textDocument/documentHighlight` handler
      - 5 unit tests (`document_highlight.rs`) + 2 integration tests
        (`crates/crbasic-lsp/tests/lsp_integration.rs`)
  - [x] `codeActionProvider` (quick fixes for diagnostics) ✅ Resolved
    - Added `SemanticErrorKind` (`crates/crbasic-parser/src/semantic.rs`):
      a structured classification alongside each `SemanticError`'s free-text
      `message`, so downstream consumers don't have to parse prose to tell
      which validation rule fired. Re-exported from the crate root
    - Added `CodeActionProvider` and `TruncateVariableNameData`
      (`crates/crbasic-lsp/src/code_action.rs`): offers a "truncate this
      variable name" quick fix for `MaxLengthExceeded` and
      `RecommendedLengthExceeded` diagnostics, reusing
      `ReferencesProvider::find_all_references` to rename every occurrence
      in one `WorkspaceEdit`
      - `TruncationCollision` diagnostics intentionally get no quick fix --
        truncating one side of a collision to an arbitrary shorter name has
        no guaranteed-correct outcome
      - The fix data (`variableName`, `targetLength`) is computed once, at
        diagnostic-publish time, and round-tripped through the standard LSP
        `Diagnostic::data`/`codeAction` `context.diagnostics` mechanism
        (`CRBasicLanguageServer::code_action_data` in `backend.rs`) instead
        of being re-derived from the diagnostic's message text
      - Wired into `backend.rs`: `code_action_provider` capability
        (scoped to `CodeActionKind::QUICKFIX`) plus the
        `textDocument/codeAction` handler
      - 8 unit tests (`code_action.rs`) + 3 unit tests covering the
        `data`/`code` embedding (`backend.rs`) + 2 integration tests
        (`crates/crbasic-lsp/tests/lsp_integration.rs`)
  - [x] `foldingRangeProvider` ✅ Resolved
    - Added `FoldingRangeProvider` (`crates/crbasic-lsp/src/folding.rs`):
      `If`/`For`/`Do`/`Function`/`Sub` statements already carry a span
      through their closing keyword (`EndIf`/`Next`/`Loop`/`EndFunction`/
      `EndSub`), so each maps directly to one folding range; the provider
      recurses into `If`'s branches and each block's body to fold nested
      statements too
    - `BeginProg`/`EndProg` and `DataTable`/`EndTable` are parsed as
      independent flat `ProgramStructure` statements rather than a single
      spanning one (see `symbols.rs`'s document symbols, which has the same
      shape), so the provider pairs them back up itself with one stack per
      keyword pair; unmatched open/close markers are silently skipped
      rather than treated as an error, since a mid-edit document is
      expected to be transiently unbalanced
    - Single-line ranges are dropped (nothing to fold)
    - Wired into `backend.rs`: `folding_range_provider` capability plus the
      `textDocument/foldingRange` handler
    - 12 unit tests (`folding.rs`) + 2 integration tests
      (`crates/crbasic-lsp/tests/lsp_integration.rs`)
    - Note: `client/language-configuration.json` already has a
      regex-based `folding.markers` fallback for editors/scenarios where
      the LSP server isn't available; this AST-based provider takes
      precedence in VSCode once the server is running, the same
      TextMate-first/LSP-refines relationship ADR-002 established for
      syntax highlighting
  - [x] `workspaceSymbolProvider` ✅ Resolved
    - Added `WorkspaceSymbolProvider` (`crates/crbasic-lsp/src/workspace_symbol.rs`):
      reuses the existing `textDocument/documentSymbol` extraction
      (`symbols::extract_document_symbols`) across every open document,
      flattening each document's nested symbol tree into flat
      `SymbolInformation` entries and attaching that document's URI, rather
      than re-walking the AST with new logic
      - Matching is a case-insensitive substring check against `query`; an
        empty query returns every symbol, matching the common "list all
        symbols" convention
      - **Scope note**: search only covers currently *open* documents, not
        every file in the project on disk -- this server has no workspace
        file-indexing infrastructure, only `DocumentManager`'s in-memory
        map of documents the client has opened. A full-project index would
        need a new file-walking/background-indexing layer, out of scope
        here
    - Added `DocumentManager::analyzed_documents()`
      (`crates/crbasic-lsp/src/document.rs`) to iterate every open
      document's URI and cached AST, needed to feed the cross-document
      search
    - Wired into `backend.rs`: `workspace_symbol_provider` capability plus
      the `workspace/symbol` handler
    - 2 unit tests (`document.rs`) + 7 unit tests (`workspace_symbol.rs`) +
      2 integration tests (`crates/crbasic-lsp/tests/lsp_integration.rs`)
  - [x] `inlayHintProvider` ✅ Resolved
    - Added `InlayHintProvider` (`crates/crbasic-lsp/src/inlay_hint.rs`):
      shows each recognized function's parameter name inline before its
      argument at a call site (e.g. `Scan(Interval:1, Units:Sec, ...)`)
      - Built-in parameter names are reused from
        `SignatureProvider::get_function_signature` (the same database
        signature help already uses) rather than a second hardcoded list;
        user-defined `Function`/`Sub` parameter names are read straight
        from their AST declaration
      - Call sites are collected by walking the full statement *and*
        expression tree (assignment values, conditions, loop bounds,
        nested call arguments, array indices), not just top-level call
        statements, so `x = Sqrt(y)` gets a hint the same as a bare
        `Scan(...)` statement
      - Arguments past the last known parameter are silently dropped
        (`Iterator::zip` truncates to the shorter side) rather than
        guessed at
      - Hints are filtered to the `range` the client requested, per the
        `textDocument/inlayHint` spec (editors typically request only the
        visible viewport)
    - Wired into `backend.rs`: `inlay_hint_provider` capability plus the
      `textDocument/inlayHint` handler
    - 9 unit tests (`inlay_hint.rs`) + 2 integration tests
      (`crates/crbasic-lsp/tests/lsp_integration.rs`)
  - [x] `codeLensProvider` ✅ Resolved
    - Added `CodeLensProvider` (`crates/crbasic-lsp/src/code_lens.rs`):
      shows a "N references" lens above every `Public`/`Dim`/`Const`
      variable and `Function`/`Sub` declaration
      - Declaration sites come from `DefinitionProvider::extract_definitions`
        and the occurrence search from `ReferencesProvider::find_all_references`
        -- both reused as-is rather than re-derived
      - The declaring occurrence is excluded from the count using the same
        "first identifier token on the declaration's source line" heuristic
        `semantic_tokens.rs` already relies on (documented there: CRBasic
        requires declaration before use, so this holds for every real
        program)
      - An unused symbol still gets a "0 references" lens rather than being
        hidden, since that's useful for spotting dead code
      - The lens's `Command` is built eagerly (`editor.action.showReferences`
        with the pre-resolved locations as arguments) instead of deferring
        to `codeLens/resolve`, since all the data needed is already on hand
        at request time
    - Wired into `backend.rs`: `code_lens_provider` capability
      (`resolve_provider: Some(false)`) plus the `textDocument/codeLens`
      handler
    - 9 unit tests (`code_lens.rs`) + 2 integration tests
      (`crates/crbasic-lsp/tests/lsp_integration.rs`)
  - [x] `callHierarchyProvider` ✅ Resolved
    - Added `crates/crbasic-lsp/src/call_sites.rs` first, as a planned
      refactor separate from the feature commit: extracted the
      "walk every statement/expression looking for function calls" logic
      `inlay_hint.rs` already had into a shared `collect_call_sites`, adding
      the one thing `inlay_hint.rs` didn't need -- tracking which named
      `Function`/`Sub` (if any) each call is made from. `inlay_hint.rs` now
      consumes the shared walker; its existing test suite passed unmodified,
      confirming no behavior change
    - Added `CallHierarchyProvider`
      (`crates/crbasic-lsp/src/call_hierarchy.rs`) implementing all three
      call hierarchy requests:
      - `prepare`: resolves the identifier under the cursor (declaration or
        reference) to its `Function`/`Sub` declaration via
        `DefinitionProvider::extract_definitions`; returns `None` for
        variables or non-identifier positions
      - `incoming_calls`: walks every open document's call sites, keeping
        only calls whose `enclosing` is `Some` (i.e. made from inside a
        named `Function`/`Sub`), grouped per caller into one entry with all
        of that caller's call-site ranges
      - `outgoing_calls`: locates the target's own body (searching every
        open document, since a call target isn't necessarily in the
        document that asked), walks it for calls, and resolves each
        distinct callee's declaration the same way -- callees that don't
        resolve to a known user-defined `Function`/`Sub` (e.g. built-ins
        like `Scan`) are skipped, since call hierarchy items need a
        navigable declaration location that built-ins don't have
      - **Scope note** (same as `workspaceSymbolProvider`): search covers
        currently open documents only, not the whole project on disk
      - **Design note**: a call made directly from the main program body
        (inside `BeginProg`/`EndProg`, not inside any named `Function`/`Sub`)
        has no enclosing callable symbol, so it's deliberately left out of
        incoming calls rather than represented with an invented
        "<module>"-style node -- matches how call hierarchy is
        conventionally defined as edges between named callable symbols
    - Wired into `backend.rs`: `call_hierarchy_provider` capability plus
      the `textDocument/prepareCallHierarchy`, `callHierarchy/incomingCalls`,
      and `callHierarchy/outgoingCalls` handlers
    - 7 unit tests (`call_sites.rs`) + 13 unit tests (`call_hierarchy.rs`) +
      2 integration tests (`crates/crbasic-lsp/tests/lsp_integration.rs`)
  - Diagnostics also never populate `related_information` (hardcoded to
    `None` in both places `backend.rs` builds a `Diagnostic`)
- [x] Rust test coverage measurement in CI ✅ Resolved
  - Added a `coverage` recipe (`justfile`) and a "Check test coverage" step
    in `.github/workflows/ci.yml`'s `rust` job, both running
    `cargo llvm-cov --workspace --fail-under-lines 80
    --fail-under-functions 90` -- the line/function targets stated above
    are now enforced, not just documented
  - Measured at the time of adding this gate: 90.11% line / 98.20%
    function, both already clear of the 80%/90% targets
  - Branch coverage is deliberately not gated: `cargo-llvm-cov --branch`
    requires a nightly toolchain (`-Z coverage-options=branch` is a
    nightly-only rustc flag), and running it locally
    (`cargo +nightly llvm-cov --workspace --branch`) segfaulted inside
    LLVM's coverage-mapping code rather than producing a report -- not
    reliable enough to gate CI on. The stated 75% branch target remains
    unverified
  - `just coverage` is also wired into `just verify`
- [x] Dependency review: `tower-lsp` ✅ Resolved
  - Documented in [ADR-005](./adrs/adr-005-tower-lsp-server-migration.md):
    migrated from `tower-lsp = "0.20"` (last released 2024-08-15, no newer
    version since) to `tower-lsp-server = "0.23"`, the `tower-lsp-community`
    organization's actively-released fork (1.4M+ downloads, updated
    2025-12-07) -- chosen over the also-considered `tower-lsp-f` personal
    fork for its stronger community-governance signal
  - Beyond the crate rename (`tower_lsp` -> `tower_lsp_server`,
    `lsp_types` -> `ls_types`), pulled in three real API changes across
    `crates/crbasic-lsp/src`: `Url` replaced by a `fluent_uri`-backed `Uri`
    type (including rewriting `Document::detect_model` to use
    `Uri::to_file_path()` + `Path::extension()` instead of string-splitting
    `uri.path()`), `#[async_trait]` dropped in favor of native `async fn`
    in traits, and `workspace/symbol` now returning
    `WorkspaceSymbolResponse` instead of a bare `Vec<SymbolInformation>`
  - Verified: `cargo build --workspace`, `cargo test --workspace` (429
    lib/integration tests, no regressions; the pre-existing local-only
    doctest linker failure noted in Phase 8 is unrelated), `cargo clippy
    --all-targets --all-features -- -D warnings`, and `cargo fmt --all
    --check` all pass
- [x] Dependency review: `thiserror` ✅ Resolved
  - Audited every crate for `thiserror::Error` usage before attempting the
    assumed `1.0`→`2.0.20` version bump: none exists anywhere in the
    workspace. All error types (`ParseError`, `SemanticErrorKind`, etc.)
    are plain hand-written structs/enums, not `thiserror`-derived
  - Removed the unused dependency from the workspace `Cargo.toml` and both
    `crbasic-parser`/`crbasic-lsp` crate manifests instead of bumping a
    version nothing calls
  - Verified: `cargo build --workspace`, `cargo test --workspace` (201 +
    32 + 143 + 3 + 4 + 27 + 18 + 1 = 429 lib/integration tests, all
    passing; the pre-existing local-only doctest linker failure is
    unrelated, see Phase 8 CI/CD note above), `cargo fmt --all --check`,
    and `cargo clippy --all-targets --all-features -- -D warnings` all
    pass

### Reference Implementation & Official Docs Comparison, Round 26 (2026-08-11)

Found while re-auditing operator *semantics* rather than mere
recognition: prior rounds hunted for missing/unparseable operators and
keywords, but had not verified that already-implemented operators bind in
the order Campbell Scientific actually documents. Both findings below were
verified directly against the raw HTML of help.campbellsci.com's Operators
page (fetched via `curl`, not just a summarizing fetch tool) after Rounds
2/8's prior claims that this page "doesn't specify shift precedence" and
"doesn't state precedence explicitly" for `Imp` turned out to be wrong --
the page has always had an explicit "Precedence/Order of Operation" table,
apparently missed by earlier passes.

- [x] Operator precedence chain contradicted the documented precedence
  table (bug) ✅ Resolved
  - [help.campbellsci.com's Operators page](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/operators1.htm)
    states, verbatim, highest-to-lowest: `(^)`, `(+ [positive], –
    [negative], NOT)`, `(*, /, INTDV, MOD)`, `(+ [addition], –
    [subtraction], string concatenation [+,&])`, `(=, <>, <, <=, >, >=,
    Is)`, `(<<, >>, And, Or, Xor, Eqv, Imp)` -- with same-tier operators
    "evaluated in the order they are written". This project's parser
    diverged in two ways, both built from generic "common BASIC-family
    convention" assumptions per the original Round 2/8 commit notes
    rather than from this table:
    - `parse_power` called `parse_unary` for its operand, making unary
      `+`/`-`/`NOT` bind *tighter* than `^` -- the opposite of the
      documented tightest-binding tier. `-2 ^ 2` parsed as `(-2) ^ 2 = 4`
      instead of the documented `-(2 ^ 2) = -4`.
    - `<<`/`>>`/`AND`/`OR`/`XOR`/`IMP` were each given their own nested
      precedence tier (shift tighter than comparison; AND tighter than
      XOR tighter than OR tighter than IMP) instead of the documented
      single flat, left-to-right tier. `x OR y AND z` parsed as `x OR (y
      AND z)` instead of the documented `(x OR y) AND z`, and `x + 1 <<
      2 = 5` parsed as `((x + 1) << 2) = 5` instead of `(x + 1) << (2 =
      5)`.
  - `Eqv` remains unimplemented and undismissed as before (Round 1): it
    appears only inside the page's illustrative logical-operator truth
    table, never as its own row in the main operator list or the
    precedence table's own text -- unlike `Imp`, which has both. Not
    acted on without that same evidentiary bar being met.
  - Fixed by swapping `parse_power`/`parse_unary`'s call order (`parse_unary`'s
    non-prefix fallthrough now goes to `parse_power`, and `parse_power`'s
    right operand recurses through `parse_unary` so a sign can still
    attach directly to an exponent, e.g. `2 ^ -3`) and collapsing the four
    nested logical functions (`parse_logical_imp`/`_or`/`_xor`/`_and`)
    plus the standalone `parse_shift` into one flat, left-associative
    `parse_shift_and_logical` tier
  - While rewriting the precedence tests to prove this red-first, found a
    second, unrelated defect that had been masking confidence in this
    exact area: roughly 30 pre-existing tests across the
    arithmetic/comparison/shift/logical/unary/parenthesized-expression
    test modules matched `if let Statement::FunctionCall { arguments, ..
    }` against inputs that are not actually function calls (e.g. `"1 +
    2"`, `"x AND y"`, `"-5"`). A bare top-level expression like this
    parses to `Statement::Expression` (or `Statement::Assignment` when it
    starts with `identifier =`), so the `if let` body silently never ran
    and the test passed without checking anything -- including, ironically,
    the very `logical_and_has_higher_precedence_than_or` /
    `imp_has_lower_precedence_than_or` /
    `shift_binds_tighter_than_comparison_but_looser_than_addition` tests
    that should have caught this precedence bug outright. Rewrote all of
    them to match the statement shape the parser actually produces, with
    an else-branch panic so a future mismatch fails loudly instead of
    vanishing again. Two tests (`parses_equality`,
    `comparison_has_higher_precedence_than_logical`) needed their source
    changed too (wrapped in a call, e.g. `Invoke(x = 5)`), since a bare
    `x = 5` is unconditionally an assignment and can never exercise `=`
    as a comparison operator at statement level.
  - **Not fully resolved**: this audit and fix was scoped to the
    expression/operator-precedence test modules only. Whether the same
    `Statement::FunctionCall`-for-a-non-call vacuous-match pattern recurs
    elsewhere across the other ~270 tests in `parser.rs`'s wider test
    suite was not checked here -- flagged for a future round rather than
    guessed at.
  - 2 new parser tests (`power_binds_tighter_than_unary_minus`,
    `power_exponent_may_carry_a_unary_sign`) + 3 renamed/rewritten
    precedence tests (`logical_operators_share_precedence_evaluated_left_to_right`,
    `implication_shares_precedence_with_or_evaluated_left_to_right`,
    `shift_shares_the_loosest_precedence_tier_with_logical_operators`)
    added/updated Red-first; full workspace `build`/`test`/`clippy`/`fmt`
    gate passes (all 303 `crbasic-parser` tests, including the ~30
    rewritten ones, now genuinely assert instead of vacuously passing)
- [x] `INTDV` keyword-form integer-division operator not implemented
  ✅ Resolved
  - Same Operators page lists `INTDV` as its own named entry in the main
    operator list (alongside `AND`/`MOD`/`NOT`/`OR`/`XOR`, all already
    implemented) and again in the precedence table's multiplicative tier
    next to `*`/`/`/`MOD` -- a keyword-form synonym for `\`, meeting the
    same evidentiary bar as the already-accepted `\`/`MOD`/`IMP`, unlike
    the still-unconfirmed `Eqv`/`IntDv`-as-VB6-copy-paste dismissal from
    Round 1 (that dismissal covered a casual mention of `IntDv`'s *name*
    without checking whether the page had a dedicated operator-list
    entry for it, which it does)
  - Registered in `keywords.json`'s `logical` category, parsed in
    `parse_multiplicative` alongside `Backslash` (same
    `BinaryOperator::IntegerDivide`, same precedence tier), with matching
    hover/completion coverage and TextMate highlighting (added to the
    hand-written `keyword.operator.logical.crbasic` regex in
    `generate-grammar.js`, alongside `AND`/`OR`/`NOT`/`XOR`/`MOD`/`IMP` --
    that regex is hand-authored rather than derived from `keywords.json`,
    per the Round 2 "Keyword/instruction list unification" note)
  - 1 new parser test (`parses_intdv_as_a_keyword_synonym_for_integer_division`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` and
    client `lint`/`format:check`/`test` gates pass

Not flagged as gaps (verified during the same round):

- Numeric literal formats (`&H`/`&B` hex/binary prefixes, scientific
  notation), string literal escape handling, fixed-length string
  declarations (`As String * N`), and `Const`'s restricted type list: all
  re-confirmed already correct against their respective docs pages: no
  new gaps.
- No new statement-level construct found in the two reference repos'
  snippet/source directories beyond what Rounds 24/25 already exhausted.

### Reference Implementation & Official Docs Comparison, Round 27 (2026-08-11)

Found while re-auditing operator *associativity* (not just tier
membership/precedence, which Round 26 already fixed) and re-verifying
Round 26's own dismissal reasoning for `Eqv` against a fresh raw-HTML
fetch of help.campbellsci.com's Operators page.

- [x] Chained `^` (power) was right-associative; documented rule says
  left-to-right ✅ Resolved
  - `parse_power`'s right operand recursed through `parse_unary` (i.e.
    right-recursive), making `2 ^ 3 ^ 2` parse as `2 ^ (3 ^ 2) = 512`.
    [operators1.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/operators1.htm)
    (re-fetched via raw `curl`, per Round 26's own lesson that a
    summarizing fetch tool misses this page's table content) lists `(^)`
    as its own sole-member precedence tier, followed by the blanket rule:
    "Operators with the same precedence are evaluated in the order they
    are written inside the expression." Since `^` is alone in its tier,
    this rule can only describe chained-`^` evaluation order --
    left-to-right, i.e. `(2 ^ 3) ^ 2 = 64` -- the same left-associative
    convention VB documents for its own `^`. This is the one sub-case
    Round 26's `-2 ^ 2` fix didn't itself test (repeated `^`, vs. `^`
    combined with unary).
  - Fixed by changing `parse_power` from a single `if` (one application)
    to a `while` loop that accumulates left-associatively, with a new
    `parse_power_exponent` helper for the right operand -- it still
    allows a unary sign to attach directly to the exponent (e.g. `2^-3`,
    the existing documented convention) but, unlike `parse_unary`, does
    not fall through to `parse_power` itself, which would have
    re-introduced right-associative chaining
  - 1 new parser test (`power_chains_left_to_right`) added Red-first;
    full workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] `Select Case`'s "Is omitted" bare-comparison form (`Case < 10`)
  was unparseable ✅ Resolved
  - [selectcase.htm](https://help.campbellsci.com/crbasic/cr1000x/Content/Instructions/selectcase.htm),
    verbatim: "The Case Is statement is a keyword that is used before a
    comparison operator ... If the Is keyword is omitted (for example,
    `Case < 10`), it is implied." `parse_case_condition_term` only
    recognized the explicit `Is <op> Expression` form; a bare leading
    comparison operator fell through to `parse_additive`, which has no
    path to consume it, producing a hard `Unexpected token` parse error
    on valid, officially documented syntax.
  - Fixed by extracting the operator-matching arm into
    `match_case_comparison_operator` and checking for it before falling
    back to plain-value/range parsing, regardless of whether `Is` was
    written; the explicit-`Is`-with-no-operator-following case still
    errors the same as before
  - 1 new parser test (`parses_case_comparison_with_implied_is`) added
    Red-first; full workspace `build`/`test`/`clippy`/`fmt` gate passes
- [x] `Eqv` logical operator: Round 26's dismissal reasoning contained a
  factual error; reassessed and implemented ✅ Resolved
  - Round 26 justified continuing to dismiss `Eqv` by claiming it
    "appears only inside the page's illustrative logical-operator truth
    table, never as its own row in the main operator list or **the
    precedence table's own text** -- unlike `Imp`, which has both." A
    fresh raw-HTML fetch of the same page shows this is false: the
    Precedence/Order of Operation section's own text reads `(<<, >>,
    And, Or, Xor, Eqv, Imp)` -- `Eqv` sits in the identical sentence and
    tier as `Imp`. It still lacks a dedicated named row in the main
    operator list (unlike `Imp`/`INTDV`, which both have one), so it
    didn't clear the *stricter* dual-bar Round 26 used for `INTDV` -- but
    it does clear the *weaker* bar Round 3 used to accept `Imp` (tier
    membership in the precedence table's own text).
  - Consulted the user given the judgment call on which bar applies;
    decided to implement `Eqv` now
  - Added `BinaryOperator::Equivalence`, parsed in
    `parse_shift_and_logical` alongside `Imp` (same precedence tier,
    same left-to-right evaluation), registered in `keywords.json`'s
    `logical` category, with matching completion/hover coverage and the
    hand-authored TextMate operator regex (`generate-grammar.js`,
    alongside `AND`/`OR`/`NOT`/`XOR`/`MOD`/`IMP`/`INTDV`)
  - 2 new parser tests (`parses_eqv_operation`,
    `equivalence_shares_precedence_with_or_evaluated_left_to_right`)
    added Red-first; full workspace `build`/`test`/`clippy`/`fmt` and
    client `lint`/`format:check`/`test` gates pass

Not flagged as gaps (verified during the same round):

- Swept all 17 remaining `if let Statement::FunctionCall { .. } = ...`
  sites in `parser.rs` that Round 26's own vacuous-match fix didn't
  itself rewrite: every one uses a genuinely function-call-shaped source
  with a panicking `else` branch -- no further instance of that bug.
- Mechanically cross-checked all `keywords.json` `languageKeywords`
  entries against `parser.rs`'s dispatch sites: zero misses, no new
  instance of the "advertised via `keywords.json` but absent from the
  parser" bug class (`Mod`/`ElseIf`/`Select Case`/`Exit` statements were
  prior instances).
- `Is` as a general/standalone comparison operator: `selectcase.htm`
  confirms `Is` is exclusively a `Case`-clause keyword, not usable as an
  ordinary binary operator in `If`/assignment expressions -- the current
  scoping of `Is`-handling to `parse_case_condition_term` only is
  correct as-is.
- Every other precedence tier (`parse_shift_and_logical`/
  `parse_comparison`/`parse_additive`/`parse_multiplicative`) already
  uses a left-to-right accumulating loop matching the documented
  "evaluated in the order they are written" rule -- `parse_power`'s
  former right-recursion was the only tier that diverged from this
  pattern.
- LSP provider capability surface unchanged since Rounds 11/21's
  exhaustive audits: no applicable LSP 3.17 provider missing.

### Builtin Function Completion Parity (2026-08-11)

Closes the scope gap the "Keyword/instruction list unification" entry
above deliberately deferred: `completion.rs::get_builtin_function_completions`
only covered 52 of `BUILTIN_FUNCTIONS`' 130 entries (78 missing, not the
"~35" estimated at the time -- that count undercounted the real gap).

- [x] Full completion-snippet parity with `BUILTIN_FUNCTIONS` ✅ Resolved
  - Added snippets for all 78 previously-uncompleted functions, grouped
    into 7 commits by category (measurement, communication, data,
    string, math, time, menu) plus a standalone one for `IIf`. Every
    parameter name and order was verified against
    help.campbellsci.com's own syntax diagram for that instruction
    (not inferred from generic BASIC-family conventions), following
    the same evidentiary bar as the Reference Implementation & Official
    Docs Comparison rounds above
  - Notable signature findings surfaced during verification:
    - `Rnd` and `NaN` take no parentheses at all (`variable = RND`,
      bare `NAN`) -- confirmed via their docs' own syntax diagrams,
      unlike every other entry in this batch
    - `FieldNames` takes exactly one comma-separated string parameter,
      not a multi-parameter list
    - `ModbusMaster` has no live doc page of its own; Campbell
      Scientific renamed it `ModbusClient` and the current page states
      the parameter list is unchanged, so that page's signature was
      used
    - `MenuPick` has an unbounded variadic item list; represented with
      two placeholders as a starting point rather than a fixed arity
    - `Resistance`, `BrHalf4W`, `BrFull6W`, `MoveBytes`, `DisplayMenu`,
      and `SubMenu` each have one documented-optional trailing
      parameter, included in the snippet per the existing project
      convention of listing the full documented parameter set (see the
      `Resistance`/`BrHalf4W`/`BrFull6W`/`PortSet` entries in Round 26's
      measurement research)
    - `EmailRelay`/`FTPClient`/`HTTPPost`/`HTTPPut` share a compound
      `NumRecs/TimeIntoInterval` parameter in Campbell's own docs;
      spelled `NumRecsOrTimeIntoInterval` in the snippet since a literal
      `/` in a placeholder name is unconventional
  - Added `every_canonical_builtin_function_has_a_completion_item`, the
    reverse of the pre-existing
    `every_builtin_function_completion_is_a_known_canonical_name` check,
    so this parity is now enforced going forward rather than a one-time
    fix
  - 79 new tests (one exact-`insert_text` assertion per function, plus
    the coverage test) added across the 8 commits; full workspace
    `build`/`test`/`clippy`/`fmt` gate passes after each

Not flagged as gaps (out of scope for this pass):

- `hover.rs` and `signature.rs` still only cover a small subset of
  `BUILTIN_FUNCTIONS` (hover: 4/130; signature help: 32/130 -- this
  entry originally said "no coverage at all" / "33", both off by a
  small amount; corrected in the next section below once actually
  re-checked against the code) -- both are separate, larger content
  efforts than the completion-snippet parity closed here, consistent
  with how the "Keyword/instruction list unification" entry already
  scoped `hover.rs`/`signature.rs` prose out of that round too.

### Hover Builtin Function Parity (2026-08-12)

Closes the `hover.rs` half of the gap the previous section deliberately
deferred. Re-checking the code before starting found the previous
section's own numbers were off: hover.rs had 4/130 covered (WindVector,
TCDiff, Resistance, SDI12Recorder), not 0, and `signature.rs` had 32/130
(33 match arms, but one -- `DataTable` -- is a statement, not a
`BUILTIN_FUNCTIONS` entry), not 33.

- [x] Full hover-text parity with `BUILTIN_FUNCTIONS` ✅ Resolved
  - Added hover coverage for the remaining 126 functions, grouped into
    8 commits by `keywords.json` category (scan, measurement,
    communication, data, string, math, time, logical+menu), each with
    its own `all_<category>_functions_have_hover_info` test
  - Descriptions reuse the already-verified one-line prose from
    `completion.rs::get_builtin_function_completions` (authored and
    checked against help.campbellsci.com during the prior "Builtin
    Function Completion Parity" round) rather than re-deriving them
    independently -- the same facts, a second surface. `Rnd`'s and
    `NaN`'s hover text each additionally note they take no
    parentheses, per that round's own signature-shape finding
  - Relocated `SDI12Recorder`'s pre-existing hover entry from
    `get_measurement_function_description` into a new
    `get_communication_function_description` -- its real
    `keywords.json` category -- instead of leaving it in the wrong
    bucket now that categories are being tracked deliberately
  - Split into one `get_<category>_function_description` helper per
    category (mirroring the pre-existing `get_measurement_function_description`
    shape) rather than one large flat match, chained through a new
    `get_builtin_function_description` dispatcher
  - Added `every_canonical_builtin_function_has_hover_info`, the
    hover.rs equivalent of completion.rs's pre-existing
    `every_canonical_builtin_function_has_a_completion_item`, so this
    parity is enforced going forward
  - **Process note**: the first two commits of this round briefly
    committed that completeness test before the remaining categories
    were filled in, leaving 2 commits in the middle of the history
    with a failing test (caught by `cargo test`, not by the
    pre-commit hooks, which don't run the Rust test suite). Fixed by
    a follow-up commit removing the premature test and restoring a
    fully green state, then re-adding it correctly in the final
    commit once every category actually had coverage -- worth
    remembering for the `signature.rs` round below: don't add a
    forward-looking completeness test until the category work it
    depends on is actually done.
  - 8 new category tests + 1 completeness test added across the 8
    commits; full workspace `build`/`test`/`clippy`/`fmt` gate passes
    (356 `crbasic-lsp` lib tests, up from 347)

### Signature Help Builtin Function Parity (2026-08-12)

Closes the `signature.rs` half of the same gap the "Builtin Function
Completion Parity" round deferred (see the corrected 4/32 counts in its
own section above). `signature.rs` needs a per-parameter description for
every function, not just a one-line summary, so unlike the `hover.rs`
round this couldn't reuse `completion.rs`'s prose directly -- 6
categories (measurement, communication, data, string, time, menu) were
researched and verified against help.campbellsci.com by parallel research
agents, each given the exact parameter names/order already fixed by
`completion.rs` and instructed to flag rather than silently rename any
mismatch found. Scan/logical (`SubScan`/`IIf`) and math (single-`Value`-
parameter trig/log/rounding functions) were low-risk enough to write
directly without dispatching a research pass.

- [x] Full per-parameter signature-help parity with `BUILTIN_FUNCTIONS`
  ✅ Resolved
  - Added signature help for the remaining 98 functions, grouped into
    9 commits (scan+logical, measurement, communication, data, string,
    math, time, menu, plus a final commit for the completeness test),
    each with its own `has_<category>_functions`/`all_remaining_<category>_functions_have_a_signature`
    test following the file's existing lighter spot-check convention
    (a handful of `is_some()` assertions per category, not an
    exhaustive list per function -- `hover.rs`'s exhaustive-list style
    is that file's own convention, not carried over here)
  - Added `every_canonical_builtin_function_has_a_signature`, the
    `signature.rs` equivalent of `completion.rs`'s/`hover.rs`'s
    existing completeness tests, in its own final commit once every
    category actually had coverage -- deliberately following the
    hover.rs round's own lesson (see its Process Note above) rather
    than repeating the premature-red-test mistake
  - `Rnd` and `NaN` take no parentheses (per the earlier "Builtin
    Function Completion Parity" round's finding); both represented
    with an empty parameter list and their own dedicated
    `_takes_no_parameters` test, mirroring `PPPClose` (zero parameters,
    confirmed via `completion.rs`'s existing verified `"PPPClose()"`
    snippet)
  - **Parameter-naming discrepancies found during research, deliberately
    not acted on**: kept every parameter name exactly as already fixed
    by `completion.rs` (cross-surface consistency with the existing,
    already-shipped completion snippets and the coming autocomplete
    experience), even where a research agent found the official docs
    use a different name for the same slot. Recorded here rather than
    silently ignored:
    - `Therm107`/`Therm108`/`Therm109`: official sensor manuals
      (`107.pdf`/`108.pdf`/`109.pdf` at s.campbellsci.com -- no live
      CRBasic Editor Help page exists for these three) name the 4th
      parameter `VxChan` (given: `Excite`) and the 6th `Integ/fN1`, a
      dual name split across older/newer datalogger models (given:
      `Integ`)
    - `ExciteV`: CR6/CR1000X docs show an undocumented-here 4th
      optional parameter, `DiffEx`, after `Delay` -- not added, since
      doing so would desync this function's arity from
      `completion.rs`'s existing 3-parameter snippet
    - `UDPSocketRecv`: official docs spell the 5th parameter
      `RemoteIPAddr` (given: `RemoteIPAdd`, missing the trailing `r`)
    - `LowerCase`/`UpperCase`/`Trim`/`RTrim`/`LTrim`/`Replace`: official
      docs use more specific names (`SourceString`, `TrimString`,
      `SearchString`/`SubString`/`ReplaceString`) than the generic
      `String`/`Find`/`ReplaceWith` already established by
      `completion.rs`
    - `MenuItem`: Campbell's own docs are internally inconsistent here
      -- the syntax line names the 2nd parameter `Variable` (matching
      what's already given), but that same page's parameter-description
      table heading calls it `MenuVariable`
  - 12 new tests added across the 9 commits; full workspace
    `build`/`test`/`clippy`/`fmt` gate passes (368 `crbasic-lsp` lib
    tests, up from 356)

### Reference Implementation & Official Docs Comparison, Round 28 (2026-08-12)

A fresh audit round (research-only, no code changes) re-checked four
angles not covered by Rounds 1-27's grammar diff: the reference
extension's snippets file and `extension.js`, `docs/researches/research-001`
cross-checked line-by-line against the current implementation, the health
of the three parity-enforcing completeness tests, and a full (not
sampled) name-by-name diff of both reference repos' function lists
against `keywords.json`'s `builtinFunctions`.

- [x] Newly-discovered builtin functions missing from `BUILTIN_FUNCTIONS`
  ✅ Resolved
  - The first three angles were clean (no new gap): the reference
    extension's 11 snippets and its 82-line `extension.js` (a `.crb`
    text-paste command plus a Windows-only `PC400.exe` launcher, no
    CRBasic language logic) had nothing not already covered;
    `research-001` was re-verified in full against the parser/semantic
    analyzer with no new discrepancy beyond the already-fixed
    While/Wend omission; all three completeness tests
    (`every_canonical_builtin_function_has_a_completion_item`/
    `_hover_info`/`_a_signature`) were confirmed non-vacuous (each
    iterates the real 130-entry `BUILTIN_FUNCTIONS`, not a hardcoded
    list)
  - The fourth angle (full name diff) found a 354-name union of
    functions in one or both reference repos absent from
    `keywords.json`. The overwhelming majority is the same
    already-acknowledged, deliberately-deferred content backlog from
    the "Builtin Function Completion Parity" round above (GOES/ARGOS,
    DNP, CDM_*/SDM* peripherals, CSAT3/LI7200/LI7700 sensors, PakBus,
    plus many garbled/typoed entries in one reference repo's data) --
    not re-actioned here
  - 9 of the 354 names were individually verified against
    help.campbellsci.com and confirmed as real, currently-undocumented,
    single-page CRBasic instructions small enough to add outright
    rather than defer: `SecsSince1990`, `WatchdogTimer`,
    `TimeIsBetween`, `PWM`, `GPS`, `Randomize`, `DewPoint`,
    `EthernetPower`, `I2COpen`. A follow-up check on `I2COpen`'s two
    companion instructions (`I2CRead`/`I2CWrite`, both undocumented on
    the same page as `I2COpen`) confirmed both independently, bringing
    the total to 11
  - Added all 11 to `keywords.json` (categorized: `WatchdogTimer`/`PWM`/
    `DewPoint` under `measurement`; `GPS`/`EthernetPower`/`I2COpen`/
    `I2CRead`/`I2CWrite` under `communication`; `Randomize` under
    `math`; `SecsSince1990`/`TimeIsBetween` under `time` -- matching the
    category of each function's closest existing sibling, e.g.
    `WatchdogTimer` alongside `PortSet` since both are Campbell's own
    "Datalogger Status/Control" doc category, which this project folds
    into `measurement`), regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 11, keeping the existing parity enforced by
    the completeness tests; every parameter name/order was taken
    directly from each function's own help.campbellsci.com syntax
    diagram (not inferred), following the same evidentiary bar as prior
    rounds
  - 20 new tests (11 exact-`insert_text` completion tests, plus
    additions to the existing category spot-check lists in
    `hover.rs`/`signature.rs`) added in a single commit (all 11
    functions share one concern -- this round's own verification pass
    -- rather than the multi-commit-per-category split the earlier,
    separately-researched parity rounds used); full workspace
    `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass (379 `crbasic-lsp` lib tests,
    up from 368)

Not flagged as gaps (out of scope for this pass):

- The remaining ~345 unmatched names from the full diff remain the same
  acknowledged, deliberately-deferred content backlog first named in
  the "Keyword/instruction list unification" round above -- each would
  need its own per-function docs verification before being added, not
  a mechanical list sync.

### Reference Implementation & Official Docs Comparison, Round 29 (2026-08-12)

Picked one category out of Round 28's ~345-name deferred backlog to close:
the CSAT3/LI-COR eddy-covariance sensor family (sonic anemometer plus
closed/open-path gas analyzers), the smallest and best-documented of the
candidate categories surveyed (PakBus, CDM_\*/SDM\*, GOES/ARGOS were the
others). Each of the 7 functions below was verified directly against
help.campbellsci.com (CSAT3, LI7200, LI7700) or, where no live help page
exists for the instrument, the official PDF manual (CSAT3B/CSAT3BMonitor in
the CSAT3B manual; EC100/EC100Configure in the IRGASON manual) -- the same
evidentiary bar as the Therm107/108/109 precedent from the signature-help
parity round above.

- [x] `CSAT3`, `CSAT3B`, `CSAT3BMonitor`, `EC100`, `EC100Configure`,
  `LI7200`, `LI7700` missing from `BUILTIN_FUNCTIONS` ✅ Resolved
  - All 7 are real, documented CRBasic instructions for controlling and
    reading SDM-connected sensors common in micrometeorology/eddy-covariance
    programs; none existed anywhere in `keywords.json`,
    `completion.rs`/`hover.rs`/`signature.rs`, or the reference extensions'
    correctly-spelled form (one reference repo's list had `CSAT3b`, a
    case-variant typo of `CSAT3B` not added as a separate entry, since
    CRBasic function names are matched case-insensitively)
  - Added all 7 to `keywords.json` under the `measurement` category
    (matching the closest existing sibling functions, e.g. `Therm107`),
    regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 7, taking every parameter name/order directly
    from each function's own official syntax line/manual page (not
    inferred), following the same evidentiary bar as prior rounds
  - `EC100Configure`'s `ConfigCmd` parameter's option table (referenced by
    the manual as "TABLE 10-4") was not itself transcribed into the
    parameter documentation -- out of scope for a syntax/parameter-name
    verification pass, consistent with how `signature.rs` documents what a
    parameter *is* rather than enumerating every valid value across this
    file
  - 7 new completion tests (one exact-`insert_text` test per function,
    following the Round 28 convention) plus additions to the existing
    `measurement` category spot-check lists in `hover.rs`/`signature.rs`
    added across 3 commits (one per LSP layer, following the file-by-file
    commit convention used since the signature-help parity round); full
    workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass (386 `crbasic-lsp` lib tests, up
    from 379)

### Reference Implementation & Official Docs Comparison, Round 30 (2026-08-12)

Picked PakBus (Campbell Scientific's proprietary datalogger-to-datalogger
networking protocol) as the next category out of Round 28's ~345-name
deferred backlog, following the same one-category-per-round approach as
Round 29's CSAT3/LI-COR work. 20 PakBus-related candidate names were found
in one reference extension's TextMate grammar; two parallel research
agents independently verified each against help.campbellsci.com before
anything was added, following the same evidentiary bar as prior rounds.

- [x] `AcceptDataRecords`, `Broadcast`, `ClockReport`, `DataGram`,
  `EncryptExempt`, `GetDataRecord`, `GetFile`, `GetVariables`,
  `PakBusClock`, `Route`, `Routes`, `SendData`, `SendFile`,
  `SendGetVariables`, `SendTableDef`, `SendVariables`, `StaticRoute`,
  `TimeUntilTransmit` missing from `BUILTIN_FUNCTIONS` ✅ Resolved
  - All 18 are real, documented CRBasic instructions/functions for
    PakBus networking (routing, clock synchronization, remote
    file/variable/data-record transfer between dataloggers); none
    existed anywhere in `keywords.json`,
    `completion.rs`/`hover.rs`/`signature.rs`
  - Two of the reference grammar's 20 candidates turned out to be
    garbled spellings of real functions rather than distinct
    instructions: `NetWorkPakBusCLock` is `PakBusClock`, and
    `SenfVariables` is `SendVariables` -- neither was added as a
    separate entry
  - One candidate, `RouteNeighbors`, could not be corroborated against
    any help.campbellsci.com page or official manual after a dedicated
    search effort; left out rather than guessed at, unlike the
    confirmed typos above where the real target was independently
    verifiable
  - `Route` (a function returning the neighbor/route address for a
    given `PakBusAddr`) is distinct from `Routes` (an instruction that
    fills an array with the full dynamic route table) -- confirmed as
    two separate, correctly-spelled instructions, not a singular/plural
    typo of each other
  - `TimeUntilTransmit` is used bare with no parentheses (an expression,
    like `Rnd`/`NaN`/the data-type case), not a zero-argument function
    call like `PPPClose()` -- confirmed against its own syntax line
  - Added all 18 to `keywords.json` under the `communication` category
    (matching the existing `GPS`/`I2COpen` PakBus-adjacent networking
    entries from Round 28), regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 18, keeping the existing parity enforced by
    all three completeness tests; every parameter name/order was taken
    directly from each function's own help.campbellsci.com syntax
    diagram (not inferred), following the same evidentiary bar as prior
    rounds. `SendGetVariables`'s own official page names its 9th
    parameter `GetVariable` in the syntax line but `GetVariables`
    (plural) in the parameter table -- kept as `GetVariable` (matching
    the syntax line), the same resolution strategy used for prior
    syntax/table naming mismatches (e.g. `MenuItem` in the
    signature-help parity round)
  - 19 new tests: 18 completion tests (one exact-`insert_text` test per
    function) and 1 signature test (`timeuntiltransmit_takes_no_parameters`,
    mirroring the existing `pppclose_takes_no_parameters` convention)
    added across 3 commits (one per LSP layer, following the Round 29
    commit convention); full workspace `build`/`test`/`clippy`/`fmt`
    gate passes (405 `crbasic-lsp` lib tests, up from 386)

Not flagged as gaps (out of scope for this pass):

- The remaining backlog from Round 28's full name diff (CDM_\*/SDM\*
  peripherals, GOES/ARGOS satellite telemetry, DNP, plus the garbled/
  typoed entries already dismissed there) remains deferred -- PakBus was
  this round's single chosen category, consistent with the
  one-category-per-round approach Round 29 established.

### Reference Implementation & Official Docs Comparison, Round 31 (2026-08-12)

Sized all three of Round 28's remaining backlog categories (CDM_\*/SDM\*
peripherals, GOES/ARGOS satellite telemetry, DNP) before picking one, using
the same "smallest and best-documented" heuristic as Round 29's choice.
CDM_\*/SDM\* sized to 40 confirmed instructions / 329 parameters (well
documented, but 2-3x larger than any prior round, plus a 4-instrument
CDM-VW300 subfamily documented only in a device manual rather than the
standard instruction reference); GOES/ARGOS sized to 12 confirmed
instructions / 48 parameters (individually well documented, but the
parameter semantics assume satellite-telemetry hardware context --
ST-20 buffer numbering, GOES transmission windows -- carrying higher risk
of subtly wrong descriptions); DNP sized to 3 confirmed instructions / 19
parameters with zero ambiguity and no parser/AST work needed. DNP was
picked as this round's category.

- [x] `DNP`, `DNPUpdate`, `DNPVariable` missing from `BUILTIN_FUNCTIONS`
  ✅ Resolved
  - All 3 are real, documented CRBasic instructions for configuring a
    datalogger as a DNP3 (Distributed Network Protocol, used in electric/
    water utility SCADA systems) outstation device; verified directly
    against their own help.campbellsci.com pages (syntax line, parameter
    table, and introductory description paragraph) rather than the
    reference grammar alone
  - `DNPUpdateZDNPVariable`, present in one reference extension's grammar,
    is a concatenation artifact of `DNPUpdate` and `DNPVariable` (the same
    grammar-scrape failure mode as `FillStopZGOESField` found while sizing
    the GOES/ARGOS alternative) -- not added as its own entry
  - None of the three is a block construct (no `End*` keyword, no nested
    body), so this needed no parser/AST changes -- purely
    `keywords.json`/completion/hover/signature-help work, unlike a
    `DataTable`-style declaration block
  - Added all 3 to `keywords.json` under the `communication` category
    (alongside the existing PakBus/GPS/I2C networking entries),
    regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 3, keeping the existing parity enforced by all
    three completeness tests; every parameter name/order was taken
    directly from each instruction's own help.campbellsci.com syntax
    diagram and parameter table (not inferred)
  - 4 new tests: 3 completion tests (one exact-`insert_text` test per
    function, following the Round 29/30 convention) added across 3
    commits (one per LSP layer, following the Round 29/30 commit
    convention); full workspace `build`/`test`/`clippy`/`fmt` gate and
    client `lint`/`format:check`/`test` gate pass (408 `crbasic-lsp` lib
    tests, up from 405)

Not flagged as gaps (out of scope for this pass):

- CDM_\*/SDM\* peripherals (40 confirmed instructions) and GOES/ARGOS
  satellite telemetry (12 confirmed instructions) remain deferred to
  future rounds per the sizing above -- each is a larger, single-category
  batch of its own, consistent with the one-category-per-round approach.

### Reference Implementation & Official Docs Comparison, Round 32 (2026-08-12)

Picked GOES/ARGOS satellite telemetry as the next category out of Round 31's
remaining backlog (CDM_\*/SDM\* peripherals, GOES/ARGOS), following the same
"smallest and best-documented" heuristic Round 29 established -- GOES/ARGOS
was the smaller of the two remaining categories. Two parallel research
agents independently verified the Argos-family and GOES-family candidates
against help.campbellsci.com before anything was added, following the same
evidentiary bar as prior rounds.

- [x] `ArgosData`, `ArgosDataRepeat`, `ArgosError`, `ArgosSetup`,
  `ArgosTransmit`, `GOESData`, `GOESField`, `GOESGPS`, `GOESSetup`,
  `GOESStatus`, `GOESTable` missing from `BUILTIN_FUNCTIONS` ✅ Resolved
  - All 11 are real, documented CRBasic instructions for configuring a
    datalogger to transmit data via the Argos or GOES satellite systems
    (common in remote environmental-monitoring stations with no
    cellular/radio coverage); none existed anywhere in `keywords.json`,
    `completion.rs`/`hover.rs`/`signature.rs`
  - `GOESCommand`, present in one reference extension's grammar, could not
    be corroborated against any help.campbellsci.com page or official PDF
    manual after a dedicated search effort -- left out rather than guessed
    at, the same treatment as `RouteNeighbors` in the PakBus round.
    `FillStopZGOESField`, present in another reference grammar, was
    independently confirmed as a concatenation artifact of `FillStop` and
    `GOESField` (the same grammar-scrape failure mode as
    `DNPUpdateZDNPVariable` in the DNP round) -- not added as its own
    entry. One reference grammar also spelled `ArgosData` as `ArgosDat`
    (missing the trailing "a") and `GOESTable` as `GoesTable`
    (lowercase-`oes`) -- both confirmed as the third-party grammar's own
    typos, not alternate official spellings
  - `ArgosError`'s official parameter table names its one parameter
    `ErrorMessage`, but the instruction's own syntax line calls it
    `ErrorCodes` -- kept as `ErrorCodes` (matching the syntax line), the
    same syntax-line-wins resolution used for the `SendGetVariables`
    naming mismatch in the PakBus round
  - Added all 11 to `keywords.json` under the `communication` category
    (alongside the existing PakBus/DNP/GPS/I2C networking entries),
    regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 11, keeping the existing parity enforced by all
    three completeness tests; every parameter name/order was taken
    directly from each instruction's own help.campbellsci.com syntax
    diagram (not inferred), following the same evidentiary bar as prior
    rounds
  - 11 new completion tests (one exact-`insert_text` test per function,
    following the Round 29-31 convention) added across 3 commits (one per
    LSP layer, following the Round 29-31 commit convention); full
    workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass (419 `crbasic-lsp` lib tests, up
    from 408)

Not flagged as gaps (out of scope for this pass):

- CDM_\*/SDM\* peripherals (40 confirmed instructions) remain deferred to
  a future round per the Round 31 sizing above -- the last remaining
  category from that backlog, and the largest one surveyed so far.

### Reference Implementation & Official Docs Comparison, Round 33 (2026-08-12)

CDM_\*/SDM\* (Round 31's sizing: 40 confirmed instructions / 329
parameters) was too large for a single one-category-per-round pass, so
before implementing anything this round re-scraped both reference
extensions' grammars for the raw candidate name list and split it into
three natural sub-batches by evidentiary source and hardware grouping,
confirmed with the user before implementation began:

- Round 33a (this round): `SDM*` peripheral-bus instructions (13
  candidates)
- Round 33b (deferred): `CDM_*` general-purpose measurement instructions
  (~27 candidates)
- Round 33c (deferred): `CDM_VW300` subfamily (4 candidates),
  kept separate per Round 31's note that it's documented only in a
  device manual rather than the standard instruction reference

Two parallel research agents independently verified all 13 `SDM*`
candidates against help.campbellsci.com (one cross-checking against raw
PDF manual text via `pdftotext`, not just HTML help) before anything was
added, following the same evidentiary bar as prior rounds.

- [x] `SDMAO4`, `SDMAO4A`, `SDMBeginPort`, `SDMCAN`, `SDMCD16AC`,
  `SDMCVO4`, `SDMGeneric`, `SDMINT8`, `SDMIO16`, `SDMSIO4`, `SDMSpeed`,
  `SDMTrigger`, `SDMX50` missing from `BUILTIN_FUNCTIONS` ✅ Resolved
  - All 13 are real, documented CRBasic instructions for configuring and
    operating SDM (Synchronous Device for Measurement) bus peripherals
    -- unlike every prior round's candidate batch, both research passes
    found zero typos, case variants, or grammar-scrape concatenation
    artifacts in this set; every name resolved to its own dedicated
    help.campbellsci.com page
  - None is a block construct (no `End*` keyword, no nested body), so
    this needed no parser/AST changes -- purely
    `keywords.json`/completion/hover/signature-help work
  - `SDMCAN`'s official help page *title* renders it "SDMCan" (mixed
    case), but the page's own syntax code block renders it `SDMCAN` --
    kept as `SDMCAN`, extending the existing syntax-line-wins resolution
    (previously applied only to parameter naming, e.g. `SendGetVariables`
    in the PakBus round) to the instruction name itself
  - `SDMTrigger`'s official syntax line has no parentheses (`SDMTrigger`,
    not `SDMTrigger()`), the same bare-expression treatment as
    `TimeUntilTransmit` in the PakBus round rather than the empty-parens
    `PPPClose()` convention
  - `SDMIO16`'s official syntax code block has a stray space in one
    parameter name ("Mode 4_1") -- normalized to `Mode4_1` to match its
    three sibling parameters (`Mode16_13`, `Mode12_9`, `Mode8_5`), since
    this is a formatting slip on Campbell's own page rather than an
    intentional alternate spelling
  - `SDMGeneric`'s and `SDMCAN`'s syntax lines and parameter description
    tables disagree on two parameter names each (`NumValuesOut`/`In` vs.
    table's `NumValsOut`/`In`; `Multiplier, Offset` vs. table's
    abbreviated "Mult, Offset") -- kept the syntax-line forms in both
    cases, consistent with the resolution above
  - Related `SDM*` instructions surfaced during research but explicitly
    out of scope for this round (not part of the original 13-name
    candidate list): `SDMCD16Mask` (bitmask alternative to `SDMCD16AC`),
    `SDMSW8A` (SDM-SW8A 8-channel switch module), and the modern
    `SerialOpen`/`SerialIn`/`SerialOut`/`SerialFlush` instructions that
    replaced the legacy `SDMSIO4`/`SDMSIO1A` model for newer hardware --
    left for a future round if the SDM family is revisited
  - Added all 13 to `keywords.json` under the `communication` category
    (alongside the existing PakBus/DNP/GOES/ARGOS/GPS/I2C networking
    entries), regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 13, keeping the existing parity enforced by
    all three completeness tests; every parameter name/order was taken
    directly from each instruction's own help.campbellsci.com syntax
    line (not inferred), following the same evidentiary bar as prior
    rounds
  - 14 new tests: 13 completion tests (one exact-`insert_text` test per
    function, following the Round 29-32 convention) and 1 signature test
    (`sdmtrigger_takes_no_parameters`, mirroring the existing
    `pppclose_takes_no_parameters`/`timeuntiltransmit_takes_no_parameters`
    convention) added across 3 commits (one per LSP layer, following the
    Round 29-32 commit convention); full workspace
    `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass (433 `crbasic-lsp` lib tests,
    up from 419)

Not flagged as gaps (out of scope for this pass):

- `CDM_*` general-purpose measurement instructions (~27 candidates,
  Round 33b) and the `CDM_VW300` subfamily (4 candidates, Round 33c)
  remain deferred to future rounds per the split above.

### Reference Implementation & Official Docs Comparison, Round 33b (2026-08-12)

Picked up the `CDM_*` general-purpose measurement instructions (28
candidates) deferred from Round 33's split. Two parallel research agents
independently verified all 28 candidates; unlike every prior round, their
findings *disagreed* on several names, which required a third,
direct-verification step before implementation could start.

- [x] `CDM_ACPower`, `CDM_Battery`, `CDM_BrFull`, `CDM_BrFull6W`,
  `CDM_BrHalf`, `CDM_BrHalf3W`, `CDM_BrHalf4W`, `CDM_CurrentDiff`,
  `CDM_Delay`, `CDM_ExciteI`, `CDM_ExciteV`, `CDM_MuxSelect`,
  `CDM_PanelTemp`, `CDM_PeriodAvg`, `CDM_PulsePort`, `CDM_Resistance`,
  `CDM_Resistance3W`, `CDM_SW12`, `CDM_SW5`, `CDM_SWPower`, `CDM_TCComp`,
  `CDM_TCDiff`, `CDM_TCSE`, `CDM_Therm107`, `CDM_Therm108`,
  `CDM_Therm109`, `CDM_VoltDiff`, `CDM_VoltSE` missing from
  `BUILTIN_FUNCTIONS` ✅ Resolved
  - One research agent found and cached dedicated
    `help.campbellsci.com/crbasic/cr6/Content/Instructions/cdm{name}.htm`
    pages for 25 of the 28 candidates (a URL pattern the other agent's
    search strategy never landed on) and reported clean, fully-sourced
    syntax for all of them; the second agent, searching PDF manuals and
    forum posts instead, corroborated most of the same 25 but explicitly
    flagged `CDM_CurrentDiff` and `CDM_SWPower` as "likely not real" after
    failing to find either
  - Rather than picking a side, directly fetched and read the first
    agent's own cached page files for `CDM_CurrentDiff` and `CDM_SWPower`
    (plus several other disputed/uncertain entries) before adding
    anything -- both turned out to be genuine, internally-consistent
    Campbell Scientific help pages (matching the exact glossary-hover-text
    and "CPI Calulator" typo fingerprint every other confirmed page in
    this project's history has had), directly refuting the second agent's
    "not real" call. This is the first round where the two independent
    verification passes materially disagreed rather than just differing
    in phrasing, and it confirms the value of the two-agent process: a
    single agent's negative result (absence of evidence) is not
    equivalent to evidence of absence when the other agent's positive
    result is independently checkable
  - The same direct-read step also resolved two parameter-name
    discrepancies between the two agents' reports: `CDM_PanelTemp`'s 5th
    parameter is `ThermChan` (per the cached official page), not
    `StartThermistor` (the second agent's own placeholder guess, invented
    because it never saw a page with the real name); `CDM_PulsePort`'s
    3rd/4th parameters are `Port`/`Delay` (per the cached official page's
    syntax line), not `PulseWidth` (same placeholder-guess issue)
  - `CDM_Therm107`, `CDM_Therm108`, `CDM_Therm109` have no dedicated help
    page (confirmed independently by both agents -- every URL pattern that
    worked for the other 25 candidates 404's for these three), but their
    existence is confirmed by a first-party source: `CDM_ExciteV`'s own
    official Remarks paragraph names all three by number ("Instructions
    that will not return the excitation to its former state are:
    CDM_PanelTemp, CDM_Therm107, 108, and 109..."). Their syntax was
    derived by extending the already-verified base `Therm107`/`Therm108`/
    `Therm109` signatures (already present in this codebase's
    `signature.rs`) with the `CDMType, CPIAddress` prefix shared by every
    other `CDM_` instruction in this family -- the same prepend pattern
    every other confirmed `CDM_` instruction follows relative to its
    non-CDM base instruction -- and cross-checked against one agent's
    directly-quoted real-world example code for `CDM_Therm107`/
    `CDM_Therm109` (`CDM_Therm108` has no confirmed example, but shares an
    identical parameter template with its two siblings)
  - `CDM_VoltSE`/`CDM_TCSE`: both the official page *title* and the
    formal *syntax line* render these with a capitalized `SE` (only the
    informal in-page example code sometimes lowercases it to `Se`) --
    kept as `CDM_VoltSE`/`CDM_TCSE`, extending the syntax-line-wins
    precedent to a case where title and syntax line already agree with
    each other and only the example disagrees. This also matches the
    existing non-CDM `TCSE` keyword already in this codebase's
    `keywords.json`
  - Several syntax-line vs. parameter-table naming disagreements
    confirmed directly from the cached pages, all resolved via the
    established syntax-line-wins rule: `CDM_PulsePort`/`CDM_SW5` (syntax
    `Port`, table heading `SW5Port`); `CDM_SW12`/`CDM_SW5`/`CDM_SWPower`
    (syntax `SWOption`, table heading `Option`); `CDM_Resistance`/
    `CDM_Resistance3W` (syntax `EXuA`, table heading `ExuA`, case only);
    `CDM_ExciteI` (syntax `IxuA`, table heading `IxUA`, case only)
  - None of the 28 is a block construct (no `End*` keyword, no nested
    body), so this needed no parser/AST changes -- purely
    `keywords.json`/completion/hover/signature-help work
  - Added all 28 to `keywords.json` under the `measurement` category
    (alongside the existing `Battery`/`BrFull`/`Therm107` measurement
    entries), regenerating the lexer keyword table and
    `client/syntaxes/crbasic.tmLanguage.json` via
    `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 28, keeping the existing parity enforced by
    all three completeness tests. `CDM_BrFull6W`/`CDM_BrHalf4W`'s
    optional `ReturnV1` and `CDM_Resistance`/`CDM_Resistance3W`'s
    optional `MeasCurrent` are included in the snippet, matching the
    existing non-CDM `BrFull6W`/`BrHalf4W`/`Resistance` snippets' own
    convention of including optional trailing parameters
  - 28 new completion tests (one exact-`insert_text` test per function,
    following the Round 29-33a convention) added across 3 commits (one
    per LSP layer, following the Round 29-33a commit convention); full
    workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass (461 `crbasic-lsp` lib tests,
    up from 433)

Not flagged as gaps (out of scope for this pass):

- The `CDM_VW300` subfamily (4 candidates, Round 33c) remains deferred
  per the Round 33 split above -- documented only in a device manual
  rather than the standard instruction reference, unlike every other
  candidate in this round.

### Reference Implementation & Official Docs Comparison, Round 33c (2026-08-12)

Picked up the `CDM_VW300` subfamily (4 candidates) deferred from Round
33's split -- the last remaining category from the entire Round 28 name-
diff backlog. Two parallel research agents independently located and read
the official CDM-VW300 device manual PDF (no dedicated help.campbellsci.com
page exists for this subfamily, confirmed by both agents) and agreed on
every syntax line with no conflicts, unlike Round 33b.

- [x] `CDM_VW300Config`, `CDM_VW300Dynamic`, `CDM_VW300RainFlow`,
  `CDM_VW300Static` missing from `BUILTIN_FUNCTIONS` ✅ Resolved
  - All 4 are real, documented CRBasic instructions for the CDM-VW300
    vibrating-wire spectrum-analyzer module, verified directly against
    the official CDM-VW300 instruction manual PDF (§7.6.1.2-7.6.1.6);
    the same syntax is also reused verbatim in the related VWIRE 305
    manual, which one agent found independently and used to
    corroborate the first agent's reading of the CDM-VW300 manual
  - The candidate list's `CDM_VW300Rainflow` (lowercase `f`) is a case
    typo of the manual's own `CDM_VW300RainFlow` (capital `F`) --
    confirmed by both agents directly against the manual text, not
    inferred; `CDM_VW300Rainfkow`, present in one reference extension's
    grammar, does not appear anywhere in the manual and is a further
    `k`/`l` typo of the same name -- neither was added as a separate
    entry
  - `CDM_VW300Config`'s official parameter-description table (Table
    7-2) names its 3rd parameter `SysOptions` (plural), but the
    instruction's own syntax line and every code example in the manual
    use `SysOption` (singular) -- kept as `SysOption`, the same
    syntax-line-wins resolution used for prior naming inconsistencies
    in Campbell Scientific's own docs
  - None of the 4 is a block construct (no `End*` keyword, no nested
    body, confirmed by grepping the manual text for `EndCDM`), so this
    needed no parser/AST changes -- purely
    `keywords.json`/completion/hover/signature-help work
  - Added all 4 to `keywords.json` under the `measurement` category
    (alongside the existing Round 33b `CDM_*` entries), regenerating
    the lexer keyword table and `client/syntaxes/crbasic.tmLanguage.json`
    via `scripts/generate-grammar.js`
  - Added matching completion snippets, hover text, and per-parameter
    signature help for all 4, keeping the existing parity enforced by
    all three completeness tests; every parameter name/order was taken
    directly from the manual's own syntax lines (not inferred)
  - 4 new completion tests (one exact-`insert_text` test per function,
    following the Round 29-33b convention) added across 3 commits (one
    per LSP layer, following the Round 29-33b commit convention); full
    workspace `build`/`test`/`clippy`/`fmt` gate and client
    `lint`/`format:check`/`test` gate pass (465 `crbasic-lsp` lib tests,
    up from 461)

This closes out the entire Round 28 builtin-function name-diff backlog
(PakBus, DNP, CSAT3/EC100/LI-COR, GOES/ARGOS, SDM, CDM general, and
CDM_VW300 -- Rounds 29 through 33c).
