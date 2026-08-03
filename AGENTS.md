# Project Agents.md Guide

This is a Rust + TypeScript project: a Language Server Protocol implementation
for CRBasic (the programming language used in Campbell Scientific data
loggers), shipped as a VSCode extension via WebAssembly.

## Language Convention

This project is intended for open-source distribution. All of the following
must be written in **English**:

- Commit messages
- Code comments
- Documentation (including `AGENTS.md`, `README.md`, ADRs, etc.)
- Test names
- Error messages
- Variable/function names

## Project Structure

This project uses a **layered architecture**:

```text
Parser Layer (crbasic-parser)
    ↓
LSP Layer (crbasic-lsp)
    ↓
WASM Layer (crbasic-wasm)
    ↓
Client Layer (TypeScript, client/)
```

- `crates/crbasic-parser/` — lexer, parser, and AST; no LSP dependencies.
- `crates/crbasic-lsp/` — LSP server; depends on the parser, not on WASM.
- `crates/crbasic-wasm/` — thin `wasm-bindgen` bindings only.
- `client/` — VSCode extension (TypeScript); only calls the WASM public API.
- `docs/adrs/` — Architecture Decision Records for significant design choices.
- `docs/researches/` — CRBasic language specification research.

Respect these layer boundaries: never introduce a direct dependency that skips
a layer (e.g., the client calling into `crbasic-parser` directly, or the
parser depending on `tower-lsp`).

## Coding convention

**Rust**:

- Format with `cargo fmt`; pass `cargo clippy --all-targets --all-features -- -D warnings`.
- Use `Result` for error handling; avoid `unwrap()`/`expect()` in production code.
- Write rustdoc comments for all public APIs.
- Avoid `unsafe` unless justified in an ADR.

**TypeScript**:

- Format with Prettier; pass ESLint with no warnings.
- Use strict TypeScript settings (`strict: true`).
- Write JSDoc comments for exported functions.

**CRBasic language handling** (domain-specific rules the parser must enforce):

- Keywords are case-insensitive (`BeginProg` = `BEGINPROG` = `beginprog`);
  normalize to canonical form (e.g., `BeginProg`) and match case-insensitively
  in the lexer.
- Detect the datalogger model from the file extension (e.g., `.cr1` → CR200X)
  and apply model-specific variable name length validation:
  - CR200X: error if >16 chars, warn if >12 chars; also detect 12-char
    truncation collisions between field names.
  - CR6/GRANITE: error if >39 chars, warn if >35 chars.
- Treat `Public` variables as global regardless of declaration location;
  distinguish `Public` (monitored) from `Dim` (scratch) variables.

## Tooling

- `cargo build` / `cargo test --workspace` — build and test the Rust workspace.
- `wasm-pack build --target web` (run from `crates/crbasic-wasm/`) — build the
  WASM package consumed by the client.
- `cd client && npm test` — run TypeScript tests (Vitest).
- `just verify` — run the full CI-equivalent check (Rust fmt/clippy/test,
  TypeScript lint/format/test) before opening a PR.
- **pre-commit hooks** — catch formatting and lint issues on every commit
  (`pre-commit run --all-files` to run manually).

Performance targets to keep in mind when touching the parser or LSP server:

- Lexer: <1ms for a 1000-line file.
- Parser: <10ms for a 1000-line file (WASM).
- Diagnostics: <50ms for full-file validation.
- IntelliSense: <100ms response time.

## Development Philosophy

### Red/Green TDD (Detroit school)

- Red → Green → Refactor cycle strictly followed.
- Use real objects; mocks are only permitted at external boundaries (file
  system, WASM host bindings, network).
- Write tests BEFORE implementation; run tests AFTER implementation.
- Coverage targets: 80% line, 75% branch, 90% function.

### Domain Object Design

- Rich domain objects: pair data and logic in the same type.
- Prefer immutability; avoid mutable state unless necessary.
- Distinguish entities (identity-based) from value objects (value-based).
- Enforce layer boundaries through abstract types; no direct dependency on
  concrete implementations across layers.

### Evergreen Tests

- Test names describe WHAT business rule is being verified, not HOW.
- Test names must not reference implementation details.
- Test code serves as living documentation of the system's behavior.

### Code Comments

- Do NOT write code comments unless explicitly permitted by the user.
- Let the code speak for itself; let tests document the behavior.
- Code = How, Tests = What, Commit messages = Why.

## Git Conventions

### Format

```text
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

| Type | Description |
| :--- | :--- |
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Code style (formatting, whitespace) |
| `refactor` | Code change that is neither a fix nor a feature |
| `tidy` | Small, safe cleanup (< 2 min; no behavior change) |
| `test` | Adding or updating tests |
| `chore` | Build process, tooling, or config changes |
| `ci` | CI/CD pipeline changes (GitHub Actions, workflows) |
| `perf` | Performance improvement |

### Scopes

Scope is optional; use the crate or area name when the change targets a
specific part of the codebase (e.g., `parser`, `lexer`, `lsp`, `wasm`,
`client`). Omit for project-wide changes.

### Type vs. Scope Precedence

The type vocabulary above mixes two axes: an **impact axis** (`feat`, `fix`,
`perf`, `refactor` — the SemVer-relevant effect of a change) and a **domain
axis** (`docs`, `style`, `test`, `chore`, `ci`, `tidy` — a layer with no
runtime/SemVer effect). When a change is fully contained within a domain, use
that domain as `type` (e.g. `docs: fix typo`); do not use it as `scope` on an
impact-axis type (avoid `fix(docs): ...`). `scope` sub-divides whatever `type`
already established (e.g. `feat(parser)`); it is not a substitute
classification axis.

### Subject Line

- Use the imperative mood: "add", "fix", "remove" — not "added" or "adds".
- 72 characters max.
- No trailing period.

### Body (optional)

- Wrap at 72 characters.
- Explain **why**, not what — the diff already shows what changed.
- Leave one blank line between subject and body.

### Footer (optional)

- `BREAKING CHANGE: <description>` for breaking changes.
- `Closes #123` or `Fixes #456` to link issues.

### Branch naming

`feat/xxx`, `fix/xxx`, `docs/xxx`.
