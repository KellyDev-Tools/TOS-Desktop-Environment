use tokio::net::{TcpListener};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use std::sync::{Arc};
use tos_alpha2::services::marketplace::MarketplaceService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let port = 7004;
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    println!("TOS-MARKETPLACED: Operational on {}", addr);
    
    loop {
        let (socket, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("[MARKETPLACED] Client error: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream) -> anyhow::Result<()> {
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
            "discover" => {
                let path = std::path::PathBuf::from(args[0]);
                match MarketplaceService::discover_module(path) {
                    Ok(m) => serde_json::to_string(&m).unwrap_or_else(|_| "ERROR: Serialization failed".to_string()),
                    Err(e) => format!("ERROR: {}", e),
                }
            },
            "verify" => {
                let manifest_json = args[0];
                match serde_json::from_str::<tos_alpha2::services::marketplace::ModuleManifest>(manifest_json) {
                    Ok(m) => {
                         match MarketplaceService::get_trusted_public_key() {
                             Ok(pk) => {
                                 if MarketplaceService::verify_manifest(&m, &pk) {
                                     "VALID".to_string()
                                 } else {
                                     "INVALID".to_string()
                                 }
                             },
                             Err(e) => format!("ERROR: PK retrieval failed: {}", e),
                         }
                    },
                    Err(e) => format!("ERROR: Invalid manifest JSON: {}", e),
                }
            },
            _ => "ERROR: Unknown command".to_string(),
        };

        writer.write_all(format!("{}\n", response).as_bytes()).await?;
        writer.flush().await?;
    }
    Ok(())
}
