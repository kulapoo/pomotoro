# Scripts Directory

This directory contains utility scripts for the Pomotoro project.

## Available Scripts

### `install-deps.sh`

**Purpose**: Automated system dependency installer for Linux distributions.

**Description**: Detects your Linux distribution and installs all required system libraries and development dependencies needed to build Pomotoro.

**Supported Distributions**:
- ✅ Debian/Ubuntu/Pop!_OS
- ✅ Fedora/RHEL/CentOS  
- ✅ Arch Linux/Manjaro
- ✅ Alpine Linux

**Usage**:
```bash
# From project root
./scripts/install-deps.sh
```

**What it installs**:
- WebKit2GTK development libraries
- GTK3 development libraries
- SQLite development libraries
- ALSA (audio) development libraries
- OpenSSL development libraries
- System tray libraries
- SVG rendering libraries
- Build tools and compilers

**Note**: This script requires `sudo` privileges to install system packages.

---

## Adding New Scripts

When adding new utility scripts to this directory:

1. Make scripts executable: `chmod +x scripts/your-script.sh`
2. Add proper shebang: `#!/usr/bin/env bash` or `#!/bin/bash`
3. Include help/usage information in the script
4. Document the script in this README
5. Use consistent error handling (`set -e` at minimum)

## Task Runner

**All development workflows use `just` as the single task runner.**

The `justfile` in the project root provides all development commands:
- Running the development server (`just dev`)
- Building for production (`just build`)
- Testing and code quality checks (`just test`, `just fmt`, `just clippy`)
- Installing dependencies (`just install-deps`)

Run `just` without arguments to see all available commands.

Refer to the main [README.md](../README.md) for complete usage details.