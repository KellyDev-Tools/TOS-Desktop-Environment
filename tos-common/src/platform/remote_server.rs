use crate::brain::ipc_handler::IpcHandler;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use rcgen::generate_simple_self_signed;

pub struct RemoteServer {
    ipc: Arc<IpcHandler>,
}

impl RemoteServer {
    pub fn new(ipc: Arc<IpcHandler>) -> Self {
        Self { ipc }
    }

    /// §12.1: Start the Remote Server daemons
    pub async fn run(&self, port: u16) -> anyhow::Result<()> {
        let tcp_addr = format!("[::]:{}", port);
        let ws_addr = format!("[::]:{}", port + 1); // e.g. 7001 for WebSocket
        let uds_path = "/tmp/brain.sock";

        // Bind with retry — previous Brain instance may have just been killed
        let tcp_listener = Self::bind_with_retry(&tcp_addr).await?;
        let ws_listener = Self::bind_with_retry(&ws_addr).await?;

        // §5.2: Advertise TOS Brain via mDNS (_tos-brain._tcp)
        let _mdns_handle = self.register_mdns(port);

        // Remove existing UDS if present (exists(), not is_file() — sockets aren't regular files)
        if std::path::Path::new(uds_path).exists() {
            let _ = std::fs::remove_file(uds_path);
        }
        let uds_listener = tokio::net::UnixListener::bind(uds_path)?;

        tracing::info!("[REMOTE_SERVER] TCP Listening on {}", tcp_addr);
        tracing::info!("[REMOTE_SERVER] WS  Listening on {}", ws_addr);
        tracing::info!("[REMOTE_SERVER] UDS (Discovery Gate) on {}", uds_path);

        let ipc_clone = self.ipc.clone();

        let tls_config = Self::generate_tls_config().expect("Failed to generate TLS cert");
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

        // Spawn TCP daemon
        let tcp_ipc = ipc_clone.clone();
        let tcp_tls = tls_acceptor.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = tcp_listener.accept().await {
                    let h_ipc = tcp_ipc.clone();
                    let h_tls = tcp_tls.clone();
                    tokio::spawn(async move {
                        if let Ok(tls_stream) = h_tls.accept(socket).await {
                            if let Err(e) = Self::handle_tcp_client(tls_stream, h_ipc).await {
                                tracing::error!("[REMOTE_SERVER] TCP Client error: {}", e);
                            }
                        } else {
                            tracing::error!("[REMOTE_SERVER] TCP TLS handshake failed");
                        }
                    });
                }
            }
        });

        // Spawn WebSocket daemon
        let ws_ipc = ipc_clone.clone();
        let ws_tls = tls_acceptor.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, addr)) = ws_listener.accept().await {
                    tracing::info!("[REMOTE_SERVER] WS Client connecting from {}", addr);
                    let h_ipc = ws_ipc.clone();
                    let h_tls = ws_tls.clone();
                    tokio::spawn(async move {
                        if let Ok(tls_stream) = h_tls.accept(socket).await {
                            if let Err(e) = Self::handle_ws_client(tls_stream, h_ipc).await {
                                tracing::error!("[REMOTE_SERVER] WS Client error ({}): {}", addr, e);
                            }
                        } else {
                            tracing::error!("[REMOTE_SERVER] WS TLS handshake failed ({})", addr);
                        }
                    });
                }
            }
        });

        // Spawn UDS daemon
        let uds_ipc = ipc_clone.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = uds_listener.accept().await {
                    let h_ipc = uds_ipc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_uds_client(socket, h_ipc).await {
                            tracing::error!("[REMOTE_SERVER] UDS Client error: {}", e);
                        }
                    });
                }
            }
        });

        // Keep the server (and its local variables like _mdns_handle) alive.
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }

    /// Attempt to bind a TCP port, retrying up to 3 times on AddrInUse.
    async fn bind_with_retry(addr: &str) -> anyhow::Result<TcpListener> {
        let max_retries = 3;
        for attempt in 1..=max_retries {
            match TcpListener::bind(addr).await {
                Ok(listener) => return Ok(listener),
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && attempt < max_retries => {
                    tracing::warn!(
                        "[REMOTE_SERVER] Port {} in use, retrying in 1s ({}/{})",
                        addr,
                        attempt,
                        max_retries
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
        unreachable!()
    }

    fn generate_tls_config() -> anyhow::Result<ServerConfig> {
        let cert = generate_simple_self_signed(vec![
            "localhost".into(),
            "127.0.0.1".into(),
            "tos-brain.local".into(),
        ])?;
        let key = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());
        let cert_der = CertificateDer::from(cert.cert.der().to_vec());
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key)?;
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(config)
    }

    async fn handle_uds_client(
        mut socket: tokio::net::UnixStream,
        ipc: Arc<IpcHandler>,
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

            let command = line.trim();
            if !command.is_empty() {
                let response = ipc.handle_request(command);
                writer
                    .write_all(format!("{}\n", response).as_bytes())
                    .await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn handle_tcp_client(socket: tokio_rustls::server::TlsStream<TcpStream>, ipc: Arc<IpcHandler>) -> anyhow::Result<()> {
        let (reader, mut writer) = tokio::io::split(socket);
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                break;
            }

            let command = line.trim();
            if !command.is_empty() {
                let response = ipc.handle_request(command);
                writer
                    .write_all(format!("{}\n", response).as_bytes())
                    .await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn handle_ws_client(socket: tokio_rustls::server::TlsStream<TcpStream>, ipc: Arc<IpcHandler>) -> anyhow::Result<()> {
        let ws_stream = accept_async(socket).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let push_ipc = ipc.clone();
        let push_tx = mpsc_tx.clone();
        
        let push_task = tokio::spawn(async move {
            let mut last_version = 0;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let request = format!("get_state_delta:{}", last_version);
                let response = push_ipc.handle_request(&request);
                if response != "NO_CHANGE" && !response.starts_with("ERROR") {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response) {
                        if let Some(v) = parsed.get("version").and_then(|v| v.as_u64()) {
                            last_version = v;
                        }
                    }
                    if push_tx.send(format!("state_delta:{}", response)).is_err() {
                        break;
                    }
                }
            }
        });

        loop {
            tokio::select! {
                Some(msg_result) = ws_rx.next() => {
                    let msg = match msg_result {
                        Ok(m) => m,
                        Err(_) => break,
                    };
                    if msg.is_text() {
                        if let Ok(command) = msg.to_text() {
                            let response = ipc.handle_request(command);
                            if mpsc_tx.send(response).is_err() {
                                break;
                            }
                        }
                    } else if msg.is_close() {
                        break;
                    }
                }
                Some(send_msg) = mpsc_rx.recv() => {
                    if ws_tx.send(tokio_tungstenite::tungstenite::Message::Text(send_msg)).await.is_err() {
                        break;
                    }
                }
                else => {
                    break;
                }
            }
        }
        
        push_task.abort();
        Ok(())
    }

    /// Registers the Brain as an mDNS service for zero-config discovery.
    fn register_mdns(&self, port: u16) -> Option<mdns_sd::ServiceDaemon> {
        use mdns_sd::{ServiceDaemon, ServiceInfo};
        use std::collections::HashMap;

        let mdns = match ServiceDaemon::new() {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("[MDNS] Failed to start mDNS daemon: {}", e);
                return None;
            }
        };

        let service_type = "_tos-brain._tcp.local.";
        let instance_name = "TOS-Brain";
        let host_name = "tos-brain.local.";
        let mut properties = HashMap::new();
        properties.insert("version".to_string(), "0.1.0-beta.0".to_string());
        properties.insert("vendor".to_string(), "TOS-Foundation".to_string());

        let my_service = ServiceInfo::new(
            service_type,
            instance_name,
            host_name,
            "", // IP is resolved automatically by mdns-sd
            port,
            Some(properties),
        )
        .expect("Failed to create mDNS service info");

        match mdns.register(my_service) {
            Ok(_) => {
                tracing::info!(
                    "[MDNS] Registered service: {} at port {}",
                    service_type,
                    port
                );
                Some(mdns)
            }
            Err(e) => {
                tracing::error!("[MDNS] Registration failed: {}", e);
                None
            }
        }
    }
}
