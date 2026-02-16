## Full Architecture for a Native Horizon OS Client for TOS (Option B)

### Overview
The native Horizon OS client is a dedicated Android application (since Horizon OS is based on Android) that connects to a remote TOS instance via the TOS Remote Server protocol. It provides a fully immersive VR/AR interface for interacting with remote sectors, applications, and files, while leveraging the headset’s native capabilities (controllers, hand tracking, spatial audio, haptics). The client also implements two‑way file synchronization between the remote host and the headset’s local storage.

The client is designed as a modular system with the following high‑level components:
- **Connection Manager** – handles discovery, authentication, and secure transport.
- **Rendering Engine** – decodes and displays the streamed visual output from the remote host.
- **Input Processor** – translates native Horizon OS input events into TOS semantic events.
- **File Sync Service** – manages bidirectional file synchronization between remote and local file systems.
- **Collaboration Module** – handles multi‑user presence and role‑based interactions.
- **Local UI Layer** – provides overlay elements (e.g., sync status, connection info) and optional native menus.

All components communicate via well‑defined internal APIs and share state through a central **Session Manager**.

---

### 1. System Architecture (Client‑Side)

```

+---------------------+       +---------------------+

|   Horizon OS (Android)      |    TOS Remote Server   |

|                     |       |      (Host Machine)    |

|  +---------------+  |       |  +-------------------+ |

|  |   Connection  |  | <-->  |  |  Control Channel | |

|  |    Manager    |  |       |  |  (WebSocket/TLS) | |

|  +---------------+  |       |  +-------------------+ |

|          |          |       |  +-------------------+ |

|  +---------------+  |       |  |   Video Stream    | |

|  |   Rendering   |  | <-->  |  |   (WebRTC/H.264)  | |

|  |    Engine     |  |       |  +-------------------+ |

|  +---------------+  |       |  +-------------------+ |

|          |          |       |  |   File Transfer   | |

|  +---------------+  |       |  |   (WebDAV/rsync)  | |

|  |    Input      |  | <-->  |  +-------------------+ |

|  |   Processor   |  |       |                        |

|  +---------------+  |       |                        |

|          |          |       |                        |

|  +---------------+  |       |                        |

|  |  File Sync    |  | <-->  |                        |

|  |    Service    |  |       |                        |

|  +---------------+  |       |                        |

|          |          |       |                        |

|  +---------------+  |       |                        |

|  | Collaboration |  | <-->  |                        |

|  |    Module     |  |       |                        |

|  +---------------+  |       |                        |

|          |          |       |                        |

|  +---------------+  |       |                        |

|  |   Local UI    |  |       |                        |

|  |    Overlay    |  |       |                        |

|  +---------------+  |       |                        |
+---------------------+       +------------------------+

```


#### Key Data Flows
- The **Connection Manager** establishes a persistent control channel (e.g., WebSocket over TLS) for command/event exchange.
- The **Rendering Engine** receives a video stream (via WebRTC or a custom UDP‑based protocol) and displays it as a texture in the VR scene.
- **Input events** from controllers, hands, or gaze are captured by the Horizon OS, translated by the **Input Processor** into TOS semantic events, and sent over the control channel.
- **File synchronization** uses a separate channel (e.g., WebDAV or a custom protocol) to list, upload, and download files. The **File Sync Service** runs in the background, monitoring local and remote changes.
- **Collaboration data** (user presence, cursors, role changes) flows over the control channel; the **Collaboration Module** renders remote user avatars/cursors and handles role enforcement.
- The **Local UI Overlay** displays connection status, sync progress, and other native elements that are composited on top of the remote video.

---

### 2. Communication with TOS Remote Server

The client communicates with the TOS Remote Server using multiple parallel channels:

#### a) Control Channel
- **Protocol**: WebSocket (or a custom TCP‑based protocol) secured with TLS.
- **Authentication**: SSH keys (client presents a key), password, or time‑limited invite tokens. The client stores credentials securely using Android Keystore.
- **Message Format**: JSON‑RPC or a compact binary format (e.g., MessagePack) for efficiency.
- **Messages include**:
  - Session establishment (handshake, authentication).
  - Input events (zoom, select, mode changes, text input).
  - Viewport commands (split, close, resize).
  - Collaboration events (user join/leave, cursor positions, role changes).
  - Heartbeats and keep‑alives.

#### b) Video/Audio Stream
- **Protocol**: WebRTC (for NAT traversal and adaptive bitrate) or a custom UDP‑based RTP stream.
- **Codec**: H.264 or H.265 hardware‑decoded on Quest for low latency.
- **Stream Content**: The TOS compositor outputs a dedicated video stream for the requested sector or viewport. Multiple streams can be requested (e.g., for splits).
- **Audio**: Three‑layer audio (ambient, tactical, voice) is mixed on the host and streamed as a separate audio track.

#### c) File Transfer Channel
- **Protocol**: WebDAV over HTTPS, or a custom protocol over the same WebSocket (with binary frames). WebDAV provides a standard way to list, read, write, and manage files.
- **Authentication**: Reuses the control channel credentials or a separate token.
- **Operations**: Directory listing, file download/upload, move, delete, metadata retrieval.

---

### 3. Display and Rendering Pipeline

The client renders the remote interface in a fully immersive VR environment. Key aspects:

- **Video Decoding**: The incoming video stream is fed to the Quest’s hardware decoder (via Android’s `MediaCodec` API). Decoded frames are converted to OpenGL textures.
- **Scene Graph**: The client maintains a 3D scene with a “virtual screen” (a quad or curved surface) onto which the remote video texture is mapped. The user can resize, reposition, and curve the screen using controller interactions.
- **Multiple Viewports**: If the remote sector is using split viewports, the host can either:
  - Send a single video containing the entire composited view (simpler), or
  - Send separate streams for each viewport (more complex but allows independent spatial positioning). The client then places each stream on a separate 3D surface.
- **Overlay Compositing**: Local UI elements (sync status, connection indicator, native menus) are rendered on top of the video texture using a separate render pass, ensuring they remain readable.
- **Performance Optimizations**:
  - Foveated rendering (if supported) to reduce pixel shading in peripheral vision.
  - Adaptive resolution based on network conditions and head movement.
  - Frame interpolation to smooth occasional drops.

---

### 4. Input Handling and Mapping

Horizon OS provides access to various input devices: controllers (trigger, grip, thumbstick, buttons), hand tracking (pinch, point, grab), and gaze. The client’s **Input Processor** translates these into TOS semantic events.

#### Mapping Rules
- **Controller trigger** → `select` (tap/click)
- **Controller grip** → `secondary_select` (right‑click / context menu)
- **Thumbstick up/down** → `zoom_in` / `zoom_out` (when in appropriate context)
- **Thumbstick click** → `toggle_bezel_expanded`
- **Hand pinch** → `select`
- **Hand grab** → `drag` (to move surfaces or resize viewports)
- **Gaze + dwell** → `select` (if eye tracking available)
- **Voice commands** → transcribed locally (or sent as raw audio) and mapped to `voice_command_start` with the transcribed text.

The mapping is user‑configurable via a settings panel. The client also supports simultaneous input from multiple devices (e.g., using controllers for navigation while using voice for commands).

---

### 5. File Synchronization

The **File Sync Service** provides bidirectional synchronization between a user‑specified folder on the remote host and a corresponding folder on the headset’s local storage.

#### Architecture
- **Sync Engine**: A background service that monitors changes using:
  - On the remote side: the TOS Remote Server’s file system API (polling or inotify‑based events over the control channel).
  - On the client side: Android’s `FileObserver` to detect local changes.
- **Conflict Resolution**: Configurable strategies (e.g., “keep both”, “remote wins”, “local wins”, or manual resolution via a UI).
- **Transfer Protocol**: Uses the file transfer channel (WebDAV) to upload/download files. Large files are chunked to allow resumption.
- **Selective Sync**: Users can choose which folders or file types to sync. A local UI shows sync status (progress, conflicts, errors).

#### Integration with TOS Directory Mode
- The client can present the remote file system as if it were local, using the file transfer channel. Directory Mode in TOS (when accessed remotely) already provides file browsing; the client can enhance it by adding a “Sync this folder” button.
- Alternatively, the client can expose a native file manager overlay that shows both remote and local files, with drag‑and‑drop between them.

---

### 6. Collaboration Support

The client fully participates in TOS collaboration features as a guest.

- **User Presence**: When the client connects, it registers as a guest with a specified role (Viewer, Commenter, Operator, Co‑owner). The host broadcasts the guest’s presence to all participants.
- **Cursor Sharing**: The client sends its cursor position (in 3D space projected onto the virtual screen) to the host, which then distributes it to other guests. Remote cursors are rendered as colored avatars/pointers in the client’s scene.
- **Following Mode**: The client can request to follow another user’s view. The host sends viewport state (zoom level, splits, focused surface) and the client replicates it.
- **Role Enforcement**: The host enforces permissions; the client simply disables UI elements that are not allowed for its role (e.g., a Viewer cannot type in the prompt).
- **Auditory/Haptic Cues**: Collaboration events (user join, role change, hand raise) trigger the client’s audio and haptic systems as per TOS specifications.

---

### 7. Performance Considerations

- **Latency Management**:
  - Local echo for input (e.g., showing button presses immediately) while waiting for remote confirmation.
  - Adaptive video bitrate based on network conditions.
  - Prediction of remote state (e.g., cursor movement) to smooth rendering.
- **Battery Life**:
  - Use hardware decoding to offload CPU.
  - Throttle background sync when on battery.
  - Reduce frame rate in low‑activity periods.
- **Memory**:
  - Limit the number of cached video frames.
  - Unload textures for off‑screen viewports.

---

### 8. Security

- **Authentication**: Credentials stored in Android Keystore; SSH keys generated locally and never transmitted.
- **Transport**: All channels encrypted with TLS; certificates verified against known host keys or CA.
- **File Sync**: Files transferred over HTTPS; local storage encrypted if the headset supports it.
- **Privacy**: The client respects TOS logging policies; guest actions are logged on the host, and the client displays a privacy notice upon connection.

---

### 9. Integration with Horizon OS

- **System UI**: The client can be launched from the Quest’s app library. When active, it may request permissions (storage, microphone for voice, etc.).
- **Passthrough**: For AR scenarios, the client can use Quest’s passthrough API to blend the virtual screen with the real environment.
- **Multitasking**: Since Horizon OS supports multiple 2D apps, the client could run alongside other apps (e.g., a browser) using the system’s windowing capabilities, but for full immersion it likely takes over the entire display.
- **Notifications**: System notifications from the headset (e.g., low battery) are handled by the OS and may be displayed on top of the client’s UI.

---

### 10. Summary

The native Horizon OS client for TOS is a sophisticated but feasible project that leverages the existing TOS Remote Server protocol and the Quest’s hardware capabilities. By building a modular architecture with separate channels for control, video, and file transfer, the client can deliver a seamless remote desktop experience in VR with full input, collaboration, and file synchronization features. The design emphasizes performance, security, and deep integration with the headset’s native input and output systems, while remaining faithful to TOS’s spatial command philosophy.


