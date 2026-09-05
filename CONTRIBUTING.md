# Contributing

## Prerequisites

- [rustup](https://www.rust-lang.org/tools/install) — installs the exact Rust toolchain pinned in `rust-toolchain.toml` automatically
- [nvm](https://github.com/nvm-sh/nvm) — recommended for managing the Node.js version
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [just](https://just.systems/) — task runner
- [pre-commit](https://pre-commit.com/) — hook runner

## Setup

```sh
pip install pre-commit   # or: brew install pre-commit
git clone https://github.com/connect0459/crbasic-lsp-rs.git
cd crbasic-lsp-rs
nvm use   # installs/activates the Node.js version pinned in .nvmrc
just setup
```

Any `cargo`/`rustup` command run inside the repo picks up the Rust version, components, and `wasm32-unknown-unknown` target pinned in `rust-toolchain.toml` automatically (installing it on first use if missing) — no equivalent of `nvm use` is needed for Rust.

`just setup` installs the pre-commit git hook (`pre-commit install`), then runs `cargo build` and `npm install` (in `client/`) to fetch dependencies for both workspaces. It requires the `pre-commit` CLI to already be installed, hence the first line above.

## Development workflow

| Command | Purpose |
| :--- | :--- |
| `cargo test --workspace` | Run all Rust tests |
| `just test-crate <name>` | Run tests for a single crate (e.g. `crbasic-parser`) |
| `cd client && npm run test:run` | Run TypeScript tests (Vitest) |
| `cargo fmt` | Format all Rust source files |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lint Rust code |
| `cd client && npm run lint` | Lint TypeScript code (ESLint) |
| `cd client && npm run format` | Format TypeScript code (Prettier) |
| `just build-wasm` | Build the `crbasic-wasm` package |
| `just build-extension` | Build the `crbasic-lsp` server binary and copy it into `client/server/` |
| `just install-local` | Package a `.vsix` for this machine and install it into VS Code |
| `just verify` | Run the full CI-equivalent check locally |

The pre-commit hooks enforce formatting and lint checks on every commit. To run them manually across all files:

```sh
pre-commit run --all-files
```

Before opening a pull request, run the full verification suite:

```sh
just verify
```

This mirrors the CI checks: Rust formatting and clippy across the workspace, `cargo test`, coverage thresholds via `cargo llvm-cov`, a check that `crbasic.tmLanguage.json`/`keywords_generated.rs` are up to date with `keywords.json`, and ESLint/Prettier/`tsc --noEmit`/Vitest for the client.

## Running the extension locally

There are two ways to try a change in a real VS Code instance instead of relying on unit tests alone.

**Extension Development Host** — fastest for iterating:

```sh
just build-extension   # builds crbasic-lsp and copies it into client/server/
cd client && npm run build
```

Then open this repository in VS Code and press `F5`. This launches an Extension Development Host with the extension loaded from `client/`, using the checked-in `.vscode/launch.json` and `.vscode/tasks.json`. Its `preLaunchTask` already runs `just build-extension` plus the client build for you, so the two commands above are only needed if you want to run them manually (e.g. outside VS Code).

**Installing a real `.vsix`** — for verifying the packaged artifact itself:

```sh
just install-local
```

This builds `crbasic-lsp` for your machine, packages a `.vsix` (see `client/scripts/package-vsix.js`), and installs it via `code --install-extension`. If the Marketplace version of the extension is already installed, disable or uninstall it first so only one version is active.

## Testing guidelines

This project follows **Red → Green → Refactor** (Detroit-school TDD):

- Write a failing test first, then implement.
- Use real objects; mocks are only permitted at external boundaries (file system, network, WASM host bindings).
- Test names describe **what business rule** is verified, not how.
- Coverage targets: 80% line, 75% branch, 90% function.

Rust unit tests live in-file under `#[cfg(test)] mod tests`; integration tests live under each crate's `tests/` directory. TypeScript tests are colocated as `*.test.ts` files next to their source.

```sh
cargo test --workspace
cd client && npm run test:run
```

## Commit format

```text
<type>(<scope>): <subject>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `tidy`, `test`, `chore`, `ci`, `perf`

**Scope**: crate or area name when the change targets a specific part of the codebase (`parser`, `lexer`, `lsp`, `wasm`, `client`, etc.); omit for project-wide changes.

**Subject**: imperative mood, 72 characters max, no trailing period.

Examples:

```text
feat(parser): add lexer for CRBasic single-quote comments
fix(lsp): handle case-insensitive keyword matching in hover
tidy(client): remove unused wasm-loader import
```

## Pull request process

1. Fork the repository and create a branch: `feat/xxx`, `fix/xxx`, `docs/xxx`.
2. Follow the Red → Green → Refactor cycle.
3. Run `just verify` and commit any resulting diffs.
4. If the change touches a public API (parser AST, LSP capabilities, WASM bindings), update the relevant rustdoc/JSDoc comments and `docs/ARCHITECTURE.md` so documentation stays in sync with the implementation.
5. If the change is an architectural decision, add an ADR under `docs/adrs/` following `docs/adrs/adr-000-template.md`.
6. Open a pull request — CI runs `cargo test`, `cargo clippy`, `cargo fmt --check`, and the client's lint/test/format checks.

## Code style

- No code comments unless the **why** is genuinely non-obvious.
- Respect layer boundaries: `crbasic-parser` has no LSP dependencies, `crbasic-lsp` depends on the parser (not WASM), and `crbasic-wasm` is thin bindings only. See `docs/ARCHITECTURE.md` for details.
- Prefer immutability; avoid mutable state unless necessary.
- Use `Result` for error handling in Rust; avoid `unwrap()`/`expect()` outside tests.
- All user-facing strings (test names, error messages, doc comments) must be in **English**.
