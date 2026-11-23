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

/// Phase 2 — three-stream narrowing.
///
/// The only command prefixes a remote client (WebSocket or TCP) may
/// invoke. Together with the Brain→face pushes (`display_update`,
/// `state_delta`, `res:` replies) this list is the operative
/// definition of the three streams the roadmap names — terminal,
/// chip state, and prompt:
///
/// - terminal: `display_subscribe`, `terminal_key`, `terminal_text`
///   (chip taps and the prompt bar also type through these),
///   `terminal_resize`, `terminal_signal`, `display_scroll`
/// - chip state: `get_state`, `get_state_delta`,
///   `chips_set_show_hidden` (the chips panel's dotfiles switch,
///   Phase 1.7 — it only flips a flag on the hub the streams
///   already describe)
/// - prompt: `prompt_submit`, `force_prompt_submit`
/// - session plumbing for those streams: `tos_ports` (discovery
///   probe), `face_register` (a face announces itself, then streams)
///
/// Everything else — every AI, marketplace, editor, kanban, process,
/// session-transfer, settings and sector command, and every grey-area
/// command not listed here — belongs to a parked subsystem and is
/// refused over remote (see `remote_refusal`). The UDS / local path
/// keeps the full surface; narrowing applies only where the caller
/// arrived over the network.
const REMOTE_ALLOWED_PREFIXES: &[&str] = &[
    "display_subscribe",
    "terminal_key",
    "terminal_text",
    "terminal_resize",
    "terminal_signal",
    "display_scroll",
    "prompt_submit",
    "force_prompt_submit",
    "placement_sync_apps",
    "placement_choose",
    "placement_relaunch",
    "get_state",
    "get_state_delta",
    "chips_set_show_hidden",
    "tos_ports",
    "face_register",
];

/// If `request` is a command outside the three streams, returns the
/// refusal response to send back; otherwise returns None and the
/// caller dispatches normally.
///
/// The refusal is loud on purpose — an `ERROR:` the caller can see,
/// in the same framing the real response would have used (a
/// `cmd:<id>:`-wrapped request gets a `res:<id>:`-wrapped refusal) —
/// so a parked caller fails visibly instead of hanging.
fn remote_refusal(request: &str) -> Option<String> {
    // Unwrap the WebSocket `cmd:<id>:` framing exactly the way
    // IpcHandler::handle_request does, so the id survives.
    let (id, command): (Option<&str>, &str) = match request.strip_prefix("cmd:") {
        Some(rest) => match rest.split_once(':') {
            Some((id, actual)) => (Some(id), actual),
            None => (None, request),
        },
        None => (None, request),
    };
    let prefix = command.split(':').next().unwrap_or(command).trim();
    if REMOTE_ALLOWED_PREFIXES.contains(&prefix) {
        return None;
    }
    let message = format!(
        "ERROR: {} is not available over a remote connection (three-stream narrowing: terminal, chip state, prompt only)",
        prefix
    );
    Some(match id {
        Some(id) => format!("res:{}:{}", id, message),
        None => message,
    })
}

/// The Remote Server manages TCP, WebSocket, and UDS connections to the Brain.
/// 
/// It handles TLS encryption for network transport and provides discovery 
/// via mDNS as specified in §12.1 and §5.2.
pub struct RemoteServer {
    ipc: Arc<IpcHandler>,
    config: crate::config::RemoteConfig,
    webrtc_sessions: Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, Arc<webrtc::peer_connection::RTCPeerConnection>>>>,
}

impl RemoteServer {
    /// Creates a new RemoteServer instance with the provided IPC handler.
    pub fn new(ipc: Arc<IpcHandler>, config: crate::config::RemoteConfig) -> Self {
        Self {
            ipc,
            config,
            webrtc_sessions: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// The host the network listeners effectively bind to.
    ///
    /// Remote access off means loopback only, whatever bind_address
    /// says — fail closed, so a Brain is never reachable from the
    /// network unless remote access was explicitly enabled. Remote
    /// access on binds the configured address; a wildcard stays
    /// dual-stack `[::]`, as before Phase 2.
    fn effective_bind_host(config: &crate::config::RemoteConfig) -> String {
        if !config.remote_access {
            return "127.0.0.1".to_string();
        }
        match config.bind_address.trim() {
            "" | "0.0.0.0" | "::" | "[::]" => "[::]".to_string(),
            other => other.to_string(),
        }
    }

    /// Format a host:port bind address, bracketing IPv6 hosts.
    fn bind_addr(host: &str, port: u16) -> String {
        if host.contains(':') && !host.starts_with('[') {
            format!("[{}]:{}", host, port)
        } else {
            format!("{}:{}", host, port)
        }
    }

    /// Subject alt names for the self-signed TLS certificate: the
    /// loopback identities, plus — when remote access is on — the
    /// configured bind identity (e.g. the tailnet IP a face will
    /// dial) and this machine's hostname, so a face connecting by
    /// that name or address finds it in the certificate.
    fn tls_subject_alt_names(config: &crate::config::RemoteConfig) -> Vec<String> {
        let mut sans: Vec<String> = vec![
            "localhost".into(),
            "127.0.0.1".into(),
            "[::1]".into(),
            "tos-brain.local".into(),
        ];
        if config.remote_access {
            let bind = config.bind_address.trim();
            if !bind.is_empty()
                && bind != "0.0.0.0"
                && bind != "::"
                && !sans.iter().any(|s| s == bind)
            {
                sans.push(bind.to_string());
            }
            if let Ok(host) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
                let host = host.trim().to_string();
                if !host.is_empty() && !sans.contains(&host) {
                    sans.push(host);
                }
            }
        }
        sans
    }

    /// §12.1: Start the Remote Server daemons
    pub async fn run(&self) -> anyhow::Result<()> {
        let host = Self::effective_bind_host(&self.config);
        let tcp_addr = Self::bind_addr(&host, self.config.anchor_port);
        let ws_addr = Self::bind_addr(&host, self.config.ws_port);
        let uds_path = self.config.uds_path.as_str();
        if self.config.remote_access {
            tracing::info!(
                "[REMOTE_SERVER] Remote access ON — binding {} (config bind_address: {})",
                host,
                self.config.bind_address
            );
        } else {
            tracing::info!(
                "[REMOTE_SERVER] Remote access OFF — network listeners are loopback-only"
            );
        }

        // Bind with retry — previous Brain instance may have just been killed
        let tcp_listener = Self::bind_with_retry(&tcp_addr).await?;
        let ws_listener = Self::bind_with_retry(&ws_addr).await?;

        // §5.2: Advertise TOS Brain via mDNS (_tos-brain._tcp) — only
        // when remote access is on and discovery is enabled; a
        // loopback-only Brain has no business announcing itself.
        let _mdns_handle = if self.config.remote_access && self.config.mdns_enabled {
            self.register_mdns(self.config.anchor_port)
        } else {
            None
        };

        // Remove existing UDS if present (exists(), not is_file() — sockets aren't regular files)
        if std::path::Path::new(uds_path).exists() {
            let _ = std::fs::remove_file(uds_path);
        }
        let uds_listener = tokio::net::UnixListener::bind(uds_path)?;

        tracing::info!("[REMOTE_SERVER] TCP Listening on {}", tcp_addr);
        tracing::info!("[REMOTE_SERVER] WS  Listening on {}", ws_addr);
        tracing::info!("[REMOTE_SERVER] UDS (Discovery Gate) on {}", uds_path);

        let ipc_clone = self.ipc.clone();

        // Build the TLS acceptor ONCE and share it with both accept
        // loops. (Before Phase 2 the loops regenerated a fresh
        // self-signed certificate for every connection.)
        let tls_config = Self::generate_tls_config(&Self::tls_subject_alt_names(&self.config))?;
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

        let server = Arc::new(Self::new(self.ipc.clone(), self.config.clone()));
        
        // Spawn TCP daemon
        let tcp_server = server.clone();
        let tcp_tls = tls_acceptor.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = tcp_listener.accept().await {
                    let h_server = tcp_server.clone();
                    let conn_tls = tcp_tls.clone();
                    tokio::spawn(async move {
                        if let Ok(tls_stream) = conn_tls.accept(socket).await {
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
        let ws_tls = tls_acceptor.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, addr)) = ws_listener.accept().await {
                    tracing::info!("[REMOTE_SERVER] WS Client connecting from {}", addr);
                    let h_server = ws_server.clone();
                    let conn_tls = ws_tls.clone();
                    tokio::spawn(async move {
                        match conn_tls.accept(socket).await {
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

    fn generate_tls_config(sans: &[String]) -> anyhow::Result<ServerConfig> {
        // §12.1: Ensure a crypto provider is installed for rustls 0.23+
        let _ = rustls::crypto::ring::default_provider().install_default();

        let cert = generate_simple_self_signed(sans.to_vec())?;
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
                // Phase 2 narrowing: the old OpenSearch HTTP shim
                // dispatched `search:` for any unauthenticated GET.
                // Search is a parked subsystem, so the shim is gone:
                // HTTP requests to this port are refused outright.
                if command.starts_with("GET /") {
                    let body = "Not available over a remote connection (three-stream narrowing: terminal, chip state, prompt only)\n";
                    let response = format!(
                        "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    writer.write_all(response.as_bytes()).await?;
                    writer.flush().await?;
                    return Ok(());
                }

                let response = match remote_refusal(command) {
                    Some(refusal) => refusal,
                    None => self.ipc.handle_request(command),
                };
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
        self.ipc.register_broadcaster(mpsc_tx.clone());
        // No replay on connect: the face subscribes to the
        // session's display state and the snapshot it answers with
        // is the whole truth (Architecture §26.3).

        let push_ipc = self.ipc.clone();
        let push_tx = mpsc_tx.clone();
        
        let push_task = tokio::spawn(async move {
            let mut last_version = 0;
            let mut idle_ticks: u8 = 0;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let request = format!("get_state_delta:{}", last_version);
                let response = push_ipc.handle_request(&request);
                if response != "NO_CHANGE" && !response.starts_with("ERROR") {
                    idle_ticks = 0;
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response) {
                        if let Some(v) = parsed.get("version").and_then(|v| v.as_u64()) {
                            last_version = v;
                        }
                    }
                    if push_tx.send(format!("state_delta:{}", response)).is_err() {
                        break;
                    }
                } else {
                    // Keep face liveness checks satisfied without forcing full
                    // state serialization when nothing changed.
                    idle_ticks = idle_ticks.saturating_add(1);
                    if idle_ticks >= 2 {
                        idle_ticks = 0;
                        if push_tx.send("heartbeat".to_string()).is_err() {
                            break;
                        }
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
                                // WebRTC faces are parked, and the
                                // signalling side-channel sits outside
                                // the three streams.
                                "ERROR: webrtc_signalling is not available over a remote connection (three-stream narrowing: terminal, chip state, prompt only)".to_string()
                            } else if let Some(refusal) = remote_refusal(command) {
                                refusal
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RemoteConfig;

    fn cfg(remote_access: bool, bind: &str) -> RemoteConfig {
        RemoteConfig {
            remote_access,
            bind_address: bind.to_string(),
            ..RemoteConfig::default()
        }
    }

    #[test]
    fn remote_off_forces_loopback_even_with_a_bind_address() {
        // Fail closed: a configured tailnet bind must not expose the
        // Brain while remote access is off.
        assert_eq!(
            RemoteServer::effective_bind_host(&cfg(false, "100.64.0.7")),
            "127.0.0.1"
        );
    }

    #[test]
    fn remote_on_binds_the_configured_address() {
        assert_eq!(
            RemoteServer::effective_bind_host(&cfg(true, "100.64.0.7")),
            "100.64.0.7"
        );
    }

    #[test]
    fn remote_on_wildcard_stays_dual_stack() {
        assert_eq!(
            RemoteServer::effective_bind_host(&cfg(true, "0.0.0.0")),
            "[::]"
        );
    }

    #[test]
    fn bind_addr_brackets_ipv6_only() {
        assert_eq!(RemoteServer::bind_addr("127.0.0.1", 7001), "127.0.0.1:7001");
        assert_eq!(RemoteServer::bind_addr("[::]", 7001), "[::]:7001");
        assert_eq!(RemoteServer::bind_addr("::1", 7001), "[::1]:7001");
    }

    #[test]
    fn cert_covers_the_tailnet_bind_only_when_remote_is_on() {
        let on = RemoteServer::tls_subject_alt_names(&cfg(true, "100.64.0.7"));
        assert!(on.iter().any(|s| s == "100.64.0.7"));
        let off = RemoteServer::tls_subject_alt_names(&cfg(false, "100.64.0.7"));
        assert!(!off.iter().any(|s| s == "100.64.0.7"));
        assert!(off.iter().any(|s| s == "localhost"));
    }

    #[test]
    fn three_stream_commands_pass_the_remote_gate() {
        for cmd in [
            "display_subscribe:",
            "terminal_key:{\"key\":\"a\"}",
            "terminal_text:{\"text\":\"ls\"}",
            "terminal_resize:80;24",
            "terminal_signal:SIGINT",
            "display_scroll:;5",
            "prompt_submit:ls",
            "force_prompt_submit:ls",
            "placement_sync_apps:[]",
            "placement_choose:x;host",
            "placement_relaunch:x",
            "get_state:",
            "get_state_delta:0",
            "tos_ports:",
            "face_register:ambient",
            // cmd:-wrapped forms the face actually sends
            "cmd:1:get_state:",
            "cmd:2:terminal_key:{\"key\":\"a\"}",
        ] {
            assert!(remote_refusal(cmd).is_none(), "should pass: {}", cmd);
        }
    }

    #[test]
    fn parked_commands_are_refused_loudly_over_remote() {
        for (cmd, prefix) in [
            ("marketplace_home:", "marketplace_home"),
            ("ai_submit:hello", "ai_submit"),
            ("set_mode:CommandHub", "set_mode"),
            ("sector_create:work", "sector_create"),
            ("session_save:x", "session_save"),
            ("process_signal:1;9", "process_signal"),
            ("system_reset:", "system_reset"),
            ("app_launch:terminal", "app_launch"),
            // The retired raw-stream commands do not survive
            // alongside the display-state protocol.
            ("terminal_input_hex:6869", "terminal_input_hex"),
            ("pty_output:6869", "pty_output"),
        ] {
            let refusal = remote_refusal(cmd).expect("should be refused");
            assert!(
                refusal.starts_with(&format!("ERROR: {} is not available", prefix)),
                "refusal for {} should name it, got: {}",
                cmd,
                refusal
            );
        }
    }

    #[test]
    fn refusal_keeps_the_cmd_wrapper_id() {
        let refusal = remote_refusal("cmd:42:ai_submit:hello").unwrap();
        assert!(refusal.starts_with("res:42:ERROR: ai_submit is not available"));
    }
}
