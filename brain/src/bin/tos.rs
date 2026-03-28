use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("TOS // Tactical Command Utility");
        println!("Usage: tos <command> [args]");
        println!("Commands:");
        println!("  ports  List all active Brain-managed services");
        return Ok(());
    }

    let cmd = &args[1];
    match cmd.as_str() {
        "ports" => {
            let socket_path = "/tmp/brain.sock";
            if !std::path::Path::new(socket_path).exists() {
                return Err(anyhow::anyhow!(
                    "Brain discovery gate not found at {}. Is the Brain running?",
                    socket_path
                ));
            }

            let mut stream = UnixStream::connect(socket_path).await?;
            stream.write_all(b"tos_ports:\n").await?;

            let (reader, _) = stream.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            if line.starts_with("ERROR") {
                println!("{}", line);
                return Ok(());
            }

            let entries: Vec<Value> = match serde_json::from_str(line.trim()) {
                Ok(v) => v,
                Err(_) => {
                    println!("ERROR: Failed to parse port map response: {}", line);
                    return Ok(());
                }
            };

            println!(
                "{:<22} {:<8} {:<15} {}",
                "SERVICE", "PORT", "HOST", "STATUS"
            );
            println!("{}", "-".repeat(56));
            for e in entries {
                let name = e["name"].as_str().unwrap_or("-");
                let port = e["port"].as_u64().unwrap_or(0);
                let host = e["host"].as_str().unwrap_or("-");
                let status = e["status"].as_str().unwrap_or("-");

                let status_fmt = if status == "ACTIVE" {
                    "\x1b[32mACTIVE\x1b[0m"
                } else {
                    "\x1b[31mDEAD\x1b[0m"
                };

                println!("{:<22} {:<8} {:<15} {}", name, port, host, status_fmt);
            }
        }
        _ => println!("ERROR: Unknown command '{}'", cmd),
    }

    Ok(())
}
