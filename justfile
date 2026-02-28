# Pomotoro Development Commands

# Run development server with info-level logging
dev:
    cd tauri-app && RUST_LOG=info cargo tauri dev

# Run development server with debug-level logging
dev-debug:
    cd tauri-app && RUST_LOG=debug cargo tauri dev

# Run development server with trace-level logging (very verbose)
dev-trace:
    cd tauri-app && RUST_LOG=trace cargo tauri dev

# Build for production
build:
    cd tauri-app && cargo tauri build

# Build frontend only
build-frontend:
    trunk build

# Run frontend dev server only
serve:
    trunk serve

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist

# Run tests
test:
    cargo test --workspace

# Run only infrastructure tests
test-infra:
    cargo test -p infra

# Run only domain tests
test-domain:
    cargo test -p domain

# Run only usecases tests
test-usecases:
    cargo test -p usecases

# Check code
check:
    cargo check --workspace

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Install dependencies
install:
    cargo build --workspace
    cd tauri-app && cargo tauri build --help > /dev/null || cargo install tauri-cli

# Verify infrastructure decoupling
verify:
    ./verify-decoupling.sh

# Build just the framework-agnostic core
build-core:
    cargo build -p infra -p domain -p usecases
