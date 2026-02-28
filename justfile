# Pomotoro Development Commands

# List all available commands (default)
default:
    @just --list

# ==============================================================================
# Development & Building
# ==============================================================================

# Run development server with info-level logging
dev:
    cd apps/tauri-app && RUST_LOG=info cargo tauri dev

# Run development server with debug-level logging
dev-debug:
    cd apps/tauri-app && RUST_LOG=debug cargo tauri dev

# Run development server with trace-level logging (very verbose)
dev-trace:
    cd apps/tauri-app && RUST_LOG=trace cargo tauri dev

# Build for production
build:
    cd apps/tauri-app && cargo tauri build

# Build frontend only
build-frontend:
    trunk build

# Build just the framework-agnostic core
build-core:
    cargo build -p infra -p domain -p usecases

# Run frontend dev server only
serve:
    trunk serve

# ==============================================================================
# Testing & Quality
# ==============================================================================

# Run all tests
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

# Run all checks (test, check, fmt, clippy)
ci: test check fmt clippy
    @echo "✅ All checks passed!"

# Check code without building
check:
    cargo check --workspace

# Format code
fmt:
    cargo fmt --all

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
clippy:
    cargo clippy --workspace -- -D warnings

# ==============================================================================
# Setup & Installation
# ==============================================================================

# Install system dependencies (Linux only)
install-deps:
    ./scripts/install-deps.sh

# Build all Rust dependencies
install:
    cargo build --workspace

# ==============================================================================
# Cleanup
# ==============================================================================

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist
