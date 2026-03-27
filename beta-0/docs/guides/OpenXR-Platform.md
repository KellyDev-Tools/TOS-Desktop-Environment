# OpenXR & Spatial Platform (Quest/Meta)

The TOS Spatial Face provides a high-fidelity immersive "Cockpit" environment.

## 1. Spatial Registry (§3.3.5)

The Spatial Face MUST register with the `spatial` profile. 

```json
{
  "type": "face_register",
  "profile": "spatial",
  "capabilities": ["hand_tracking", "gaze", "spatial_audio"],
  "viewport": { "w": 3840, "h": 2160 }
}
```

## 2. Rendering Strategy (§15.3)

TOS Spatial Faces use "World-Space Compositing":
- The UI is projected onto curved cylinders and quads in a virtual cockpit.
- **Performance**: High-throughput terminal rendering uses EGLImage to avoid CPU stalls (§15.3.2).

## 3. Remote Synchronization

Spatial Faces often act as "Thin Clients" for a remote TOS Brain:
1. Connecting via WebSocket (`7001`).
2. Receiving state deltas and high-resolution terminal textures.
3. Sending normalized spatial inputs (raycasts/hand triggers).
