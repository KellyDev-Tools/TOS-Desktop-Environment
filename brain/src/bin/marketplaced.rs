use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use tos_common::marketplace::*;
use tos_lib::services::marketplace::MarketplaceService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();

    tracing::info!("TOS-MARKETPLACED: Operational on port {}", port);

    // §4.1: Dynamic Port Registration Gate
    tos_lib::daemon::register_with_brain("tos-marketplaced", port).await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                tracing::error!("[MARKETPLACED] Client error: {}", e);
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
                    tracing::info!("[MARKETPLACED] Starting install for {}", mod_id);
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
                let query = payload.to_lowercase();
                let results = get_all_mock_modules()
                    .into_iter()
                    .filter(|m| {
                        m.name.to_lowercase().contains(&query)
                            || m.module_type.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>();
                serde_json::to_string(&results).unwrap_or_default()
            }
            "marketplace_install_cancel" => {
                let _mod_id = payload.trim();
                "CANCELLED".to_string()
            }
            "discover" => {
                let path = std::path::PathBuf::from(payload);
                match MarketplaceService::discover_module_local(path) {
                    Ok(m) => serde_json::to_string(&m)
                        .unwrap_or_else(|_| "ERROR: Serialization failed".to_string()),
                    Err(e) => format!("ERROR: {}", e),
                }
            }
            "verify" => {
                match serde_json::from_str::<tos_lib::services::marketplace::ModuleManifest>(
                    payload,
                ) {
                    Ok(m) => match MarketplaceService::get_trusted_public_key() {
                        Ok(pk) => {
                            if MarketplaceService::verify_manifest_local(&m, &pk) {
                                "VALID".to_string()
                            } else {
                                "INVALID".to_string()
                            }
                        }
                        Err(e) => format!("ERROR: PK retrieval failed: {}", e),
                    },
                    Err(e) => format!("ERROR: Invalid manifest JSON: {}", e),
                }
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

fn get_all_mock_modules() -> Vec<MarketplaceModuleSummary> {
    let mut modules = Vec::new();
    let mut base_path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    base_path.push(".config/tos/modules");

    // Recurse through all categories
    let categories = vec!["ai", "terminal", "themes", "renderers", "templates"];
    for cat in categories {
        let mut cat_path = base_path.clone();
        cat_path.push(cat);

        if let Ok(entries) = std::fs::read_dir(cat_path) {
            for entry in entries.flatten() {
                if let Ok(manifest) = MarketplaceService::discover_module_local(entry.path()) {
                    modules.push(MarketplaceModuleSummary {
                        id: manifest.id,
                        name: manifest.name,
                        module_type: manifest.module_type,
                        author: manifest.author,
                        icon: manifest.icon,
                        rating: 4.5, // Default for mock
                        price: "Free".to_string(),
                        installed: true, // If we found it in .config/tos/modules, it's "installed"
                    });
                }
            }
        }
    }

    // Add some "uninstalled" ones for variety if we have few
    modules
}

fn get_mock_home() -> MarketplaceHome {
    let all = get_all_mock_modules();
    let mut featured = Vec::new();

    // Pick first few as featured if they exist
    if !all.is_empty() {
        featured.push(all[0].clone());
    }
    if all.len() > 3 {
        featured.push(all[3].clone());
    }
    if all.len() > 5 {
        featured.push(all[5].clone());
    }

    MarketplaceHome {
        featured,
        categories: vec![
            MarketplaceCategory {
                id: "ai".to_string(),
                name: "AI Behaviors".to_string(),
                icon: "🧠".to_string(),
                module_count: all
                    .iter()
                    .filter(|m| m.module_type.to_lowercase().contains("ai"))
                    .count() as u32,
            },
            MarketplaceCategory {
                id: "shell".to_string(),
                name: "Shell Modules".to_string(),
                icon: "🐚".to_string(),
                module_count: all
                    .iter()
                    .filter(|m| m.module_type.to_lowercase().contains("shell"))
                    .count() as u32,
            },
            MarketplaceCategory {
                id: "theme".to_string(),
                name: "Themes".to_string(),
                icon: "🎨".to_string(),
                module_count: all
                    .iter()
                    .filter(|m| m.module_type.to_lowercase().contains("theme"))
                    .count() as u32,
            },
            MarketplaceCategory {
                id: "renderer".to_string(),
                name: "Terminal Modules".to_string(),
                icon: "📺".to_string(),
                module_count: all
                    .iter()
                    .filter(|m| {
                        m.module_type.to_lowercase().contains("renderer")
                            || m.module_type.to_lowercase().contains("terminaloutput")
                    })
                    .count() as u32,
            },
            MarketplaceCategory {
                id: "template".to_string(),
                name: "Templates".to_string(),
                icon: "📝".to_string(),
                module_count: all
                    .iter()
                    .filter(|m| m.module_type.to_lowercase().contains("template"))
                    .count() as u32,
            },
        ],
    }
}

fn get_mock_category_modules(cat_id: &str) -> Vec<MarketplaceModuleSummary> {
    get_all_mock_modules()
        .into_iter()
        .filter(|m| {
            let m_type = m.module_type.to_lowercase();
            match cat_id {
                "ai" => m_type.contains("ai"),
                "shell" => m_type.contains("shell"),
                "theme" => m_type.contains("theme"),
                "renderer" => m_type.contains("renderer") || m_type.contains("terminaloutput"),
                "template" => m_type.contains("template"),
                _ => true,
            }
        })
        .collect()
}

fn get_mock_detail(id: &str) -> MarketplaceModuleDetail {
    let summary = get_all_mock_modules()
        .into_iter()
        .find(|m| m.id == id)
        .unwrap_or_else(|| MarketplaceModuleSummary {
            id: id.to_string(),
            name: "Unknown Module".to_string(),
            module_type: "Unknown".to_string(),
            author: "Unknown".to_string(),
            icon: None,
            rating: 0.0,
            price: "N/A".to_string(),
            installed: false,
        });

    let description = match id {
        "tos-observer" => "A built-in AI behavior that monitors terminal output and suggests fixes for errors (127) or long-running tasks. Non-intrusive and non-blocking.".to_string(),
        "tos-chat" => "Your primary conversational interface for TOS. Supports deep context awareness, code staging, and multi-sector history.".to_string(),
        "tos-shell-fish" => "The canonical TOS shell module. High-performance, with full OSC sequence support for mode-aware transitions and heuristic path completion.".to_string(),
        "tos-aurora-theme" => "A premium theme inspired by the Northern Lights. Deep blues and vibrant teals with glassmorphic transparency.".to_string(),
        "tos-cinematic" => "A wide-screen terminal rendering module with high-fidelity typography and dynamic layout adjustments for presentation-grade output.".to_string(),
        "tos-retro-crt" => "Brings back the glow. Simulated phosphor persistence, scanlines, and subtle spherical curvature for that classic mainframe feel.".to_string(),
        "tos-monochrome" => "Peak efficiency. A black-and-white theme with high contrast and zero distractions. Optimized for clarity.".to_string(),
        "tos-sentinel" => "Enterprise-grade security monitoring. Analyzes command patterns for suspicious privilege escalation and bulk destructive operations.".to_string(),
        "tos-dev-layout" => "A pre-configured Split Viewport template optimized for Rust/Svelte development. Includes a 3-pane layout with dedicated terminal, logs, and a file browser.".to_string(),
        _ => "No detailed description available for this module.".to_string(),
    };

    let permissions = match id {
        "tos-observer" | "tos-sentinel" => {
            vec!["terminal_read".to_string(), "system_log_write".to_string()]
        }
        "tos-chat" => vec![
            "terminal_read".to_string(),
            "terminal_write".to_string(),
            "filesystem_read".to_string(),
        ],
        "tos-shell-fish" => vec!["pty_access".to_string(), "filesystem_full".to_string()],
        "tos-aurora-theme" | "tos-monochrome" => vec!["ui_style_override".to_string()],
        "tos-cinematic" | "tos-retro-crt" => vec!["terminal_render_hook".to_string()],
        _ => vec![],
    };

    MarketplaceModuleDetail {
        summary,
        description,
        screenshots: vec!["https://tos.live/assets/modules/preview.png".to_string()],
        permissions,
        reviews: vec![MarketplaceReview {
            author: "Archer".to_string(),
            rating: 5,
            comment: "Essential for any TOS installation.".to_string(),
            date: "2026-03-01".to_string(),
        }],
    }
}
