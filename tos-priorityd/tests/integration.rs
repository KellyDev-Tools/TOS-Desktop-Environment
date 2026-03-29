use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tos_common::MockBrain;

#[tokio::test]
async fn test_priorityd_integration() -> anyhow::Result<()> {
    let mock_brain: MockBrain = MockBrain::new().await?;
    let bin_path = env!("CARGO_BIN_EXE_tos-priorityd");
    let mut child = Command::new(bin_path).spawn()?;
    
    let (_name, port) = mock_brain.handle_one_registration().await?;
    assert_eq!(_name, "tos-priorityd");
    
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    stream.write_all(b"get_priority:test-sector\n").await?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    assert!(line.contains("total_score"));
    
    child.kill().await?;
    Ok(())
}
