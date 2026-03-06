use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};

use tos_alpha2::services::marketplace::MarketplaceService;
use tos_protocol::marketplace::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    
    println!("TOS-MARKETPLACED: Operational on port {}", port);

    // Register with Brain via Unix socket
    let brain_sock_path = "/tmp/tos.brain.sock";
    tokio::spawn(async move {
        for _attempt in 1..=10 {
            match tokio::net::UnixStream::connect(brain_sock_path).await {
                Ok(mut sock) => {
                    let register_cmd = format!("service_register:tos-marketplaced;{}\n", port);
                    let _ = sock.write_all(register_cmd.as_bytes()).await;
                    println!("[MARKETPLACED] Registered with Brain on port {}", port);
                    break;
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    });
    
    loop {
        let (socket, _) = listener.accept().await?;
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("[MARKETPLACED] Client error: {}", e);
            }
        });
    }
}

async fn handle_client(socket: TcpStream) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { break; }

        let request = line.trim();
        if request.is_empty() { continue; }

        let parts: Vec<&str> = request.splitn(2, ':').collect();
        let prefix = parts[0];
        let payload = parts.get(1).unwrap_or(&"");

        let response = match prefix {
            "marketplace_home" => {
                let home = get_mock_home();
                serde_json::to_string(&home).unwrap_or_default()
            }
            "marketplace_category" => {
                let cat_id = payload.trim();
                let modules = get_mock_category_modules(cat_id);
                serde_json::to_string(&modules).unwrap_or_default()
            }
            "marketplace_detail" => {
                let mod_id = payload.trim();
                let detail = get_mock_detail(mod_id);
                serde_json::to_string(&detail).unwrap_or_default()
            }
            "marketplace_install" => {
                let mod_id = payload.trim().to_string();
                tokio::spawn(async move {
                    // Simulate install
                    println!("[MARKETPLACED] Starting install for {}", mod_id);
                });
                "INSTALLING".to_string()
            }
            "marketplace_status" => {
                let mod_id = payload.trim();
                let progress = InstallProgress {
                    module_id: mod_id.to_string(),
                    progress: 0.5,
                    status: "Downloading".to_string(),
                    error: None,
                };
                serde_json::to_string(&progress).unwrap_or_default()
            }
            "marketplace_search_ai" => {
                let query = payload.trim();
                let results = vec![get_mock_home().featured[0].clone()]; // Mock search
                serde_json::to_string(&results).unwrap_or_default()
            }
            "marketplace_install_cancel" => {
                let _mod_id = payload.trim();
                "CANCELLED".to_string()
            }
            "discover" => {
                let path = std::path::PathBuf::from(payload);
                match MarketplaceService::discover_module_local(path) {
                    Ok(m) => serde_json::to_string(&m).unwrap_or_else(|_| "ERROR: Serialization failed".to_string()),
                    Err(e) => format!("ERROR: {}", e),
                }
            },
            "verify" => {
                match serde_json::from_str::<tos_alpha2::services::marketplace::ModuleManifest>(payload) {
                    Ok(m) => {
                         match MarketplaceService::get_trusted_public_key() {
                             Ok(pk) => {
                                 if MarketplaceService::verify_manifest_local(&m, &pk) {
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

fn get_mock_home() -> MarketplaceHome {
    MarketplaceHome {
        featured: vec![
            MarketplaceModuleSummary {
                id: "tos-observer".to_string(),
                name: "Passive Observer".to_string(),
                module_type: "AI Behavior".to_string(),
                author: "TOS Team".to_string(),
                icon: Some("✦".to_string()),
                rating: 4.8,
                price: "Free".to_string(),
                installed: true,
            },
            MarketplaceModuleSummary {
                id: "tos-aurora-theme".to_string(),
                name: "Aurora Borealis".to_string(),
                module_type: "Theme".to_string(),
                author: "TOS Art".to_string(),
                icon: Some("⊞".to_string()),
                rating: 4.5,
                price: "$5.00".to_string(),
                installed: false,
            }
        ],
        categories: vec![
            MarketplaceCategory { id: "ai".to_string(), name: "AI Behaviors".to_string(), icon: "🧠".to_string(), module_count: 12 },
            MarketplaceCategory { id: "shell".to_string(), name: "Shell Modules".to_string(), icon: "🐚".to_string(), module_count: 5 },
            MarketplaceCategory { id: "theme".to_string(), name: "Themes".to_string(), icon: "🎨".to_string(), module_count: 24 },
        ]
    }
}

fn get_mock_category_modules(_cat_id: &str) -> Vec<MarketplaceModuleSummary> {
    vec![
        MarketplaceModuleSummary {
            id: "tos-observer".to_string(),
            name: "Passive Observer".to_string(),
            module_type: "AI Behavior".to_string(),
            author: "TOS Team".to_string(),
            icon: Some("✦".to_string()),
            rating: 4.8,
            price: "Free".to_string(),
            installed: true,
        },
        MarketplaceModuleSummary {
            id: "tos-chat".to_string(),
            name: "Chat Companion".to_string(),
            module_type: "AI Behavior".to_string(),
            author: "TOS Team".to_string(),
            icon: Some("💬".to_string()),
            rating: 4.9,
            price: "Free".to_string(),
            installed: true,
        },
    ]
}

fn get_mock_detail(id: &str) -> MarketplaceModuleDetail {
    MarketplaceModuleDetail {
        summary: MarketplaceModuleSummary {
            id: id.to_string(),
            name: "Passive Observer".to_string(),
            module_type: "AI Behavior".to_string(),
            author: "TOS Team".to_string(),
            icon: Some("✦".to_string()),
            rating: 4.8,
            price: "Free".to_string(),
            installed: true,
        },
        description: "A built-in AI behavior that monitors terminal output and suggests fixes for errors (127) or long-running tasks. Non-intrusive and non-blocking.".to_string(),
        screenshots: vec!["https://example.com/obs1.png".to_string()],
        permissions: vec!["terminal_read".to_string(), "system_log_write".to_string()],
        reviews: vec![
            MarketplaceReview {
                author: "Archer".to_string(),
                rating: 5,
                comment: "Saved me from typos a hundred times already!".to_string(),
                date: "2026-02-15".to_string(),
            }
        ],
    }
}
