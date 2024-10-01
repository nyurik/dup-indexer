#!/usr/bin/env just --justfile

@_default:
    just --list --unsorted

# Clean all build artifacts
clean:
    cargo clean
    rm -f Cargo.lock

update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

build:
    cargo build --all-targets

# Run cargo clippy
clippy:
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-targets --all-features -- -D warnings

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

# Run cargo fmt
fmt:
    cargo +nightly fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Quick compile
check:
    RUSTFLAGS='-D warnings' cargo check --workspace --all-targets

# Run all tests
test:
    RUSTFLAGS='-D warnings' cargo test --workspace --all-targets

# Test documentation
test-doc:
    RUSTFLAGS='-D warnings' cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

rust-info:
    rustc --version
    cargo --version

# Run all tests as expected by CI
ci-test: rust-info test-fmt clippy check test test-doc

# Run benchmarks
bench:
    cargo bench -p bench
    open target/criterion/DupIndexer/report/index.html

# Run Miri test
miri: rust-info
    RUSTFLAGS='-D warnings' cargo +nightly miri test
