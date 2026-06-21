# Project Overview

This project is a Pomodoro timer application called Pomotoro. It is built with a Rust backend using the Tauri framework and a reactive web frontend using React and TypeScript. The project follows a hexagonal architecture, with the codebase organized into four main crates: `domain`, `usecases`, `infra`, and `tauri-app`.

- **`domain`**: Contains the core business logic and data structures of the application.
- **`usecases`**: Implements the application's use cases, orchestrating the interaction between the `domain` and `infra` layers.
- **`infra`**: Provides the infrastructure for the application, including the Tauri setup, database access, and other external services.
- **`tauri-app`**: The Tauri desktop client (command handlers, plugins, UI emission). The React/TypeScript frontend lives in `apps/react-ui/`.

## Building and Running

The project uses `just` as a command runner to simplify common development tasks.

- **Run in development mode:** `just dev`
- **Build for production:** `just build`
- **Run tests:** `just test`
- **Check code:** `just check`
- **Format code:** `just fmt`
- **Run clippy:** `just clippy`
- **Install dependencies:** `just install`

## Development Conventions

The project uses `rustfmt` for code formatting and `clippy` for linting. The `justfile` provides convenient commands for running these tools. The project also has a multi-crate structure, which helps to keep the code organized and modular.


> IMPORTANT: Dont push the code into github
