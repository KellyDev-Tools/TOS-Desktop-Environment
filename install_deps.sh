#!/bin/bash
# TOS Dream Complete - Dependency Installation Script
# This script installs all required system dependencies for building TOS

set -e

echo "=== TOS Dream Complete - Dependency Installer ==="
echo "Detecting distribution..."

# Detect distribution
if [ -f /etc/debian_version ]; then
    DISTRO="debian"
    echo "Detected Debian/Ubuntu-based distribution"
elif [ -f /etc/fedora-release ]; then
    DISTRO="fedora"
    echo "Detected Fedora-based distribution"
elif [ -f /etc/arch-release ]; then
    DISTRO="arch"
    echo "Detected Arch-based distribution"
else
    echo "Unknown distribution. Please install dependencies manually."
    exit 1
fi

# Install based on distribution
case $DISTRO in
    debian)
        echo "Installing Debian/Ubuntu dependencies..."
        sudo apt-get update
        sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libsoup-3.0-dev \
            libjavascriptcoregtk-4.1-dev \
            pkg-config \
            libgtk-3-dev \
            libglib2.0-dev \
            libcairo2-dev \
            libgdk-pixbuf2.0-dev \
            libpango1.0-dev \
            libatk1.0-dev \
            build-essential \
            curl \
            git
        ;;
    fedora)
        echo "Installing Fedora dependencies..."
        sudo dnf install -y \
            webkit2gtk4.1-devel \
            libsoup3-devel \
            javascriptcoregtk4.1-devel \
            pkgconf \
            gtk3-devel \
            glib2-devel \
            cairo-devel \
            gdk-pixbuf2-devel \
            pango-devel \
            atk-devel \
            gcc \
            curl \
            git
        ;;
    arch)
        echo "Installing Arch dependencies..."
        sudo pacman -S --needed \
            webkit2gtk-4.1 \
            libsoup3 \
            javascriptcoregtk-4.1 \
            pkgconf \
            gtk3 \
            glib2 \
            cairo \
            gdk-pixbuf2 \
            pango \
            atk \
            base-devel \
            curl \
            git
        ;;
esac

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

echo ""
echo "=== Verifying Installation ==="

# Check each dependency
DEPS_OK=true

check_pkg_config() {
    if pkg-config --exists "$1"; then
        echo "✓ $1 found: $(pkg-config --modversion $1)"
    else
        echo "✗ $1 NOT FOUND"
        DEPS_OK=false
    fi
}

check_pkg_config webkit2gtk-4.1
check_pkg_config libsoup-3.0
check_pkg_config javascriptcoregtk-4.1
check_pkg_config gtk+-3.0
check_pkg_config glib-2.0

echo ""
if [ "$DEPS_OK" = true ]; then
    echo "=== All dependencies installed successfully! ==="
    echo "You can now build TOS with: cd tos-dream && cargo build"
    exit 0
else
    echo "=== Some dependencies are missing ==="
    echo "Please check the error messages above and install manually."
    exit 1
fi
