# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build the library
cargo build --lib

# Build the GUI application
cargo build --features gui

# Run all tests (276 tests)
cargo test

# Run a specific test file
cargo test --test bezel_state_test

# Run tests with specific features
cargo test --features accessibility,saas
```

## Feature Flags

Available feature flags in `tos-dream/Cargo.toml`:

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `gui` | GUI with Tao/Wry | wry, tao |
| `gamepad` | Game controller support | gilrs |
| `accessibility` | Screen reader, high-contrast, switch access | rodio, speech-dispatcher, atspi, dark-light |
| `live-feed` | WebSocket streaming for remote observation | tokio-tungstenite |
| `voice-system` | Voice commands and STT | cpal, whisper-rs |
| `script-engine` | JavaScript/Lua scripting | rquickjs, mlua |
| `remote-desktop` | SSH and WebSocket remote connections | russh, russh-keys |
| `wayland` | Native Wayland compositor | smithay, wayland-server |
| `openxr` | OpenXR/Android XR support | openxr |
| `saas` | SaaS/cloud infrastructure | bollard, kube, aws-config, vaultrs |

**Combined flags:**
- `full` (default) = gui + gamepad + accessibility + live-feed + saas + wayland + openxr

## Architecture Overview

### Three-Level Hierarchy (Tactical System Design)

TOS uses a recursive zoom hierarchy inspired by LCARS (Star Trek interface):

1. **Global Overview (Level 1)**: Bird's-eye view of all sectors (tiles)
2. **Command Hub (Level 2)**: Central control for a sector with three modes
3. **Application Focus (Level 3)**: Full-screen application with Tactical Bezel

Deeper inspection levels (4-5) available for debugging:
- **Level 4**: Detail Inspector - CPU, memory, uptime, event history
- **Level 5**: Buffer Inspector - Raw memory hex dump (requires elevation)

### Core Data Structures

| Type | Description |
|------|-------------|
| `TosState` | Main state container with all sectors, viewports, and managers |
| `Sector` | Self-contained workspace with hubs, connection info, participants |
| `CommandHub` | Level 2 control point with mode, prompt, applications |
| `Viewport` | Current view state with zoom level and bezel state |
| `Application` | Running app with PID, icon, bezel actions, decoration policy |

### Key Modules

| Module | Purpose |
|--------|---------|
| `system/` | PTY, IPC, security, voice, AI, search, performance, shell API |
| `ui/` | Rendering, minimap, SVG engine |
| `modules/` | Hot-reloadable Sector Types and Application Models |
| `marketplace/` | Package discovery, installation, verification |
| `containers/` | Docker/Kubernetes containerization |
| `saas/` | Multi-tenant cloud infrastructure |
| `platform/` | Platform abstractions (Wayland, OpenXR, Android) |

### Command Hub Modes

| Mode | Purpose |
|------|---------|
| `Command` | CLI input with shell integration |
| `Directory` | File browser with OSC 777 synchronization |
| `Activity` | Process/application manager |
| `Search` | Federated search across files, logs, sectors |
| `AI` | Natural language queries with Ollama/OpenAI backends |

## Development Workflow

1. **修改代码**: Make changes in `tos-dream/src/`
2. **Run tests**: `cargo test` (276 tests)
3. **Build**: `cargo build --features gui` for UI testing
4. **Run UI**: `cargo run --features gui`

## Platform Support

- **Linux Wayland**: Full desktop experience via Smithay compositor
- **Android XR**: OpenXR spatial computing support
- **Android Phone**: Touch interface via Android backend

## Key Patterns

- **Input Abstraction**: All physical devices map to `SemanticEvent`
- **Module System**: Hot-reloadable modules via `TosModule` trait
- **Modular Code**: Reduce repeated patterns into common function calls
- **Shell API**: OSC 777 sequences for shell-compositor communication
- **Security**: Dangerous commands require tactile confirmation
- **Priority System**: Visual indicators (border chips, chevrons) based on weighted scoring

## AI Development Standards

Any AI agent working on this codebase must adhere to these tactical principles to maintain architectural integrity and visual excellence:

### 1. AI Commenting Rules
- **NEVER** use document names (e.g., "Phase 10", "TOS AI Development Standards") in code or doc comments. It is confusing. Use the distinct name of the feature/module instead.
- Use the `§` symbol when referencing sections from the Architectural Specification (e.g., `// See §3.4`).

### 2. Testing Expectations & TDD
- **Test-Driven Development (TDD)** is strictly required. Write the test first, ensure it fails, then write the code to pass it.
- **"No Code Without Tests"**: Every new feature must include an integration test verifying the "Hierarchy Round-Trip".
- You must run `cargo check` and `cargo test` after *every* significant file change to ensure zero regressions.
- **Task Completion Validation**: Always run the full test suite (`cargo test`) upon completing a task. Verify all tests pass before marking the task complete. No regressions are acceptable.

### 3. Visual Excellence & UI Standards
- **LCARS Aesthetic**: Use the curated palette in `variables.css` (`--lcars-orange`, `--lcars-blue`, `--lcars-gold`, `--lcars-red`).
- **Glassmorphism**: Use semi-transparent backgrounds with `backdrop-filter: blur()` for overlays. Prefer high-density grids and "Elbow" paths over traditional window borders.
- **Premium Micro-Animations**: Interfaces must feel "alive." Any state change (mode switch, zoom) must be accompanied by a subtle transition or animation (e.g., `recursive-zoom` keyframes). Avoid generic layouts.

### 4. Rust Coding Standards (Safety & Modularity)
- Use custom enums for state management instead of strings or booleans (e.g., `CommandHubMode`).
- Favor `Result` and `Option` over `unwrap()`. Errors in tactical systems must be handled gracefully.
- **Compiler Protections**: Do not disable compiler protections. Avoid `#[allow(warnings)]` and only use `unsafe` if strictly required for FFI/performance and if thoroughly documented and explicitly approved.
- **Unused Imports**: **Never** use `#[allow(unused_imports)]`. Instead, comment out the imports and document why they are unused or explain why they are needed.
- **Error Reporting**: All errors must be reported via the centralized log system (`src/system/log.rs`). Use `LogManager` to record errors with the appropriate `LogType` (e.g., `LogType::System` or `LogType::Security`) with appropriate region context.
- Every public function and struct must have a doc comment explaining its role in the tactical environment.

### 5. Interaction Etiquette
- **Be Proactive**: If a change requires a new CSS rule or a document update, perform it immediately.
- **Be Aesthetic**: If generating UI, ensure it explicitly looks "Tactical" and "Premium."
- **Be Structured**: Utilize headers and bold text in responses to make technical details straightforward to parse.
