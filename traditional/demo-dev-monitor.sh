#!/bin/bash

# Dev Monitor Demo Script

echo "=================================================="
echo "  TOS Development Monitor Demo"
echo "=================================================="
echo ""
echo "This script will:"
echo "  1. Start the dev monitor server on port 3000"
echo "  2. Wait for you to open the browser"
echo "  3. Run visual tests for you to watch"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Please run this from the /8TB/tos/traditional directory"
    exit 1
fi

# Start the dev server in the background
echo "Starting dev monitor server..."
cargo run --features dev-monitor --bin dev-server -- --port 3000 &
SERVER_PID=$!

# Wait for server to start
sleep 3

echo ""
echo "=================================================="
echo "  🌐 BROWSER TIME!"
echo "=================================================="
echo ""
echo "  Open your browser to:"
echo "  http://127.0.0.1:3000"
echo ""
echo "  You should see:"
echo "  - The LCARS TOS interface"
echo "  - A red '🔴 DEV MONITOR ACTIVE' badge in top-right"
echo ""
read -p "Press ENTER when your browser is ready..." 

echo ""
echo "=================================================="
echo "  Running Visual Tests..."
echo "=================================================="
echo ""
echo "Watch your browser! You'll see:"
echo "- UI updates in real-time"
echo "- Test steps and assertions"
echo "- Zoom levels morphing"
echo "- All 3 visual tests running"
echo ""

# Run the visual tests
cargo test --features dev-monitor --test visual_navigation -- --include-ignored

echo ""
echo "=================================================="
echo "  Demo Complete!"
echo "=================================================="
echo ""
echo "The server is still running. You can:"
echo "  - Re-run tests manually"
echo "  - Create your own visual tests"
echo "  - Keep the browser open to inspect"
echo ""
read -p "Press ENTER to stop the server and exit..." 

# Kill the server
kill $SERVER_PID 2>/dev/null

echo ""
echo "Dev monitor stopped. Goodbye!"
