# ADR-001: Rust + WASM for LSP Server Implementation

**Status**: Accepted
**Date**: 2025-11-21
**Decision Makers**: Project Team
**Tags**: #architecture #lsp #rust #wasm

## Context and Problem Statement

We need to implement a Language Server Protocol (LSP) server for CRBasic that provides advanced language features (diagnostics, IntelliSense, navigation) within a VSCode extension.

The LSP server must:

1. Parse CRBasic source code with complex domain-specific semantics
2. Provide real-time diagnostics with model-dependent validation rules
3. Run efficiently in VSCode's extension host environment
4. Be maintainable and testable with high code quality standards

The key decision is: **What technology stack should we use for the LSP server implementation?**

## Decision Drivers

- **Performance**: Real-time parsing and diagnostics require efficient execution
- **Type Safety**: CRBasic has complex semantic rules (variable name truncation, scope rules) that benefit from strong typing
- **VSCode Integration**: Must run in VSCode's extension host (Node.js/Electron environment)
- **Maintainability**: Code must be testable, documented, and follow best practices
- **Memory Safety**: Parsing untrusted/malformed code should not cause crashes
- **WASM Support**: Ability to run in browser environments for potential web IDE support

## Considered Options

### Option 1: TypeScript LSP Server (Pure Node.js)

**Pros**:

- Native VSCode integration (no additional runtime needed)
- Large ecosystem of LSP libraries (`vscode-languageserver`)
- Easy debugging in VSCode environment
- Faster initial development (no compilation step)

**Cons**:

- Runtime performance overhead compared to native/WASM
- Lacks strong type safety for complex parsing logic
- Memory management issues with large ASTs
- No WASM portability for future web IDE integration

### Option 2: Rust LSP Server (Native Binary)

**Pros**:

- Excellent performance (native speed)
- Strong type safety with Rust's type system
- Memory safety without garbage collection overhead
- Rich parsing libraries (e.g., `nom`, `pest`)
- Excellent LSP support via `tower-lsp`

**Cons**:

- Requires separate binary distribution for each platform (Windows, macOS, Linux)
- Complex build/packaging process for VSCode extension
- Additional runtime dependency management

### Option 3: Rust + WASM LSP Server (Selected)

**Pros**:

- Near-native performance with WASM
- Strong type safety and memory safety from Rust
- **Platform-independent**: Single WASM binary works everywhere
- Seamless VSCode integration via `wasm-bindgen`
- Future-proof: Can be reused in web-based IDEs
- Excellent tooling: `wasm-pack` for npm packaging
- Comprehensive testing with Rust's test framework
- No external binary dependencies (everything bundled in extension)

**Cons**:

- Slight performance overhead vs. native (typically <10%)
- Additional WASM build step in development workflow
- WASM binary size consideration (mitigated by optimization)

## Decision Outcome

**Chosen option**: **Option 3: Rust + WASM LSP Server**

### Rationale

1. **Platform Independence**: WASM eliminates the need for platform-specific binaries, simplifying distribution and ensuring consistent behavior across all platforms.

2. **Type Safety for Complex Semantics**: CRBasic's domain-specific rules require precise semantic analysis.
   - e.g., CR200X 12-character variable truncation, Public variable global scope regardless of declaration location
   - Rust's type system allows us to model these rules safely:

   ```rust
   enum DataloggerModel {
       CR200X { max_var_len: usize, truncate_len: usize },
       CR6 { max_var_len: usize, recommended_len: usize },
   }
   ```

3. **Memory Safety**: Parsing arbitrary/malformed CRBasic code requires robust error handling. Rust's ownership system prevents memory leaks and crashes without garbage collection overhead.

4. **Performance**: Real-time diagnostics demand efficient parsing. WASM provides near-native performance, which is sufficient for CRBasic programs (typically <1000 lines).

5. **Future-Proofing**: WASM enables potential reuse in web-based development environments (e.g., browser-based CRBasic editor for quick prototyping).

6. **Testing & Quality**: Rust's built-in test framework and `cargo-tarpaulin` for coverage align with our TDD approach and coverage targets (80% line, 75% branch, 90% function).

### Implementation Strategy

- **Crate Structure**:
  - `crbasic-parser`: Pure Rust parser (no LSP dependencies)
  - `crbasic-lsp`: LSP server using `tower-lsp`
  - `crbasic-wasm`: WASM bindings using `wasm-bindgen`

- **Build Pipeline**:
  1. `cargo test` for Rust unit/integration tests
  2. `wasm-pack build` to generate WASM + TypeScript bindings
  3. Vite bundles WASM with TypeScript client
  4. VSCode extension packages everything

- **Performance Optimization**:
  - Use `wasm-opt` for WASM size reduction
  - Implement incremental parsing for large files
  - Cache AST and symbol tables between edits

## Consequences

### Positive

- ✅ Single codebase works on all platforms (Windows, macOS, Linux)
- ✅ Strong type safety reduces bugs in complex semantic analysis
- ✅ Memory-safe parsing of untrusted code
- ✅ Fast development iteration with Rust's tooling
- ✅ Comprehensive testing with Rust's test framework
- ✅ Future extensibility to web-based IDEs

### Negative

- ⚠️ WASM build step adds to development workflow (mitigated by watch mode)
- ⚠️ Slight performance overhead vs. native (~5-10%)
- ⚠️ Developers need Rust knowledge (in addition to TypeScript)

### Neutral

- 🔹 WASM binary size (~500KB estimated, acceptable for VSCode extension)
- 🔹 Learning curve for `wasm-bindgen` (well-documented, one-time cost)

## Validation

We will validate this decision by:

1. **Performance Benchmark**: Parse sample CRBasic programs (<1ms for typical 100-line program)
2. **Memory Usage**: Monitor WASM memory consumption (<10MB for typical workspace)
3. **Development Velocity**: Track time from test-writing to green (target: <2 minutes)
4. **Code Quality**: Achieve coverage targets (80% line, 75% branch, 90% function)

If WASM performance becomes a bottleneck (>100ms parsing time), we will re-evaluate Option 2 (native Rust binary) or implement a hybrid approach (WASM for small files, native for large files).

## Affected Files

### Initial Implementation (2025-11-21)

- `Cargo.toml`: Workspace configuration for Rust crates
- `crates/crbasic-parser/`: Core parser implementation
- `crates/crbasic-lsp/`: LSP server implementation
- `crates/crbasic-wasm/`: WASM bindings
- `client/package.json`: WASM dependency management
- `client/src/extension.ts`: WASM LSP client initialization
- `.github/workflows/ci.yml`: CI pipeline for WASM builds (future)

### ADR Numbering Fix (2026-08-06)

- `docs/adrs/adr-001-rust-wasm-lsp-architecture.md`: removed the stale
  "ADR-003: Datalogger model detection strategy (future)" placeholder link
  once ADR-003 was claimed by a different, unrelated decision (see
  [ADR-003](./adr-003-release-process.md))

## Related Decisions

- [ADR-002](./adr-002-textmate-grammar-first.md): TextMate Grammar for initial syntax highlighting

Datalogger model detection strategy was implemented directly as
`ValidationProfile` (`crates/crbasic-parser/src/semantic.rs`) without a
dedicated ADR; see `docs/todo.md`'s "Datalogger-specific validation
profiles" entry for the rationale.

## References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/): Language Server Protocol
- [tower-lsp](https://docs.rs/tower-lsp/): Rust LSP framework
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/): Rust-WASM bindings
- [wasm-pack](https://rustwasm.github.io/wasm-pack/): WASM build tool
- [Research Document](../researches/research-001-crbasic-for-vscode.md): CRBasic language analysis
