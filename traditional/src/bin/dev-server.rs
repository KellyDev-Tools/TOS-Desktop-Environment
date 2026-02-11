// Development Monitor Server
// Usage: cargo run --features dev-monitor --bin dev-server -- --port 3000

#![cfg(feature = "dev-monitor")]

use tos_comp::dev_monitor::DevMonitor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|arg| {
            if arg == "--port" {
                std::env::args().nth(2).and_then(|p| p.parse().ok())
            } else {
                arg.parse().ok()
            }
        })
        .unwrap_or(3000);

    println!("{}", "=".repeat(60));
    println!("  TOS Development Monitor");
    println!("{}", "=".repeat(60));
    println!();
    println!("  HTTP Server: http://127.0.0.1:{}", port);
    println!("  WebSocket:   ws://127.0.0.1:{}/ws", port);
    println!();
    println!("  1. Open http://127.0.0.1:{} in your browser", port);
    println!("  2. Run tests with: cargo test --features dev-monitor \\");
    println!("                     --test visual_navigation -- --include-ignored");
    println!();
    println!("{}", "=".repeat(60));
    println!();

    let monitor = DevMonitor::new(port);
    
    // Initialize global broadcaster
    tos_comp::dev_monitor::init_global_monitor(monitor.get_broadcaster());
    
    // Start server
    monitor.run().await?;

    Ok(())
}
