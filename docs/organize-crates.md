# Organize Crates: Core + Apps Structure

## Context

The project is preparing for multiple client applications (CLI, Cosmic DE applet, Tauri desktop, Leptos web) that share the same pomodoro engine. The current flat crate layout (`domain/`, `infra/`, `usecases/`, `tauri-app/`, `ui/`) doesn't communicate this core-vs-client distinction. Reorganizing into `core/` and `apps/` directories makes the architecture self-documenting and simplifies onboarding for new client developers.

**Good news:** `infra/` is already Tauri-free (decoupling from `docs/decouple-infra.md` is done). This is purely a directory reorganization + workspace config update.

## Target Layout

```
pomotoro/
├── Cargo.toml              # workspace root
├── core/
│   ├── domain/             # was: domain/
│   ├── infra/              # was: infra/
│   └── usecases/           # was: usecases/
├── apps/
│   ├── tauri-app/          # was: tauri-app/
│   ├── leptos-ui/          # was: ui/  (renamed)
│   ├── pomotoro-cli/       # NEW - future CLI client
│   └── cosmic-de/          # NEW - future Cosmic DE applet
├── docs/
└── assets/
```

## Implementation Steps

### Step 1: Create directory structure

```bash
mkdir -p core apps
```

### Step 2: Move core crates

```bash
git mv domain core/domain
git mv usecases core/usecases
git mv infra core/infra
```

### Step 3: Move app crates

```bash
git mv tauri-app apps/tauri-app
git mv ui apps/leptos-ui
```

### Step 4: Update workspace root `Cargo.toml`

```toml
[workspace]
members = [
    "core/domain",
    "core/usecases",
    "core/infra",
    "apps/tauri-app",
    "apps/leptos-ui",
]
```

### Step 5: Fix all path dependencies

**`core/usecases/Cargo.toml`:**
- `domain = { path = "../domain" }` (unchanged - still relative within core/)

**`core/infra/Cargo.toml`:**
- `domain = { path = "../domain" }` (unchanged)
- `usecases = { path = "../usecases" }` (unchanged)

**`apps/tauri-app/Cargo.toml`:**
- `domain = { path = "../../core/domain" }`
- `usecases = { path = "../../core/usecases" }`
- `infra = { path = "../../core/infra" }`

**`apps/leptos-ui/Cargo.toml`:**
- `domain = { path = "../../core/domain" }` (if it has a domain dep)

**`core/infra/Cargo.toml` dev-dependencies:**
- `domain = { path = "../domain", features = ["test-utils"] }` (unchanged)

### Step 6: Fix Tauri config paths

- `apps/tauri-app/tauri.conf.json` — update `frontendDist` and any relative paths that reference `../ui/` (now `../leptos-ui/` or `../../...`)
- `apps/tauri-app/build.rs` — verify no hardcoded paths
- `apps/leptos-ui/Trunk.toml` — verify dist/output paths still work

### Step 7: Scaffold future app crates (empty stubs)

**`apps/pomotoro-cli/Cargo.toml`:**
```toml
[package]
name = "pomotoro-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
domain = { path = "../../core/domain" }
usecases = { path = "../../core/usecases" }
infra = { path = "../../core/infra" }
tokio = { workspace = true }
```

**`apps/pomotoro-cli/src/main.rs`:** minimal `fn main()` stub

**`apps/cosmic-de/Cargo.toml`:**
```toml
[package]
name = "pomotoro-cosmic"
version = "0.1.0"
edition = "2021"

[dependencies]
domain = { path = "../../core/domain" }
usecases = { path = "../../core/usecases" }
infra = { path = "../../core/infra" }
tokio = { workspace = true }
```

**`apps/cosmic-de/src/main.rs`:** minimal `fn main()` stub

Add both to workspace members.

### Step 8: Update docs references

- Grep docs/ for old paths (`infra/`, `domain/`, `usecases/`, `ui/`, `tauri-app/`) and update to new locations
- Update `docs/decouple-infra.md` paths

### Step 9: Update any CI/scripts

- Check for GitHub Actions, Makefiles, or shell scripts referencing old paths

## Files Modified

| File | Change |
|---|---|
| `Cargo.toml` (root) | Update workspace members |
| `apps/tauri-app/Cargo.toml` | Fix path deps |
| `apps/leptos-ui/Cargo.toml` | Fix path deps (if any) |
| `apps/tauri-app/tauri.conf.json` | Fix relative paths |
| `apps/leptos-ui/Trunk.toml` | Verify paths |
| `apps/pomotoro-cli/` | New stub crate |
| `apps/cosmic-de/` | New stub crate |
| `docs/*` | Update path references |

## Verification

```bash
# Build all core crates
cargo build -p domain -p usecases -p infra

# Build all app crates
cargo build -p tauri-app
cargo build -p pomotoro-cli
cargo build -p pomotoro-cosmic

# Run all tests
cargo test --workspace

# Verify Tauri app still works
cd apps/tauri-app && cargo tauri dev

# Verify no broken path deps
cargo tree --workspace
```
