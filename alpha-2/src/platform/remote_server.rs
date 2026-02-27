use tokio::net::{TcpListener, TcpStream};
use crate::brain::ipc_handler::IpcHandler;
use std::sync::Arc;

pub struct RemoteServer {
    _ipc: Arc<IpcHandler>,
}

impl RemoteServer {
    pub fn new(ipc: Arc<IpcHandler>) -> Self {
        Self { _ipc: ipc }
    }

    /// §12.1: Start the Remote Server daemon
    pub async fn run(&self, port: u16) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("TOS Remote Server listening on {}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            self.handle_client(socket).await;
        }
    }

    async fn handle_client(&self, _socket: TcpStream) {
        // §12.1: Upgrade to WebSocket/TLS and handle login/streaming
        tracing::debug!("New remote client connection attempt");
    }
}

