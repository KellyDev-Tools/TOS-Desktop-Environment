#!/bin/bash
# TOS Pre-commit Hook
# Ensures workspace-wide stability before any code is committed.

set -e

echo "--------------------------------------------------------------------------------"
echo "TOS PRE-COMMIT GUARD"
echo "--------------------------------------------------------------------------------"

# Step 1: Fast Workspace Check
echo "[1/2] Checking workspace syntax and types..."
cargo check --workspace

# Step 2: Full Integration Test Suite
echo "[2/2] Running integration tests (Common, Brain, Shell, Search)..."
make test-all

echo "--------------------------------------------------------------------------------"
echo "SUCCESS: Workspace is stable. Proceeding with commit."
echo "--------------------------------------------------------------------------------"
exit 0
