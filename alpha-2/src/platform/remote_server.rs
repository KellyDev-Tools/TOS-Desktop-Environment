use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use crate::brain::ipc_handler::IpcHandler;
use std::sync::Arc;

pub struct RemoteServer {
    ipc: Arc<IpcHandler>,
}

impl RemoteServer {
    pub fn new(ipc: Arc<IpcHandler>) -> Self {
        Self { ipc }
    }

    /// §12.1: Start the Remote Server daemon
    pub async fn run(&self, port: u16) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        eprintln!("[REMOTE_SERVER] Listening on {}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            let ipc_clone = self.ipc.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(socket, ipc_clone).await {
                    eprintln!("[REMOTE_SERVER] Client error: {}", e);
                }
            });
        }
    }

    async fn handle_client(mut socket: TcpStream, ipc: Arc<IpcHandler>) -> anyhow::Result<()> {
        let (reader, mut writer) = socket.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 { break; } 

            let command = line.trim();
            if !command.is_empty() {
                eprintln!("[REMOTE_SERVER] Executing: {}", command);
                let response = ipc.handle_request(command);
                writer.write_all(format!("{}\n", response).as_bytes()).await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }
}
