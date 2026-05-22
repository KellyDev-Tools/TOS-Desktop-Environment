use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" || args[1] == "help" {
        println!("\x1b[1;36mTOS // Tactical Command Utility\x1b[0m");
        println!("");
        println!("Usage: tos <command> [args]");
        println!("");
        println!("Commands:");
        println!("  ports   List all active Brain-managed services");
        println!("  status  Check Brain core health and connectivity");
        println!("  help    Show this help message");
        println!("");
        println!("Examples:");
        println!("  tos ports");
        return Ok(());
    }

    let cmd = &args[1];
    let socket_path = "/tmp/brain.sock";

    match cmd.as_str() {
        "ports" => {
            if !std::path::Path::new(socket_path).exists() {
                return Err(anyhow::anyhow!(
                    "Brain discovery gate not found at {}. Is the Brain running?",
                    socket_path
                ));
            }

            let mut stream = match UnixStream::connect(socket_path).await {
                Ok(s) => s,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to connect to Brain discovery gate at {}: {}",
                        socket_path, e
                    ));
                }
            };
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
                "\x1b[1m{:<22} {:<8} {:<15} STATUS\x1b[0m",
                "SERVICE", "PORT", "HOST"
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
        "status" => {
            print!("Checking Brain Core... ");
            if std::path::Path::new(socket_path).exists() {
                match UnixStream::connect(socket_path).await {
                    Ok(_) => println!("\x1b[32mONLINE\x1b[0m (UDS Socket Active)"),
                    Err(e) => println!("\x1b[31mUNREACHABLE\x1b[0m (Connect failed: {})", e),
                }
            } else {
                println!("\x1b[31mOFFLINE\x1b[0m (Socket missing)");
            }
        }
        _ => println!("ERROR: Unknown command '{}'. Try 'tos help'.", cmd),
    }

    Ok(())
}
