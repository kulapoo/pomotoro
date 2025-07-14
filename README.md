# Pomotoro

A modern Pomodoro timer application built with Tauri and Leptos, combining the power of Rust with a fast, reactive web frontend.

## Features

- 🍅 Classic Pomodoro technique (25min work, 5min break, 15min long break)
- ⏱️ Visual circular timer with progress indication
- 🎮 Simple controls: Start, Pause, Reset, Skip
- 📊 Session tracking and statistics
- 🔔 Desktop notifications for phase transitions
- ⚙️ Customizable timer durations
- 🎨 Clean, modern UI with smooth animations
- 📱 Cross-platform support (Windows, macOS, Linux)

## Technology Stack

- **Frontend**: [Leptos](https://leptos.dev/) - Fast, reactive Rust web framework
- **Backend**: [Tauri](https://tauri.app/) - Secure, lightweight desktop app framework
- **Language**: Rust - Memory safe, fast native performance
- **Build Tool**: [Trunk](https://trunkrs.dev/) - WASM web application bundler

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for npm dependencies)
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

### Development

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd pomotoro
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Run in development mode:
   ```bash
   cargo tauri dev
   ```

### Building

Create a production build:
```bash
cargo tauri build
```

The built application will be available in `src-tauri/target/release/bundle/`.

## Project Structure

```
pomotoro/
├── src/                 # Leptos frontend source
│   ├── app.rs          # Main app component
│   └── main.rs         # Frontend entry point
├── src-tauri/          # Tauri backend
│   ├── src/
│   │   ├── lib.rs      # Timer logic and commands
│   │   └── main.rs     # App entry point
│   └── tauri.conf.json # App configuration
├── public/             # Static assets
├── styles.css          # Global styles
└── index.html          # HTML template
```

## Usage

1. **Start Timer**: Click the play button to begin a 25-minute focus session
2. **Pause/Resume**: Toggle the timer as needed
3. **Skip Phase**: Jump to the next break or work session
4. **Reset**: Return to the beginning of the current phase
5. **Settings**: Customize work and break durations

## Development Commands

- `cargo tauri dev` - Start development server with hot reload
- `cargo tauri build` - Create production build
- `trunk serve` - Serve frontend only (for UI development)
- `cargo test` - Run Rust tests
- `cargo clippy` - Lint Rust code
- `cargo fmt` - Format Rust code

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and test thoroughly
4. Commit with clear messages: `git commit -m "Add feature"`
5. Push and create a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by the [Pomodoro Technique](https://francescocirillo.com/pages/pomodoro-technique) by Francesco Cirillo
- Built with the amazing [Tauri](https://tauri.app/) and [Leptos](https://leptos.dev/) frameworks
