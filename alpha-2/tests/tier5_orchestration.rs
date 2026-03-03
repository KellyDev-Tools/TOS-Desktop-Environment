use tokio::net::TcpStream;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_service_orchestration_health() -> anyhow::Result<()> {
    println!("\n\x1B[1;35m[TOS ORCHESTRATION TEST]\x1B[0m");
    
    let services = vec![
        ("Brain Core (TCP)", 7000),
        ("Brain UI Sync (WS)", 7001),
        ("Settings Daemon", 7002),
        ("Log Daemon", 7003),
        ("Marketplace Daemon", 7004),
        ("Priority Engine", 7005),
        ("Web UI Server", 8080),
    ];

    println!("Verifying tactical service constellation...\n");

    let mut failed = false;
    for (name, port) in services {
        print!("Checking {:<20} [Port {}] ... ", name, port);
        
        let mut connected = false;
        // Retry a few times as services might be booting
        for _ in 0..5 {
            if let Ok(_) = TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                connected = true;
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }

        if connected {
            println!("\x1B[1;32mONLINE\x1B[0m");
        } else {
            println!("\x1B[1;31mOFFLINE\x1B[0m");
            failed = true;
        }
    }

    if failed {
        println!("\n\x1B[1;31m[CRITICAL] ORCHESTRATION FAILURE: One or more services are unreachable.\x1B[0m");
        println!("Check orchestration logs in logs/*.log for details.");
        return Err(anyhow::anyhow!("Orchestration Health Check Failed"));
    }

    println!("\n\x1B[1;32m[SUCCESS] Full tactical stack is orchestrated and responsive.\x1B[0m");
    Ok(())
}
