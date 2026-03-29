use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tos_common::SettingsStore;
use tos_common::services::settings::SettingsService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Bind to ephemeral port for dynamic registration (§4.1)
    let listener = match TcpListener::bind("0.0.0.0:0").await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(
                "TOS-SETTINGSD ERROR: Failed to bind to ephemeral port: {}",
                e
            );
            return Err(e.into());
        }
    };
    let port = listener.local_addr()?.port();
    tracing::info!("TOS-SETTINGSD: Listening on port {}", port);

    // §4.1: Dynamic Port Registration Gate
    tos_common::register_with_brain("tos-settingsd", port).await?;

    // The actual SettingsService logic (I/O, persistence)
    let service = Arc::new(SettingsService::new());
    // Use local load to avoid recursion if a previous daemon resides on the port
    let current_settings = match service.load_local() {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            tracing::error!("TOS-SETTINGSD ERROR: Failed to load local settings: {}", e);
            return Err(e.into());
        }
    };

    tracing::info!("TOS-SETTINGSD: Operational on port {}", port);

    loop {
        let (socket, _) = listener.accept().await?;
        let service_clone = service.clone();
        let settings_clone = current_settings.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, service_clone, settings_clone).await {
                tracing::error!("[SETTINGSD] Client error: {}", e);
            }
        });
    }
}

async fn handle_client(
    mut socket: TcpStream,
    service: Arc<SettingsService>,
    settings: Arc<Mutex<SettingsStore>>,
) -> anyhow::Result<()> {
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
        if parts.len() < 2 {
            writer.write_all(b"ERROR: Malformed request\n").await?;
            continue;
        }

        let prefix = parts[0];
        let payload = parts[1];
        let args: Vec<&str> = payload.split(';').collect();

        let response = match prefix {
            "get_all" => {
                let lock = settings.lock().unwrap();
                serde_json::to_string(&*lock)
                    .unwrap_or_else(|_| "ERROR: Serialization failed".to_string())
            }
            "get_setting" => {
                let key = args[0];
                let lock = settings.lock().unwrap();
                lock.global.get(key).cloned().unwrap_or_default()
            }
            "set_setting" => {
                if args.len() < 2 {
                    "ERROR: Key and value required".to_string()
                } else {
                    let key = args[0].to_string();
                    let val = args[1].to_string();
                    let mut lock = settings.lock().unwrap();
                    lock.global.insert(key.clone(), val.clone());
                    let _ = service.save(&*lock);

                    // §2.7: Notify Brain of external setting change
                    let config = tos_common::TosConfig::load();
                    let addr = format!("127.0.0.1:{}", config.remote.anchor_port);
                    if let Ok(mut brain_stream) = std::net::TcpStream::connect_timeout(
                        &addr.parse().unwrap(),
                        std::time::Duration::from_millis(50),
                    ) {
                        let _ = brain_stream
                            .write_all(format!("set_setting:{};{}\n", key, val).as_bytes());
                    }

                    "OK".to_string()
                }
            }
            "save" => {
                let lock = settings.lock().unwrap();
                match service.save(&*lock) {
                    Ok(_) => "OK".to_string(),
                    Err(e) => format!("ERROR: {}", e),
                }
            }
            "reload" => match service.load() {
                Ok(new_settings) => {
                    let mut lock = settings.lock().unwrap();
                    *lock = new_settings;
                    "OK".to_string()
                }
                Err(e) => format!("ERROR: {}", e),
            },
            _ => "ERROR: Unknown command".to_string(),
        };

        writer
            .write_all(format!("{}\n", response).as_bytes())
            .await?;
        writer.flush().await?;
    }
    Ok(())
}
