# TOS Alpha-2 Test Driven Development (TDD) Plan

**Version:** 1.0  
**Status:** DRAFT  
**Scope:** Terminal On Steroids (TOS) Alpha-2 Core Logic (Brain), IPC, and UI Integration (Face)

## 1. Introduction
This plan outlines the TDD strategy for implementing the TOS Alpha-2 architecture. Following the "Terminal-First" philosophy, testing will focus on verifying the integrity of the command stream, the accuracy of the IPC protocol, and the safety of the modular sandbox.

## 2. Testing Taxonomy & Coverage Tiers

This plan aligns structurally with the *Test Taxonomy & Definitions* outlined in the `TOS AI Development Standards` (§4.1).

### 2.1 Unit Tests (Inline Rust & Svelte `.spec.ts`)
*   **Goal:** Microsecond-fast validation of isolated functions, structs, and pure logic. No side effects, no full state initialization.
*   **Key Components:**
    *   **Settings Resolver:** Test cascading resolution (Global -> Sector -> App).
    *   **OSC Parser Algorithm:** Verify extraction of 9000-series sequences from dirty string inputs.
    *   **Priority Math:** Verify priority scores based on mock activity factors.
    *   **Trust Classifier:** Test regex/rules engines for `privilege_escalation` and `recursive_bulk` detection.

### 2.2 Integration Tests (Headless Native Rust)
*   **Goal:** Validate cross-subsystem interaction natively by testing the core Brain IPC interface against in-memory architecture (bypassing UI).
*   **Key Components:**
    *   **IPC Protocol Matcher:** Validate `prefix:payload` parsing and state mutation (`tests/headless_brain.rs`).
    *   **Sector Tree Lifecycle:** Test creation, cloning, and destruction of active sectors and PTY backends.
    *   **TTY Buffer Wraparound:** Test the 500-line FIFO limit when pushing live data to `system_log`.
    *   **Remote ICE Teardown:** Mock socket drops and verify the graceful 500ms auto-close teardown sequence.

### 2.3 Component Tests (Isolated Functional Blocks)
*   **Goal:** Verify individual units, daemons, or UI modules completely in isolation—independent of the rest of the system. Validates that specific distributed components act correctly according to design, making debugging faster.
*   **Key Components:**
    *   **Isolated Daemons:** Verify `tos-marketplaced` API responses without a running Brain by mocking the `brain.sock`.
    *   **Brain Subsystems:** Test the `TrustService` decision logic by injecting state JSON independently of the `IpcHandler`.
    *   **Web Face (Svelte/Playwright):** Assert LCARS `.lcars-bar` and `.glass-panel` layout rules, DOM presence, and interaction state changes in isolation.
    *   **Native Face (Wayland/OpenXR):** Use string-buffer testing stubs (`tests/face_visual_states.rs`) to validate state representations headlessly without a Compositor.

### 2.4 Modular Sandbox Tests (Security)
*   **Goal:** Verify permissions boundary enforcement within the module loader.
*   **Key Components:**
    *   **Capability Enforcement:** Ensure "Standard" modules cannot access restricted system traits.
    *   **Manifest Validation:** Verify module initialization safely rejects malformed or un-signed declarations.
    *   **Theme Injection Security:** Test that CSS variables from Theme Modules are safely deserialized and cannot perform XSS or arbitrary code execution.

---

## 3. Mocking Strategy

| Component | Mocking Approach |
|-----------|------------------|
| **PTY / Shell Backend** | Use virtual pipes (e.g. `tests/stimulator.rs`) to inject known OSC byte streams and capture responses. |
| **Settings File I/O** | Use an in-memory `HashMap` overlay during testing to prevent polluting `~/.config/tos/`. |
| **Face Input Engine** | Use headless IPC socket writes or direct function calls via the `test-protocol` harness. |
| **Native Renderers** | Stub the `Renderer` traits (`src/platform/mod.rs`) to dump output as parseable text strings layout trees. |

---

## 4. Specific Test Protocols (TDD Workflow)

### 4.1 Integration: IPC Standardization Test
1.  **Define Test:** `test_ipc_semicolon_parsing`
2.  **Input:** `set_setting:theme;lcars-dark`
3.  **Expected:** `state.settings.global.get("theme") == "lcars-dark"`
4.  **Input:** `signal_app:uuid-123;SIGKILL`
5.  **Expected:** `internal_signal_event(uuid-123, SIGKILL)`

### 4.2 Terminal Buffer FIFO Test
1.  **Define Test:** `test_terminal_buffer_wrap`
2.  **Setup:** Set `terminal_buffer_limit` to 5.
3.  **Operation:** Push 6 lines of text.
4.  **Expected:** Buffer contains lines 2-6; line 1 is discarded.

### 4.3 Remote Session Failure Test
1.  **Define Test:** `test_remote_disconnect_timer`
2.  **Setup:** Established remote sector.
3.  **Action:** Kill the mock remote socket.
4.  **Assert:** Sector state changes to `Disconnected`.
5.  **Wait 5.1s:** Verify the sector is removed from `state.sectors`.

### 4.4 Action-Identifier Enforcement
1.  **Define Test:** `test_bezel_label_rejection`
2.  **Input:** `click:ZOOM OUT` (Label instead of identifier)
3.  **Expected:** Log warning; no state change.
4.  **Input:** `click:zoom_out` (Identifier)
5.  **Expected:** `state.level` decrements.

---

## 5. Success Criteria
*   100% pass rate on Tier 1 & 2 tests.
*   All `Dangerous Commands` (§17.3) require verified confirmation logic.
*   Zero bypass of `Standard Tier` sandbox permissions in Tier 4 tests.
*   Latency for IPC round-trips (Face -> Brain -> Face) stays below 16ms in local testing.
