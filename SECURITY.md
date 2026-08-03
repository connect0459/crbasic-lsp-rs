# Security Policy

## Supported Versions

Only the latest release on the `main` branch is actively maintained.
Older versions do not receive security fixes.

| Version  | Supported |
| :------- | :-------- |
| latest   | ✓         |
| < latest | ✗         |

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Use GitHub's [private vulnerability reporting][private-report] feature to
disclose issues confidentially. You will receive an acknowledgment within
**5 business days** and a resolution timeline once the report has been
triaged.

[private-report]: https://github.com/connect0459/crbasic-lsp-rs/security/advisories/new

## Scope

The following vulnerability classes are in scope for this project:

- **WASM sandbox escapes** — any path that allows the `crbasic-wasm` bindings
  or the code they execute to access the host file system, network, or other
  resources outside the VSCode extension host's explicitly granted surface.
- **Parser/lexer crashes** — any input (including malformed or adversarially
  crafted CRBasic source text) that causes a panic, SIGSEGV, or unhandled
  runtime error in the lexer or parser (`crbasic-parser`).
- **Resource exhaustion** — inputs that cause unbounded memory growth, CPU
  spin, or stack overflow in the parser or LSP server (`crbasic-lsp`) when
  processing a single file, well beyond the documented performance targets.
- **LSP/extension injection** — any diagnostic, hover, or completion response
  that allows injecting unintended content into the VSCode UI or triggers
  unsafe behavior in the TypeScript client (`client/`).

The following are **out of scope**:

- Issues in third-party dependencies (report those upstream).
- Incorrect diagnostics or IntelliSense results that are not security-relevant
  (report those as regular bugs via GitHub Issues).
- Theoretical issues without a reproducible proof-of-concept.

## Disclosure Policy

Once a fix is ready and released, a GitHub Security Advisory will be published
with full details. The typical timeline from report to public disclosure is
**30 days**, though this may be extended by mutual agreement when a fix
requires significant changes.
