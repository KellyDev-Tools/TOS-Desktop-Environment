#!/bin/bash
# scripts/release.sh - Creates the canonical beta-0 release asset bundle

set -e

VERSION="0.1.0-beta.0"
OUTPUT_DIR="targets/release_$VERSION"
TAR_NAME="tos-beta-0-$VERSION-linux-x86_64.tar.gz"

echo "[RELEASE] Establishing Beta-0 Output Directory: $OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/bin"
mkdir -p "$OUTPUT_DIR/share/xsessions"
mkdir -p "$OUTPUT_DIR/var/log/tos"

echo "[RELEASE] Compiling Svelte UI Framework..."
cd svelte_ui
npm i && npm run build
cd ..

echo "[RELEASE] Compiling Native Daemons (Release Mode)..."
cargo build --release --workspace

echo "[RELEASE] Bundling Binary Assets..."
cp target/release/tos "$OUTPUT_DIR/bin/"
cp target/release/tos-brain "$OUTPUT_DIR/bin/"
for daemon in tos-settingsd tos-marketplaced tos-sessiond tos-loggerd searchd tos-heuristicd tos-priorityd; do
    if [ -f "target/release/$daemon" ]; then
        cp "target/release/$daemon" "$OUTPUT_DIR/bin/"
    fi
done

echo "[RELEASE] Bundling Subsystems & Configurations..."
cp tos.toml "$OUTPUT_DIR/"
cp packaging/tos.desktop "$OUTPUT_DIR/share/xsessions/"

echo "[RELEASE] Packaging Tarball..."
cd targets
tar -czvf "$TAR_NAME" "release_$VERSION"
cd ..

echo "[RELEASE] SUCCESS. Artifact available at: targets/$TAR_NAME"
# Note: Code signing handled by CI Pipeline/Generate Signed Release Assets Gate.
