#!/bin/bash
# install.sh - Install TOS after compilation

set -e

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share"
XSESSIONSDIR="${DATADIR}/xsessions"

# Detect alpha version
if [ -d "alpha-2" ] && [ -f "alpha-2/Cargo.toml" ]; then
    ALPHA_DIR="alpha-2"
    echo "Detected TOS Alpha-2"
elif [ -d "alpha-1" ] && [ -f "alpha-1/Cargo.toml" ]; then
    ALPHA_DIR="alpha-1"
    echo "Detected TOS Alpha-1"
else
    echo "Error: No Alpha-1 or Alpha-2 directory found"
    exit 1
fi

echo "Building TOS..."
cd "$ALPHA_DIR"
cargo build --release
cd ..

# Create directories
echo "Creating system directories..."
sudo mkdir -p "$BINDIR" "$XSESSIONSDIR" "/etc/tos" "/var/log/tos"

# Install binary
echo "Installing binary..."
sudo cp "$ALPHA_DIR/target/release/tos" "$BINDIR/tos-session"

# Install configuration (if any)
if [ -f tos.conf ]; then
    echo "Installing configuration..."
    sudo cp tos.conf /etc/tos/
fi

# Install icon (if present)
if [ -f "packaging/tos-icon.png" ]; then
    sudo cp packaging/tos-icon.png "$DATADIR/pixmaps/tos-icon.png"
fi

# Set permissions
sudo chmod 755 "$BINDIR/tos-session"
sudo chmod 644 "$XSESSIONSDIR/tos.desktop"
sudo chmod 777 "/var/log/tos" # Allow TOS to write logs

# Detect and restart display manager
if systemctl is-active display-manager.service >/dev/null 2>&1; then
    echo "Display manager detected. You might need to log out and back in to see TOS."
    # sudo systemctl try-restart display-manager.service # Potentially disruptive, leaving commented
fi

# Install systemd service for Alpha-2 background services
if [ "$ALPHA_DIR" = "alpha-2" ]; then
    if systemctl list-unit-files | grep -q tos-session.service; then
        echo "Installing systemd service..."
        sudo cp packaging/tos-session.service /etc/systemd/system/
        sudo systemctl daemon-reload
        sudo systemctl enable tos-session.service
    fi
fi

echo "TOS installed successfully. Select 'TOS' from your login screen session menu."
