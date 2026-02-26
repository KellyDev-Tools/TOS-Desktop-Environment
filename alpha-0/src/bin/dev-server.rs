// Development Monitor Server (standalone)
// Starts the HTTP/WebSocket server for browser-based monitoring.
//
// Usage:
//   cargo run --features dev-monitor --bin dev-server
//   cargo run --features dev-monitor --bin dev-server -- 8080

#[cfg(feature = "dev-monitor")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .skip(1)
        .find(|arg| arg.parse::<u16>().is_ok())
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(3000);

    println!("{}", "═".repeat(58));
    println!("  TOS Development Monitor Server");
    println!("{}", "═".repeat(58));
    println!();
    println!("  HTTP:      http://127.0.0.1:{}", port);
    println!("  WebSocket: ws://127.0.0.1:{}/ws", port);
    println!();
    println!("  Connect a browser, then run tests or the demo backend:");
    println!();
    println!("    cargo run --features dev-monitor --bin demo-backend");
    println!("    cargo test --features dev-monitor --test visual_navigation -- --include-ignored");
    println!();
    println!("{}", "═".repeat(58));
    println!();

    let monitor = tos_comp::dev_monitor::DevMonitor::new(port);
    tos_comp::dev_monitor::init_global_monitor(monitor.get_broadcaster());

    monitor.run().await?;
    Ok(())
}

#[cfg(not(feature = "dev-monitor"))]
fn main() {
    eprintln!("Error: dev-server requires the 'dev-monitor' feature.");
    eprintln!("Run with: cargo run --features dev-monitor --bin dev-server");
    std::process::exit(1);
}
