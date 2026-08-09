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
  - `Include` (referenced by `#UnDef`'s real use case) is also entirely
    unsupported by this project -- separate, related gap, not addressed
    here
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
