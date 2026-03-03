# TOS Alpha-2 Ecosystem Orchestration

This document details the service-oriented architecture (SOA) of the TOS Alpha-2 environment and the orchestration protocols used for system-wide boot synchronization.

## Architecture Overview

TOS is designed as a constellation of independent processes communicating over a local bus (TCP/WebSocket). This decoupled approach ensures that AI failures, log overflows, or marketplace timeouts do not compromise the core Brain logic.

### 1. The Core Constellation
- **`tos-brain`**: The central authority. It manages the global state, hierarchical navigation (Levels 1-5), and coordinates the IPC flow between all other components.
- **`tos-face`**: The visual layer (Web UI). It communicates with the Brain via WebSocket (Port 7001) for high-frequency state updates and TCP (Port 7000) for command submission.

### 2. Auxiliary Daemons (§4)
Daemons are specialized services that extend the Brain's capabilities. They are launched as background processes:

- **Settings Service (`tos-settingsd`)**: Managed on Port 7002. It handles the persistence of global and sector-specific configurations.
- **Log Service (`tos-loggerd`)**: Managed on Port 7003. It aggregates structured JSONL logs from all system components.
- **Marketplace Service (`tos-marketplaced`)**: Managed on Port 7004. It performs cryptographic verification of installable modules.
- **Priority Service (`tos-priorityd`)**: Managed on Port 7005. It calculates tactical priority scores based on system telemetry.

## Orchestration Flow

The boot sequence is managed by the `Makefile` using the following hierarchy:

### Phase 1: Service Initialization (`make run-services`)
The system first spawns the auxiliary daemons in the background. Each daemon has its output redirected to `logs/*.log` to keep the primary console focused on Brain telemetry.
1. `mkdir -p logs`
2. Cleanup of lingering processes (`pkill`).
3. Background launch of all 4 daemons.

### Phase 2: Face Orchestration (`make run-web`)
Once services are initialized, the UI server and Brain core are launched.
1. Spawns `python3 -m http.server` to serve the Web UI on Port 8080.
2. Initializes the `tos-brain`.
3. Sets up a signal trap (INT/TERM) to ensure that killing the Brain also terminates the UI server and backgrounded services.

## Manual Status Check

To verify the health of the constellation, check the following:

- **Process Status**: `ps aux | grep tos-`
- **Socket Awareness**: `ss -ltn | grep -E "700[0-5]|8080"`
- **Tactical Health Check**: `make test-health` (Automated reachability verification)
- **Port Conflict Fix**: If a port is busy, use `fuser -k <port>/tcp` to force-clear it.
- **Orchestration Logs**:
  - `logs/settingsd.log`
  - `logs/loggerd.log`
  - `logs/marketplaced.log`
  - `logs/priorityd.log`
  - `logs/tos-brain.log`

## Communication Protocol

- **Brain-to-Daemon**: The Brain initiates tactical queries to daemons as needed (e.g., fetching a setting or verifying a signature).
- **Face-to-Brain**: The Face maintains a persistent WebSocket for state synchronization and sends discrete commands to the Brain's TCP IPC handler.
