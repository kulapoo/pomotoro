# Pomotoro Development Commands

# Run development server with info-level logging
dev:
    cd infra && RUST_LOG=info cargo tauri dev

# Run development server with debug-level logging
dev-debug:
    cd infra && RUST_LOG=debug cargo tauri dev

# Run development server with trace-level logging (very verbose)
dev-trace:
    cd infra && RUST_LOG=trace cargo tauri dev

# Build for production
build:
    cd infra && cargo tauri build

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
    cd infra && cargo tauri build --help > /dev/null || cargo install tauri-cli