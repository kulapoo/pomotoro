# Pomotoro 🍅🐂

A powerful Pomodoro timer application with the strength of a bull! Built with Tauri and Leptos, combining the speed of Rust with a fast, reactive web frontend.

*Stay focused and productive with Pomotoro – charge through distractions like a determined toro.*

## Features

- 🍅 **Classic Pomodoro technique** - 25min work, 5min break, 15min long break cycles
- ⏱️ **Visual circular timer** with smooth progress indication
- 🎮 **Simple controls** - Start, Pause, Reset, Skip
- 📊 **Session tracking and statistics** - Monitor your productivity
- 🔔 **Desktop notifications** for seamless phase transitions
- ⚙️ **Customizable timer durations** - Adapt to your workflow
- 🎨 **Clean, modern UI** with smooth animations
- 📱 **Cross-platform support** - Windows, macOS, Linux
- ⚡ **Lightning-fast performance** - Native speed with minimal resources

## Technology Stack

- **Frontend**: [Leptos](https://leptos.dev/) - Fast, reactive Rust web framework compiled to WebAssembly
- **Backend**: [Tauri](https://tauri.app/) - Secure, lightweight desktop app framework
- **Language**: Rust - Memory safe, blazing fast native performance
- **Database**: SQLite with Diesel ORM for session persistence
- **Architecture**: Clean Architecture with Domain-Driven Design, decoupled infrastructure layer

## Quick Start

### Prerequisites

#### 1. Rust Toolchain
- Install [Rust](https://rustup.rs/) (latest stable)
- Add WebAssembly target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
  *This target is REQUIRED for compiling the Leptos frontend to WebAssembly*

#### 2. Build Tools
Install the following CLI tools that orchestrate the build process:

```bash
# Task runner - REQUIRED for all development commands
cargo install just

# WASM application bundler (builds the Leptos frontend)
cargo install trunk

# Desktop application packager (creates native app with embedded web UI)
cargo install tauri-cli
```

## Project Structure

Pomotoro follows Clean Architecture principles with a decoupled infrastructure layer:

```
pomotoro/
├── core/
│   ├── domain/      # Business logic and entities (framework-agnostic)
│   ├── usecases/    # Application services (framework-agnostic)
│   └── infra/       # Core infrastructure (framework-agnostic)
│                    # - Repositories, Event Bus, Audio, Timer Tick Service
│                    # - Zero Tauri dependencies - reusable across clients
└── apps/
    ├── tauri-app/   # Tauri desktop client (Tauri-specific)
    │                # - Command handlers, Tauri plugins, UI emission
    │                # - Thin wrapper around infra core
    ├── leptos-ui/   # Leptos frontend (WebAssembly)
    ├── pomotoro-cli/    # CLI client (coming soon)
    └── cosmic-de/       # Cosmic DE applet (coming soon)
```

**Key Design**: The `core/infra/` crate is completely framework-agnostic and can be reused by any Rust client (Tauri, Cosmic applet, CLI, etc.) without bringing in Tauri dependencies.

# Optional: Database migration tool (requires SQLite dev libraries - see below)
cargo install diesel_cli --no-default-features --features sqlite
```

**Note**: Installing `diesel_cli` requires SQLite development libraries to be installed first. See the System Dependencies section below.

#### 3. System Dependencies

##### Linux (Debian/Ubuntu)

**Option 1: Use the automated installer script (Recommended)**
```bash
# Using just (if installed)
just install-deps

# Or run the script directly
./scripts/install-deps.sh
```

**Option 2: Manual installation**
```bash
# Install all required dependencies at once
sudo apt update
sudo apt install \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsqlite3-dev \
  libasound2-dev \
  libgtk-3-dev \
  libpango1.0-dev \
  libgdk-pixbuf2.0-dev \
  libcairo2-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev

# Breakdown:
# - libwebkit2gtk-4.1-dev: Core Tauri webview
# - build-essential: C/C++ compiler toolchain
# - libxdo-dev: Window manipulation
# - libssl-dev: Cryptography support
# - libayatana-appindicator3-dev: System tray support
# - librsvg2-dev: SVG rendering
# - libsqlite3-dev: Database (required for diesel_cli)
# - libasound2-dev: Audio/sound notifications (REQUIRED)
# - libgtk-3-dev: GTK3 development libraries
# - libpango1.0-dev: Text rendering and layout
# - libgdk-pixbuf2.0-dev: Image loading library
# - libcairo2-dev: 2D graphics library
# - libsoup-3.0-dev: HTTP library for WebKit
# - libjavascriptcoregtk-4.1-dev: JavaScript engine for WebKit
```

**For other Linux distributions:**

The `scripts/install-deps.sh` script supports multiple distributions:
- ✅ Debian/Ubuntu/Pop!_OS
- ✅ Fedora/RHEL/CentOS
- ✅ Arch Linux/Manjaro
- ✅ Alpine Linux

Just run `./scripts/install-deps.sh` and it will detect your distribution automatically!

**Manual installation commands:**
- **Fedora/RHEL/CentOS**: 
  ```bash
  sudo dnf install webkit2gtk4.1-devel gtk3-devel sqlite-devel alsa-lib-devel \
    openssl-devel libayatana-appindicator-gtk3-devel librsvg2-devel
  ```
- **Arch Linux**: 
  ```bash
  sudo pacman -S webkit2gtk-4.1 gtk3 sqlite alsa-lib openssl \
    libayatana-appindicator librsvg
  ```
- **Alpine Linux**: 
  ```bash
  sudo apk add webkit2gtk-4.1-dev gtk+3.0-dev sqlite-dev alsa-lib-dev \
    openssl-dev libayatana-appindicator-dev librsvg-dev
  ```

##### macOS
- Xcode Command Line Tools: `xcode-select --install`
- WebKit framework (included with macOS)

##### Windows
- [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- WebView2 (typically auto-installed with Windows updates)

### Development

1. **Clone the repository:**
```bash
git clone <repository-url>
cd pomotoro
```

2. **Verify installation:**
```bash
# Check all required tools are installed
rustc --version
cargo --version
trunk --version
cargo tauri --version
rustup target list --installed | grep wasm32
```

3. **Run in development mode:**
```bash
# Using just (recommended)
just dev

# Or using cargo directly
cd apps/tauri-app && cargo tauri dev
```

The app will open automatically with hot-reload enabled for both frontend and backend changes.

**Note**: Run `just` without arguments to see all available commands.

## Understanding the Build Stack

### Why Both Trunk AND Tauri CLI?

This project uses a **two-stage build process** that requires separate tools:

1. **Trunk** - WASM Bundler
   - Compiles Rust code in `ui/` crate to WebAssembly
   - Bundles WASM with JavaScript glue code
   - Serves the web application during development
   - Similar to webpack in JavaScript projects

2. **Tauri CLI** - Desktop Packager
   - Embeds the WASM web app into a native window
   - Provides system API access (notifications, file system, etc.)
   - Creates platform-specific installers
   - Similar to Electron packager but much lighter

**Analogy**: Think of it like building a hybrid car - you need one tool to build the electric motor (Trunk → WASM) and another to assemble the complete vehicle (Tauri → Desktop App).

## Project Structure

```
pomotoro/
├── core/              # Framework-agnostic core engine
│   ├── domain/        # Core business logic & entities
│   │   ├── src/
│   │   │   ├── entities/  # Domain models (Task, Timer, Session)
│   │   │   └── traits/    # Repository interfaces
│   │   └── Cargo.toml
│   ├── usecases/      # Application business rules
│   │   ├── src/
│   │   │   └── *.rs   # Use case implementations
│   │   └── Cargo.toml
│   └── infra/         # Infrastructure layer
│       ├── src/
│       │   ├── adapters/  # Event bus, audio, repositories
│       │   └── bootstrap.rs
│       ├── migrations/    # Database schema migrations
│       └── Cargo.toml
├── apps/              # Client applications
│   ├── tauri-app/     # Tauri desktop client
│   │   ├── src/
│   │   │   ├── main.rs   # Tauri application entry
│   │   │   └── commands/ # Tauri command handlers
│   │   ├── tauri.conf.json
│   │   └── Cargo.toml
│   ├── leptos-ui/     # Leptos WASM frontend
│   │   ├── src/
│   │   │   ├── pages/   # Application pages/routes
│   │   │   ├── components/ # Reusable UI components
│   │   │   └── main.rs  # Frontend entry point
│   │   ├── index.html   # HTML template
│   │   └── Cargo.toml
│   ├── pomotoro-cli/  # CLI client (stub)
│   └── cosmic-de/     # Cosmic DE applet (stub)
├── assets/            # Sounds and icons
├── scripts/           # Utility scripts
│   └── install-deps.sh # System dependency installer
├── Trunk.toml         # Frontend build configuration
└── justfile           # Task runner - all development commands
```

## Development Commands

**All development tasks are managed through `just`** - run `just` to see all available commands.

### Common Commands
| Command | Description |
|---------|-------------|
| `just` | List all available commands |
| `just dev` | Start development server (info-level logging) |
| `just dev-debug` | Start development server (debug-level logging) |
| `just dev-trace` | Start development server (trace-level logging) |
| `just build` | Create production build |
| `just serve` | Run frontend dev server only |
| `just test` | Run all tests |
| `just test-infra` | Run infrastructure tests only |
| `just test-domain` | Run domain tests only |
| `just test-usecases` | Run use cases tests only |
| `just check` | Check code without building |
| `just fmt` | Format code |
| `just fmt-check` | Check code formatting |
| `just clippy` | Run linter |
| `just ci` | Run all checks (test, check, fmt, clippy) |
| `just clean` | Clean build artifacts |
| `just install-deps` | Install system dependencies (Linux only) |

### Using Cargo/Trunk Directly (if needed)
| Command | Description |
|---------|-------------|
| `cd apps/tauri-app && cargo tauri dev` | Start development server |
| `cd apps/tauri-app && cargo tauri build` | Build for production |
| `trunk serve` | Frontend dev server only |
| `cargo test --workspace` | Run tests |
| `cargo fmt --all` | Format code |
| `cargo clippy --workspace` | Run linter |

### Database Migrations
```bash
cd core/infra
diesel migration run     # Apply migrations
diesel migration revert  # Rollback last migration
diesel migration redo    # Revert and reapply
```

## Building for Production

Create optimized builds for distribution:

```bash
# Recommended: using just
just build

# Or using cargo directly
cd apps/tauri-app && cargo tauri build
```

The built application will be available in:
- **Linux**: `target/release/bundle/deb/*.deb` or `appimage/*.AppImage`
- **macOS**: `target/release/bundle/dmg/*.dmg`
- **Windows**: `target/release/bundle/msi/*.msi`

## Troubleshooting

### Common Issues and Solutions

#### 1. "can't find crate for `core`" or `std`
**Problem**: Missing WebAssembly compilation target
**Solution**:
```bash
rustup target add wasm32-unknown-unknown
```

#### 2. "trunk: command not found"
**Problem**: Trunk not installed
**Solution**:
```bash
cargo install trunk
```

#### 3. "error: no such subcommand: `tauri`"
**Problem**: Tauri CLI not installed
**Solution**:
```bash
cargo install tauri-cli
```

#### 4. "pkg-config exited with status code 1" - Missing system libraries
**Problem**: Missing GTK3, Pango, GDK, ALSA, or other system development libraries
**Solution**:

This usually means you're missing one or more of the comprehensive dependency list. Run the full installation command:

```bash
# Debian/Ubuntu - Install ALL dependencies at once
sudo apt update
sudo apt install \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsqlite3-dev \
  libasound2-dev \
  libgtk-3-dev \
  libpango1.0-dev \
  libgdk-pixbuf2.0-dev \
  libcairo2-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev

# Then clean and rebuild
cargo clean
```

**Common specific errors:**
- `library 'alsa' required by crate 'alsa-sys' was not found` → Install `libasound2-dev`
- `library 'pango' required by crate 'pango-sys' was not found` → Install `libpango1.0-dev` and `libgtk-3-dev`
- `library 'gdk-3.0' required by crate 'gdk-sys' was not found` → Install `libgtk-3-dev`
- `library 'libsoup-3.0' was not found` → Install `libsoup-3.0-dev`

#### 5. Build fails on Linux with webkit2gtk errors
**Problem**: Missing WebKit development libraries
**Solution**:
```bash
sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev
```

#### 6. "Failed to load module" errors in development
**Problem**: Frontend not building correctly
**Solution**:
```bash
# Clean and rebuild
cargo clean
rm -rf dist
trunk build
```

#### 7. "unable to find library -lsqlite3" during diesel_cli installation
**Problem**: SQLite development libraries not installed
**Solution**:
```bash
# Debian/Ubuntu
sudo apt install libsqlite3-dev

# Fedora/RHEL/CentOS
sudo dnf install sqlite-devel

# Arch Linux
sudo pacman -S sqlite

# Alpine Linux
sudo apk add sqlite-dev

# Then retry the installation
cargo install diesel_cli --no-default-features --features sqlite
```

## Usage

1. **🎯 Start Timer** - Click the play button to begin a 25-minute focus session
2. **⏸️ Pause/Resume** - Toggle the timer as needed during sessions
3. **⏭️ Skip Phase** - Jump to the next break or work session
4. **🔄 Reset** - Return to the beginning of the current phase
5. **⚙️ Settings** - Customize work and break durations to fit your workflow

## Why Pomotoro?

Like a focused bull (toro), Pomotoro helps you charge through your tasks with unwavering determination. No fluff, no distractions – just pure productivity power delivered through modern Rust technology.

## Contributing

We welcome contributions! Here's how to get started:

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature-name`
3. **Make** your changes and test thoroughly
4. **Commit** with clear messages: `git commit -m "Add feature: description"`
5. **Push** and create a pull request

See [ROADMAP.md](ROADMAP.md) for planned features and development priorities.

## Architecture Notes

This project follows **Clean Architecture** principles:
- **Domain Layer**: Pure business logic, no external dependencies
- **Use Cases Layer**: Application-specific business rules
- **Infrastructure Layer**: Frameworks, databases, and external interfaces
- **UI Layer**: User interface and presentation logic

Dependencies flow inward: UI → Infrastructure → Use Cases → Domain

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- 🍅 Inspired by the [Pomodoro Technique](https://francescocirillo.com/pages/pomodoro-technique) by Francesco Cirillo
- 🦀 Built with the amazing [Tauri](https://tauri.app/) and [Leptos](https://leptos.dev/) frameworks
- 🐂 Powered by the strength and speed of Rust