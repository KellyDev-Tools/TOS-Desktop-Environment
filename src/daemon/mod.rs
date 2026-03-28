use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tos_protocol::ipc::{ServiceRegister, ServiceRegisterResponse};
use ed25519_dalek::{SigningKey, Signer};
use rand_core::OsRng;

/// Register a satellite daemon with the Brain on brain.sock.
/// 
/// This performs the ed25519 signature handshake required by the
/// Dynamic Port Registration Gate (§4.1).
pub async fn register_with_brain(name: &str, port: u16) -> anyhow::Result<()> {
    let uds_path = "/tmp/brain.sock";
    
    // Attempt connection with retry (daemon might start before Brain is ready)
    let mut stream = None;
    for _ in 0..5 {
        if let Ok(s) = tokio::net::UnixStream::connect(uds_path).await {
            stream = Some(s);
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    let mut stream = stream.ok_or_else(|| anyhow::anyhow!("Could not connect to brain.sock at {}", uds_path))?;
    
    // Generate an ephemeral keypair for this daemon instance.
    // In production, these should be securely stored and pre-authorized (§4.1.3).
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let public_key = signing_key.verifying_key();
    
    let signature = signing_key.sign(format!("{}:{}", name, port).as_bytes());
    
    let reg_req = ServiceRegister {
        name: name.to_string(),
        port,
        signature: hex::encode(signature.to_bytes()),
        public_key: hex::encode(public_key.to_bytes()),
    };
    
    let request_line = format!("service_register:{}\n", serde_json::to_string(&reg_req)?);
    stream.write_all(request_line.as_bytes()).await?;
    stream.flush().await?;
    
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    
    if line.is_empty() {
        return Err(anyhow::anyhow!("Brain closed connection without response"));
    }
    
    let response: ServiceRegisterResponse = serde_json::from_str(line.trim())?;
    
    if response.status == "OK" {
        tracing::info!("DAEMON: Successfully registered {} on port {} with Brain.", name, port);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Registration denied: {}", response.message))
    }
}

/// Send a log message to the Brain's unified LogManager.
pub async fn log_to_brain(text: &str, priority: u8) -> anyhow::Result<()> {
    // Connect to discovery gate
    let mut stream = tokio::net::UnixStream::connect("/tmp/brain.sock").await?;
    
    // Command format: system_log_append:text;priority
    let cmd = format!("system_log_append:{};{}\n", text, priority);
    stream.write_all(cmd.as_bytes()).await?;
    
    // We don't necessarily need to wait for response for a log
    Ok(())
}
