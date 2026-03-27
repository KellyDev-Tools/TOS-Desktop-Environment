use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use crate::brain::ipc_handler::IpcHandler;
use std::sync::Arc;

pub struct RemoteServer {
    ipc: Arc<IpcHandler>,
}

impl RemoteServer {
    pub fn new(ipc: Arc<IpcHandler>) -> Self {
        Self { ipc }
    }

    /// §12.1: Start the Remote Server daemons
    pub async fn run(&self, port: u16) -> anyhow::Result<()> {
        let tcp_addr = format!("0.0.0.0:{}", port);
        let ws_addr = format!("0.0.0.0:{}", port + 1); // e.g. 7001 for WebSocket
        let uds_path = "/tmp/tos.brain.sock";

        // Bind with retry — previous Brain instance may have just been killed
        let tcp_listener = Self::bind_with_retry(&tcp_addr).await?;
        let ws_listener = Self::bind_with_retry(&ws_addr).await?;
        
        // Remove existing UDS if present (exists(), not is_file() — sockets aren't regular files)
        if std::path::Path::new(uds_path).exists() {
            let _ = std::fs::remove_file(uds_path);
        }
        let uds_listener = tokio::net::UnixListener::bind(uds_path)?;
        
        eprintln!("[REMOTE_SERVER] TCP Listening on {}", tcp_addr);
        eprintln!("[REMOTE_SERVER] WS  Listening on {}", ws_addr);
        eprintln!("[REMOTE_SERVER] UDS Listening on {}", uds_path);

        let ipc_clone = self.ipc.clone();
        
        // Spawn TCP daemon
        let tcp_ipc = ipc_clone.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = tcp_listener.accept().await {
                    let h_ipc = tcp_ipc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_tcp_client(socket, h_ipc).await {
                            eprintln!("[REMOTE_SERVER] TCP Client error: {}", e);
                        }
                    });
                }
            }
        });

        // Spawn WebSocket daemon
        let ws_ipc = ipc_clone.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = ws_listener.accept().await {
                    let h_ipc = ws_ipc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_ws_client(socket, h_ipc).await {
                            eprintln!("[REMOTE_SERVER] WS Client error: {}", e);
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
                            eprintln!("[REMOTE_SERVER] UDS Client error: {}", e);
                        }
                    });
                }
            }
        });

        Ok(())
    }

    /// Attempt to bind a TCP port, retrying up to 3 times on AddrInUse.
    async fn bind_with_retry(addr: &str) -> anyhow::Result<TcpListener> {
        let max_retries = 3;
        for attempt in 1..=max_retries {
            match TcpListener::bind(addr).await {
                Ok(listener) => return Ok(listener),
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && attempt < max_retries => {
                    eprintln!(
                        "[REMOTE_SERVER] Port {} in use, retrying in 1s ({}/{})",
                        addr, attempt, max_retries
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
        unreachable!()
    }

    async fn handle_uds_client(mut socket: tokio::net::UnixStream, ipc: Arc<IpcHandler>) -> anyhow::Result<()> {
        let (reader, mut writer) = socket.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 { break; } 

            let command = line.trim();
            if !command.is_empty() {
                let response = ipc.handle_request(command);
                writer.write_all(format!("{}\n", response).as_bytes()).await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn handle_tcp_client(mut socket: TcpStream, ipc: Arc<IpcHandler>) -> anyhow::Result<()> {
        let (reader, mut writer) = socket.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 { break; } 

            let command = line.trim();
            if !command.is_empty() {
                let response = ipc.handle_request(command);
                writer.write_all(format!("{}\n", response).as_bytes()).await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn handle_ws_client(socket: TcpStream, ipc: Arc<IpcHandler>) -> anyhow::Result<()> {
        let mut ws_stream = accept_async(socket).await?;
        
        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if msg.is_text() {
                let command = msg.to_text()?;
                let response = ipc.handle_request(command);
                ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(response)).await?;
            }
        }
        Ok(())
    }
}
