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
    # Start Vite in its own process group so we can reliably tear it down.
    # `npm run dev` spawns `sh -> vite` as children; killing just npm leaves
    # vite orphaned and still bound to port 5173 (strictPort), which makes the
    # next `just dev` fail with "port already in use" — e.g. after quitting the
    # app from the tray (app.exit terminates Tauri but not the npm child tree).
    setsid bash -c "cd \"$ROOT/apps/react-ui\" && npm install --silent 2>/dev/null && npm run dev" &
    VITE_PGID=$!
    cleanup() { kill -- "-$VITE_PGID" 2>/dev/null || true; }
    trap cleanup EXIT INT TERM
    cd "$ROOT/apps/tauri-app" && RUST_LOG=info cargo tauri dev

# Run development server with debug-level logging
dev-debug:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    setsid bash -c "cd \"$ROOT/apps/react-ui\" && npm install --silent 2>/dev/null && npm run dev" &
    VITE_PGID=$!
    cleanup() { kill -- "-$VITE_PGID" 2>/dev/null || true; }
    trap cleanup EXIT INT TERM
    cd "$ROOT/apps/tauri-app" && RUST_LOG=debug cargo tauri dev

# Run development server with trace-level logging (very verbose)
dev-trace:
    #!/usr/bin/env bash
    set -e
    ROOT="{{justfile_directory()}}"
    setsid bash -c "cd \"$ROOT/apps/react-ui\" && npm install --silent 2>/dev/null && npm run dev" &
    VITE_PGID=$!
    cleanup() { kill -- "-$VITE_PGID" 2>/dev/null || true; }
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
