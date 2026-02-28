# Organize Crates: Core + Apps Structure - COMPLETED

**Date**: 2024
**Status**: ✅ Complete and Verified

## Summary

Successfully reorganized the Pomotoro codebase from a flat structure to a hierarchical `core/` and `apps/` layout. This makes the architecture self-documenting and clearly separates the framework-agnostic core engine from client applications.

## Changes Made

### Directory Structure Migration

**Before:**
```
pomotoro/
├── domain/
├── infra/
├── usecases/
├── tauri-app/
└── ui/
```

**After:**
```
pomotoro/
├── core/
│   ├── domain/         # moved from domain/
│   ├── infra/          # moved from infra/
│   └── usecases/       # moved from usecases/
└── apps/
    ├── tauri-app/      # moved from tauri-app/
    ├── leptos-ui/      # moved and renamed from ui/
    ├── pomotoro-cli/   # NEW - stub for future CLI
    └── cosmic-de/      # NEW - stub for future Cosmic DE applet
```

### Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Updated members to new paths, added CLI and Cosmic stubs |
| `apps/tauri-app/Cargo.toml` | Updated path deps: `../../core/domain`, `../../core/usecases`, `../../core/infra` |
| `apps/leptos-ui/Cargo.toml` | Updated domain path: `../../core/domain` |
| `apps/tauri-app/tauri.conf.json` | Updated `frontendDist: "../../dist"` |
| `Trunk.toml` | Updated target: `apps/leptos-ui/index.html`, ignore: `core/infra` |
| `justfile` | Updated all `tauri-app` references to `apps/tauri-app` |
| `dev.sh` | Updated `tauri-app` references to `apps/tauri-app` |
| `README.md` | Updated all path references and project structure diagrams |
| `docs/decouple-infra.md` | Updated all path references to new structure |

### New Stub Crates Created

**`apps/pomotoro-cli/`** - Future CLI client
- `Cargo.toml` with core dependencies
- `src/main.rs` with placeholder

**`apps/cosmic-de/`** - Future Cosmic DE applet
- `Cargo.toml` with core dependencies  
- `src/main.rs` with placeholder

Both stubs:
- Depend on `core/domain`, `core/usecases`, `core/infra`
- Are buildable and runnable
- Added to workspace members

## Verification Results

### ✅ Compilation Check
```bash
cargo check --workspace
```
**Result**: All 7 crates compiled successfully
- core/domain
- core/usecases
- core/infra
- apps/tauri-app
- apps/leptos-ui
- apps/pomotoro-cli
- apps/cosmic-de

### ✅ Build Check
```bash
cargo build --workspace
```
**Result**: All crates built successfully in 11.87s

### ✅ Development Server (just dev)
```bash
just dev
```
**Result**: Trunk serve starts correctly, frontend builds, Tauri app compiles ✅

Required wrapper scripts added to handle Tauri's working directory:
- `apps/trunk-serve.sh` - Runs trunk from project root
- `apps/trunk-build.sh` - Runs trunk from project root

### ✅ Test Suite
```bash
just test
# or: cargo test --workspace
```
**Result**: All 204 tests passed
- Domain: 122 tests ✅
- Infrastructure: 11 tests ✅
- Integration: 71 tests ✅

**Breakdown:**
- Unit tests: 133 tests
- Integration tests: 71 tests
- All pass with no failures, warnings, or ignored tests

### ✅ Stub Applications
```bash
cargo run -p pomotoro-cli
cargo run -p pomotoro-cosmic
```
**Result**: Both stubs execute successfully with placeholder messages

### ✅ Core Infrastructure Independence
```bash
cargo tree -p infra | grep -i tauri
```
**Result**: No Tauri dependencies found in `core/infra` ✅

This confirms the infrastructure layer remains framework-agnostic and reusable.

## Benefits Achieved

### 1. **Self-Documenting Architecture**
The directory structure immediately communicates:
- `core/` = framework-agnostic engine (reusable)
- `apps/` = client implementations (framework-specific)

### 2. **Simplified Onboarding**
New client developers can immediately understand:
- Where to find the core engine (`core/`)
- Where to add new clients (`apps/`)
- How dependencies flow (apps → core)

### 3. **Scalability**
Adding new clients is straightforward:
```bash
mkdir apps/my-new-client
# Create Cargo.toml with core dependencies
# Add to workspace members
```

### 4. **Workspace Organization**
Clear separation of concerns:
- **Core crates** (3): domain, usecases, infra
- **App crates** (4): tauri-app, leptos-ui, pomotoro-cli, cosmic-de

### 5. **Maintained Backward Compatibility**
- All 204 tests pass unchanged
- No functionality broken
- `just dev` and `just test` work as before
- Tauri app builds and runs identically

## Path Dependency Reference

### Within Core (unchanged)
```toml
# core/usecases/Cargo.toml
domain = { path = "../domain" }

# core/infra/Cargo.toml
domain = { path = "../domain" }
usecases = { path = "../usecases" }
```

### Apps to Core (updated)
```toml
# apps/tauri-app/Cargo.toml
domain = { path = "../../core/domain" }
usecases = { path = "../../core/usecases" }
infra = { path = "../../core/infra" }

# apps/leptos-ui/Cargo.toml
domain = { path = "../../core/domain" }

# apps/pomotoro-cli/Cargo.toml
domain = { path = "../../core/domain" }
usecases = { path = "../../core/usecases" }
infra = { path = "../../core/infra" }

# apps/cosmic-de/Cargo.toml
domain = { path = "../../core/domain" }
usecases = { path = "../../core/usecases" }
infra = { path = "../../core/infra" }
```

## Trunk Wrapper Scripts Fix

When running `cargo tauri dev` from `apps/tauri-app/`, Tauri executes the `beforeDevCommand` from the `apps/` directory, not from `apps/tauri-app/`. However, Trunk needs to run from the project root where `Trunk.toml` is located.

**Solution**: Created wrapper scripts in `apps/` directory:

**`apps/trunk-serve.sh`:**
```bash
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."  # Navigate to project root
exec trunk serve
```

**`apps/trunk-build.sh`:**
```bash
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."  # Navigate to project root
exec trunk build
```

**Updated `apps/tauri-app/tauri.conf.json`:**
```json
{
  "build": {
    "beforeDevCommand": "bash ./trunk-serve.sh",
    "beforeBuildCommand": "bash ./trunk-build.sh",
    "frontendDist": "../../dist"
  }
}
```

This ensures `just dev` works correctly with the new directory structure.

## Git History

All moves performed with `git mv` to preserve history:
```bash
git mv domain core/domain
git mv usecases core/usecases
git mv infra core/infra
git mv tauri-app apps/tauri-app
git mv ui apps/leptos-ui
```

## Future Work

With this structure in place, adding new clients is straightforward:

### CLI Client (apps/pomotoro-cli)
- Implement terminal UI with crossterm/ratatui
- Use core engine for all pomodoro logic
- Desktop notifications via notify-rust

### Cosmic DE Applet (apps/cosmic-de)
- Integrate with libcosmic and cosmic-panel
- System tray with timer display
- D-Bus notifications

### Other Potential Clients
- **GTK4 App**: Native GNOME/GTK application
- **Slint App**: Modern declarative UI
- **egui App**: Immediate mode GUI
- **Web Service**: REST API exposing core functionality

All clients can reuse `core/infra` without pulling in Tauri dependencies.

## Commands Reference

### Development
```bash
just dev          # Start Tauri app in dev mode
just test         # Run all tests
just build        # Build for production
```

### Building Specific Crates
```bash
# Core engine
cargo build -p domain -p usecases -p infra

# Specific app
cargo build -p tauri-app
cargo build -p pomotoro-cli
cargo build -p pomotoro-cosmic

# All apps
cargo build --workspace
```

### Running Stub Apps
```bash
cargo run -p pomotoro-cli      # "Pomotoro CLI - Coming soon!"
cargo run -p pomotoro-cosmic   # "Pomotoro Cosmic DE Applet - Coming soon!"
```

## Conclusion

The reorganization is **complete and fully verified**. The codebase now clearly communicates its architecture through directory structure, making it easier for new contributors to understand and extend. All existing functionality preserved with zero test failures.

The foundation is set for adding multiple client applications (CLI, Cosmic DE, etc.) that can all share the same robust, framework-agnostic core engine.