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
- **Architecture**: Clean Architecture with Domain-Driven Design

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
# WASM application bundler (builds the Leptos frontend)
cargo install trunk

# Desktop application packager (creates native app with embedded web UI)
cargo install tauri-cli

# Optional: Task runner for simplified commands
cargo install just

# Optional: Database migration tool
cargo install diesel_cli --no-default-features --features sqlite
```

#### 3. System Dependencies

##### Linux (Debian/Ubuntu)
```bash
# Core Tauri dependencies
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Audio support for sound notifications
sudo apt install libasound2-dev
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
# Using just (if installed)
just dev

# Or using cargo directly
cd infra && cargo tauri dev

# Or using the dev script
./dev.sh dev
```

The app will open automatically with hot-reload enabled for both frontend and backend changes.

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
├── domain/             # Core business logic & entities (pure Rust, no frameworks)
│   ├── src/
│   │   ├── entities/  # Domain models (Task, Timer, Session)
│   │   └── traits/    # Repository interfaces
│   └── Cargo.toml
├── usecases/          # Application business rules
│   ├── src/
│   │   └── *.rs      # Use case implementations
│   └── Cargo.toml
├── infra/             # Infrastructure layer
│   ├── src/
│   │   ├── main.rs   # Tauri application entry
│   │   ├── commands/ # Tauri command handlers
│   │   └── repos/    # Database implementations
│   ├── migrations/   # Database schema migrations
│   ├── Trunk.toml   # Frontend build configuration
│   └── tauri.conf.json
├── ui/               # Leptos WASM frontend
│   ├── src/
│   │   ├── pages/   # Application pages/routes
│   │   ├── components/ # Reusable UI components
│   │   └── main.rs  # Frontend entry point
│   └── index.html   # HTML template
├── assets/          # Sounds and icons
├── dev.sh          # Development helper script
└── justfile        # Task definitions for `just`
```

## Development Commands

### Using `just` (Recommended)
| Command | Description |
|---------|-------------|
| `just dev` | Start development server with hot reload |
| `just build` | Create production build |
| `just serve` | Run frontend dev server only |
| `just test` | Run all tests |
| `just fmt` | Format code |
| `just clippy` | Run linter |
| `just clean` | Clean build artifacts |

### Using Cargo/Trunk Directly
| Command | Description |
|---------|-------------|
| `cd infra && cargo tauri dev` | Start development server |
| `cd infra && cargo tauri build` | Build for production |
| `cd infra && trunk serve` | Frontend dev server only |
| `cargo test --workspace` | Run tests |
| `cargo fmt --all` | Format code |
| `cargo clippy --workspace` | Run linter |

### Using Dev Script
```bash
./dev.sh dev        # Start development server
./dev.sh build      # Build for production
./dev.sh test       # Run tests
./dev.sh fmt        # Format code
```

### Database Migrations
```bash
cd infra
diesel migration run     # Apply migrations
diesel migration revert  # Rollback last migration
diesel migration redo    # Revert and reapply
```

## Building for Production

Create optimized builds for distribution:

```bash
just build
# or
cd infra && cargo tauri build
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

#### 4. Audio not working on Linux
**Problem**: Missing ALSA development libraries
**Solution**:
```bash
sudo apt install libasound2-dev
```

#### 5. Build fails on Linux with webkit2gtk errors
**Problem**: Missing WebKit development libraries
**Solution**:
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

#### 6. "Failed to load module" errors in development
**Problem**: Frontend not building correctly
**Solution**:
```bash
# Clean and rebuild
cargo clean
rm -rf dist
cd infra && trunk build
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