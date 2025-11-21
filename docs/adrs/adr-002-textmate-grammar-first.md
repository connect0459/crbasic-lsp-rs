# ADR-002: TextMate Grammar for Initial Syntax Highlighting

**Status**: Accepted
**Date**: 2025-11-21
**Decision Makers**: Project Team
**Tags**: #syntax-highlighting #textmate #vscode

## Context and Problem Statement

VSCode extensions can provide syntax highlighting through two primary mechanisms:

1. **TextMate Grammar**: Declarative regex-based tokenization (`.tmLanguage.json`)
2. **Semantic Tokens (LSP)**: Programmatic tokenization via Language Server

The question is: **Should we rely solely on semantic tokens from our LSP server, or should we implement a TextMate Grammar as a first layer of syntax highlighting?**

## Decision Drivers

- **User Experience**: Syntax highlighting should appear instantly when a file is opened
- **Performance**: Large files should highlight without noticeable lag
- **Fallback Behavior**: Highlighting should work even if LSP fails to initialize
- **Development Effort**: Balance between implementation complexity and user value
- **Standards Compliance**: Follow VSCode extension best practices

## Considered Options

### Option 1: LSP Semantic Tokens Only

**Pros**:

- Single source of truth for tokenization logic
- Semantic awareness (e.g., distinguish variable references from declarations)
- No duplication of keyword lists

**Cons**:

- Delayed highlighting until LSP server initializes (noticeable lag on startup)
- No fallback if LSP fails or crashes
- Worse performance for large files (requires full parsing)
- Poor user experience: blank file until LSP activates

### Option 2: TextMate Grammar Only

**Pros**:

- Instant highlighting (VSCode built-in tokenizer)
- Extremely fast (regex-based, no parsing required)
- Always available (no LSP dependency)

**Cons**:

- Limited semantic awareness (cannot distinguish context-dependent tokens)
- Duplication of keyword definitions
- Less precise for complex language features

### Option 3: TextMate Grammar + LSP Semantic Tokens (Selected)

**Pros**:

- **Instant baseline highlighting** via TextMate Grammar
- **Enhanced semantic highlighting** via LSP when available
- **Graceful degradation**: Works even if LSP fails
- **Best user experience**: Immediate feedback, refined over time
- **Standard practice**: Follows VSCode extension conventions

**Cons**:

- Duplication of keyword lists (mitigated by shared source)
- Slightly more implementation effort

## Decision Outcome

**Chosen option**: **Option 3: TextMate Grammar + LSP Semantic Tokens**

### Rationale

1. **Instant User Feedback**: TextMate Grammar provides immediate syntax highlighting when a CRBasic file is opened, before the LSP server even starts.

2. **Graceful Degradation**: If the LSP server fails to initialize (e.g., WASM loading error, parser crash), users still get basic syntax highlighting.

3. **Performance at Scale**: For very large CRBasic programs (e.g., 1000+ lines), TextMate Grammar provides fast initial highlighting while LSP performs semantic analysis in the background.

4. **Industry Standard**: Most VSCode language extensions use this hybrid approach (e.g., TypeScript, Rust, Python extensions all provide both).

5. **CRBasic Suitability**: CRBasic's syntax is well-suited to regex-based tokenization:
   - Clear keyword boundaries (case-insensitive)
   - Simple comment syntax (`'` to end of line)
   - Distinct line continuation pattern (space + `_`)

### Implementation Strategy

**TextMate Grammar Scope Coverage**:

- `comment.line.single-quote.crbasic`: Single-quote comments
- `keyword.control.crbasic`: Control flow keywords (If, For, Do, While, etc.)
- `storage.type.crbasic`: Declaration keywords (Public, Dim, Const, Alias)
- `support.function.measurement.crbasic`: Measurement instructions (PulseCount, Battery, etc.)
- `support.function.comms.crbasic`: Communication instructions (SerialOpen, SerialOut, etc.)
- `constant.numeric.crbasic`: Numeric literals
- `string.quoted.double.crbasic`: String literals
- `punctuation.separator.continuation.crbasic`: Line continuation (`_`)

**LSP Semantic Token Enhancement** (future):

- Distinguish variable declarations vs. references
- Highlight Public variables differently from Dim variables
- Mark user-defined functions/subroutines
- Highlight model-specific validation warnings (e.g., variable names >12 chars on CR200X)

**Avoiding Duplication**:

- Maintain a single source of truth for CRBasic keywords (e.g., `keywords.json`)
- Generate TextMate Grammar from this source during build
- Use same list for LSP completion/validation

### TextMate Grammar Structure

```json
{
  "scopeName": "source.crbasic",
  "patterns": [
    { "include": "#comments" },
    { "include": "#line-continuation" },
    { "include": "#keywords" },
    { "include": "#built-in-instructions" },
    { "include": "#strings" },
    { "include": "#numbers" }
  ],
  "repository": {
    "comments": {
      "name": "comment.line.single-quote.crbasic",
      "match": "'.*$"
    },
    "line-continuation": {
      "name": "punctuation.separator.continuation.crbasic",
      "match": "\\s_$"
    },
    // ... (detailed patterns for keywords, instructions, etc.)
  }
}
```

## Consequences

### Positive

- ✅ Instant syntax highlighting on file open (< 10ms)
- ✅ Extension works even if LSP fails
- ✅ Better performance for large files (1000+ lines)
- ✅ Follows VSCode extension best practices
- ✅ Users see immediate value (highlighting works from day 1)

### Negative

- ⚠️ Keyword list duplication between TextMate and LSP (mitigated by shared source)
- ⚠️ Two layers of maintenance (TextMate patterns + LSP semantic tokens)

### Neutral

- 🔹 Initial implementation effort (~2-4 hours for TextMate Grammar)
- 🔹 Need to ensure scope names align with common VSCode themes

## Validation

We will validate this decision by:

1. **Performance**: Measure time-to-first-highlight (<50ms target for 1000-line file)
2. **Correctness**: Verify highlighting against sample CRBasic programs (no mis-tokenization)
3. **Fallback**: Test extension behavior when LSP is disabled (highlighting still works)
4. **User Feedback**: Confirm highlighting appears "instantly" on file open

## Affected Files

- `client/syntaxes/crbasic.tmLanguage.json`: TextMate Grammar definition
- `client/package.json`: Grammar registration in extension manifest
- `crates/crbasic-lsp/src/keywords.rs`: Shared keyword definitions (future)
- `scripts/generate-grammar.js`: Script to generate `.tmLanguage.json` from keywords (future)

## Related Decisions

- [ADR-001](./adr-001-rust-wasm-lsp-architecture.md): Rust + WASM LSP architecture
- [ADR-004](./adr-004-keyword-database-source.md): Keyword database source of truth (future)

## References

- [VSCode Syntax Highlight Guide](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide)
- [TextMate Grammar Reference](https://macromates.com/manual/en/language_grammars)
- [Semantic Highlighting Guide](https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide)
- [Research Document](../researches/research-001-crbasic-for-vscode.md): Section 2 (Lexical Structure)
