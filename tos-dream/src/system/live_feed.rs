//! Live Feed System - WebSocket Streaming for TOS Web Portal
//! 
//! Provides real-time streaming of TOS state and test execution
//! to the web portal for live observation and debugging.

use crate::TosState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::Duration;

/// Live feed server for streaming TOS state
#[derive(Debug, Clone)]
pub struct LiveFeedServer {
    /// Broadcast channel for state updates
    state_tx: broadcast::Sender<LiveFeedMessage>,
    /// Connected clients
    clients: Arc<RwLock<HashMap<SocketAddr, ClientInfo>>>,
    /// Server configuration
    config: LiveFeedConfig,
    /// Test recording state
    recording: Arc<RwLock<Option<TestRecording>>>,
    /// Command sender (for external use)
    command_tx: mpsc::Sender<FeedCommand>,
}

/// Live feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveFeedConfig {
    /// WebSocket bind address
    pub bind_address: String,
    /// Port for WebSocket server
    pub port: u16,
    /// Update frequency in Hz
    pub update_frequency: f32,
    /// Enable test recording
    pub enable_recording: bool,
    /// Recording directory
    pub recording_path: String,
    /// Authentication token (optional)
    pub auth_token: Option<String>,
    /// Maximum connected clients
    pub max_clients: usize,
    /// Stream compression enabled
    pub compression: bool,
}

impl Default for LiveFeedConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8765,
            update_frequency: 30.0, // 30 FPS
            enable_recording: true,
            recording_path: "/tmp/tos-recordings".to_string(),
            auth_token: None,
            max_clients: 10,
            compression: true,
        }
    }
}

/// Client connection information
#[derive(Debug, Clone)]
struct ClientInfo {
    addr: SocketAddr,
    connected_at: std::time::Instant,
    last_ping: std::time::Instant,
    subscribed_tests: Vec<String>,
    view_mode: ViewMode,
}

/// Client view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Overview,    // Global overview
    Sector(usize), // Specific sector
    Test,        // Test execution view
}

/// Live feed message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum LiveFeedMessage {
    /// Full state snapshot
    StateSnapshot(TosStateSnapshot),
    /// Delta state update
    StateDelta(StateDelta),
    /// Test event
    TestEvent(TestEvent),
    /// Performance metrics
    PerformanceMetrics(PerformanceData),
    /// Accessibility announcement
    AccessibilityEvent(AccessibilityEventData),
    /// User interaction
    UserInteraction(InteractionData),
    /// System notification
    Notification(NotificationData),
    /// Ping/keepalive
    Ping,
    /// Pong response
    Pong,
    /// Authentication required
    AuthRequired,
    /// Authentication success
    AuthSuccess,
    /// Error message
    Error(String),
}

/// TOS state snapshot (serializable subset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TosStateSnapshot {
    pub timestamp: u64,
    pub current_level: String,
    pub sectors: Vec<SectorSnapshot>,
    pub active_viewport: usize,
    pub fps: f32,
    pub performance_alert: bool,
    pub viewports: Vec<ViewportSnapshot>,
}

/// Sector snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorSnapshot {
    pub id: String,
    pub name: String,
    pub color: String,
    pub host: String,
    pub connection_type: String,
    pub portal_active: bool,
    pub portal_url: Option<String>,
    pub participant_count: usize,
    pub hubs: Vec<HubSnapshot>,
    pub active_hub_index: usize,
}

/// Hub snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubSnapshot {
    pub id: String,
    pub mode: String,
    pub prompt: String,
    pub application_count: usize,
    pub active_app_index: Option<usize>,
    pub confirmation_required: Option<String>,
}

/// Viewport snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSnapshot {
    pub id: String,
    pub sector_index: usize,
    pub hub_index: usize,
    pub current_level: String,
    pub bezel_expanded: bool,
}

/// State delta (changes only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    pub timestamp: u64,
    pub changes: Vec<StateChange>,
}

/// Individual state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub path: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
}

/// Test event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEvent {
    pub test_id: String,
    pub test_name: String,
    pub event_type: TestEventType,
    pub timestamp: u64,
    pub details: serde_json::Value,
    pub screenshot: Option<String>, // Base64 encoded
}

/// Test event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestEventType {
    Started,
    Passed,
    Failed,
    Skipped,
    InProgress,
    Assertion,
    Error,
    Completed,
}

/// Performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub timestamp: u64,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub cpu_usage: f32,
    pub memory_mb: f32,
    pub gpu_usage: Option<f32>,
    pub render_time_ms: f32,
}

/// Accessibility event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityEventData {
    pub timestamp: u64,
    pub event_type: String,
    pub description: String,
    pub aria_live: String,
}

/// User interaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionData {
    pub timestamp: u64,
    pub input_type: String,
    pub semantic_event: String,
    pub target_element: Option<String>,
    pub coordinates: Option<(f32, f32)>,
}

/// Notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
    pub source: String,
}

/// Feed control commands
#[derive(Debug, Clone)]
pub enum FeedCommand {
    StartRecording(String), // test name
    StopRecording,
    PauseStreaming,
    ResumeStreaming,
    SetViewMode(SocketAddr, ViewMode),
    Authenticate(SocketAddr, String),
    DisconnectClient(SocketAddr),
    Shutdown,
}

/// Test recording session
#[derive(Debug)]
struct TestRecording {
    test_id: String,
    test_name: String,
    started_at: std::time::Instant,
    events: Vec<TestEvent>,
    state_snapshots: Vec<TosStateSnapshot>,
    performance_log: Vec<PerformanceData>,
}

impl LiveFeedServer {
    /// Create a new live feed server
    pub fn new(config: LiveFeedConfig) -> Self {
        let (state_tx, _) = broadcast::channel(1000);
        let (command_tx, _command_rx) = mpsc::channel(32);
        
        Self {
            state_tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
            recording: Arc::new(RwLock::new(None)),
            command_tx,
        }
    }
    
    /// Get command sender for external control
    pub fn command_sender(&self) -> mpsc::Sender<FeedCommand> {
        self.command_tx.clone()
    }
    
    /// Start the live feed server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Live feed server started on {}", addr);
        
        // Accept connections
        let clients = self.clients.clone();
        let state_tx = self.state_tx.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let clients = clients.clone();
                        let state_tx = state_tx.clone();
                        let config = config.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, addr, clients, state_tx, config).await {
                                tracing::warn!("Client {} error: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Process a single control command
    pub async fn process_command(&self, cmd: FeedCommand) {
            match cmd {
                FeedCommand::StartRecording(test_name) => {
                    let mut recording = self.recording.write().await;
                    let test_id = format!("test_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs());
                    
                    *recording = Some(TestRecording {
                        test_id: test_id.clone(),
                        test_name,
                        started_at: std::time::Instant::now(),
                        events: Vec::new(),
                        state_snapshots: Vec::new(),
                        performance_log: Vec::new(),
                    });
                    
                    tracing::info!("Started recording test: {}", test_id);
                    
                    // Notify clients
                    let _ = self.state_tx.send(LiveFeedMessage::Notification(NotificationData {
                        timestamp: now_ms(),
                        level: "info".to_string(),
                        message: format!("Test recording started: {}", test_id),
                        source: "live_feed".to_string(),
                    }));
                }
                
                FeedCommand::StopRecording => {
                    let mut recording = self.recording.write().await;
                    if let Some(rec) = recording.take() {
                        self.save_recording(&rec).await;
                        tracing::info!("Stopped recording test: {}", rec.test_id);
                    }
                }
                
                FeedCommand::PauseStreaming => {
                    tracing::info!("Streaming paused");
                }
                
                FeedCommand::ResumeStreaming => {
                    tracing::info!("Streaming resumed");
                }
                
                FeedCommand::SetViewMode(addr, mode) => {
                    let mut clients = self.clients.write().await;
                    if let Some(client) = clients.get_mut(&addr) {
                        client.view_mode = mode;
                    }
                }
                
                FeedCommand::Authenticate(_addr, token) => {
                    let authenticated = self.config.auth_token.as_ref()
                        .map(|expected| expected == &token)
                        .unwrap_or(true);
                    
                    let msg = if authenticated {
                        LiveFeedMessage::AuthSuccess
                    } else {
                        LiveFeedMessage::Error("Authentication failed".to_string())
                    };
                    
                    let _ = self.state_tx.send(msg);
                }
                
                FeedCommand::DisconnectClient(addr) => {
                    let mut clients = self.clients.write().await;
                    clients.remove(&addr);
                    tracing::info!("Client {} disconnected", addr);
                }
                
                FeedCommand::Shutdown => {
                    tracing::info!("Live feed server shutting down");
                }
            }
    }
    
    /// Broadcast state update to all clients
    pub async fn broadcast_state(&self, state: &TosState) {
        let snapshot = create_state_snapshot(state);
        let msg = LiveFeedMessage::StateSnapshot(snapshot);
        
        // Record if recording is active
        if let Some(ref mut recording) = *self.recording.write().await {
            recording.state_snapshots.push(msg.clone().into_data().unwrap());
        }
        
        let _ = self.state_tx.send(msg);
    }
    
    /// Broadcast test event
    pub async fn broadcast_test_event(&self, event: TestEvent) {
        let msg = LiveFeedMessage::TestEvent(event.clone());
        
        // Record if recording is active
        if let Some(ref mut recording) = *self.recording.write().await {
            recording.events.push(event);
        }
        
        let _ = self.state_tx.send(msg);
    }
    
    /// Broadcast performance metrics
    pub async fn broadcast_performance(&self, data: PerformanceData) {
        let msg = LiveFeedMessage::PerformanceMetrics(data.clone());
        
        // Record if recording is active
        if let Some(ref mut recording) = *self.recording.write().await {
            recording.performance_log.push(data);
        }
        
        let _ = self.state_tx.send(msg);
    }
    
    /// Broadcast accessibility event
    pub async fn broadcast_accessibility(&self, event_type: &str, description: &str) {
        let msg = LiveFeedMessage::AccessibilityEvent(AccessibilityEventData {
            timestamp: now_ms(),
            event_type: event_type.to_string(),
            description: description.to_string(),
            aria_live: "polite".to_string(),
        });
        
        let _ = self.state_tx.send(msg);
    }
    
    /// Broadcast user interaction
    pub async fn broadcast_interaction(&self, input_type: &str, semantic_event: &str) {
        let msg = LiveFeedMessage::UserInteraction(InteractionData {
            timestamp: now_ms(),
            input_type: input_type.to_string(),
            semantic_event: semantic_event.to_string(),
            target_element: None,
            coordinates: None,
        });
        
        let _ = self.state_tx.send(msg);
    }
    
    /// Save recording to disk
    async fn save_recording(&self, recording: &TestRecording) {
        use std::fs;
        use std::path::Path;
        
        let path = Path::new(&self.config.recording_path);
        if !path.exists() {
            let _ = fs::create_dir_all(path);
        }
        
        let filename = format!("{}/{}.json", self.config.recording_path, recording.test_id);
        let data = serde_json::json!({
            "test_id": recording.test_id,
            "test_name": &recording.test_name,
            "duration_ms": recording.started_at.elapsed().as_millis(),
            "events": recording.events,
            "state_snapshots": recording.state_snapshots,
            "performance_log": recording.performance_log,
        });
        
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(&filename, json);
            tracing::info!("Recording saved to: {}", filename);
        }
    }
    
    /// Get current recording status
    pub async fn is_recording(&self) -> bool {
        self.recording.read().await.is_some()
    }
    
    /// Get recording info
    pub async fn recording_info(&self) -> Option<(String, String, std::time::Duration)> {
        self.recording.read().await.as_ref().map(|r| {
            (
                r.test_id.clone(),
                r.test_name.clone(),
                r.started_at.elapsed(),
            )
        })
    }
}

/// Handle a WebSocket client connection
async fn handle_client(
    _stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<SocketAddr, ClientInfo>>>,
    state_tx: broadcast::Sender<LiveFeedMessage>,
    config: LiveFeedConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Client connected: {}", addr);
    
    // Check max clients
    {
        let clients_guard = clients.read().await;
        if clients_guard.len() >= config.max_clients {
            tracing::warn!("Max clients reached, rejecting {}", addr);
            return Ok(());
        }
    }
    
    // Register client
    {
        let mut clients_guard = clients.write().await;
        clients_guard.insert(addr, ClientInfo {
            addr,
            connected_at: std::time::Instant::now(),
            last_ping: std::time::Instant::now(),
            subscribed_tests: Vec::new(),
            view_mode: ViewMode::Overview,
        });
    }
    
    // Subscribe to state updates
    let mut rx = state_tx.subscribe();
    
    // Send initial auth required if token is set
    if config.auth_token.is_some() {
        let _ = state_tx.send(LiveFeedMessage::AuthRequired);
    }
    
    // Handle messages
    loop {
        match tokio::time::timeout(Duration::from_secs(30), rx.recv()).await {
            Ok(Ok(msg)) => {
                // Send to client (in a real implementation, this would be WebSocket)
                tracing::debug!("Sending to {}: {:?}", addr, msg);
                
                // Check for ping
                if matches!(msg, LiveFeedMessage::Ping) {
                    // Update last ping
                    let mut clients_guard = clients.write().await;
                    if let Some(client) = clients_guard.get_mut(&addr) {
                        client.last_ping = std::time::Instant::now();
                    }
                }
            }
            Ok(Err(_)) => {
                // Broadcast channel closed
                break;
            }
            Err(_) => {
                // Timeout - check if client is still alive
                let clients_guard = clients.read().await;
                if let Some(client) = clients_guard.get(&addr) {
                    if client.last_ping.elapsed() > Duration::from_secs(60) {
                        tracing::warn!("Client {} timed out", addr);
                        break;
                    }
                }
            }
        }
    }
    
    // Unregister client
    {
        let mut clients_guard = clients.write().await;
        clients_guard.remove(&addr);
    }
    
    tracing::info!("Client disconnected: {}", addr);
    Ok(())
}

/// Create a state snapshot from TOS state
fn create_state_snapshot(state: &TosState) -> TosStateSnapshot {
    TosStateSnapshot {
        timestamp: now_ms(),
        current_level: format!("{:?}", state.current_level),
        sectors: state.sectors.iter().map(|s| SectorSnapshot {
            id: s.id.to_string(),
            name: s.name.clone(),
            color: s.color.clone(),
            host: s.host.clone(),
            connection_type: format!("{:?}", s.connection_type),
            portal_active: s.portal_active,
            portal_url: s.portal_url.clone(),
            participant_count: s.participants.len(),
            hubs: s.hubs.iter().map(|h| HubSnapshot {
                id: h.id.to_string(),
                mode: format!("{:?}", h.mode),
                prompt: h.prompt.clone(),
                application_count: h.applications.len(),
                active_app_index: h.active_app_index,
                confirmation_required: h.confirmation_required.clone(),
            }).collect(),
            active_hub_index: s.active_hub_index,
        }).collect(),
        active_viewport: state.active_viewport_index,
        fps: state.fps,
        performance_alert: state.performance_alert,
        viewports: state.viewports.iter().map(|v| ViewportSnapshot {
            id: v.id.to_string(),
            sector_index: v.sector_index,
            hub_index: v.hub_index,
            current_level: format!("{:?}", v.current_level),
            bezel_expanded: v.bezel_expanded,
        }).collect(),
    }
}

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Extension trait for LiveFeedMessage
trait IntoData {
    fn into_data(self) -> Option<TosStateSnapshot>;
}

impl IntoData for LiveFeedMessage {
    fn into_data(self) -> Option<TosStateSnapshot> {
        match self {
            LiveFeedMessage::StateSnapshot(data) => Some(data),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_feed_config_default() {
        let config = LiveFeedConfig::default();
        assert_eq!(config.port, 8765);
        assert!(config.enable_recording);
    }

    #[test]
    fn test_state_snapshot_creation() {
        let state = TosState::new();
        let snapshot = create_state_snapshot(&state);
        
        assert_eq!(snapshot.sectors.len(), 3);
        assert!(snapshot.fps > 0.0);
    }

    #[test]
    fn test_test_event_serialization() {
        let event = TestEvent {
            test_id: "test_1".to_string(),
            test_name: "Test One".to_string(),
            event_type: TestEventType::Started,
            timestamp: 1234567890,
            details: serde_json::json!({"step": 1}),
            screenshot: None,
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test_1"));
        assert!(json.contains("Started"));
    }

    #[tokio::test]
    async fn test_live_feed_server_creation() {
        let config = LiveFeedConfig::default();
        let server = LiveFeedServer::new(config);
        
        assert!(!server.is_recording().await);
        assert!(server.command_sender().capacity() > 0);
    }
}
