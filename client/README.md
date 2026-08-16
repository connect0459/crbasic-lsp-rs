# CRBasic Language Support

CRBasic language support as a Visual Studio Code extension.

## Features

- **Syntax Highlighting**: TextMate grammar-based highlighting for CRBasic keywords, built-in instructions, operators, and literals.
- **IntelliSense**: Context-aware code completion for CRBasic instructions and keywords, with parameter signature help.
- **Diagnostics**: Real-time validation with datalogger-model-specific rules.
  - Variable name length validation (model-dependent).
  - Duplicate field name detection (CR200X 12-character truncation collisions).
  - Structure validation (`BeginProg`/`EndProg`, `Function`/`EndFunction`, and other block constructs).
- **Navigation**: Go to definition, find all references, document symbols, workspace symbols, call hierarchy, rename, and linked editing ranges.
- **Hover Information**: Documentation for built-in instructions, including parameter descriptions.
- **Editing Aids**: Signature help, code actions (quick fixes), code lens, folding ranges, selection ranges, and inlay hints.
- **Semantic Highlighting**: Semantic tokens layered on top of the TextMate grammar.

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

No external dependencies. The language server is bundled with the extension.

## Disclaimer

This is an independent, unofficial extension and is not affiliated with or endorsed by Campbell Scientific.

## Contributing

See [CONTRIBUTING.md](https://github.com/connect0459/crbasic-lsp-rs/blob/main/CONTRIBUTING.md).

## License

[MIT](https://github.com/connect0459/crbasic-lsp-rs/blob/main/LICENSE)
