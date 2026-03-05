//! Component Test: External TCP Client exercising the Brain over the wire
//!
//! Boots a full Brain + TCP server on an ephemeral port, then acts as an
//! external client — the same way a real Face, `tos` CLI tool, or daemon
//! would interact with the Brain in production.
//!
//! Round-trip validated:
//!   Client → TCP → RemoteServer → IpcHandler → State → Response → TCP → Client

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

/// Send a single IPC command over TCP and return the response line.
async fn send_command(stream: &mut TcpStream, command: &str) -> String {
    let (reader, writer) = stream.split();
    let mut writer = writer;
    let mut reader = BufReader::new(reader);

    writer
        .write_all(format!("{}\n", command).as_bytes())
        .await
        .expect("Failed to write to Brain TCP");
    writer.flush().await.expect("Failed to flush");

    let mut response = String::new();
    reader
        .read_line(&mut response)
        .await
        .expect("Failed to read response from Brain");
    response.trim().to_string()
}

/// Short-lived connection — send one command, get one response, disconnect.
/// Simulates a CLI tool like `tos ports`.
async fn cli_command(port: u16, command: &str) -> String {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect to Brain");
    send_command(&mut stream, command).await
}

/// Strip the IPC timing suffix, e.g. `{...} (42.1µs)` → `{...}`
/// The IPC handler appends ` (duration)` to every response.
fn strip_timing(resp: &str) -> &str {
    // Find the last ` (` that's followed by a duration like `42µs)` or `1.2ms)`
    if let Some(pos) = resp.rfind(") (") {
        // " (Xµs)" is at the end — back up to include the closing paren of the JSON
        &resp[..pos + 1]
    } else if let Some(pos) = resp.rfind(" (") {
        &resp[..pos]
    } else {
        resp
    }
}

/// Boot a Brain + TCP server, return (port, brain) so tests can send commands.
async fn boot_brain_server() -> (u16, tos_alpha2::brain::Brain) {
    let brain = tos_alpha2::brain::Brain::new()
        .expect("Brain must initialize");
    let ipc = brain.ipc.clone();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral port");
    let port = listener.local_addr().unwrap().port();

    let ipc_clone = ipc.clone();
    tokio::spawn(async move {
        loop {
            if let Ok((socket, _)) = listener.accept().await {
                let h_ipc = ipc_clone.clone();
                tokio::spawn(async move {
                    let (reader, mut writer) = socket.into_split();
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        let n = reader.read_line(&mut line).await.unwrap_or(0);
                        if n == 0 { break; }
                        let command = line.trim();
                        if !command.is_empty() {
                            let response = h_ipc.handle_request(command);
                            let _ = writer.write_all(format!("{}\n", response).as_bytes()).await;
                            let _ = writer.flush().await;
                        }
                    }
                });
            }
        }
    });

    sleep(Duration::from_millis(50)).await;
    (port, brain)
}

/// Parse a `get_state:` response into a JSON Value, stripping the timing suffix.
fn parse_state_response(resp: &str) -> serde_json::Value {
    let json_str = strip_timing(resp);
    serde_json::from_str(json_str)
        .unwrap_or_else(|e| panic!("get_state response is not valid JSON: {}\nRaw (first 200 chars): {}", e, &json_str[..json_str.len().min(200)]))
}

// ---------------------------------------------------------------------------
// Full User Workflow: Navigate hierarchy, create sector
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tcp_client_navigates_hierarchy_and_creates_sector() {
    let (port, _brain) = boot_brain_server().await;

    // Start at GlobalOverview
    let resp = cli_command(port, "get_state:").await;
    assert!(resp.contains("GlobalOverview"), "Should start at GlobalOverview");

    // Zoom to CommandHub
    let resp = cli_command(port, "zoom_in:").await;
    assert!(resp.contains("ZOOMED_IN"));

    // Verify state over the wire
    let resp = cli_command(port, "get_state:").await;
    assert!(resp.contains("CommandHub"));

    // Create a new sector
    let resp = cli_command(port, "sector_create:DevOps").await;
    assert!(resp.contains("SECTOR_CREATED"));

    // Confirm the sector exists in state
    let resp = cli_command(port, "get_state:").await;
    assert!(resp.contains("DevOps"));
}

// ---------------------------------------------------------------------------
// Service discovery: register daemon, query ports, deregister
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tcp_client_registers_daemon_and_queries_port_table() {
    let (port, _brain) = boot_brain_server().await;

    // Register a mock daemon
    let resp = cli_command(port, "service_register:tos-loggerd;8888").await;
    assert!(resp.contains("SERVICE_REGISTERED"));

    // Query the port table — should include the anchor and our daemon
    let resp = cli_command(port, "tos_ports:").await;
    assert!(resp.contains("tos-brain (anchor)"));
    assert!(resp.contains("tos-loggerd"));
    assert!(resp.contains("8888"));

    // Deregister
    let resp = cli_command(port, "service_deregister:tos-loggerd").await;
    assert!(resp.contains("SERVICE_DEREGISTERED"));

    // Confirm removal
    let resp = cli_command(port, "tos_ports:").await;
    assert!(!resp.contains("tos-loggerd"));
}

// ---------------------------------------------------------------------------
// Persistent session: AI staging workflow over a single connection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tcp_client_stages_ai_command_and_accepts_over_persistent_connection() {
    let (port, _brain) = boot_brain_server().await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    // Navigate to CommandHub first
    let resp = send_command(&mut stream, "zoom_in:").await;
    assert!(resp.contains("ZOOMED_IN"));

    // Switch to AI mode
    let resp = send_command(&mut stream, "set_mode:ai").await;
    assert!(resp.contains("MODE_SET"));

    // Stage an AI suggestion
    let resp = send_command(
        &mut stream,
        r#"ai_stage_command:{"command":"kubectl get pods","explanation":"List Kubernetes pods"}"#,
    ).await;
    assert!(resp.contains("AI_COMMAND_STAGED"));

    // Accept the suggestion
    let resp = send_command(&mut stream, "ai_suggestion_accept:").await;
    assert!(resp.contains("AI_SUGGESTION_ACCEPTED"));

    // Verify via get_state
    let resp = send_command(&mut stream, "get_state:").await;
    let state = parse_state_response(&resp);
    let prompt = state["sectors"][0]["hubs"][0]["prompt"].as_str().unwrap_or("");
    assert_eq!(prompt, "kubectl get pods");
}

// ---------------------------------------------------------------------------
// Cross-scope settings: global and sector-scoped updates
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tcp_client_updates_settings_across_scopes() {
    let (port, _brain) = boot_brain_server().await;

    // Set a global setting
    let resp = cli_command(port, "set_setting:tos.ai.disabled;true").await;
    assert!(resp.contains("SETTING_UPDATE"));

    // Set a sector-scoped setting
    let resp = cli_command(port, "set_sector_setting:prod;tos.trust.privilege_escalation;trust").await;
    assert!(resp.contains("SECTOR_SETTING_UPDATE"));

    // Verify both in state
    let resp = cli_command(port, "get_state:").await;
    let state = parse_state_response(&resp);
    assert_eq!(state["settings"]["global"]["tos.ai.disabled"].as_str(), Some("true"));
    assert_eq!(
        state["settings"]["sectors"]["prod"]["tos.trust.privilege_escalation"].as_str(),
        Some("trust")
    );
}

// ---------------------------------------------------------------------------
// Error resilience: invalid commands over the wire
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tcp_client_receives_errors_for_invalid_commands() {
    let (port, _brain) = boot_brain_server().await;

    // Malformed (no colon separator)
    let resp = cli_command(port, "missing_colon").await;
    assert!(resp.contains("ERROR"));

    // Unknown command prefix
    let resp = cli_command(port, "does_not_exist:payload").await;
    assert!(resp.contains("ERROR"));

    // Invalid UUID for sector close
    let resp = cli_command(port, "sector_close:not-a-valid-uuid").await;
    assert!(resp.contains("ERROR"));

    // Invalid mode name
    let resp = cli_command(port, "set_mode:nonexistent_mode").await;
    assert!(resp.contains("ERROR"));
}
