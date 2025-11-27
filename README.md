# CRBasic LSP for Visual Studio Code

A Visual Studio Code extension providing comprehensive language support for CRBasic, the programming language used in Campbell Scientific data loggers.

## Features

- **Syntax Highlighting**: Instant syntax highlighting via TextMate Grammar
- **IntelliSense**: Context-aware code completion for CRBasic instructions and keywords
- **Diagnostics**: Real-time validation with model-specific rules
  - Variable name length validation (model-dependent)
  - Duplicate field name detection (CR200X 12-char truncation)
  - Structure validation (BeginProg/EndProg, Function/EndFunction)
- **Navigation**: Go to definition, find all references
- **Hover Information**: Documentation for built-in instructions

## Supported File Extensions

- `.cr1`, `.cr1x` - CR1000 series
- `.cr2`, `.cr3`, `.cr5` - CR200, CR3000, CR5000 series
- `.cr6` - CR6 series
- `.cr8` - CR800 series
- `.cr9`, `.cr9x`, `.c9x` - CR9000 series
- `.cr300` - CR300 series
- `.crb` - GRANITE series
- `.dld` - Generic datalogger programs

## Technology Stack

- **Client**: TypeScript + Vite
- **LSP Server**: Rust + WebAssembly (WASM)
- **Testing**: Vitest (TypeScript), Cargo test (Rust)

## Installation (Future)

> **Note**: This extension is currently in development and not yet published.

Once published, install from the VSCode Marketplace:

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "CRBasic LSP"
4. Click "Install"

## Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) v18+ and npm
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [pre-commit](https://pre-commit.com/) (optional, for git hooks)

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/crbasic-lsp-rs.git
cd crbasic-lsp-rs

# Install Rust dependencies and build
cargo build

# Build WASM package
cd crates/crbasic-wasm
wasm-pack build --target web
cd ../..

# Install TypeScript dependencies
cd client
npm install

# Build extension
npm run build

# Run in development mode
npm run dev
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run TypeScript tests
cd client
npm test

# Check code coverage
cargo tarpaulin --out Html  # Rust coverage
cd client && npm run test:coverage  # TypeScript coverage
```

### Pre-commit Hooks (Optional)

```bash
# Install pre-commit
pip install pre-commit  # or: brew install pre-commit

# Set up git hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

## Project Structure

```text
crbasic-lsp-rs/
├── crates/                      # Rust workspace
│   ├── crbasic-parser/          # Core parser
│   ├── crbasic-lsp/             # LSP server
│   └── crbasic-wasm/            # WASM bindings
├── client/                      # VSCode extension (TypeScript)
│   ├── src/                     # Extension source
│   └── syntaxes/                # TextMate Grammar
├── docs/                        # Documentation
│   ├── ARCHITECTURE.md          # Architecture overview
│   ├── adrs/                    # Architecture decision records
│   └── researches/              # Research documents
└── README.md                    # This file
```

## Documentation

- [Architecture](./docs/ARCHITECTURE.md): System architecture and design
- [ADRs](./docs/adrs/): Architecture decision records
- [Research](./docs/researches/research-001-crbasic-for-vscode.md): CRBasic language analysis

## Contributing

Contributions are welcome! Please read our development guidelines:

1. **All code, comments, and documentation must be in English** (this is an OSS project)
2. **Test-Driven Development**: Write tests before implementation
3. **Code Quality**: Pass all linters (`cargo clippy`, ESLint) and formatters
4. **Coverage Targets**: 80% line, 75% branch, 90% function coverage
5. **Conventional Commits**: Use conventional commit messages (e.g., `feat:`, `fix:`, `docs:`)

## License

[MIT License](./LICENSE) (TODO: Add license file)

## Acknowledgments

- CRBasic language specification from [Campbell Scientific](https://www.campbellsci.com/)
- Inspired by existing community extensions ([daiwalkr/cr-basic-ms-vscode](https://marketplace.visualstudio.com/items?itemName=daiwalkr.cr-basic-ms-vscode))

## Status

🚧 **Under Active Development** 🚧

Current progress:

- [x] Research and architecture design
- [ ] Project structure setup
- [ ] TextMate Grammar implementation
- [ ] Rust parser implementation
- [ ] LSP server implementation
- [ ] WASM integration
- [ ] VSCode extension client
- [ ] Testing and validation

## Contact

For questions, issues, or contributions, please open an issue on [GitHub](https://github.com/YOUR_USERNAME/crbasic-lsp-rs/issues).
