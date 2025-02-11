#!/usr/bin/env just --justfile

@_default:
    just --list

# Clean all build artifacts
clean:
    cargo clean
    rm -f Cargo.lock

# Update dependencies, including breaking changes
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:
    cargo msrv find --write-msrv -- cargo check --all-targets --workspace

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

# Quick compile for MSRV, without compiling benches
check-msrv:
    RUSTFLAGS='-D warnings' cargo check --all-targets

# Run all tests
test:
    RUSTFLAGS='-D warnings' cargo test --workspace --all-targets

# Run all tests
test-msrv:
    RUSTFLAGS='-D warnings' cargo test --all-targets

# Test documentation
test-doc:
    RUSTFLAGS='-D warnings' cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

rust-info:
    rustc --version
    cargo --version

# Run all tests as expected by CI
ci-test: rust-info test-fmt clippy check test test-doc

# Run minimal subset of tests to ensure compatibility with MSRV
ci-test-msrv: rust-info check-msrv test-msrv

# Run benchmarks
bench:
    cargo bench -p bench
    open target/criterion/DupIndexer/report/index.html

# Run Miri test
miri: rust-info
    RUSTFLAGS='-D warnings' cargo +nightly miri test

# Verify that the current version of the crate is not the same as the one published on crates.io
check-if-published:
    #!/usr/bin/env bash
    LOCAL_VERSION="$(grep '^version =' Cargo.toml | sed -E 's/version = "([^"]*)".*/\1/')"
    echo "Detected crate version:  $LOCAL_VERSION"
    CRATE_NAME="$(grep '^name =' Cargo.toml | head -1 | sed -E 's/name = "(.*)"/\1/')"
    echo "Detected crate name:     $CRATE_NAME"
    PUBLISHED_VERSION="$(cargo search ${CRATE_NAME} | grep "^${CRATE_NAME} =" | sed -E 's/.* = "(.*)".*/\1/')"
    echo "Published crate version: $PUBLISHED_VERSION"
    if [ "$LOCAL_VERSION" = "$PUBLISHED_VERSION" ]; then
        echo "ERROR: The current crate version has already been published."
        exit 1
    else
        echo "The current crate version has not yet been published."
    fi
