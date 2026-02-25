#!/bin/bash
# TOS Dream - Dependency Installation Script
# Installs all required system libraries for building TOS.
#
# Feature → Native library mapping:
#   gui               wry/tao  → webkit2gtk-4.1, libsoup-3.0, gtk3
#   accessibility     kira     → libasound2-dev (ALSA for cpal backend)
#                     speech-dispatcher → libspeechd-dev
#   voice-system      cpal     → libasound2-dev, libudev-dev
#                     whisper-rs → libclang-dev, cmake, libopenblas-dev (optional)
#   script-engine     mlua     → liblua5.4-dev
#   remote-desktop    russh    → libssl-dev
#   wayland           smithay  → libwayland-dev, libxkbcommon-dev, libseat-dev,
#                                libdrm-dev, libgbm-dev, libinput-dev, libegl-dev

set -e

echo "=== TOS Dream - Dependency Installer ==="
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
            `# --- GUI (wry / tao / webkit2gtk) ---` \
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
            `# --- Audio / accessibility (kira + cpal) ---` \
            libasound2-dev \
            `# --- Input devices (gilrs gamepad) ---` \
            libudev-dev \
            `# --- TTS / screen reader (speech-dispatcher, atspi) ---` \
            libspeechd-dev \
            libdbus-1-dev \
            `# --- Script engine (mlua → Lua 5.4) ---` \
            liblua5.4-dev \
            `# --- Voice system (whisper-rs needs libclang + cmake for bindgen) ---` \
            libclang-dev \
            cmake \
            `# --- Remote desktop (russh / reqwest need OpenSSL) ---` \
            libssl-dev \
            `# --- Wayland compositor (smithay) ---` \
            libwayland-dev \
            wayland-protocols \
            libxkbcommon-dev \
            libseat-dev \
            libdrm-dev \
            libgbm-dev \
            libinput-dev \
            libegl-dev \
            `# --- General ---` \
            curl \
            git
        ;;
    fedora)
        echo "Installing Fedora dependencies..."
        sudo dnf install -y \
            `# --- GUI ---` \
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
            `# --- Audio / accessibility ---` \
            alsa-lib-devel \
            `# --- Input devices ---` \
            libudev-devel \
            `# --- TTS / screen reader ---` \
            speech-dispatcher-devel \
            dbus-devel \
            `# --- Script engine ---` \
            lua-devel \
            `# --- Voice system ---` \
            clang-devel \
            cmake \
            `# --- Remote desktop ---` \
            openssl-devel \
            `# --- Wayland compositor ---` \
            wayland-devel \
            wayland-protocols-devel \
            libxkbcommon-devel \
            libseat-devel \
            libdrm-devel \
            mesa-libgbm-devel \
            libinput-devel \
            mesa-libEGL-devel \
            `# --- General ---` \
            gcc \
            curl \
            git
        ;;
    arch)
        echo "Installing Arch dependencies..."
        sudo pacman -S --needed \
            `# --- GUI ---` \
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
            `# --- Audio / accessibility ---` \
            alsa-lib \
            `# --- Input devices ---` \
            systemd-libs \
            `# --- TTS / screen reader ---` \
            speech-dispatcher \
            dbus \
            `# --- Script engine ---` \
            lua54 \
            `# --- Voice system ---` \
            clang \
            cmake \
            `# --- Remote desktop ---` \
            openssl \
            `# --- Wayland compositor ---` \
            wayland \
            wayland-protocols \
            libxkbcommon \
            seatd \
            libdrm \
            mesa \
            libinput \
            `# --- General ---` \
            base-devel \
            curl \
            git
        ;;
esac

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo ""
echo "=== Verifying Installation ==="

DEPS_OK=true

check_pkg_config() {
    if pkg-config --exists "$1"; then
        echo "✓ $1 found: $(pkg-config --modversion "$1")"
    else
        echo "✗ $1 NOT FOUND"
        DEPS_OK=false
    fi
}

check_bin() {
    if command -v "$1" &> /dev/null; then
        echo "✓ $1 found: $("$1" --version 2>&1 | head -1)"
    else
        echo "✗ $1 NOT FOUND"
        DEPS_OK=false
    fi
}

echo ""
echo "--- GUI (wry / webkit2gtk) ---"
check_pkg_config webkit2gtk-4.1
check_pkg_config libsoup-3.0
check_pkg_config javascriptcoregtk-4.1
check_pkg_config gtk+-3.0
check_pkg_config glib-2.0

echo ""
echo "--- Audio / voice-system (kira + cpal + whisper-rs) ---"
check_pkg_config alsa
check_bin cmake

echo ""
echo "--- TTS / screen reader (speech-dispatcher, atspi) ---"
check_pkg_config speech-dispatcher
check_pkg_config dbus-1

echo ""
echo "--- Script engine (mlua / Lua 5.4) ---"
check_pkg_config lua5.4

echo ""
echo "--- Remote desktop (russh / reqwest / OpenSSL) ---"
check_pkg_config openssl

echo ""
echo "--- Wayland compositor (smithay) ---"
check_pkg_config wayland-server
check_pkg_config xkbcommon
check_pkg_config libdrm
check_pkg_config gbm
check_pkg_config libinput
check_pkg_config egl

echo ""
if [ "$DEPS_OK" = true ]; then
    echo "=== All dependencies installed successfully! ==="
    echo "You can now build TOS with: cd tos-dream && cargo build"
    echo ""
    echo "Feature flags you can pass to cargo:"
    echo "  --features accessibility   Audio, TTS, screen reader"
    echo "  --features voice-system    Wake word + speech-to-text (whisper)"
    echo "  --features script-engine   Lua and JS scripting"
    echo "  --features remote-desktop  SSH remote sectors"
    echo "  --features wayland         Native Wayland compositor"
    echo "  --features saas            Container orchestration (Docker/K8s)"
    exit 0
else
    echo "=== Some dependencies are missing ==="
    echo "Please check the error messages above and install manually."
    exit 1
fi
