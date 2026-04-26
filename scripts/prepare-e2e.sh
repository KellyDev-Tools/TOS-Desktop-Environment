#!/bin/bash
# scripts/prepare-e2e.sh - Sequential build for WSL stability
set -e

echo "[TOS E2E] Starting sequential build pipeline..."

# Throttling to prevent WSL OOM
export CARGO_BUILD_JOBS=2

# 1. Clean (Optional but recommended if space was an issue)
# echo "[TOS E2E] Cleaning targets..."
# cargo clean

# 2. Build Daemons
echo "[TOS E2E] Building all daemons (throttled)..."
cargo build --bins

# 3. Build Face UI
echo "[TOS E2E] Building Face UI (Static)..."
cd face-svelte-ui
# npm install --no-audit --no-fund
npm run build
cd ..

# 4. Create Sentinel
echo "[TOS E2E] Environment READY."
touch .e2e_ready
