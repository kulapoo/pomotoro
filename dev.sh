#!/bin/bash

# Pomotoro development script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_msg() {
    echo -e "${GREEN}[Pomotoro]${NC} $1"
}

print_error() {
    echo -e "${RED}[Error]${NC} $1"
}

# Main command handling
case "$1" in
    dev)
        print_msg "Starting development server..."
        cd infra && cargo tauri dev
        ;;
    build)
        print_msg "Building for production..."
        cd infra && cargo tauri build
        ;;
    serve)
        print_msg "Starting frontend dev server only..."
        cd infra && trunk serve --config Trunk.toml
        ;;
    build-frontend)
        print_msg "Building frontend only..."
        cd infra && trunk build --config Trunk.toml
        ;;
    test)
        print_msg "Running tests..."
        cargo test --workspace
        ;;
    check)
        print_msg "Checking code..."
        cargo check --workspace
        ;;
    fmt)
        print_msg "Formatting code..."
        cargo fmt --all
        ;;
    clippy)
        print_msg "Running clippy..."
        cargo clippy --workspace -- -D warnings
        ;;
    clean)
        print_msg "Cleaning build artifacts..."
        cargo clean
        rm -rf dist
        ;;
    *)
        echo "Usage: $0 {dev|build|serve|build-frontend|test|check|fmt|clippy|clean}"
        echo ""
        echo "Commands:"
        echo "  dev             - Run development server"
        echo "  build           - Build for production"
        echo "  serve           - Run frontend dev server only"
        echo "  build-frontend  - Build frontend only"
        echo "  test            - Run tests"
        echo "  check           - Check code"
        echo "  fmt             - Format code"
        echo "  clippy          - Run clippy"
        echo "  clean           - Clean build artifacts"
        exit 1
        ;;
esac