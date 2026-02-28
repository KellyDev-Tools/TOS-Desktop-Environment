use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_stimulate_brain_node() -> anyhow::Result<()> {
    println!("\x1B[1;35m[TOS STIMULATOR: stimulator_brain_node]\x1B[0m");
    println!("Connecting to Brain Node at localhost:7000...");

    // Try to connect with retries as the node might be booting
    let mut stream = None;
    for _ in 0..10 {
        if let Ok(s) = TcpStream::connect("127.0.0.1:7000").await {
            stream = Some(s);
            break;
        }
        sleep(Duration::from_millis(1000)).await;
    }

    let mut stream = stream.ok_or_else(|| anyhow::anyhow!("Failed to connect to Brain Node"))?;
    println!("-> Connection established.\n");

    let commands = vec![
        "sector_create:Staging",
        "zoom_in:",
        "set_mode:directory",
        "prompt_submit:ls -la",
        "search:brain",
        "set_setting:test_run;true",
        "zoom_to:GlobalOverview",
    ];

    let (reader, mut writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut response_line = String::new();

    for (i, cmd) in commands.into_iter().enumerate() {
        println!("\x1B[1;33m[STIMULUS {}]\x1B[0m Sending: {}", i + 1, cmd);
        writer.write_all(format!("{}\n", cmd).as_bytes()).await?;
        writer.flush().await?;
        
        response_line.clear();
        if let Ok(_) = reader.read_line(&mut response_line).await {
            println!("\x1B[1;36m[BRAIN RESPONSE]\x1B[0m {}", response_line.trim());
        }
        
        sleep(Duration::from_millis(1000)).await;
    }

    println!("\x1B[1;32mSTIMULATION SEQUENCE COMPLETE.\x1B[0m");
    Ok(())
}
