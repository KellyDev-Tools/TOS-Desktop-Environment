use tos_common::modules::sandbox::{SandboxManager, SandboxProfile};
use std::fs;
use std::path::Path;

fn setup() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

fn bwrap_available() -> bool {
    std::process::Command::new("bwrap")
        .args(&["--unshare-user", "--", "true"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn test_sandbox_filesystem_isolation() {
    setup();
    if !bwrap_available() {
        println!("Skipping sandbox test: bwrap is not functional in this environment");
        return;
    }
    // Create a canary file in the host system
    let canary_path = "/tmp/tos_sandbox_canary.txt";
    fs::write(canary_path, "host_secret").unwrap();

    // Spawn a process in the default sandbox that tries to read the canary
    let output = SandboxManager::spawn_bwrap_process(
        SandboxProfile::Default,
        None,
        "ls",
        &[canary_path],
    ).expect("Failed to spawn sandboxed process");

    let status = output.wait_with_output().expect("Failed to wait for output");
    
    // Cleanup canary
    let _ = fs::remove_file(canary_path);

    // ls should fail because /tmp is a fresh empty directory in the sandbox
    assert!(!status.status.success(), "Sandbox should have blocked access to host /tmp files");
}

#[test]
fn test_sandbox_network_isolation() {
    setup();
    if !bwrap_available() {
        println!("Skipping sandbox test: bwrap is not functional in this environment");
        return;
    }
    // Attempt to ping localhost from within a default sandbox
    let output = SandboxManager::spawn_bwrap_process(
        SandboxProfile::Default,
        None,
        "ping",
        &["-c", "1", "127.0.0.1"],
    ).expect("Failed to spawn sandboxed process");

    let status = output.wait_with_output().expect("Failed to wait for output");
    
    // ping should fail because network is unshared
    assert!(!status.status.success(), "Sandbox should have blocked network access");
}

#[test]
fn test_sandbox_network_allowance() {
    setup();
    if !bwrap_available() {
        println!("Skipping sandbox test: bwrap is not functional in this environment");
        return;
    }
    // Attempt to check network interfaces from within a network-enabled sandbox
    let output = SandboxManager::spawn_bwrap_process(
        SandboxProfile::Network,
        None,
        "ip",
        &["addr"],
    ).expect("Failed to spawn sandboxed process");

    let status = output.wait_with_output().expect("Failed to wait for output");
    let stdout = String::from_utf8_lossy(&status.stdout);
    let stderr = String::from_utf8_lossy(&status.stderr);
    
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    // We expect some interfaces to be visible (like eth0, docker0, or at least lo)
    assert!(status.status.success(), "ip addr should succeed in network sandbox");
    assert!(!stdout.is_empty(), "Network sandbox should show some interfaces");
}

#[test]
fn test_sandbox_filesystem_binding() {
    setup();
    if !bwrap_available() {
        println!("Skipping sandbox test: bwrap is not functional in this environment");
        return;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tim".to_string());
    let sector_id = format!("test-sector-{}", uuid::Uuid::new_v4());
    let sector_base = Path::new(&home).join("TOS/Sectors").join(&sector_id);
    fs::create_dir_all(&sector_base).unwrap();
    
    let file_path = sector_base.join("data.txt");
    fs::write(&file_path, "sector_data").expect("Failed to write test file");

    println!("Created test file at {:?}", file_path);

    let output = SandboxManager::spawn_bwrap_process(
        SandboxProfile::FileSystem,
        Some(&sector_id),
        "cat",
        &["/mnt/sector/data.txt"],
    ).expect("Failed to spawn sandboxed process");

    let status = output.wait_with_output().expect("Failed to wait for output");
    let stdout = String::from_utf8_lossy(&status.stdout);
    let stderr = String::from_utf8_lossy(&status.stderr);

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    // Cleanup
    let _ = fs::remove_dir_all(&sector_base);

    assert_eq!(stdout.trim(), "sector_data", "FileSystem sandbox should have bound /mnt/sector");
}
