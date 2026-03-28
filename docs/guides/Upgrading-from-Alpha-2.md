# Upgrading from TOS Alpha-2 to Beta-0

This guide outlines the critical architectural changes and required code updates for existing system services transitioning to the Beta-0 environment.

## 1. Dynamic Port Registration (§4.1)

In Alpha-2, services often used hardcoded ports (e.g., 7002, 7003). In Beta-0, all services **MUST** use ephemeral ports and register their availability with the Brain's Discovery Gate.

### The Brain Discovery Gate
- **Socket Path**: `/tmp/brain.sock` (Unix Domain Socket)
- **Mechanism**: Use the `tos_lib::daemon::register_with_brain` helper.

### Code Example (Old Alpha-2):
```rust
let listener = TcpListener::bind("127.0.0.1:7002").await?;
```

### Code Example (New Beta-0):
```rust
use tos_lib::daemon::register_with_brain;

// 1. Bind to an ephemeral port (0)
let listener = TcpListener::bind("127.0.0.1:0").await?;
let port = listener.local_addr()?.port();

// 2. Register with the Brain gate
register_with_brain("tos-my-daemon", port).await?;
```

## 2. Standardized Logging (§2.1)

Beta-0 strictly enforces the use of the `tracing` framework for all system output. Use of `println!` or `eprintln!` in daemon binaries is prohibited as they bypass the system's log management and tier-based filtering.

- **Action**: Replace `println!(...)` with `tracing::info!(...)`.
- **Action**: Replace `eprintln!(...)` with `tracing::error!(...)`.

## 3. The `tos-protocol` Crate

The core state structures (`TosState`, `CommandHub`) and IPC handlers have been moved from the Brain into the `tos-protocol` crate. 

- **Action**: Update dependencies to point to the workspace member `tos-protocol`.
- **Action**: Use `tos_protocol::state::TosState` instead of local copies.

## 4. Semantic Event Abstraction (§14.1)

All physical input is now normalized into `SemanticEvent` enums.
- **Contract**: Interfaces responding to user input should consume `SemanticEvent` rather than raw keyboard/mouse events.
