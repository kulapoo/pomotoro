# 🚀 Getting Started

## Prerequisites

### Required Tools
- **Rust** (latest stable)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Node.js** (v18+) for UI development
- **Tauri CLI**
  ```bash
  cargo install tauri-cli
  ```

### Recommended Tools
- **rust-analyzer** for IDE support
- **cargo-watch** for auto-recompilation
  ```bash
  cargo install cargo-watch
  ```

## Setting Up Your Development Environment

### 1. Clone the Repository
```bash
git clone https://github.com/yourusername/pomotoro.git
cd pomotoro
```

### 2. Install Dependencies
```bash
# Rust dependencies
cargo build

# UI dependencies
cd apps/react-ui && npm install && npm run build
```

### 3. Run Tests
```bash
# Run all tests
cargo test --workspace

# Run domain tests only
cargo test -p domain

# Run with coverage
cargo tarpaulin --out Html
```

### 4. Start Development Server
```bash
# Run the Tauri app in development mode
cargo tauri dev

# Or run individual components
cargo run -p infra  # Backend only
cd apps/react-ui && npm run dev  # Frontend only
```

## Project Structure Quick Reference

```
pomotoro/
├── core/           # Framework-agnostic core (domain, usecases, infra)
├── apps/
│   ├── tauri-app/  # Tauri desktop client
│   ├── react-ui/   # React + TypeScript frontend
│   ├── pomotoro-cli/
│   └── cosmic-de/
└── docs/          # Documentation
```

## Your First Contribution

### Step 1: Understand the Domain
Start by exploring the domain layer:
```bash
# Read the domain documentation
cat docs/domain.md

# Explore key entities
ls -la domain/src/timer/
ls -la domain/src/task/
```

### Step 2: Pick a Task
Check available issues:
- Look for `good-first-issue` labels
- Read the [TODO list](../../todo.md)
- Check [MVP specs](../../specs/)

### Step 3: Create a Branch
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

### Step 4: Make Changes
Follow the workflow for your task type:
- [Adding a Feature](../workflows/adding-feature.md)
- [Fixing Bugs](../workflows/fixing-bugs.md)

### Step 5: Test Your Changes
```bash
# Run tests for affected modules
cargo test -p domain
cargo test -p usecases

# Run integration tests
cargo test --test '*'

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

### Step 6: Submit PR
1. Push your branch
2. Create a Pull Request
3. Ensure CI passes
4. Request review

## Common Commands

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build specific package
cargo build -p domain
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_timer_start

# Run benchmarks
cargo bench
```

### Development
```bash
# Watch for changes
cargo watch -x test

# Format code
cargo fmt

# Check code
cargo clippy

# Update dependencies
cargo update
```

### Tauri Specific
```bash
# Development mode
cargo tauri dev

# Build for production
cargo tauri build

# Generate icons
cargo tauri icon path/to/icon.png
```

## IDE Setup

### VS Code
1. Install `rust-analyzer` extension
2. Install `Tauri` extension
3. Configure settings:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

### IntelliJ IDEA
1. Install Rust plugin
2. Enable `cargo` integration
3. Configure code style from `rustfmt.toml`

## Troubleshooting

### Common Issues

#### 1. Build Fails
```bash
# Clean and rebuild
cargo clean
cargo build
```

#### 2. Tests Fail
```bash
# Run tests in single thread
cargo test -- --test-threads=1
```

#### 3. Tauri Issues
```bash
# Reset Tauri
rm -rf target/
cargo tauri dev
```

### Getting Help
- Check [existing issues](https://github.com/yourusername/pomotoro/issues)
- Ask in discussions
- Read the [FAQ](../FAQ.md)

## Next Steps

✅ Environment set up  
✅ Project structure understood  
➡️ Explore [Layer Guides](../layers/)  
➡️ Learn [Development Workflow](./workflow.md)  
➡️ Start contributing!