# ADR-005: Migrate from `tower-lsp` to `tower-lsp-server`

**Status**: Accepted
**Date**: 2026-08-09
**Decision Makers**: Project Team
**Tags**: #dependencies #lsp #maintenance

## Context and Problem Statement

[ADR-001](./adr-001-rust-wasm-lsp-architecture.md) chose `tower-lsp` (the
`ebkalderon/tower-lsp` crate) as the LSP framework underlying
`crbasic-lsp`. `docs/todo.md` flagged a dependency review as outstanding:
`tower-lsp = "0.20"` is the last release of the upstream crate, published
2024-08-15, with no newer release since. The repository itself is not
archived, but its release cadence has effectively stopped.

The question: **should `crbasic-lsp` stay on `tower-lsp 0.20`, or migrate to
a maintained alternative?**

## Decision Drivers

- **Maintenance signal**: whether ongoing LSP spec additions, dependency
  bumps (e.g. `lsp-types`), and bug fixes still land upstream.
- **Community governance**: a fork maintained by a project-specific
  individual (scoped to that project's own needs) is a weaker signal than
  one maintained by a dedicated community organization accepting outside
  contributions.
- **Migration cost**: how much of `crbasic-lsp`'s 37 `tower_lsp`-referencing
  call sites would need to change, and whether the change is a mechanical
  rename or a deeper API/type migration.
- **Architectural fit**: must remain compatible with the existing
  `tower-lsp`/stdio transport architecture ADR-001 and
  [ADR-004](./adr-004-multi-platform-packaging.md) already depend on --
  this is a dependency swap, not a transport or architecture change.

## Considered Options

### Option 1: Stay on `tower-lsp 0.20`

**Pros**:

- Zero migration cost; every existing call site keeps compiling as-is.
- Not archived; still installable and functional today.

**Cons**:

- No release in over a year; any future LSP spec addition or upstream
  `lsp-types` fix will not reach this crate without forking anyway.
- Silent drift was explicitly called out in `docs/todo.md` as the outcome
  to avoid -- staying requires a deliberate decision, not defaulting to
  inertia.

### Option 2: Migrate to `tower-lsp-f` (`neocmakelsp/tower-lsp-f`)

**Pros**:

- Newer releases than upstream (0.24.0 vs. 0.20.0 at review time).

**Cons**:

- Originally created as a fork for the maintainer's own `neocmakelsp`
  project, not as a general-purpose community successor -- weaker
  governance signal than a dedicated fork organization.
- No evidence of a broader contributor base or an explicit migration guide
  for consumers coming from upstream `tower-lsp`.

### Option 3: Migrate to `tower-lsp-server` (`tower-lsp-community/tower-lsp-server`) (Selected)

**Pros**:

- Explicitly positioned as "a community fork of tower-lsp", maintained by a
  dedicated GitHub organization rather than a single project's maintainer.
- Actively released: 1.4M+ cumulative downloads on crates.io, last updated
  2025-12-07 (latest at review time: 0.23.0).
- Ships a `CHANGELOG.md` documenting breaking changes per version, easing
  future upgrades.
- `default = ["runtime-tokio"]` matches this project's existing `tokio`
  runtime with no extra feature flags needed.

**Cons**:

- Swapped its LSP type definitions from the (also unmaintained)
  `gluon-lang/lsp-types` to its own `ls-types` fork, which:
  - Renames the `tower_lsp::lsp_types` module to `tower_lsp_server::ls_types`.
  - Replaces `Url` (from the `url` crate) with its own `fluent_uri`-backed
    `Uri` type -- a real API change, not just a rename, touching every
    document-URI call site.
- Dropped the `#[async_trait]` macro in favor of native `async fn` in
  traits (v0.21.0), requiring the `impl LanguageServer` block's attribute
  to be removed.
- `workspace/symbol` (`fn symbol`) now returns
  `Result<Option<WorkspaceSymbolResponse>>` instead of
  `Result<Option<Vec<SymbolInformation>>>`, requiring the response to be
  wrapped in `WorkspaceSymbolResponse::Flat`.

## Decision Outcome

**Chosen option**: **Option 3: Migrate to `tower-lsp-server`**

### Rationale

1. Between the two forks, `tower-lsp-server`'s community-organization
   governance and independently-verified release cadence (1.4M+ downloads,
   updated within the last two months at review time) are a materially
   stronger maintenance signal than `tower-lsp-f`'s single-maintainer,
   single-project origin.
2. The migration cost -- while real (`Url` to `Uri`, `#[async_trait]`
   removal, `workspace/symbol`'s return type) -- is bounded and mechanical:
   every affected call site was identified and fixed without touching
   business logic, and the existing 429+ test suite is the regression net
   that confirms no behavioral change. This is a dependency swap, not a
   rewrite.
3. Staying on `tower-lsp 0.20` (Option 1) trades this one-time, bounded
   migration cost for indefinitely accumulating drift from upstream LSP
   spec and `lsp-types` fixes -- the exact outcome `docs/todo.md` flagged
   as needing a deliberate decision rather than silent inertia.

### Implementation Strategy

- `Cargo.toml` (workspace) and `crates/crbasic-lsp/Cargo.toml`:
  `tower-lsp = "0.20"` → `tower-lsp-server = "0.23"`.
- Every `tower_lsp::` reference renamed to `tower_lsp_server::`, and
  `tower_lsp::lsp_types` to `tower_lsp_server::ls_types`, across
  `crates/crbasic-lsp/src` and `crates/crbasic-lsp/tests`.
- Every `Url` type/parameter renamed to `Uri`; every
  `Url::parse(EXPR).expect(...)` rewritten as `EXPR.parse::<Uri>().expect(...)`
  (`Uri` implements `FromStr`, not a `Url`-style `::parse` associated
  function).
- `Document::detect_model` (`crates/crbasic-lsp/src/document.rs`) rewritten
  to use `Uri::to_file_path()` + `Path::extension()` instead of the
  previous `uri.path().rsplit('.')` string-splitting -- `ls_types::Uri`'s
  `.path()` (via `Deref` to `fluent_uri::Uri`) returns a percent-encoded
  path segment type with no `rsplit`, and the `Path`-based approach is more
  correct regardless (handles percent-encoding and platform path
  separators that manual string splitting on `.path()` did not).
- `#[tower_lsp_server::async_trait]` removed from
  `impl LanguageServer for CRBasicLanguageServer`
  (`crates/crbasic-lsp/src/backend.rs`) -- native `async fn` in traits
  needs no macro desugaring.
- `CRBasicLanguageServer::symbol` (`backend.rs`) changed to return
  `Result<Option<WorkspaceSymbolResponse>>`, wrapping
  `WorkspaceSymbolProvider::search`'s result in
  `WorkspaceSymbolResponse::Flat`; the two `workspace/symbol` integration
  tests updated to match on the `Flat` variant.
- `InitializeResult`'s construction gained a trailing
  `..Default::default()` to stay forward-compatible with fields `ls-types`
  may add (e.g. `offset_encoding`) without every future field needing an
  explicit value here.

## Consequences

### Positive

- ✅ `crbasic-lsp` now depends on an actively-maintained LSP framework
  fork instead of one with no release in over a year.
- ✅ No behavioral change: `cargo build --workspace`,
  `cargo test --workspace` (429 lib/integration tests), `cargo clippy
  --all-targets --all-features -- -D warnings`, and `cargo fmt --all
  --check` all pass identically to before the migration.
- ✅ `Document::detect_model`'s new `Path`-based extension detection is
  more correct than the string-splitting it replaced (handles
  percent-encoded URIs correctly).

### Negative

- ⚠️ `ls-types`'s own crates.io version is `0.0.6` (pre-1.0, unstable
  versioning) -- a second dependency-review cycle may be needed sooner than
  a typical 1.0+ crate would require.
- ⚠️ `Uri` is a new, less widely known type (`fluent_uri`-backed) compared
  to the well-known `url::Url` -- future contributors need to know
  `to_file_path()`/`from_file_path()` exist for path conversions instead of
  reaching for `url`-crate-style APIs.

### Neutral

- 🔹 This ADR does not revisit ADR-001's core Rust+WASM+LSP architecture or
  ADR-004's native-binary packaging -- both remain unaffected, since
  `tower-lsp-server` is a drop-in framework replacement using the same
  stdio-transport, native-binary shape.

## Validation

We will validate this decision by:

1. Already done this session: `cargo build --workspace`,
   `cargo test --workspace`, `cargo clippy --all-targets --all-features --
   -D warnings`, and `cargo fmt --all --check` all pass with zero
   regressions against the pre-migration test count (429 lib/integration
   tests; the pre-existing local-only doctest linker failure documented in
   Phase 8 of `docs/todo.md` is unrelated and unaffected).
2. Ongoing: watch `tower-lsp-community/tower-lsp-server`'s release cadence;
   if it too stalls, this ADR's Option 1/2 comparison should be revisited.

## Affected Files

### Initial Implementation (2026-08-09)

- `Cargo.toml`: `tower-lsp = "0.20"` → `tower-lsp-server = "0.23"`.
- `crates/crbasic-lsp/Cargo.toml`: dependency renamed to match.
- `crates/crbasic-lsp/src/*.rs` (15 files): `tower_lsp` → `tower_lsp_server`,
  `lsp_types` → `ls_types`, `Url` → `Uri`.
- `crates/crbasic-lsp/src/document.rs`: `detect_model` rewritten for
  `Uri::to_file_path()` + `Path::extension()`.
- `crates/crbasic-lsp/src/backend.rs`: `#[async_trait]` removed; `symbol()`
  return type changed to `WorkspaceSymbolResponse`; `InitializeResult`
  gained `..Default::default()`.
- `crates/crbasic-lsp/tests/lsp_integration.rs`: same renames, plus
  `workspace/symbol` assertions updated for `WorkspaceSymbolResponse::Flat`.
- `docs/todo.md`: dependency review item checked off.

## Related Decisions

- [ADR-001](./adr-001-rust-wasm-lsp-architecture.md): originally chose
  `tower-lsp` as the LSP framework; this ADR keeps that choice's rationale
  (Rust + `tower`-based LSP server) intact while swapping the specific
  crate providing it.

## References

- [tower-lsp-server on crates.io](https://crates.io/crates/tower-lsp-server): release history and download counts reviewed for this decision.
- [tower-lsp-server GitHub repository](https://github.com/tower-lsp-community/tower-lsp-server): community fork governance and `CHANGELOG.md`.
- [ls-types on crates.io](https://crates.io/crates/ls-types): the `lsp-types` fork `tower-lsp-server` depends on, source of the `Url` → `Uri` change.
