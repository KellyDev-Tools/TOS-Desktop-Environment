// Development Monitor: HTTP + WebSocket server for browser-based UI debugging
// This allows you to watch UI tests execute in real-time via a browser

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::sync::broadcast;

pub type ClientId = usize;

/// Message to broadcast to all connected browser clients
#[derive(Debug, Clone)]
pub enum BroadcastMessage {
    UpdateViewport {
        html_content: String,
        zoom_level: u8,
        is_red_alert: bool,
    },
    UpdateDashboard(String),
    ZoomLevel(u8),
    TestEvent {
        test_name: String,
        event_type: String, // "started", "assertion", "completed"
        details: String,
    },
}

/// Development Monitor Server
pub struct DevMonitor {
    port: u16,
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
    clients: Arc<Mutex<HashMap<ClientId, String>>>,
}

impl DevMonitor {
    /// Create a new dev monitor on the specified port
    pub fn new(port: u16) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            port,
            broadcast_tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a sender handle for broadcasting messages
    pub fn get_broadcaster(&self) -> DevMonitorBroadcaster {
        DevMonitorBroadcaster {
            tx: self.broadcast_tx.clone(),
        }
    }

    /// Start the HTTP and WebSocket server (async)
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        use warp::Filter;
        use futures_util::{StreamExt, SinkExt};

        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        
        println!("[Dev Monitor] Starting on http://{}", addr);
        println!("[Dev Monitor] WebSocket on ws://{}/ws", addr);

        // Serve static files from ui/ directory at root
        let assets = warp::path("assets")
            .and(warp::fs::dir("./ui/assets"));

        // Serve index.html at root
        let index = warp::path::end()
            .and(warp::fs::file("./ui/index.html"));

        let clients = self.clients.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let next_client_id = Arc::new(Mutex::new(0usize));

        // WebSocket endpoint
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let clients = clients.clone();
                let broadcast_tx = broadcast_tx.clone();
                let next_id = Arc::clone(&next_client_id);

                ws.on_upgrade(move |websocket| async move {
                    let client_id = {
                        let mut id = next_id.lock().unwrap();
                        let current = *id;
                        *id += 1;
                        current
                    };

                    println!("[Dev Monitor] Client {} connected", client_id);
                    clients.lock().unwrap().insert(client_id, format!("Browser {}", client_id));

                    let (mut ws_tx, mut ws_rx) = websocket.split();
                    let mut rx = broadcast_tx.subscribe();

                    // Spawn task to forward broadcast messages to this client
                    let forward_task = tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            let json = match msg {
                                BroadcastMessage::UpdateViewport { html_content, zoom_level, is_red_alert } => {
                                    serde_json::json!({
                                        "type": "viewport",
                                        "html": html_content,
                                        "zoom": zoom_level,
                                        "redAlert": is_red_alert,
                                    })
                                }
                                BroadcastMessage::UpdateDashboard(html) => {
                                    serde_json::json!({
                                        "type": "dashboard",
                                        "html": html,
                                    })
                                }
                                BroadcastMessage::ZoomLevel(level) => {
                                    serde_json::json!({
                                        "type": "zoom",
                                        "level": level,
                                    })
                                }
                                BroadcastMessage::TestEvent { test_name, event_type, details } => {
                                    serde_json::json!({
                                        "type": "test_event",
                                        "test": test_name,
                                        "event": event_type,
                                        "details": details,
                                    })
                                }
                            };

                            let text = serde_json::to_string(&json).unwrap();
                            if ws_tx.send(warp::ws::Message::text(text)).await.is_err() {
                                break;
                            }
                        }
                    });

                    // Handle incoming messages from browser (optional - for input)
                    while let Some(result) = ws_rx.next().await {
                        if let Ok(msg) = result {
                            if let Ok(text) = msg.to_str() {
                                println!("[Dev Monitor] Received from client {}: {}", client_id, text);
                                // Could forward input commands here if needed
                            }
                        }
                    }

                    println!("[Dev Monitor] Client {} disconnected", client_id);
                    clients.lock().unwrap().remove(&client_id);
                    forward_task.abort();
                })
            });

        let routes = index.or(assets).or(ws_route);

        warp::serve(routes).run(addr).await;

        Ok(())
    }
}

/// Handle for broadcasting messages to the dev monitor
#[derive(Clone)]
pub struct DevMonitorBroadcaster {
    tx: broadcast::Sender<BroadcastMessage>,
}

impl DevMonitorBroadcaster {
    pub fn send(&self, msg: BroadcastMessage) {
        let _ = self.tx.send(msg); // Ignore error if no receivers
    }

    pub fn update_viewport(&self, html: String, zoom: u8, red_alert: bool) {
        self.send(BroadcastMessage::UpdateViewport {
            html_content: html,
            zoom_level: zoom,
            is_red_alert: red_alert,
        });
    }

    pub fn update_dashboard(&self, html: String) {
        self.send(BroadcastMessage::UpdateDashboard(html));
    }

    pub fn zoom_level(&self, level: u8) {
        self.send(BroadcastMessage::ZoomLevel(level));
    }

    pub fn test_event(&self, test_name: impl Into<String>, event_type: impl Into<String>, details: impl Into<String>) {
        self.send(BroadcastMessage::TestEvent {
            test_name: test_name.into(),
            event_type: event_type.into(),
            details: details.into(),
        });
    }
}

/// Global dev monitor instance (optional)
static DEV_MONITOR: once_cell::sync::OnceCell<DevMonitorBroadcaster> = once_cell::sync::OnceCell::new();

/// Initialize the global dev monitor broadcaster
pub fn init_global_monitor(broadcaster: DevMonitorBroadcaster) {
    let _ = DEV_MONITOR.set(broadcaster);
}

/// Get the global dev monitor broadcaster (if initialized)
pub fn get_monitor() -> Option<&'static DevMonitorBroadcaster> {
    DEV_MONITOR.get()
}
