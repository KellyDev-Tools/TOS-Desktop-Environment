use std::io::BufRead;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use tos_common::brain::Brain;

#[tokio::test]
async fn test_crash_reporting_thread() {
    // 1. Start a mock loggerd on port 7003 (the fallback port used by LoggerService)
    // We use a TcpListener to intercept the "crash:..." message.
    let listener = TcpListener::bind("127.0.0.1:7003").expect("Failed to bind to port 7003 for testing");
    
    let received_messages = Arc::new(Mutex::new(Vec::<String>::new()));
    let msgs_clone = received_messages.clone();
    
    thread::spawn(move || {
        // Accept connections and read lines until we find the crash report
        while let Ok((socket, _)) = listener.accept() {
            let mut reader = std::io::BufReader::new(socket);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 { break; }
                let trimmed = line.trim().to_string();
                if trimmed.starts_with("crash:") {
                    let mut lock = msgs_clone.lock().unwrap();
                    lock.push(trimmed);
                    return; // Found it!
                }
                line.clear();
            }
        }
    });

    // 2. Initialize Brain
    // This will load the default config and set up LoggerService.
    let brain = Brain::new().expect("Failed to initialize Brain");
    
    // 3. Send a crash report via IPC
    // The IpcHandler should route this to self.services.logger.crash_report(payload)
    let crash_payload = "SIGSEGV at 0x41414141";
    let res = brain.ipc.handle_request(&format!("crash:{}", crash_payload));
    
    assert_eq!(res, "OK");

    // 4. Verify mock loggerd received the message
    // Give it a moment to complete the TCP connection and write.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    let lock = received_messages.lock().unwrap();
    assert_eq!(lock.len(), 1, "Mock loggerd should have received exactly 1 message");
    assert_eq!(lock[0], format!("crash:{}", crash_payload));
}
