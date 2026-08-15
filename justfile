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

# Branch coverage needs a nightly toolchain and crashed locally with this
# host's cargo-llvm-cov, so only line/function coverage is gated here.
coverage:
    cargo llvm-cov --workspace --fail-under-lines 80 --fail-under-functions 90

# Run TypeScript tests
test-client:
    cd client && npm run test.run

# Build the WASM package
build-wasm:
    cd crates/crbasic-wasm && wasm-pack build --target web

# Regenerate keywords_generated.rs and crbasic.tmLanguage.json from keywords.json
generate-grammar:
    node scripts/generate-grammar.js

# Verify code quality and all workspaces (matches CI)
verify:
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings
    just test-rust
    just coverage
    node scripts/generate-grammar.js --check
    cd client && npm run lint
    cd client && npm run format.check
    just test-client
