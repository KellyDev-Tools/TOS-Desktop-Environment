use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use std::fs::{OpenOptions};
use std::io::Write;
use chrono::Local;

#[derive(serde::Serialize, serde::Deserialize)]
struct LogRecord {
    ts: i64,
    level: String,
    source: String,
    event: String,
    data: String,
}

#[derive(serde::Deserialize)]
struct QueryRequest {
    surface: Option<String>,
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let port = 7003;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    // Log file management (JSONL format for Alpha-2.1)
    let log_path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join(".local/share/tos/system.jsonl");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    println!("TOS-LOGGERD: Operational on {}", addr);
    println!("TOS-LOGGERD: Storage: {:?}", log_path);
    
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

        let response = match prefix {
            "log" => {
                let args: Vec<&str> = payload.split(';').collect();
                let message = args[0];
                let level = args.get(1).unwrap_or(&"1");
                let source = args.get(2).unwrap_or(&"system");
                
                let record = LogRecord {
                    ts: Local::now().timestamp(),
                    level: level.to_string(),
                    source: source.to_string(),
                    event: "log".to_string(),
                    data: message.to_string(),
                };
                
                let json_entry = serde_json::to_string(&record).unwrap_or_default();
                println!("[{}][LVL-{}] [{}] {}", Local::now().format("%H:%M:%S"), level, source, message);
                
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                    let _ = writeln!(file, "{}", json_entry);
                }

                if let Ok(mut brain_stream) = std::net::TcpStream::connect_timeout(&"127.0.0.1:7000".parse().unwrap(), std::time::Duration::from_millis(50)) {
                    let _ = brain_stream.write_all(format!("system_log_append:{};{}\n", level, message).as_bytes());
                }

                "OK".to_string()
            },
            "query" => {
                let req: QueryRequest = match serde_json::from_str(payload) {
                    Ok(r) => r,
                    Err(e) => {
                        writer.write_all(format!("ERROR: Invalid query JSON: {}\n", e).as_bytes()).await?;
                        continue;
                    }
                };

                let limit = req.limit.unwrap_or(50);
                let mut results = Vec::new();

                if let Ok(file) = std::fs::File::open(&log_path) {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(file);
                    // For Alpha, we do a simple reverse scan (limited)
                    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
                    for line in lines.iter().rev() {
                        if results.len() >= limit { break; }
                        if let Ok(record) = serde_json::from_str::<LogRecord>(line) {
                            // Simple filtering
                            if let Some(s) = &req.surface {
                                if record.source != *s { continue; }
                            }
                            results.push(record);
                        }
                    }
                }

                let output = serde_json::json!({
                    "query_id": uuid::Uuid::new_v4().to_string(),
                    "results": results
                });
                serde_json::to_string(&output).unwrap_or_default()
            },
            _ => "ERROR: Unknown command".to_string(),
        };

        writer.write_all(format!("{}\n", response).as_bytes()).await?;
        writer.flush().await?;
    }
    Ok(())
}
