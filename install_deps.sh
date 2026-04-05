#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "=========================================================="
echo " TOS Beta-0 Desktop Environment - Dependency Installer"
echo "=========================================================="

# ─────────────────────────────────────────────────────────────────────────────
# Platform Detection
# ─────────────────────────────────────────────────────────────────────────────

IS_WINDOWS=false
case "$OSTYPE" in
    msys*|mingw*|cygwin*) IS_WINDOWS=true ;;
esac

# Fallback: check uname if OSTYPE isn't set
if [ "$IS_WINDOWS" = false ] && uname -s 2>/dev/null | grep -qiE "MINGW|MSYS|CYGWIN|Windows_NT"; then
    IS_WINDOWS=true
fi

if [ "$IS_WINDOWS" = true ]; then
    echo "Detected Platform: Windows ($(uname -s))"
    echo ""
    echo "NOTE: Native Linux packages (wayland, sway, etc.) are not applicable on Windows."
    echo "      The Electron Face and Svelte UI components will be set up."
    echo ""
else
    # Linux root check (not applicable on Windows/MSYS)
    if [ "$EUID" -eq 0 ]; then
        echo "Please run this script as a normal user. It will ask for sudo when necessary."
        exit 1
    fi

    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        VERSION_MAJOR=$(echo "$VERSION_ID" | cut -d. -f1)
        
        # Smart detection: Normalize Fedora/RHEL/CentOS/Rocky/Alma/Oracle to "rhel"
        # but keep track of the specific version for the "rhel9" request.
        IS_RHEL_FAMILY=false
        if [[ "$ID" =~ ^(fedora|rhel|centos|rocky|alma|ol)$ ]]; then
            IS_RHEL_FAMILY=true
        elif [[ " $ID_LIKE " == *" rhel "* ]] || [[ " $ID_LIKE " == *" centos "* ]] || [[ " $ID_LIKE " == *" fedora "* ]]; then
            IS_RHEL_FAMILY=true
        fi

        if [ "$IS_RHEL_FAMILY" = true ]; then
            if [ "$VERSION_MAJOR" = "9" ]; then
                OS="rhel9"
            else
                OS="rhel"
            fi
        fi
    else
        echo "Could not detect OS. /etc/os-release not found."
        exit 1
    fi

    echo "Detected OS: $OS ($PRETTY_NAME)"
    echo ""
fi

# ─────────────────────────────────────────────────────────────────────────────
# Linux System Packages (skipped on Windows)
# ─────────────────────────────────────────────────────────────────────────────

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
        fish \
        unzip \
        openjdk-17-jdk
}

install_fedora() {
    echo "Installing dependencies for RHEL/Fedora/CentOS-based system..."
    
    # Enable CRB (CodeReady Builder) for development headers
    if [[ "$OS" == "rhel9" ]] || [[ "$ID" == "rocky" ]]; then
        echo "Ensuring CRB and EPEL are enabled..."
        sudo dnf install -y epel-release
        sudo dnf config-manager --set-enabled crb || true
    fi

    # Core development tools and libraries
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
        unzip \
        java-17-openjdk-devel \
        perl-FindBin \
        perl-core

    # Optional desktop components (might be missing in some RHEL 9 repos)
    echo "Attempting to install optional Wayland testing tools (Sway, Waybar, etc.)..."
    sudo dnf install -y \
        sway \
        waybar \
        alacritty \
        fish \
        weston || echo "WARNING: Some optional desktop packages (sway, waybar, alacritty) were not found. Using XDG fallback mode."
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
        fish \
        unzip \
        jdk17-openjdk
}

if [ "$IS_WINDOWS" = false ]; then
    case $OS in
        ubuntu|debian|pop|linuxmint|elementary|kali|raspbian)
            install_debian
            ;;
        fedora|centos|rhel|rhel9|rocky|alma|ol|amazon)
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
fi

# ─────────────────────────────────────────────────────────────────────────────
# Build Tools (Make)
# ─────────────────────────────────────────────────────────────────────────────

echo "=========================================================="
echo "Checking Build Tools (Make)..."
echo "=========================================================="

if ! command -v make &> /dev/null; then
    if [ "$IS_WINDOWS" = true ]; then
        echo "Make not found. Installing ezwinports GNU Make via winget..."
        winget install --id ezwinports.make -e --accept-package-agreements --accept-source-agreements || true
        
        # Determine winget bin path for Make
        MAKE_WINGET_BIN="$LOCALAPPDATA/Microsoft/WinGet/Packages/ezwinports.make_Microsoft.Winget.Source_8wekyb3d8bbwe/bin"
        if [ -d "$MAKE_WINGET_BIN" ]; then
            export PATH="$MAKE_WINGET_BIN:$PATH"
        fi
        
        if ! command -v make &> /dev/null; then
            echo "WARNING: Make installed but not yet accessible in the current PATH."
            echo "Please restart this terminal after the script finishes to use 'make'."
        else
            echo "Make is ready: $(make --version | head -n 1)"
        fi
    else
        echo "Make not found. Skipping (handled by OS package manager in earlier step)."
    fi
else
    echo "Make is ready: $(make --version | head -n 1)"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Rust Toolchain
# ─────────────────────────────────────────────────────────────────────────────

echo "=========================================================="
echo "Checking Rust Toolchain..."
echo "=========================================================="

# On Windows/MINGW, cargo may be installed but not in the MSYS PATH.
# Add the standard rustup install location so we can find it.
if [ "$IS_WINDOWS" = true ]; then
    CARGO_BIN="$HOME/.cargo/bin"
    # Also check the Windows-native USERPROFILE path
    if [ -n "$USERPROFILE" ]; then
        WIN_CARGO_BIN="$(cygpath "$USERPROFILE")/.cargo/bin"
        if [ -d "$WIN_CARGO_BIN" ] && [[ ":$PATH:" != *":$WIN_CARGO_BIN:"* ]]; then
            export PATH="$WIN_CARGO_BIN:$PATH"
        fi
    fi
    if [ -d "$CARGO_BIN" ] && [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
        export PATH="$CARGO_BIN:$PATH"
    fi
fi

# Also try sourcing .cargo/env if it exists
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

if ! command -v cargo &> /dev/null; then
    echo "Rust not found in PATH. Installing via rustup..."
    if [ "$IS_WINDOWS" = true ]; then
        RUSTUP_INIT="$HOME/.tmp/rustup-init.exe"
        mkdir -p "$HOME/.tmp"
        echo "Downloading rustup-init.exe..."
        curl --proto '=https' --tlsv1.2 -sSf -o "$RUSTUP_INIT" https://win.rustup.rs/x86_64
        echo "Running rustup-init.exe (unattended, GNU toolchain)..."
        # Use the GNU target to avoid MSVC link.exe conflicts in MINGW64.
        # The MSVC target requires Visual Studio Build Tools and its linker
        # collides with MINGW's /usr/bin/link (GNU coreutils).
        "$RUSTUP_INIT" -y --default-toolchain stable --default-host x86_64-pc-windows-gnu
        rm -f "$RUSTUP_INIT"

        # Add cargo to PATH for this session
        WIN_CARGO_BIN="$(cygpath "$USERPROFILE" 2>/dev/null)/.cargo/bin"
        if [ -d "$WIN_CARGO_BIN" ]; then
            export PATH="$WIN_CARGO_BIN:$PATH"
        fi
        if [ -d "$HOME/.cargo/bin" ]; then
            export PATH="$HOME/.cargo/bin:$PATH"
        fi
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        fi

        if ! command -v cargo &> /dev/null; then
            echo "WARNING: cargo still not in PATH. You may need to restart your terminal."
        else
            echo "Rust installed: $(cargo --version)"
        fi
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
else
    echo "Rust is installed: $(cargo --version)"
    echo "Updating Rust to ensure it is the latest stable version..."
    rustup update

    # On Windows/MINGW, ensure the GNU toolchain is the default to avoid
    # MSVC link.exe conflicts with MINGW's /usr/bin/link.
    if [ "$IS_WINDOWS" = true ]; then
        CURRENT_HOST=$(rustc -vV 2>/dev/null | grep "host:" | awk '{print $2}')
        if [[ "$CURRENT_HOST" == *"msvc"* ]]; then
            echo "Switching Rust default from MSVC to GNU toolchain (avoids linker conflicts)..."
            rustup toolchain install stable-x86_64-pc-windows-gnu
            rustup default stable-x86_64-pc-windows-gnu
            echo "Rust toolchain switched to: $(rustc -vV | grep host)"
        fi
    fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# Node.js Toolchain
# ─────────────────────────────────────────────────────────────────────────────

echo "=========================================================="
echo "Checking Node.js Toolchain..."
echo "=========================================================="

if [ "$IS_WINDOWS" = true ]; then
    # On Windows, Node.js is typically installed natively (not via unix NVM).
    # Ensure common Windows Node install paths are in MINGW PATH.
    for NODE_DIR in \
        "$PROGRAMFILES/nodejs" \
        "$PROGRAMW6432/nodejs" \
        "$LOCALAPPDATA/Programs/nodejs" \
        "$(cygpath "$PROGRAMFILES" 2>/dev/null)/nodejs" \
        "$(cygpath "$APPDATA" 2>/dev/null)/nvm-windows/current"; do
        if [ -d "$NODE_DIR" ] && [[ ":$PATH:" != *":$NODE_DIR:"* ]]; then
            export PATH="$NODE_DIR:$PATH"
        fi
    done

    if command -v node &> /dev/null; then
        NODE_VER=$(node -v)
        echo "Node.js detected: $NODE_VER"

        # Check minimum version (v18+)
        NODE_MAJOR=$(echo "$NODE_VER" | sed 's/v\([0-9]*\).*/\1/')
        if [ "$NODE_MAJOR" -lt 18 ]; then
            echo "WARNING: Node.js $NODE_VER is too old. TOS requires Node.js v18+."
            echo "Please update via: https://nodejs.org or 'winget install OpenJS.NodeJS.LTS'"
        else
            echo "Node.js $NODE_VER meets requirements (v18+ required)."
        fi
    else
        echo "Node.js not found."
        echo ""
        echo "Install Node.js v20 LTS using one of:"
        echo "  • Download from: https://nodejs.org"
        echo "  • winget:        winget install OpenJS.NodeJS.LTS"
        echo "  • nvm-windows:   https://github.com/coreybutler/nvm-windows"
        echo ""
        read -rp "Press Enter after installing Node.js to continue, or Ctrl+C to abort..."
        if ! command -v node &> /dev/null; then
            echo "ERROR: node still not found. Please restart your terminal."
            exit 1
        fi
    fi
else
    # Linux: use NVM
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
fi

# ─────────────────────────────────────────────────────────────────────────────
# NPM Project Dependencies
# ─────────────────────────────────────────────────────────────────────────────

echo "=========================================================="
echo "Installing NPM project dependencies..."
echo "=========================================================="

if command -v npm &> /dev/null; then
    echo "Installing face-svelte-ui dependencies..."
    (cd face-svelte-ui && npm install) || echo "WARNING: face-svelte-ui npm install failed"

    echo "Installing face-electron-any dependencies..."
    (cd face-electron-any && npm install) || echo "WARNING: face-electron-any npm install failed"
else
    echo "WARNING: npm not found, skipping project dependency install."
fi

# ─────────────────────────────────────────────────────────────────────────────
# Playwright
# ─────────────────────────────────────────────────────────────────────────────

echo "=========================================================="
echo "Installing Playwright dependencies..."
echo "=========================================================="

if command -v npx &> /dev/null; then
    npm install -g playwright
    if [ "$IS_WINDOWS" = true ] || [[ "$OS" =~ ^(rhel|rhel9|centos|rocky|alma|ol)$ ]]; then
        # On Windows and RHEL-likes, --with-deps is not supported (it uses apt-get internally)
        npx playwright install chromium
    else
        npx playwright install --with-deps
    fi
else
    echo "WARNING: npx not found, skipping Playwright install."
fi

# ─────────────────────────────────────────────────────────────────────────────
# Summary (platform-specific notes)
# ─────────────────────────────────────────────────────────────────────────────

echo "=========================================================="
echo "Core dependencies installed successfully!"
if [ "$IS_WINDOWS" = true ]; then
    echo ""
    echo "WINDOWS NOTES:"
    echo "  • The Electron Face is the primary UI on Windows."
    echo "  • Build the UI:     cd face-svelte-ui && npm run build"
    echo "  • Build Electron:   cd face-electron-any && npm run build"
    echo "  • Package for Win:  cd face-electron-any && npm run pack:win"
    echo "  • Dev mode:         cd face-electron-any && npm run dev"
    echo "  • Wayland Face is Linux-only and will not build on Windows."
else
    echo ""
    echo "NOTE on Wayland Compositor Requirements:"
    echo "TOS uses 'wlr-layer-shell' to draw desktop UI elements overlaid on the screen."
    echo "If your current desktop uses Weston, GNOME/Mutter, or an incompatible shell,"
    echo "it will reject the TOS UI protocols, causing Brain to run in headless mode."
    echo ""
    echo "To see the UI, log out and select a 'Sway' session, or simply type 'sway' "
    echo "in your terminal to start a nested compositor. Run 'make run' inside it!"
fi
echo "=========================================================="

# ─────────────────────────────────────────────────────────────────────────────
# Android NDK & Gradle Setup (Optional - for Android APK builds)
# ─────────────────────────────────────────────────────────────────────────────

echo ""
echo "=========================================================="
echo "Android Build Tools Detection"
echo "=========================================================="

echo "Installing cargo-ndk and rust targets for Android..."
if [ "$IS_WINDOWS" = true ]; then
    echo "Downloading pre-compiled cargo-ndk for Windows..."
    # The binary on GitHub is named cargo-ndk-windows.exe or similar? Let's use cargo-binstall instead
    # to perfectly handle the download and avoid missing URLs.
    if ! command -v cargo-binstall &> /dev/null; then
        echo "Installing cargo-binstall..."
        curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
    fi
    cargo binstall -y cargo-ndk || true
else
    cargo install cargo-ndk || true
fi
rustup target add aarch64-linux-android

export ANDROID_HOME="$HOME/android-sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"

if [ -d "$ANDROID_NDK_HOME" ]; then
    echo "Android NDK already installed at: $ANDROID_NDK_HOME"
else
    echo "Android NDK not detected. Installing Android NDK r26d..."
    mkdir -p "$ANDROID_HOME/cmdline-tools"
    
    # Determine the correct sdkmanager command (Windows uses .bat)
    SDKMANAGER="sdkmanager"
    if [ "$IS_WINDOWS" = true ]; then
        SDKMANAGER_BAT="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager.bat"
    else
        SDKMANAGER_BAT="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
    fi

    if ! command -v sdkmanager &> /dev/null && [ ! -f "$SDKMANAGER_BAT" ]; then
        echo "Downloading Android cmdline-tools..."
        if [ "$IS_WINDOWS" = true ]; then
            CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip"
        else
            CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
        fi
        curl -o "$ANDROID_HOME/cmdline-tools.zip" "$CMDLINE_TOOLS_URL"
        unzip -q -o "$ANDROID_HOME/cmdline-tools.zip" -d "$ANDROID_HOME/cmdline-tools"
        rm "$ANDROID_HOME/cmdline-tools.zip"
        
        # Move extracted cmdline-tools to 'latest'
        if [ -d "$ANDROID_HOME/cmdline-tools/cmdline-tools" ]; then
            mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
        fi
        export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
    fi

    # Verify Java is available (required by sdkmanager)
    if ! command -v java &> /dev/null; then
        if [ "$IS_WINDOWS" = true ]; then
            # Try common Windows JDK paths
            for JAVA_DIR in \
                "$PROGRAMFILES/Java" \
                "$PROGRAMFILES/Eclipse Adoptium" \
                "$PROGRAMFILES/Microsoft/jdk-17"* \
                "$(cygpath "$PROGRAMFILES" 2>/dev/null)/Java" \
                "$(cygpath "$PROGRAMFILES" 2>/dev/null)/Eclipse Adoptium"; do
                if [ -d "$JAVA_DIR" ]; then
                    JDK_BIN=$(find "$JAVA_DIR" -name "java.exe" -path "*/bin/*" 2>/dev/null | head -1)
                    if [ -n "$JDK_BIN" ]; then
                        JAVA_BIN_DIR=$(dirname "$JDK_BIN")
                        export PATH="$JAVA_BIN_DIR:$PATH"
                        export JAVA_HOME=$(dirname "$JAVA_BIN_DIR")
                        echo "Found Java at: $JAVA_HOME"
                        break
                    fi
                fi
            done
        fi

        if ! command -v java &> /dev/null; then
            echo "WARNING: Java not found. sdkmanager requires Java 17+."
            if [ "$IS_WINDOWS" = true ]; then
                echo "Installing Microsoft OpenJDK 17 via winget..."
                winget install --id Microsoft.OpenJDK.17 -e --accept-package-agreements --accept-source-agreements || true
                
                # Check path again after winget
                for JAVA_DIR in \
                    "$PROGRAMFILES/Microsoft/jdk-17"* \
                    "$(cygpath "$PROGRAMFILES" 2>/dev/null)/Microsoft/jdk-17"*; do
                    if [ -d "$JAVA_DIR" ]; then
                        JDK_BIN=$(find "$JAVA_DIR" -name "java.exe" -path "*/bin/*" 2>/dev/null | head -1)
                        if [ -n "$JDK_BIN" ]; then
                            JAVA_BIN_DIR=$(dirname "$JDK_BIN")
                            export PATH="$JAVA_BIN_DIR:$PATH"
                            export JAVA_HOME=$(dirname "$JAVA_BIN_DIR")
                            echo "Found Java at: $JAVA_HOME"
                            break
                        fi
                    fi
                done
                
                if ! command -v java &> /dev/null; then
                    echo "WARNING: Still could not locate Java in PATH after installation."
                    echo "Skipping Android NDK installation (restart your terminal later)."
                    SKIP_SDK=true
                fi
            else
                echo "Skipping Android NDK installation."
                SKIP_SDK=true
            fi
        fi
    fi

    if [ "${SKIP_SDK:-false}" = false ]; then
        # Use the correct sdkmanager binary
        if [ -f "$SDKMANAGER_BAT" ]; then
            SDKMANAGER="$SDKMANAGER_BAT"
        fi
        
        yes | "$SDKMANAGER" --licenses >/dev/null 2>&1 || true
        "$SDKMANAGER" --sdk_root="$ANDROID_HOME" "ndk;26.1.10909125"
    fi
fi

if command -v gradle &> /dev/null; then
    echo "Gradle already installed at: $(command -v gradle)"
else
    echo "Gradle not detected. Installing Gradle 8.5..."
    GRADLE_VERSION="8.5"
    GRADLE_DIST_ZIP="$HOME/.tmp/gradle-$GRADLE_VERSION-bin.zip"
    mkdir -p "$HOME/.tmp"
    curl -L "https://services.gradle.org/distributions/gradle-$GRADLE_VERSION-bin.zip" -o "$GRADLE_DIST_ZIP"
    unzip -q -o "$GRADLE_DIST_ZIP" -d "$HOME/gradle"
    rm -f "$GRADLE_DIST_ZIP"
    export GRADLE_HOME="$HOME/gradle/gradle-$GRADLE_VERSION"
    export PATH="$GRADLE_HOME/bin:$PATH"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Shell Profile Updates
# ─────────────────────────────────────────────────────────────────────────────

# Determine the shell profile file to update
if [ "$IS_WINDOWS" = true ]; then
    SHELL_PROFILE="$HOME/.bashrc"
else
    SHELL_PROFILE="$HOME/.bashrc"
fi

update_shell_profile() {
    local target_var=$1
    local value=$2
    if ! grep -q "$target_var=" "$SHELL_PROFILE" 2>/dev/null; then
        echo "export $target_var=\"$value\"" >> "$SHELL_PROFILE"
    fi
}

update_shell_profile "ANDROID_HOME" "$ANDROID_HOME"
update_shell_profile "ANDROID_NDK_HOME" "$ANDROID_NDK_HOME"

if ! grep -q "cmdline-tools/latest/bin" "$SHELL_PROFILE" 2>/dev/null; then
    EXTRA_PATHS="\$ANDROID_HOME/cmdline-tools/latest/bin:\$ANDROID_NDK_HOME:\$HOME/gradle/gradle-8.5/bin"
    if [ "$IS_WINDOWS" = true ]; then
        EXTRA_PATHS="\$LOCALAPPDATA/Microsoft/WinGet/Packages/ezwinports.make_Microsoft.Winget.Source_8wekyb3d8bbwe/bin:\$LOCALAPPDATA/Microsoft/WindowsApps:\$USERPROFILE/.cargo/bin:$EXTRA_PATHS"
    fi
    echo "export PATH=\"$EXTRA_PATHS:\$PATH\"" >> "$SHELL_PROFILE"
    echo "Added paths to $SHELL_PROFILE"
fi

echo ""
echo "Android Build Environment Ready!"
echo "  NDK: $ANDROID_NDK_HOME"
echo "  Gradle: $(command -v gradle 2>/dev/null || echo 'not in PATH')"
echo "Please run 'source $SHELL_PROFILE' to ensure paths are loaded."
