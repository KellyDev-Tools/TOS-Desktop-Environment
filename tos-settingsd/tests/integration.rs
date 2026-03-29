use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tos_common::MockBrain;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_settingsd_integration() -> anyhow::Result<()> {
    // 1. Setup Mock Brain for registration
    let mock_brain = MockBrain::new().await?;
    
    // 2. Start the daemon binary
    let bin_path = env!("CARGO_BIN_EXE_tos-settingsd");
    let mut child = Command::new(bin_path)
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    
    // 3. Handle registration and capture port
    let (_name, port) = mock_brain.handle_one_registration().await?;
    assert_eq!(_name, "tos-settingsd");
    
    // 4. Connect and test IPC
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    
    // Test get_all
    stream.write_all(b"get_all:\n").await?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    
    let state: serde_json::Value = serde_json::from_str(line.trim())?;
    assert!(state.get("global").is_some());
    
    // 5. Cleanup
    child.kill().await?;
    Ok(())
}
