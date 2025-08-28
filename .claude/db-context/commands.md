# Development Commands Reference

## Primary Commands (justfile)

### Development
```bash
just dev           # Run development server with hot reload
just serve         # Run frontend dev server only
```

### Building
```bash
just build         # Build production release
just build-frontend # Build frontend only
```

### Testing
```bash
just test          # Run all workspace tests
cargo test --package domain    # Test domain layer
cargo test --package usecases  # Test use cases
cargo test --package infra     # Test infrastructure
```

### Code Quality
```bash
just check         # Type check entire workspace
just fmt           # Format all code
just clippy        # Run linter with warnings
```

### Cleanup
```bash
just clean         # Remove build artifacts
cargo clean        # Clean cargo build
```

### Installation
```bash
just install       # Install dependencies and tools
```

## Cargo Commands

### Build Commands
```bash
cargo build --workspace         # Build debug
cargo build --release          # Build optimized
cargo build -p domain          # Build specific package
```

### Test Commands
```bash
cargo test                     # Run all tests
cargo test -- --nocapture      # Show print output
cargo test test_name           # Run specific test
cargo test --lib               # Unit tests only
cargo test --test "*"          # Integration tests
```

### Check Commands
```bash
cargo check                    # Quick compile check
cargo clippy                   # Linting
cargo fmt --check              # Format check
```

### Documentation
```bash
cargo doc --open               # Generate and open docs
cargo doc --no-deps            # Docs without dependencies
```

## Tauri Commands

### Development
```bash
cd infra && cargo tauri dev    # Run Tauri dev mode
RUST_LOG=debug cargo tauri dev # With debug logging
RUST_LOG=infra=info cargo tauri dev # Specific logging
```

### Building
```bash
cd infra && cargo tauri build  # Production build
cargo tauri build --debug      # Debug build
```

### Icons
```bash
cargo tauri icon               # Generate app icons
```

## Database Commands

### Diesel CLI
```bash
cd infra
diesel setup                   # Create database
diesel migration run           # Run migrations
diesel migration revert        # Revert last migration
diesel migration redo          # Revert and rerun
diesel print-schema            # Print schema.rs
```

### Creating Migrations
```bash
cd infra
diesel migration generate migration_name
```

## Frontend Commands (Trunk)

### Development
```bash
cd infra
trunk serve --config Trunk.toml
trunk serve --port 8080        # Custom port
```

### Building
```bash
cd infra
trunk build --config Trunk.toml
trunk build --release          # Optimized build
```

### Watching
```bash
trunk watch                    # Auto rebuild on changes
```

## Testing Patterns

### Run Specific Test File
```bash
cargo test --test timer_tests
cargo test --test task_tests
```

### Run with Features
```bash
cargo test --features test-utils
cargo test --all-features
```

### Integration Tests
```bash
cd infra
cargo test --test "*" -- --test-threads=1
```

### Show Test Output
```bash
cargo test -- --show-output
cargo test -- --nocapture
```

## Debugging Commands

### Logging
```bash
RUST_LOG=debug cargo run       # All debug logs
RUST_LOG=domain=debug cargo run # Domain debug only
RUST_LOG=infra::adapters=trace cargo run # Specific module
```

### Backtrace
```bash
RUST_BACKTRACE=1 cargo run     # Short backtrace
RUST_BACKTRACE=full cargo run  # Full backtrace
```

### Profile
```bash
cargo build --profile=release-with-debug
cargo flamegraph               # Generate flamegraph
```

## Git Workflow

### Status Check
```bash
git status
git diff
git log --oneline -10
```

### Branching
```bash
git checkout -b feature/name
git checkout main
git merge feature/name
```

### Committing
```bash
git add -A
git commit -m "feat: description"
git push origin branch-name
```

## Environment Variables

### Development
```bash
RUST_LOG=debug                 # Logging level
DATABASE_URL=sqlite://app.db   # Database location
CARGO_TARGET_DIR=/tmp/target   # Custom build directory
```

### Testing
```bash
TEST_DATABASE_URL=:memory:     # In-memory test DB
RUST_TEST_THREADS=1            # Serial test execution
```

## Quick Workflows

### Start Development
```bash
just dev
```

### Run Tests Before Commit
```bash
just test && just check && just fmt
```

### Full Quality Check
```bash
just fmt && just clippy && just test
```

### Clean Build
```bash
just clean && just build
```

### Database Reset
```bash
cd infra
diesel migration revert --all
diesel migration run
```

## Package-Specific Commands

### Domain Only
```bash
cargo build -p domain
cargo test -p domain
cargo doc -p domain --open
```

### Use Cases Only
```bash
cargo build -p usecases
cargo test -p usecases
```

### Infrastructure Only
```bash
cargo build -p infra
cargo test -p infra
cd infra && cargo run
```

### UI Only
```bash
cargo build -p ui
cd infra && trunk build
```

## Common Flags

### Cargo Flags
- `--workspace` - All packages
- `--package name` or `-p name` - Specific package
- `--release` - Optimized build
- `--features feat` - Enable features
- `--no-default-features` - Disable defaults
- `--target target` - Cross compile
- `--verbose` or `-v` - Verbose output
- `--quiet` or `-q` - Minimal output

### Test Flags
- `--nocapture` - Show print output
- `--test-threads=1` - Serial execution
- `--ignored` - Run ignored tests
- `--include-ignored` - Run all tests