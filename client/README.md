# CRBasic Language Support

Language support for CRBasic, the programming language used in Campbell
Scientific data loggers.

## Features

- **Syntax Highlighting**: TextMate grammar-based highlighting for CRBasic
  keywords, built-in instructions, operators, and literals.
- **IntelliSense**: Context-aware code completion for CRBasic instructions
  and keywords, with parameter signature help.
- **Diagnostics**: Real-time validation with datalogger-model-specific
  rules.
  - Variable name length validation (model-dependent).
  - Duplicate field name detection (CR200X 12-character truncation
    collisions).
  - Structure validation (`BeginProg`/`EndProg`, `Function`/`EndFunction`,
    and other block constructs).
- **Navigation**: Go to definition, find all references, document symbols,
  rename, and linked editing ranges.
- **Hover Information**: Documentation for built-in instructions, including
  parameter descriptions.

## Supported File Extensions

| Extension | Datalogger model |
| --- | --- |
| `.cr2` | CR200(X) series (16-character variable name limit) |
| `.cr1`, `.cr1x` | CR1000, CR1000X series |
| `.cr3`, `.cr5` | CR3000, CR5000 series |
| `.cr6` | CR6 series |
| `.cr8` | CR800 series |
| `.cr9`, `.cr9x`, `.c9x` | CR9000 series |
| `.cr300` | CR300 series |
| `.crb`, `.dld` | Generic datalogger programs (shared across multiple models, including GRANITE-series; no model-specific validation) |

## Requirements

No external dependencies. The language server is bundled with the
extension.

## Known Limitations

This extension does not integrate with Campbell Scientific's official
toolchain (CRBasic Editor, LoggerNet, Short Cut) -- diagnostics are based
on this extension's own parser, not the official compiler.

## Feedback and Contributions

This is an independent, unofficial extension and is not affiliated with or
endorsed by Campbell Scientific. Source code, issue tracker, and
contribution guidelines are on
[GitHub](https://github.com/connect0459/crbasic-lsp-rs).

## License

[MIT](./LICENSE)
