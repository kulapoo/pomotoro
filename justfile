# Pomotoro Development Commands

# List all available commands (default)
default:
    @just --list

# ==============================================================================
# Development & Building
# ==============================================================================

# Run development server (starts Vite + Tauri together)
dev:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    cd "$ROOT/apps/react-ui" && npm install --silent 2>/dev/null && npm run dev &
    VITE_PID=$!
    cleanup() { kill "$VITE_PID" 2>/dev/null || true; }
    trap cleanup EXIT INT TERM
    cd "$ROOT/apps/tauri-app" && RUST_LOG=info cargo tauri dev

# Run development server with debug-level logging
dev-debug:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    cd "$ROOT/apps/react-ui" && npm install --silent 2>/dev/null && npm run dev &
    VITE_PID=$!
    cleanup() { kill "$VITE_PID" 2>/dev/null || true; }
    trap cleanup EXIT INT TERM
    cd "$ROOT/apps/tauri-app" && RUST_LOG=debug cargo tauri dev

# Run development server with trace-level logging (very verbose)
dev-trace:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    cd "$ROOT/apps/react-ui" && npm install --silent 2>/dev/null && npm run dev &
    VITE_PID=$!
    cleanup() { kill "$VITE_PID" 2>/dev/null || true; }
    trap cleanup EXIT INT TERM
    cd "$ROOT/apps/tauri-app" && RUST_LOG=trace cargo tauri dev

# Build for production (builds React UI first, then Tauri app)
build: clippy fmt-check build-react
    cd apps/tauri-app && cargo tauri build

# Build React UI frontend only
build-react:
    cd apps/react-ui && npm run build

# Run React UI dev server only (without Tauri)
serve-react:
    cd apps/react-ui && npm run dev

# Install React UI npm dependencies
install-react:
    cd apps/react-ui && npm install

# Build just the framework-agnostic core
build-core:
    cargo build -p infra -p domain -p usecases

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

# Install git hooks
install-hooks:
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

# Build all Rust dependencies
install: install-hooks
    cargo build --workspace

# ==============================================================================
# Cleanup
# ==============================================================================

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist
