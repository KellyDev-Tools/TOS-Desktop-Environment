# TOS Alpha-2 Project Structure

This document outlines the organization of the source code for the Tactical Operating System (TOS) Alpha-2 implementation.

## 1. Source Directory (`src/`)

### 1.1 Core Processes
- **`main.rs`**: The system entry point. Responsible for initializing the IPC channel and spawning the **Brain** (Logic) and **Face** (UI) threads or processes.
- **`brain/`**: The "Authoritative State." Handles all logic, command execution, and sector management. No rendering code lives here.
- **`face/`**: The "Representative Layer." Handles input capture and visual rendering. It receives state snapshots from the Brain.

### 1.2 Support Systems
- **`common/`**: Shared data structures (e.g., `Sector`, `CommandHub`) and IPC serialization logic used by both Brain and Face.
- **`services/`**: Independent daemons defined in Section 4 of the spec (Settings, Logging, AI, etc.).
- **`modules/`**: The runtime environment for plugins. Includes the sandbox logic and the SDK for building standard modules.
- **`platform/`**: Concrete implementations of the `Renderer`, `InputSource`, and `SystemServices` traits for different operating systems.

## 2. External Assets
- **`ui/`**: Static web assets if the Face utilizes a web-based rendering engine (HTML/CSS/JS).
- **`modules/`**: Local repository for installed `.tos-terminal`, `.tos-theme`, and `.tos-appmodel` files.
- **`scripts/`**: Integration files that users source in their external shells to enable OSC communication.

## 3. Quality Assurance
- **`tests/`**: Tiered test suites as defined in the **TDD Plan**.
  - `test-core`: Brain logic unit tests.
  - `test-shell`: PTY/OSC integration tests.
  - `test-sec`: Sandbox and permission verification.
  - `test-brain`: Brain component tests.
  - `test-ui`: Playwright-based frontend/bridge validation.
