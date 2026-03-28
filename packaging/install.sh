#!/bin/bash
# install.sh - Install TOS after compilation

set -e

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"
DATADIR="${PREFIX}/share"
XSESSIONSDIR="${DATADIR}/xsessions"

# Detect beta version
if [ -f "Cargo.toml" ]; then
    VERSION_DIR="."
    echo "Detected TOS directory"
else
    echo "Error: Must be run from the TOS source directory"
    exit 1
fi

echo "Building TOS Services..."
make build-services

# Create directories
echo "Creating system directories..."
sudo mkdir -p "$BINDIR" "$XSESSIONSDIR" "/etc/tos" "/var/log/tos"

# Install binaries
echo "Installing binaries..."
sudo cp "$VERSION_DIR/target/release/tos" "$BINDIR/tos"
sudo cp "$VERSION_DIR/target/release/tos-brain" "$BINDIR/tos-brain"
sudo cp "$VERSION_DIR/target/release/marketplaced" "$BINDIR/marketplaced"
sudo cp "$VERSION_DIR/target/release/settingsd" "$BINDIR/settingsd"
sudo cp "$VERSION_DIR/target/release/prioritd" "$BINDIR/priorityd" || true
sudo cp "$VERSION_DIR/target/release/sessiond" "$BINDIR/sessiond"
sudo cp "$VERSION_DIR/target/release/loggerd" "$BINDIR/loggerd"
sudo cp "$VERSION_DIR/target/release/searchd" "$BINDIR/searchd"
sudo cp "$VERSION_DIR/target/release/heuristicd" "$BINDIR/heuristicd" || true

# Install configuration (if any)
if [ -f "$VERSION_DIR/tos.toml" ]; then
    echo "Installing configuration..."
    sudo cp "$VERSION_DIR/tos.toml" /etc/tos/
fi

# Install desktop file
if [ -f "$VERSION_DIR/packaging/tos.desktop" ]; then
    sudo cp "$VERSION_DIR/packaging/tos.desktop" "$XSESSIONSDIR/tos.desktop"
    sudo chmod 644 "$XSESSIONSDIR/tos.desktop"
fi

# Set permissions
sudo chmod 755 "$BINDIR/"tos*
sudo chmod 777 "/var/log/tos" # Allow TOS daemons to write logs

echo "TOS Beta-0 installed successfully. Select 'TOS' from your login screen session menu."
