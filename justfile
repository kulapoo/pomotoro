# Pomotoro Development Commands

# List all available commands (default)
default:
    @just --list

# ==============================================================================
# Development & Building
# ==============================================================================

# Run development server (starts Vite + Tauri together)
dev: (_dev "info")

# Run development server with debug-level logging
dev-debug: (_dev "debug")

# Run development server with trace-level logging (very verbose)
dev-trace: (_dev "trace")

# (private) Shared dev recipe — starts Vite then launches Tauri.
# `npm run dev` spawns `sh -> vite` as children; killing just npm leaves vite
# orphaned and still bound to port 5173 (strictPort), which makes the next
# `just dev` fail with "port already in use" — e.g. after quitting the app from
# the tray (app.exit terminates Tauri but not the npm child tree).
# `setsid` creates a new session so we can kill the whole process group on
# Linux; on macOS (where `setsid` is unavailable) we fall back to freeing the
# port directly via `lsof`.
_dev level:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    VITE_CMD="cd \"$ROOT/apps/react-ui\" && npm install --silent 2>/dev/null && npm run dev"
    if command -v setsid >/dev/null 2>&1; then
        setsid bash -c "$VITE_CMD" &
    else
        bash -c "$VITE_CMD" &
    fi
    VITE_PID=$!
    cleanup() {
        kill "$VITE_PID" 2>/dev/null || true
        pkill -P "$VITE_PID" 2>/dev/null || true
        if command -v lsof >/dev/null 2>&1; then
            lsof -ti:5173 2>/dev/null | xargs kill 2>/dev/null || true
        fi
    }
    trap cleanup EXIT INT TERM
    cd "$ROOT/apps/tauri-app" && RUST_LOG="{{level}}" cargo tauri dev

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

# Run all checks (test, check, fmt, clippy, react checks)
ci: test check fmt clippy check-react
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

# Lint + typecheck the React UI (no build artifacts produced)
check-react:
    cd apps/react-ui && npm run lint && npm run typecheck

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
