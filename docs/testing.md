# Testing Guide - Pomotoro

## Quick Commands

```bash
# Run all tests
cargo test

# Domain tests only
cargo test -p domain

# Infrastructure tests only
cargo test -p infra

# Specific modules
cargo test -p domain task
cargo test -p infra app
cargo test -p infra timer

# Run specific test by name
cargo test test_name -- --exact

# Run tests matching a pattern
cargo test test_pattern

# Verbose output
cargo test -- --nocapture

# Single test with output
cargo test test_name -- --exact --nocapture
```

## Test Structure

```
pomotoro/
├── domain/              # Unit tests in source files (#[cfg(test)])
├── usecases/           # Use case tests (if applicable)
├── infra/tests/        # Infrastructure integration tests
│   ├── app/           # Application integration tests
│   │   ├── task.rs    # Task-related tests
│   │   ├── timer.rs   # Timer-related tests
│   │   └── setup.rs   # Setup tests
│   ├── core/          # Test utilities and helpers
│   │   ├── context/   # Test context builders
│   │   ├── database/  # Test database utilities
│   │   ├── fixtures/  # Test data fixtures
│   │   ├── mocks/     # Mock implementations
│   │   └── utils.rs   # Test utilities
│   └── main.rs        # Test entry point
└── ui/                 # UI tests (if applicable)
```

## Domain Tests (`domain/`)

Unit tests embedded in source files testing pure business logic:

```bash
cargo test -p domain                    # All domain tests
cargo test -p domain task              # Task logic
cargo test -p domain timer             # Timer logic
cargo test -p domain shared_kernel     # Value objects
cargo test -p domain audio             # Audio logic
```

## Infrastructure Tests (`infra/`)

Integration tests in dedicated `tests/` directory:

```bash
cargo test -p infra                    # All infra tests
cargo test -p infra app::task          # Task integration tests
cargo test -p infra app::timer         # Timer integration tests
cargo test -p infra app::setup         # Setup tests
cargo test -p infra --test main        # Run all integration tests explicitly
```

## Running Specific Tests

### By Test Function Name

```bash
# Run a specific test function
cargo test -p infra test_timer_start -- --exact

# Run all tests containing "timer" in their name
cargo test -p infra timer

# Run tests in a specific module
cargo test -p infra app::timer::
```

### By File or Module Path

```bash
# Run tests from a specific module path
cargo test -p infra app::task

# Run tests with full module path
cargo test -p infra tests::app::timer



cargo test --package infra --test main -- --nocapture -- app::timer:: --exact --show-output
cargo test --package infra --test main  -- app::timer::timer_should_start_from_idle_state --exact --show-output
```

### With Filters and Options

```bash
# Run tests matching pattern with output
cargo test -p infra timer -- --nocapture

# Run specific test showing println! output
cargo test -p infra test_create_task -- --exact --nocapture

# cargo test -p infra  -- --exact --nocapture


# Run ignored tests only
cargo test -p infra -- --ignored

# Run all tests including ignored
cargo test -p infra -- --include-ignored
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

# Watch specific package tests
cargo watch -x "test -p infra"

# Run with backtrace on failure
RUST_BACKTRACE=1 cargo test

# Run with full backtrace
RUST_BACKTRACE=full cargo test
```

## Test Organization Tips

### Workspace-Level Testing

```bash
# Run all workspace tests
cargo test --workspace

# Run tests for specific workspace members
cargo test -p domain -p infra

# Exclude certain packages
cargo test --workspace --exclude ui
```

### Performance Testing

```bash
# Run tests with release optimizations
cargo test --release

# Profile test execution time
cargo test -- --report-time --test-threads=1
```
