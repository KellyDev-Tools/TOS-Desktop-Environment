❗ Critical Gaps (Need Immediate Specification)

1. Remote & Collaboration Protocols

· TOS Remote Server Protocol (§12.1) – no message definitions, authentication handshake, or WebRTC signalling details.
· Collaboration Data Channel (§13) – payloads for presence, following mode, cursor sync, role changes are unspecified.
· Web Portal Security – token generation, expiry, one‑time URLs for exported sectors.

2. Module API Contracts

· Terminal Output Module – exact Rust trait / C ABI functions (e.g., init(context, config), render(lines), on_click(x,y)).
· AI Backend Module – how the Brain invokes the module (streaming, function calling), response format.
· Shell Module – PTY lifecycle: spawn(), write(), resize(), signal handling. OSC sequence expectations.
· Theme Module – CSS variable injection mechanism, dynamic updates, asset loading.
· Bezel Component Module – API for receiving state updates and rendering into slot.

3. IPC Message Schemas

· State Delta – JSON structure for Brain→Face updates (sectors, hubs, terminal lines, priority scores).
· Settings IPC – exact messages for get_setting, set_setting, change notifications.
· TOS Log Query – query language and response format for log service.

4. Platform Backend Details 

- should these be a part of the brain or their own process so that one brain node can service any one of them?

· Wayland – how the Face renders as a layer shell, input forwarding to native windows.
· Android XR – OpenXR action mapping for gaze, pinch, hand tracking.
· Native Application Embedding – how Wayland/X11 windows are composited into Level 3.

5. Security & Sandbox Implementation

· Sandbox Profiles – bubblewrap/Flatpak rules for each permission set.
· Permission List – exhaustive list of possible permissions (e.g., filesystem:read, network:client) and their enforcement.
· Audit Log Format – schema for non‑disableable audit events.


