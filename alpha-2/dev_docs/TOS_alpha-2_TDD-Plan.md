# TOS Alpha-2 Test Driven Development (TDD) Plan

**Version:** 1.0  
**Status:** DRAFT  
**Scope:** Tactical Operating System (TOS) Alpha-2 Core Logic (Brain), IPC, and UI Integration (Face)

## 1. Introduction
This plan outlines the TDD strategy for implementing the TOS Alpha-2 architecture. Following the "Terminal-First" philosophy, testing will focus on verifying the integrity of the command stream, the accuracy of the IPC protocol, and the safety of the modular sandbox.

## 2. Testing Tiers

### 2.1 Tier 1: Brain Core & Logic (Unit Tests)
*   **Goal:** Verify state transitions, command parsing, and settings resolution without a UI or PTY.
*   **Key Components:**
    *   **Settings Resolver:** Test cascading resolution (Global -> Sector -> App).
    *   **Sector Tree:** Test creation, cloning, and destruction of sectors and hubs.
    *   **Priority Engine:** Verify priority scores based on mock activity factors.
    *   **IPC Parser:** Validate `prefix:payload` and semicolon-delimited arguments.

### 2.2 Tier 2: Shell API & PTY (Integration Tests)
*   **Goal:** Verify bidirectional communication between the Brain and the underlying PTY/Shell.
*   **Key Components:**
    *   **OSC Parser:** Test all 9000-series sequences (9002 results, 9003 CWD, 9012 Priority).
    *   **Command Result Capturing:** Verify Base64 decoding of `command_result`.
    *   **TTY Buffer:** Test the 500-line FIFO limit and user-adjustable overrides.
    *   **Remote Disconnect:** Mock connection drops and verify the 5-second auto-close timer.

### 2.3 Tier 3: Communication & UI (E2E Integration)
*   **Goal:** Verify the **Face-to-Brain** IPC contract using mock UI events.
*   **Key Components:**
    *   **Action Identifier Enforcement:** Verify that UI messages use identifiers, not labels.
    *   **Tactile/Voice Confirmation:** Test the confirmation state machine (Slide -> Progress -> Execute).
    *   **Level Transitions:** Verify state snapshots when zooming from Level 1 to Level 2.

### 2.4 Tier 4: Modular Sandbox (Security Tests)
*   **Goal:** Verify permissions and isolation.
*   **Key Components:**
    *   **Trust Tiers:** Ensure "Standard" modules cannot access restricted system traits.
    *   **Manifest Validation:** Verify module loading fails if permissions are not declared.
    *   **Theme Injection:** Test that CSS variables from Theme Modules are safely injected.

---

## 3. Mocking Strategy

| Component | Mocking Approach |
|-----------|------------------|
| **PTY** | Use a virtual pipe to simulate shell stdout/stderr and inject OSC sequences. |
| **Shell** | Use a minimal reference script that emits known OSC sequences for testing. |
| **UI (Face)** | Use a JSON-RPC test harness to simulate bezel clicks and prompt submissions. |
| **Android SAF / Wayland** | Mock the `SystemServices` trait (§15.1) to return canned directory listings and metrics. |

---

## 4. Specific Test Protocols (TDD Workflow)

### 4.1 IPC Standardization Test
1.  **Define Test:** `test_ipc_semicolon_parsing`
2.  **Input:** `set_setting:theme;lcars-dark`
3.  **Expected:** `state.settings.get("theme") == "lcars-dark"`
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
