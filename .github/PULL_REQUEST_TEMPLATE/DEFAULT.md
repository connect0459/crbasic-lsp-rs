<!-- # PULL_REQUEST_TEMPLATE -->

<!-- Remove unnecessary sections to keep the review focused -->

## Related Links

- Issues
  - <!-- <https://github.com/connect0459/crbasic-lsp-rs/issues/xxx> -->
- PRs
  - <!-- <https://github.com/connect0459/crbasic-lsp-rs/pull/xxx> -->

## [Required] Overview

- Describe the problem being solved, its background, and what changes when this PR is merged.
- Links to `docs/todo.md` entries, ADRs under `docs/adrs/`, or other references are welcome.

```txt
It is difficult to review without knowing the specifications and background.
```

## Scope of Change

- [ ] `crbasic-parser`
- [ ] `crbasic-lsp`
- [ ] `crbasic-wasm`
- [ ] `client`
- [ ] Tooling / CI
- [ ] Documentation (`docs/todo.md`, `docs/ARCHITECTURE.md`, README)

## Breaking Changes

- [ ] No breaking changes
- [ ] Breaking changes (describe below)

<!--
If this changes a public API (parser AST, LSP capabilities, WASM bindings),
describe what breaks and why the breakage is justified, and update
docs/ARCHITECTURE.md to match.
-->

## Deferred Items and TODOs

- Items intentionally deferred and the reasons why.

```txt
If you deferred something due to time constraints, document it here.
Reviewers cannot tell whether something was intentionally skipped or overlooked
without this information.
```

## Test Items

- Describe the tests added, following Red/Green TDD (which test was written first, and what it confirmed failed before the implementation existed).
- Note coverage if it changed meaningfully (targets: 80% line, 75% branch, 90% function — see `CONTRIBUTING.md`).
- Confirm `just verify` passes with no regressions (Rust fmt/clippy/test/coverage, client lint/format/type-check/test).

## [Required] Quality Checklist

**Please check all items before merging.**

- [ ] **CI Workflow Execution**: All checks passed on the [CI workflow](../actions/workflows/ci.yml) for this PR.
- [ ] **Code Comments**: Limited to rustdoc/JSDoc and non-obvious WHY/WHY-NOT explanations, per this project's comment policy.
- [ ] **Reference Docs**: `docs/todo.md` updated to check off completed items and record any design decisions made along the way.

> **Important**: This checklist ensures quality. Please verify all items before requesting review.
