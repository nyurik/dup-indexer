#!/usr/bin/env just --justfile

@_default:
    just --list --unsorted

# Run cargo fmt and cargo clippy
lint: fmt clippy

# Run cargo fmt
fmt:
    cargo +nightly fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate

# Run cargo clippy
clippy:
    cargo clippy --workspace --all-targets --bins --tests --lib --benches -- -D warnings

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Run all tests
test:
    ./.cargo-husky/hooks/pre-push

# Run Miri test
miri:
    cargo +nightly miri test

# Run benchmarks
bench:
    cargo bench -p bench
    open target/criterion/DupIndexer/report/index.html