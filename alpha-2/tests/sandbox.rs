use tos_alpha2::modules::sandbox::SandboxManager;

#[test]
fn test_sandbox_process_creation() {
    // We attempt to spawn a process using the Kernel-Level Sandboxing.
    // In a standard unprivileged test environment, user namespaces are required.
    // Our SandboxManager handles this by passing CLONE_NEWUSER along with other isolation flags.
    
    // Attempt to run `echo "hello from sandbox"`
    let child = SandboxManager::spawn_sandboxed_process("echo", &["hello from sandbox"]);
    
    // Depending on the CI/Runner environment, unshare might still be blocked by seccomp or apparmor.
    // We check if it either succeeded (returned Ok(Child)) or gracefully failed with a known OS error.
    match child {
        Ok(mut c) => {
            let status = c.wait().expect("Failed to wait on sandboxed child");
            assert!(status.success(), "Sandboxed process did not exit successfully");
        }
        Err(e) => {
            let err_msg = e.to_string();
            println!("Sandboxing failed as expected in restricted test environment: {}", err_msg);
            // It's acceptable for this to fail with "Operation not permitted" (EPERM)
            // if the host kernel blocks user namespaces for unprivileged users.
            assert!(
                err_msg.contains("Operation not permitted") || 
                err_msg.contains("No space left on device") || // sometimes hit with user_namespaces limit
                err_msg.contains("Invalid argument"),          // some kernels reject flag combos
                "Expected either success or a permission-related error, got: {}", err_msg
            );
        }
    }
}
