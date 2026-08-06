# ADR-000: <Short Decision Title>

**Status**: Proposed | Accepted | Deprecated | Superseded by ADR-XXX
**Date**: YYYY-MM-DD
**Decision Makers**: <names, or "Project Team">
**Tags**: #tag1 #tag2

## Context and Problem Statement

Describe the situation that forces a decision. State the question being
decided as a single, explicit sentence (e.g. "Should we use X or Y for
Z?").

## Decision Drivers

- <Driver 1 — a constraint or goal that makes some options more viable than
  others (performance, maintainability, compatibility, effort, ...)>
- <Driver 2>

## Considered Options

### Option 1: <Name>

**Pros**:

- <...>

**Cons**:

- <...>

### Option 2: <Name> (Selected)

**Pros**:

- <...>

**Cons**:

- <...>

<Add as many options as were genuinely considered. Mark the chosen one
"(Selected)" in its heading.>

## Decision Outcome

**Chosen option**: **Option N: <Name>**

### Rationale

<Why this option won over the alternatives above. Numbered if there are
several distinct reasons.>

### Implementation Strategy

<Optional. Concrete steps, crate/module layout, or a code sketch the
decision implies.>

## Consequences

### Positive

- ✅ <...>

### Negative

- ⚠️ <...>

### Neutral

- 🔹 <...>

## Validation

We will validate this decision by:

1. <A concrete, checkable signal — a benchmark target, a test, an observed
  usage pattern — not just "it seems to work".>

## Affected Files

List every file this decision touches, grouped under a dated heading per
revision (the initial implementation, then any later modification driven by
a follow-up decision):

### Initial Implementation (YYYY-MM-DD)

- `path/to/file`: <what changed and why>

### <Revision Label> (YYYY-MM-DD)

- `path/to/file`: <what changed and why>

## Related Decisions

- [ADR-XXX](./adr-xxx-slug.md): <how it relates — supersedes, depends on,
  is superseded by, ...>

## References

- [Title](https://example.com): <why this reference matters>
