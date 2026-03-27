# Development Monitor - Visual UI Testing

The Development Monitor allows you to watch UI tests execute in real-time in your browser, making it easy to debug and understand the UI state changes.

## Quick Start

### 1. Start the Dev Monitor Server

In one terminal, start the HTTP/WebSocket server:

```bash
cargo run --features dev-monitor --bin dev-server -- --port 3000
```

The server will output:

```
============================================================
  TOS Development Monitor
============================================================

  HTTP Server: http://127.0.0.1:3000
  WebSocket:   ws://127.0.0.1:3000/ws

  1. Open http://127.0.0.1:3000 in your browser
  2. Run tests with: cargo test --features dev-monitor \
                     --test visual_navigation -- --include-ignored

============================================================
```

### 2. Open Your Browser

Navigate to `http://127.0.0.1:3000` (or whatever port you specified).

You should see:
- The TOS LCARS interface
- A "🔴 DEV MONITOR ACTIVE" indicator in the top-right corner

### 3. Run Visual Tests

In a second terminal, run the visual tests:

```bash
# Run all visual tests
cargo test --features dev-monitor --test visual_navigation -- --include-ignored

# Run specific test
cargo test --features dev-monitor --test visual_navigation visual_full_navigation_session -- --include-ignored
```

### 4. Watch the Magic! ✨

As tests execute, you'll see:
- **Real-time UI updates** showing exactly what the test is doing
- **Test step annotations** appearing in the top-right panel
- **Assertions** color-coded (green ✓ for pass, red ✗ for fail)
- **Zoom level changes** morphing in real-time
- **Split views, red alerts**, and all other UI states

## Available Visual Tests

### `visual_full_navigation_session`
Demonstrates complete navigation through all 5 zoom levels:
1. Start at Level 1 (Root/Sectors)
2. Zoom into Work Sector (Level 2)
3. Multi-window picker (Level 3a)
4. Single window focus (Level 3)
5. Detail view (Level 4)
6. Raw buffer view (Level 5)
7. Intelligent zoom back out

### `visual_split_view_test`
Shows split-screen functionality:
- Entering split view with two surfaces
- Swapping primary/secondary panels
- Exiting split view

### `visual_red_alert_test`
Demonstrates red alert system:
- Normal state
- Critical notification triggers red alert
- Visual theme changes
- Clearing alert

## Creating Your Own Visual Tests

Use the `VisualTestEnv` helper to create broadcast-enabled tests:

```rust
#[test]
#[ignore]
fn my_visual_test() {
    let mut vt = VisualTestEnv::new("my_visual_test");
    
    vt.step("Setup environment");
    vt.env.dashboard.add_widget(Box::new(ClockWidget));
    vt.update_viewport();
    
    vt.step("Do something");
    vt.env.navigator.zoom_in(0);
    vt.update_viewport();
    
    vt.assert(
        vt.env.navigator.current_level == ZoomLevel::Level2Sector,
        "Should be at level 2"
    );
    
    vt.finish();
}
```

### Key Methods

- `vt.step(description)` - Announce a test step
- `vt.assert(condition, message)` - Make an assertion
- `vt.update_viewport()` - Broadcast viewport changes
- `vt.update_dashboard()` - Broadcast dashboard changes
- `vt.finish()` - Mark test complete

## Custom Port

To use a different port:

```bash
# Start server on port 8080
cargo run --features dev-monitor --bin dev-server -- --port 8080
```

Then update the WebSocket URL in `ui/index.html` if needed, or the HTML automatically retries connection.

## Architecture

```
┌─────────────────┐         ┌──────────────────┐
│  Visual Test    │         │  Dev Monitor     │
│  (Rust)         │────────▶│  Server (Rust)   │
│                 │ Broadcast│  HTTP + WS       │
└─────────────────┘         └────────┬─────────┘
                                     │
                                     │ WebSocket
                                     ▼
                            ┌─────────────────┐
                            │   Browser       │
                            │   (index.html)  │
                            │   Real-time UI  │
                            └─────────────────┘
```

### How It Works

1. **Dev Monitor Server** runs an HTTP server (serves UI files) and WebSocket server (for real-time updates)
2. **Visual Tests** use `VisualTestEnv` which automatically broadcasts all state changes
3. **Browser** connects to WebSocket and updates UI in real-time as messages arrive
4. **Global Broadcaster** allows any code to send updates via `dev_monitor::get_monitor()`

## Troubleshooting

### "Connection refused"
Make sure the dev-server is running first before opening the browser.

### "No updates in browser"
- Check browser console for WebSocket errors
- Verify tests are using `VisualTestEnv`
- Ensure `--include-ignored` flag is used (visual tests are ignored by default)

### "Tests run too fast"
The visual test utilities include automatic delays (300ms per step, 200ms per assertion). You can adjust these in `visual_test_utils.rs`.

## Performance Note

The dev monitor adds ~100-300ms delay per step for visualization. This is intentional to allow human observation. Production tests without `--features dev-monitor` run at full speed.
