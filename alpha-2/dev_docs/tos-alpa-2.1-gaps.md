✅ Critical Gaps (Resolved in Alpha-2.1 Specification)

1. Remote & Collaboration Protocols (Resolved in Architecture Spec §12, §13)

· TOS Remote Server Protocol (§12.1) – Added handshake, message definitions, and WebRTC signalling details.
· Collaboration Data Channel (§13.7) – Specified payloads for presence, following mode, cursor sync, and role changes.
· Web Portal Security (§13.8) – Defined token generation, expiry, and one-time URL structure.

2. Module API Contracts (Resolved in Ecosystem Spec §1)

· Terminal Output Module (§1.5.1) – Defined Rust trait and Web/Native profile functions (init, push_lines, render, on_click).
· AI Backend Module (§1.3.1) – Specified Brain invocation (ai_query, ai_tool_call) and JSON response format.
· Shell Module (§1.7) – Defined PTY lifecycle (spawn, write, resize, signal) and OSC sequence expectations.
· Theme Module (§1.6) – Clarified CSS variable injection and dynamic state updates.
· Bezel Component Module (§1.8) – Specified API for state updates, rendering, and projection.

3. IPC Message Schemas (Resolved in Architecture Spec §3.3)

· State Delta (§3.3.2) – Defined JSON structure for Brain→Face updates.
· Settings IPC (§3.3.3) – Defined messages for get/set/notify with standardized semicolon delimiters.
· TOS Log Query (§3.3.4) – Defined query language and JSON response format.

4. Platform Backend Details (Resolved in Architecture Spec §15)

· Wayland (§15.2) – Specified layer shell usage, DMABUF surface embedding, and input forwarding.
· Android XR (§15.3) – Defined OpenXR action mapping for gaze, pinch, and hand tracking.
· Native Application Embedding (§15.6) – Specified Wayland/X11 window composition into Level 3 viewports.

5. Security & Sandbox Implementation (Resolved in Architecture Spec §17)

· Sandbox Profiles (§17.2.1) – Defined Bubblewrap rules for default, network, and filesystem isolation.
· Permission List (§17.2.2) – Provided exhaustive list of standard permissions and enforcement mechanisms.
· Audit Log Format (§17.5.1) – Defined JSON Lines schema for non-disableable security events.
