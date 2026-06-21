#!/usr/bin/env bash

# Pomotoro Dependency Installation Script
# This script installs all required system dependencies for building Pomotoro on Linux

set -e

echo "🍅 Pomotoro Dependency Installer"
echo "================================="
echo ""

# Detect the Linux distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    OS_LIKE=$ID_LIKE
else
    echo "❌ Cannot detect Linux distribution"
    exit 1
fi

echo "📦 Detected OS: $OS"
echo ""

case "$OS" in
    ubuntu|debian|pop)
        echo "🔧 Installing dependencies for Debian/Ubuntu-based system..."
        echo ""
        sudo apt update
        sudo apt install -y \
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
        ;;

    fedora|rhel|centos)
        echo "🔧 Installing dependencies for Fedora/RHEL/CentOS..."
        echo ""
        sudo dnf install -y \
            webkit2gtk4.1-devel \
            gtk3-devel \
            sqlite-devel \
            alsa-lib-devel \
            openssl-devel \
            libayatana-appindicator-gtk3-devel \
            librsvg2-devel \
            gcc \
            gcc-c++ \
            make
        ;;

    arch|manjaro)
        echo "🔧 Installing dependencies for Arch Linux..."
        echo ""
        sudo pacman -Sy --needed \
            webkit2gtk-4.1 \
            gtk3 \
            sqlite \
            alsa-lib \
            openssl \
            libayatana-appindicator \
            librsvg \
            base-devel
        ;;

    alpine)
        echo "🔧 Installing dependencies for Alpine Linux..."
        echo ""
        sudo apk add \
            webkit2gtk-4.1-dev \
            gtk+3.0-dev \
            sqlite-dev \
            alsa-lib-dev \
            openssl-dev \
            libayatana-appindicator-dev \
            librsvg-dev \
            build-base
        ;;

    *)
        echo "❌ Unsupported distribution: $OS"
        echo ""
        echo "Please refer to the README.md for manual installation instructions."
        echo "If your distribution is similar to Debian/Ubuntu, Fedora, or Arch, you can try:"
        echo "  - Debian-like: Run this script with OS=ubuntu"
        echo "  - Fedora-like: Run this script with OS=fedora"
        echo "  - Arch-like: Run this script with OS=arch"
        exit 1
        ;;
esac

echo ""
echo "✅ All dependencies installed successfully!"
echo ""
echo "📋 Next steps:"
echo "   1. Install Rust toolchain if not already installed:"
echo "      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo ""
echo "   2. Install build tools:"
echo "      cargo install tauri-cli just"
echo ""
echo "   3. Install database CLI (optional):"
echo "      cargo install diesel_cli --no-default-features --features sqlite"
echo ""
echo "   4. Start development:"
echo "      just dev"
echo ""
echo "🐂 Ready to charge through your tasks with Pomotoro!"
