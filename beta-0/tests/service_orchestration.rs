use tokio::net::{TcpStream, UnixStream};
use std::time::Duration;
use tokio::time::sleep;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::Value;

#[tokio::test]
async fn test_service_orchestration_health() -> anyhow::Result<()> {
    println!("\n\x1B[1;35m[TOS ORCHESTRATION TEST]\x1B[0m");
    
    let socket_path = "/tmp/brain.sock";
    
    // 1. Resolve Port Map from Discovery Gate
    println!("Resolving tactical service constellation via Discovery Gate ({}) ...", socket_path);
    
    let mut stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(_) => {
            println!("\x1B[1;31m[CRITICAL] FAILED to connect to Discovery Gate. Is the Brain running?\x1B[0m");
            return Err(anyhow::anyhow!("Discovery Gate Unreachable"));
        }
    };

    stream.write_all(b"tos_ports:\n").await?;
    let (reader, _) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let entries: Vec<Value> = match serde_json::from_str(line.trim()) {
        Ok(v) => v,
        Err(_) => {
            println!("ERROR: Failed to parse port map response: {}", line);
            return Err(anyhow::anyhow!("Invalid Discovery Response"));
        }
    };

    println!("Found {} registered services. Verifying reachability...\n", entries.len());

    let mut failed = false;
    for e in entries {
        let name = e["name"].as_str().unwrap_or("-");
        let port = e["port"].as_u64().unwrap_or(0);
        let host = e["host"].as_str().unwrap_or("127.0.0.1");
        
        // Handle "0.0.0.0" as "127.0.0.1" for local checks
        let connect_host = if host == "0.0.0.0" { "127.0.0.1" } else { host };

        print!("Checking {:<22} [Port {}] ... ", name, port);
        
        let mut connected = false;
        // Retry a few times as services might be booting
        for _ in 0..3 {
            if let Ok(_) = TcpStream::connect(format!("{}:{}", connect_host, port)).await {
                connected = true;
                break;
            }
            sleep(Duration::from_millis(200)).await;
        }

        if connected {
            println!("\x1B[1;32mONLINE\x1B[0m");
        } else {
            println!("\x1B[1;31mOFFLINE\x1B[0m");
            failed = true;
        }
    }

    if failed {
        println!("\n\x1B[1;31m[CRITICAL] ORCHESTRATION FAILURE: One or more services are unreachable.\x1B[0m");
        println!("Check orchestration logs in logs/*.log for details.");
        return Err(anyhow::anyhow!("Orchestration Health Check Failed"));
    }

    println!("\n\x1B[1;32m[SUCCESS] Full tactical stack is orchestrated and responsive.\x1B[0m");
    Ok(())
}
