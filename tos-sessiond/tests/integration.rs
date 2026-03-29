use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tos_common::MockBrain;

#[tokio::test]
async fn test_sessiond_integration() -> anyhow::Result<()> {
    let mock_brain: MockBrain = MockBrain::new().await?;
    let bin_path = env!("CARGO_BIN_EXE_tos-sessiond");
    let mut child = Command::new(bin_path).spawn()?;
    
    let (_name, port) = mock_brain.handle_one_registration().await?;
    assert_eq!(_name, "tos-sessiond");
    
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    stream.write_all(b"session_list:global\n").await?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    assert!(line.trim().starts_with("[")); // JSON list
    
    child.kill().await?;
    Ok(())
}
