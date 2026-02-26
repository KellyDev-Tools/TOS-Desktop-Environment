#!/bin/bash
# install.sh - Install TOS after compilation

set -e

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share"
XSESSIONSDIR="${DATADIR}/xsessions"

echo "Building TOS..."
cd alpha-1
cargo build --release
cd ..

# Create directories
echo "Creating system directories..."
sudo mkdir -p "$BINDIR" "$XSESSIONSDIR" "/etc/tos" "/var/log/tos"

# Install binary
echo "Installing binary..."
sudo cp alpha-1/target/release/tos "$BINDIR/tos-session"

# Install session desktop file
echo "Installing desktop entry..."
sudo cp packaging/tos.desktop "$XSESSIONSDIR/"

# Install configuration (if any)
if [ -f tos.conf ]; then
    echo "Installing configuration..."
    sudo cp tos.conf /etc/tos/
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

echo "TOS installed successfully. Select 'TOS' from your login screen session menu."
