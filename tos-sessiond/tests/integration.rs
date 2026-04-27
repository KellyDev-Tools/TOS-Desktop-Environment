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

#[tokio::test]
async fn test_session_handoff() -> anyhow::Result<()> {
    let mock_brain: MockBrain = MockBrain::new().await?;
    let bin_path = env!("CARGO_BIN_EXE_tos-sessiond");
    let mut child = Command::new(bin_path).spawn()?;
    
    let (_, port) = mock_brain.handle_one_registration().await?;
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    
    // 1. Prepare Handoff
    let mock_json = "{\"version\":1, \"sectors\":[]}";
    let msg = format!("session_handoff_prepare:{}\n", mock_json);
    let (reader_stream, mut writer_stream) = TcpStream::connect(format!("127.0.0.1:{}", port)).await?.into_split();
    writer_stream.write_all(msg.as_bytes()).await?;
    
    let mut reader = BufReader::new(reader_stream);
    let mut token = String::new();
    reader.read_line(&mut token).await?;
    let token = token.trim();
    assert_eq!(token.len(), 6);
    
    // 2. Claim Handoff
    let (reader_stream, mut writer_stream) = TcpStream::connect(format!("127.0.0.1:{}", port)).await?.into_split();
    writer_stream.write_all(format!("session_handoff_claim:{}\n", token).as_bytes()).await?;
    
    let mut reader = BufReader::new(reader_stream);
    let mut claimed_data = String::new();
    reader.read_line(&mut claimed_data).await?;
    assert_eq!(claimed_data.trim(), mock_json);
    
    // 3. Claim again (should fail)
    let (reader_stream, mut writer_stream) = TcpStream::connect(format!("127.0.0.1:{}", port)).await?.into_split();
    writer_stream.write_all(format!("session_handoff_claim:{}\n", token).as_bytes()).await?;
    
    let mut reader = BufReader::new(reader_stream);
    let mut error_msg = String::new();
    reader.read_line(&mut error_msg).await?;
    assert!(error_msg.contains("ERROR: Invalid token"));
    
    child.kill().await?;
    Ok(())
}
