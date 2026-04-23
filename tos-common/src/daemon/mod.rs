use ed25519_dalek::{Signer, SigningKey};
use rand_core::OsRng;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::ipc::{ServiceRegister, ServiceRegisterResponse};

/// Register a satellite daemon with the Brain on brain.sock.
///
/// This performs the ed25519 signature handshake required by the
/// Dynamic Port Registration Gate (§4.1).
pub async fn register_with_brain(name: &str, port: u16) -> anyhow::Result<()> {
    let uds_path = "/tmp/brain.sock";

    // Attempt connection with exponential backoff (daemon might start before Brain is ready).
    // Starts at 100ms, doubles each attempt, caps at 10s per interval, max 10 retries.
    let mut stream = None;
    let mut backoff_ms: u64 = 100;
    let max_retries: u32 = 10;
    let max_backoff_ms: u64 = 10_000;
    for attempt in 1..=max_retries {
        match tokio::net::UnixStream::connect(uds_path).await {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => {
                tracing::warn!(
                    "DAEMON: brain.sock connection attempt {}/{} failed ({}), retrying in {}ms",
                    attempt,
                    max_retries,
                    e,
                    backoff_ms,
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(max_backoff_ms);
            }
        }
    }

    let mut stream =
        stream.ok_or_else(|| anyhow::anyhow!("Could not connect to brain.sock at {}", uds_path))?;

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
        tracing::info!(
            "DAEMON: Successfully registered {} on port {} with Brain.",
            name,
            port
        );
        Ok(())
    } else {
        Err(anyhow::anyhow!("Registration denied: {}", response.message))
    }
}

/// A simplified Mock Brain for testing satellite daemons (§4.1).
#[cfg(feature = "test-utils")]
pub struct MockBrain {
    pub listener: tokio::net::UnixListener,
}

#[cfg(feature = "test-utils")]
impl MockBrain {
    pub async fn new() -> anyhow::Result<Self> {
        let path = "/tmp/brain.sock";
        let _ = std::fs::remove_file(path);
        let listener = tokio::net::UnixListener::bind(path)?;
        Ok(Self { listener })
    }

    pub async fn handle_one_registration(&self) -> anyhow::Result<(String, u16)> {
        let (mut stream, _) = self.listener.accept().await?;
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        
        if let Some(payload) = line.strip_prefix("service_register:") {
            let reg: ServiceRegister = serde_json::from_str(payload.trim())?;
            let resp = ServiceRegisterResponse {
                status: "OK".to_string(),
                message: "Mock authorization".to_string(),
            };
            stream.write_all(format!("{}\n", serde_json::to_string(&resp)?).as_bytes()).await?;
            Ok((reg.name, reg.port))
        } else {
            Err(anyhow::anyhow!("Malformed registration: {}", line))
        }
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
