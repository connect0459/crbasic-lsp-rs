# Setup after clone
setup:
    pre-commit install
    cargo build
    cd client && npm install

# Run Rust tests for a single crate (e.g. `just test-crate crbasic-parser`)
test-crate crate:
    cargo test -p {{crate}}

# Run all Rust tests
test-rust:
    cargo test --workspace

# Run TypeScript tests
test-client:
    cd client && npm test -- --run

# Build the WASM package
build-wasm:
    cd crates/crbasic-wasm && wasm-pack build --target web

# Verify code quality and all workspaces (matches CI)
verify:
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings
    just test-rust
    cd client && npm run lint
    cd client && npm run format:check
    just test-client
