# ADR-006: Relocate Real-World Sample Fixtures Out of `docs/`

**Status**: Accepted
**Date**: 2026-08-28
**Decision Makers**: Project Team
**Tags**: #testing #project-structure

## Context and Problem Statement

While fixing #28 (a parenthesis-less `Call` parsing bug), `docs/sample-codes/sample-complex-realworld.CR6:120` turned out not to be exercised by any test: `crates/crbasic-parser/tests/sample_files.rs` hardcoded a hand-maintained per-file list in each of its five test modules (`tokenization`, `parsing`, `ast_structure`, `semantic_analysis`, `real_world_validation`), and that list still only covered the directory's original 10 files. The 11th file, the repository's most complex sample, silently sat outside regression coverage — which is exactly why the bug shipped unnoticed (#29).

Investigating that gap surfaced a broader structural issue: `docs/sample-codes/` is described by its own neighboring `docs/examples/README.md` as "real-world programs used as parser regression tests" — i.e. test input data — yet it physically lived under `docs/`, alongside genuinely reader-facing content (`docs/examples/`, `docs/researches/`, `docs/adrs/`). The directory had no README of its own and, as the coverage gap showed, no guarantee that everything inside it was actually tested.

The question: **should these regression fixtures move out of `docs/`, and if so, to where — and should the hardcoded per-file test lists be replaced with something that can't silently omit a file again?**

`docs/examples/` (small, heavily commented, feature-showcase programs with a different audience — see its own README) is a related but distinct case, deliberately out of scope for this ADR; its placement is tracked separately.

## Decision Drivers

- **Type mismatch with `docs/`**: every other entry under `docs/` (`adrs/`, `researches/`, `examples/`) is prose or a curated, human-read artifact. `docs/sample-codes/`'s own neighboring documentation self-describes it as test input, not reader-facing content — a category `docs/` does not otherwise hold.
- **Single consumer**: `crates/crbasic-parser/tests/sample_files.rs` is the only thing that reads these files; no other crate (`crbasic-lsp`, `crbasic-wasm`) or the `client/` package depends on them independently.
- **Avoid premature generality**: introducing a repo-root fixtures directory in anticipation of a second, currently nonexistent consumer would design for a hypothetical future requirement rather than the codebase as it stands.
- **Regression safety**: whatever replaces the current layout must make it structurally harder (not just documented as a convention) for a new fixture file to be added without being picked up by every relevant test category.

## Considered Options

### Option 1: Keep `docs/sample-codes/` where it is

**Pros**:

- Zero migration cost.
- Physically adjacent to `docs/examples/`, so both "real CRBasic program" directories are discoverable from the same parent.

**Cons**:

- Perpetuates the type mismatch: test input data living under a directory otherwise reserved for prose documentation.
- Does nothing to address the root cause of the coverage gap that let #28 ship unnoticed — the hardcoded test lists would need fixing regardless of location.

### Option 2: Move to `crates/crbasic-parser/tests/fixtures/` (Selected)

**Pros**:

- Colocated with the crate that is the fixtures' sole consumer — the standard Rust convention for integration-test input data (`tests/fixtures/` alongside the `tests/*.rs` files that read it).
- Removes the type mismatch: `docs/` no longer holds anything that isn't reader-facing documentation.
- Makes the "iterate the directory instead of hardcoding a list" fix (needed regardless of location, see Implementation Strategy) live directly next to the code it protects.

**Cons**:

- `CLAUDE.local.md` informally pointed contributors/agents at `docs/sample-codes/` as a CRBasic-syntax reference; a `tests/fixtures/` path reads less obviously as "browse this to learn the language" than a `docs/`-rooted path did. Mitigated by updating that reference to the new path rather than removing it.

### Option 3: Move to a repo-root `fixtures/` or `test-data/` directory

**Pros**:

- Would be appropriate if more than one crate needed to share the same fixture set.

**Cons**:

- No such second consumer exists today. Per the "avoid premature generality" driver, introducing shared-scope infrastructure for a hypothetical future need is not justified by the current codebase.

## Decision Outcome

**Chosen option**: **Option 2: Move to `crates/crbasic-parser/tests/fixtures/`**

### Rationale

1. `crates/crbasic-parser/tests/sample_files.rs` is the fixtures' only consumer today; placing them inside that crate's `tests/` directory follows the same "closest to what consumes it" principle this project already applies to its layered architecture (parser → LSP → WASM → client), without inventing shared-scope infrastructure Option 3 would require.
2. Every entry remaining under `docs/` after this move is prose or curated human-read content; the fixtures' own neighboring documentation already called them test input, not documentation, so removing the mismatch makes `docs/`'s contents consistent with what a reader of that directory now expects.
3. The coverage gap that motivated this review is fixed at the same time: `sample_files.rs` now reads `tests/fixtures/` via `fs::read_dir` instead of hardcoded per-file arrays, so a newly added fixture is automatically included in every test category instead of requiring a matching manual edit in five places.

### Implementation Strategy

- `git mv docs/sample-codes crates/crbasic-parser/tests/fixtures` (11 files, history preserved).
- `crates/crbasic-parser/tests/sample_files.rs` rewritten:
  - `fixture_filenames()` reads `tests/fixtures/` and returns a sorted list of every file present, replacing the five hardcoded arrays.
  - `model_for_fixture()` derives each fixture's `DataloggerModel` from its own file extension via `DataloggerModel::from_extension`, rather than a hand-maintained `(filename, model)` tuple list.
  - `tokenization`/`parsing`/`ast_structure`/`real_world_validation` each collapse to one test that loops over `fixture_filenames()`; `semantic_analysis` keeps its existing per-model tests, since those document specific, non-obvious extension-to-model mapping facts (e.g. `.cr1` sharing CR6's validation profile) rather than asserting directory completeness.
- Cross-references updated: `CLAUDE.local.md` (CRBasic-syntax reference path), `docs/examples/README.md` (its comparison link to the fixtures directory), `docs/ARCHITECTURE.md` (project structure tree).
- `docs/todo.md`'s historical entries referencing the old `docs/sample-codes/` path are left as-is (they describe past state accurately); this decision is logged as a new dated entry instead.

## Consequences

### Positive

- ✅ A newly added fixture file can no longer silently sit outside regression coverage — the exact gap that let #28 ship unnoticed is closed structurally, not just by convention.
- ✅ `docs/` now contains only reader-facing content, consistent with `docs/adrs/`, `docs/researches/`, and `docs/examples/`.
- ✅ `sample_files.rs` shrank from five hand-maintained per-file lists to two small directory-driven helpers.

### Negative

- ⚠️ `CLAUDE.local.md`'s informal "browse here to learn CRBasic" use case now points inside a crate's `tests/` directory, which reads less obviously as human-facing than a `docs/`-rooted path did.

### Neutral

- 🔹 `docs/examples/` is not addressed by this decision. Its own purpose (showcasing extension features to VSCode users, not general CRBasic reference material) and audience differ enough from `docs/sample-codes/`'s that the same reasoning does not mechanically transfer; its placement is tracked as a separate follow-up.

## Validation

We will validate this decision by:

1. Already done this session: `cargo test --package crbasic-parser --test sample_files` passes (9 tests), including `sample-complex-realworld.CR6` now covered by all five test categories for the first time.
2. Ongoing: any future fixture file added to `crates/crbasic-parser/tests/fixtures/` is automatically picked up by `sample_files.rs` without a corresponding code change — the property this ADR exists to guarantee.

## Affected Files

### Initial Implementation (2026-08-28)

- `docs/sample-codes/*` → `crates/crbasic-parser/tests/fixtures/*`: 11 files moved via `git mv`.
- `crates/crbasic-parser/tests/sample_files.rs`: rewritten to iterate `tests/fixtures/` instead of hardcoding per-file lists.
- `CLAUDE.local.md`: CRBasic-reference path updated.
- `docs/examples/README.md`: cross-link to the fixtures directory updated.
- `docs/ARCHITECTURE.md`: project structure tree updated.
- `docs/todo.md`: new dated entry logging this decision; historical entries left unchanged.

## Related Decisions

- None yet. A follow-up issue tracks whether `docs/examples/` should also move; if it does, that decision should cross-reference this ADR.

## References

- [#28](https://github.com/connect0459/crbasic-lsp-rs/issues/28): the parenthesis-less `Call` bug that surfaced this gap.
- [#29](https://github.com/connect0459/crbasic-lsp-rs/issues/29): the issue this ADR resolves.
