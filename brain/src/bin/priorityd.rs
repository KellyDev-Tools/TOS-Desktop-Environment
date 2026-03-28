use std::collections::HashMap;
use std::io::Write;
use sysinfo::System;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Bind to ephemeral port for dynamic registration (§4.1)
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    tracing::info!("TOS-PRIORITYD: Listening on port {}", port);

    // §4.1: Dynamic Port Registration Gate
    tos_lib::daemon::register_with_brain("tos-priorityd", port).await?;

    let mut sys = System::new_all();

    loop {
        let (socket, _) = listener.accept().await?;
        // Refresh system metrics periodically or on request
        sys.refresh_all();

        let cpu_load = sys.global_cpu_info().cpu_usage();
        let mem_usage = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, cpu_load, mem_usage).await {
                tracing::error!("[PRIORITYD] Client error: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, cpu_load: f32, mem_usage: f32) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let request = line.trim();
        if request.is_empty() {
            continue;
        }

        let parts: Vec<&str> = request.splitn(2, ':').collect();
        let prefix = parts[0];
        let payload = parts.get(1).unwrap_or(&"");

        let response = match prefix {
            "get_priority" => {
                let sector_id = payload;
                // §21.2: Scoring heuristics
                let mut factors = HashMap::new();

                // Mocking recency and focus data for Alpha
                factors.insert("recency".to_string(), 0.75);
                factors.insert("activity_cpu".to_string(), cpu_load / 100.0);
                factors.insert("activity_mem".to_string(), mem_usage / 100.0);
                factors.insert(
                    "sector_context".to_string(),
                    if sector_id.is_empty() { 0.0 } else { 0.1 },
                );

                let total_score = (0.4 * 0.75) + (0.15 * (cpu_load / 100.0)) + (0.1 * 0.1); // Simplified
                let rank = (total_score * 4.0).round() as u8 + 1;

                let score = serde_json::json!({
                    "total_score": total_score,
                    "rank": rank,
                    "factors": factors
                });

                // Trigger tactile feedback if priority is high (§21.1)
                if rank >= 4 {
                    let config = tos_lib::config::TosConfig::load();
                    let addr = format!("127.0.0.1:{}", config.remote.anchor_port);
                    if let Ok(mut haptic_stream) = std::net::TcpStream::connect_timeout(
                        &addr.parse().unwrap(),
                        std::time::Duration::from_millis(50),
                    ) {
                        let _ = haptic_stream.write_all(b"trigger_haptic:priority_alert\n");
                    }
                }

                serde_json::to_string(&score).unwrap()
            }
            _ => "ERROR: Unknown command".to_string(),
        };

        writer
            .write_all(format!("{}\n", response).as_bytes())
            .await?;
        writer.flush().await?;
    }
    Ok(())
}
