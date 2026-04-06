#!/usr/bin/env bash
set -e

# Add flutter and cargo to PATH
export PATH="$HOME/flutter/bin:$HOME/.cargo/bin:$PATH"

echo "Generating Rust bridge code..."
mkdir -p lib/src/rust
flutter_rust_bridge_codegen generate \
    --rust-input crate::api \
    --rust-root rust/ \
    --dart-output lib/src/rust/
echo "Generation complete."
