use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use std::sync::{Arc, Mutex};
use std::fs::{OpenOptions};
use std::io::Write;
use chrono::Local;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let port = 7003;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    // Log file management
    let log_path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join(".local/share/tos/system.log");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    println!("TOS-LOGGERD: Operational on {}", addr);
    println!("TOS-LOGGERD: Writing to {:?}", log_path);
    
    loop {
        let (socket, _) = listener.accept().await?;
        let log_path_clone = log_path.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, log_path_clone).await {
                eprintln!("[LOGGERD] Client error: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, log_path: std::path::PathBuf) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { break; }

        let request = line.trim();
        if request.is_empty() { continue; }

        let parts: Vec<&str> = request.splitn(2, ':').collect();
        if parts.len() < 2 {
             writer.write_all(b"ERROR: Malformed request\n").await?;
             continue;
        }

        let prefix = parts[0];
        let payload = parts[1];
        let args: Vec<&str> = payload.split(';').collect();

        let response = match prefix {
            "log" => {
                let message = args[0];
                let level = args.get(1).unwrap_or(&"1");
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                
                let log_entry = format!("[{}][LVL-{}] {}\n", timestamp, level, message);
                print!("{}", log_entry);
                
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                    let _ = file.write_all(log_entry.as_bytes());
                }

                // §2.7: Act as a true external IPC client by notifying the Brain
                if let Ok(mut brain_stream) = std::net::TcpStream::connect("127.0.0.1:7000") {
                    let _ = brain_stream.write_all(format!("system_log_append:{};{}\n", level, message).as_bytes());
                }

                "OK".to_string()
            },
            "query" => {
                // Future: Implement log query logic (§19.1)
                "ERROR: Query not implemented in Alpha".to_string()
            },
            _ => "ERROR: Unknown command".to_string(),
        };

        writer.write_all(format!("{}\n", response).as_bytes()).await?;
        writer.flush().await?;
    }
    Ok(())
}
