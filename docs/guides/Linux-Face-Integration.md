# Linux (Wayland) Face Integration

This guide defines how a Native Linux "Face" (application) interacts with the TOS Brain core.

## 1. Connection Lifecycle

### Discovery
1. The Face **MUST** first attempt to connect to the Brain's Discovery Gate at `/tmp/brain.sock`.
2. Upon connect, the Face **MUST** send a `face_register` message matching the architecture spec §3.3.5.

### FALLBACK
If `/tmp/brain.sock` is unavailable, the Face falls back to acting as a Remote Client over the network via the TOS Remote Server (§12.1), typically at `127.0.0.1:7000` (TCP) or `7001` (WebSocket).

## 2. Face Registration Protocol (§3.3.5)

Immediately after connection, the Face MUST register its capability profile:

```json
{
  "type": "face_register",
  "face_id": "unique-uuid-per-installation",
  "profile": "desktop",
  "capabilities": ["mouse", "keyboard", "wayland"],
  "viewport": { "w": 1920, "h": 1080 }
}
```

## 3. Input Normalization (§14.1)

All physical events (key down, touch, pointer) MUST be normalized into a `SemanticEvent` prefix before being dispatched to the Brain.

- **Example**: A `Ctrl + ]` key combination in the Face is normalized to `zoom_in:` before transmission.
- **Example**: An application launch request is sent as `app_launch:<name>`.

## 4. UI Rendering & Wayland Layer Shell

The TOS architecture expects a Native Face to render using the Wayland `layer-shell` protocol:
1. Surfaces MUST use `wlr-layer-shell` on the `TOP` layer.
2. The Face acts as a composite surface, embedding native application buffers via `dmabuf` frame sharing (§15.2).
