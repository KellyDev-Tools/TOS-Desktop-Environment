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
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::interceptor::registry::Registry;

/// The Remote Server manages TCP, WebSocket, and UDS connections to the Brain.
/// 
/// It handles TLS encryption for network transport and provides discovery 
/// via mDNS as specified in §12.1 and §5.2.
pub struct RemoteServer {
    ipc: Arc<IpcHandler>,
    webrtc_sessions: Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, Arc<webrtc::peer_connection::RTCPeerConnection>>>>,
}

impl RemoteServer {
    /// Creates a new RemoteServer instance with the provided IPC handler.
    pub fn new(ipc: Arc<IpcHandler>) -> Self {
        Self { 
            ipc,
            webrtc_sessions: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
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

        let tls_config = Self::generate_tls_config()?;
        let _tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

        let server = Arc::new(Self::new(self.ipc.clone()));
        
        // Spawn TCP daemon
        let tcp_server = server.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = tcp_listener.accept().await {
                    let h_server = tcp_server.clone();
                    tokio::spawn(async move {
                        if let Ok(tls_stream) = h_server.tls_acceptor().accept(socket).await {
                            if let Err(e) = h_server.handle_tcp_client(tls_stream).await {
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
        let ws_server = server.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, addr)) = ws_listener.accept().await {
                    tracing::info!("[REMOTE_SERVER] WS Client connecting from {}", addr);
                    let h_server = ws_server.clone();
                    tokio::spawn(async move {
                        match h_server.tls_acceptor().accept(socket).await {
                            Ok(tls_stream) => {
                                if let Err(e) = h_server.handle_ws_client(tls_stream, addr).await {
                                    tracing::error!("[REMOTE_SERVER] WS Client error ({}): {}", addr, e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("[REMOTE_SERVER] WS TLS handshake failed ({}): {}", addr, e);
                            }
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
            "[::1]".into(),
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

    fn tls_acceptor(&self) -> TlsAcceptor {
        let tls_config = Self::generate_tls_config().expect("Failed to generate TLS config");
        TlsAcceptor::from(Arc::new(tls_config))
    }

    async fn handle_tcp_client(&self, socket: tokio_rustls::server::TlsStream<TcpStream>) -> anyhow::Result<()> {
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
                // §19.3: OpenSearch HTTP compatibility
                if command.starts_with("GET /opensearch.xml") {
                    let xml = self.generate_opensearch_xml();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/opensearchdescription+xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        xml.len(),
                        xml
                    );
                    writer.write_all(response.as_bytes()).await?;
                    writer.flush().await?;
                    return Ok(());
                } else if command.starts_with("GET /search?q=") {
                    let query = command.split("q=").nth(1).unwrap_or("").split(' ').next().unwrap_or("");
                    let ipc_cmd = format!("search:{}", query);
                    let results = self.ipc.handle_request(&ipc_cmd);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        results.len(),
                        results
                    );
                    writer.write_all(response.as_bytes()).await?;
                    writer.flush().await?;
                    return Ok(());
                }

                let response = self.ipc.handle_request(command);
                writer
                    .write_all(format!("{}\n", response).as_bytes())
                    .await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn handle_ws_client(&self, socket: tokio_rustls::server::TlsStream<TcpStream>, _addr: std::net::SocketAddr) -> anyhow::Result<()> {
        let ws_stream = accept_async(socket).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let push_ipc = self.ipc.clone();
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
                            let response = if command.starts_with("webrtc_signalling:") {
                                match self.handle_webrtc_signalling(command).await {
                                    Ok(res) => res,
                                    Err(e) => format!("ERROR: WebRTC Signalling failed: {}", e),
                                }
                            } else {
                                self.ipc.handle_request(command)
                            };
                            
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
        properties.insert("version".to_string(), "0.2.0-beta.0".to_string());
        properties.insert("vendor".to_string(), "TOS-Foundation".to_string());

        let my_service = ServiceInfo::new(
            service_type,
            instance_name,
            host_name,
            "", // IP is resolved automatically by mdns-sd
            port,
            Some(properties),
        )
        .map_err(|e| {
            tracing::error!("[MDNS] Failed to create service info: {}", e);
            e
        })
        .ok()?;

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

    /// Handles WebRTC signalling messages (SDP/ICE) from the Face.
    async fn handle_webrtc_signalling(&self, command: &str) -> anyhow::Result<String> {
        let payload = &command["webrtc_signalling:".len()..];
        let msg: crate::collaboration::WebRtcPayload = serde_json::from_str(payload)?;
        
        match msg {
            crate::collaboration::WebRtcPayload::SdpOffer { user, sdp } => {
                let pc = self.get_or_create_peer(user).await?;
                pc.set_remote_description(RTCSessionDescription::offer(sdp)?).await?;
                let answer = pc.create_answer(None).await?;
                let mut gather_complete: tokio::sync::mpsc::Receiver<()> = pc.gathering_complete_promise().await;
                pc.set_local_description(answer).await?;
                let _ = gather_complete.recv().await;

                if let Some(local_desc) = pc.local_description().await {
                    let response = crate::collaboration::WebRtcPayload::SdpAnswer {
                        user: uuid::Uuid::nil(),
                        sdp: local_desc.sdp,
                    };
                    return Ok(format!("webrtc_signalling:{}", serde_json::to_string(&response)?));
                }
                Err(anyhow::anyhow!("Failed to generate local description"))
            }
            crate::collaboration::WebRtcPayload::IceCandidate { user, candidate } => {
                let pc = self.get_or_create_peer(user).await?;
                pc.add_ice_candidate(webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
                    candidate,
                    ..Default::default()
                }).await?;
                Ok("OK".to_string())
            }
            _ => Ok("ERROR: Unsupported signalling variant".to_string()),
        }
    }

    /// Retrieves an existing PeerConnection or creates a new one for a participant.
    async fn get_or_create_peer(&self, user_id: uuid::Uuid) -> anyhow::Result<Arc<webrtc::peer_connection::RTCPeerConnection>> {
        let mut sessions = self.webrtc_sessions.lock().await;
        if let Some(pc) = sessions.get(&user_id) {
            return Ok(pc.clone());
        }

        let mut m = MediaEngine::default();
        m.register_default_codecs()?;
        
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let pc = Arc::new(api.new_peer_connection(config).await?);
        
        // §12.1: Add video track for streaming the Face UI
        let video_track = Arc::new(webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample::new(
            webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                mime_type: webrtc::api::media_engine::MIME_TYPE_VP8.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "tos-stream".to_owned(),
        ));
        
        pc.add_track(video_track.clone()).await?;

        // Handle connection state changes
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            tracing::info!("[WEBRTC] Peer {} state changed: {}", user_id, s);
            Box::pin(async {})
        }));

        sessions.insert(user_id, pc.clone());
        Ok(pc)
    }

    pub fn generate_opensearch_xml(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/">
  <ShortName>TOS Brain</ShortName>
  <Description>Search the TOS Brain indexed content</Description>
  <Tags>tos brain search ai</Tags>
  <Contact>admin@tos-brain.local</Contact>
  <Url type="application/json" method="GET" template="http://localhost:7000/search?q={searchTerms}"/>
  <Url type="text/html" method="GET" template="http://localhost:7000/search?q={searchTerms}"/>
</OpenSearchDescription>"#.to_string()
    }
}
