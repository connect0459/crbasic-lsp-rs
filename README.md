# crbasic-lsp-rs

[![CI](https://github.com/connect0459/crbasic-lsp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/connect0459/crbasic-lsp-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/connect0459/crbasic-lsp-rs/blob/main/LICENSE)
[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Marketplace-blue.svg?logo=visualstudiocode&logoColor=white)](https://marketplace.visualstudio.com/items?itemName=connect0459.crbasic-lsp-rs-vsce)

A Visual Studio Code extension providing comprehensive language support for CRBasic, the programming language used in Campbell Scientific data loggers.

## Features

- **Syntax Highlighting**: Instant syntax highlighting via TextMate Grammar
- **IntelliSense**: Context-aware code completion for CRBasic instructions and keywords
- **Diagnostics**: Real-time validation with model-specific rules
  - Variable name length validation (model-dependent)
  - Duplicate field name detection (CR200X 12-char truncation)
  - Structure validation (BeginProg/EndProg, Function/EndFunction)
- **Navigation**: Go to definition, find all references, document symbols, workspace symbols, call hierarchy, and rename (with linked editing ranges)
- **Hover Information**: Documentation for built-in instructions
- **Editing Aids**: Signature help, code actions (quick fixes), code lens, folding ranges, selection ranges, and inlay hints
- **Semantic Highlighting**: Semantic tokens layered on top of the TextMate grammar

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

## Installation

Install from the VSCode Marketplace:

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "CRBasic LSP"
4. Click "Install"

## Disclaimer

This is an independent, unofficial extension and is not affiliated with or endorsed by Campbell Scientific.

## Documentation

- [Architecture](https://github.com/connect0459/crbasic-lsp-rs/blob/main/docs/ARCHITECTURE.md): System architecture and design
- [ADRs](https://github.com/connect0459/crbasic-lsp-rs/blob/main/docs/adrs/): Architecture decision records

## Contributing

See [CONTRIBUTING.md](https://github.com/connect0459/crbasic-lsp-rs/blob/main/CONTRIBUTING.md).

## License

[MIT](https://github.com/connect0459/crbasic-lsp-rs/blob/main/LICENSE)
