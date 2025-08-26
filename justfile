# Pomotoro Development Commands

# Run development server
dev:
    cd infra && RUST_LOG=infra=info cargo tauri dev

# Build for production
build:
    cd infra && cargo tauri build

# Build frontend only
build-frontend:
    cd infra && trunk build --config Trunk.toml

# Run frontend dev server only
serve:
    cd infra && trunk serve --config Trunk.toml

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