#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "=========================================================="
echo " TOS Desktop Environment - Dependency Installer"
echo "=========================================================="

if [ "$EUID" -eq 0 ]; then
  echo "Please run this script as a normal user. It will ask for sudo when necessary."
  exit 1
fi

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    VERSION_ID=$VERSION_ID
else
    echo "Could not detect OS. /etc/os-release not found."
    exit 1
fi

echo "Detected OS: $OS"
echo ""

install_debian() {
    echo "Installing dependencies for Debian/Ubuntu-based system..."
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        curl \
        git \
        pkg-config \
        libwayland-dev \
        wayland-protocols \
        libegl1-mesa-dev \
        libxkbcommon-dev \
        libdbus-1-dev \
        libasound2-dev \
        libssl-dev \
        libvulkan-dev \
        libfontconfig1-dev \
        clang \
        sway \
        waybar \
        kitty \
        fish
}

install_fedora() {
    echo "Installing dependencies for Fedora-based system..."
    sudo dnf install -y \
        gcc-c++ \
        curl \
        git \
        pkgconf-pkg-config \
        wayland-devel \
        wayland-protocols-devel \
        mesa-libEGL-devel \
        libxkbcommon-devel \
        dbus-devel \
        alsa-lib-devel \
        openssl-devel \
        vulkan-devel \
        fontconfig-devel \
        clang \
        sway \
        waybar \
        alacritty \
        fish
}

install_arch() {
    echo "Installing dependencies for Arch Linux-based system..."
    sudo pacman -Syu --noconfirm --needed \
        base-devel \
        curl \
        git \
        pkgconf \
        wayland \
        wayland-protocols \
        libxkbcommon \
        alsa-lib \
        openssl \
        vulkan-headers \
        fontconfig \
        clang \
        sway \
        waybar \
        alacritty \
        fish
}

# Distro specific logic
case $OS in
    ubuntu|debian|pop|linuxmint|elementary)
        install_debian
        ;;
    fedora|centos|rhel)
        install_fedora
        ;;
    arch|manjaro|endeavouros)
        install_arch
        ;;
    *)
        echo "Unsupported OS: $OS. Please install dependencies manually."
        echo "Required: wayland, wayland-protocols, libxkbcommon, rust/cargo."
        echo "Crucial Compositor: sway (to test the wlr-layer-shell protocol)"
        ;;
esac

echo "=========================================================="
echo "Checking Rust Toolchain..."
echo "=========================================================="

if ! command -v cargo &> /dev/null; then
    echo "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust is installed: $(cargo --version)"
    echo "Updating Rust to ensure it is the latest stable version..."
    rustup update
fi

echo "=========================================================="
echo "Checking Node.js Toolchain (via NVM)..."
echo "=========================================================="

export NVM_DIR="$HOME/.nvm"
if [ ! -d "$NVM_DIR" ]; then
    echo "NVM not found. Installing NVM..."
    unset NVM_DIR
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
fi

# Load NVM for the current session
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

echo "Installing and using Node.js v20 (LTS)..."
nvm install 20
nvm use 20
echo "Node $(node -v) installed."

echo "=========================================================="
echo "Installing Playwright dependencies..."
echo "=========================================================="
# Since playwright usually installs to project, we will install it globally
# and command it to fetch its system-level dependencies.
npm install -g playwright
npx playwright install --with-deps

echo "=========================================================="
echo "Dependencies installed successfully!"
echo "NOTE on Wayland Compositor Requirements:"
echo "TOS uses 'wlr-layer-shell' to draw desktop UI elements overlaid on the screen."
echo "If your current desktop uses Weston, GNOME/Mutter, or an incompatible shell,"
echo "it will reject the TOS UI protocols, causing Brain to run in headless mode."
echo ""
echo "To see the UI, log out and select a 'Sway' session, or simply type 'sway' "
echo "in your terminal to start a nested compositor. Run 'make run' inside it!"
echo "=========================================================="
