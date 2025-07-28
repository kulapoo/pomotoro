# Testing Guide - Pomotoro

## Quick Commands

```bash
# Run all tests
cargo test

# Domain tests only
cargo test -p pomotoro-domain

# Application tests only  
cargo test -p pomotoro

# Specific modules
cargo test -p pomotoro-domain task
cargo test -p pomotoro timer::integration

# Verbose output
cargo test -- --nocapture

# Single test
cargo test test_name -- --exact --nocapture
```

## Test Structure

```
pomotoro/
├── pomotoro-domain/     # Unit tests in source files (#[cfg(test)])
└── src-tauri/tests/     # Integration & E2E tests
    ├── task/
    ├── timer/
    ├── audio/
    └── core/
```

## Domain Tests (`pomotoro-domain/`)

Unit tests embedded in source files testing pure business logic:

```bash
cargo test -p pomotoro-domain task          # Task logic
cargo test -p pomotoro-domain timer         # Timer logic  
cargo test -p pomotoro-domain shared_kernel # Value objects
cargo test -p pomotoro-domain audio         # Audio logic
```

## Application Tests (`src-tauri/`)

Integration and E2E tests in dedicated `tests/` directory:

```bash
cargo test -p pomotoro task::integration    # Task use cases
cargo test -p pomotoro timer::integration   # Timer workflows
cargo test -p pomotoro task::e2e           # End-to-end flows
cargo test -p pomotoro --test main         # All integration tests
```

## Debugging

```bash
# Serial execution (for stateful tests)
cargo test -- --test-threads=1

# Show timing
cargo test -- --report-time

# Run ignored tests  
cargo test -- --ignored

# Watch mode
cargo watch -x test
```