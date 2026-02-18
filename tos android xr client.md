## Full Architecture for a Native Android XR Client for TOS (Option B)



### Overview
The native Android XR client for TOS is an OpenXR‑based application built on the Android XR platform. It connects to a remote TOS instance via the TOS Remote Server protocol, delivering a fully immersive spatial computing experience across the Android XR ecosystem—from high‑performance VR headsets (e.g., Samsung’s “Project Moohan”) to lightweight AI‑driven smart glasses. The client leverages the platform’s native capabilities: gaze and hand tracking, the Gemini AI assistant, hardware‑accelerated video decoding, and advanced rendering features like foveation and space warp. It also provides bidirectional file synchronization between the remote host and the device’s local storage, adapting its behaviour to the capabilities of the specific XR form factor.

The client is modular, with components designed to work across a range of devices:

- **Connection Manager** – handles discovery, authentication, and secure transport.
- **Rendering Engine** – built on OpenXR; decodes and displays the streamed visual output, integrates passthrough and spatial anchoring.
- **Input Processor** – translates native Android XR input events (gaze, hand gestures, voice) into TOS semantic events, with Gemini AI integration for natural language commands.
- **File Sync Service** – manages bidirectional file synchronization; adapts to device storage capabilities (full sync on headsets, on‑demand access on glasses).
- **Collaboration Module** – handles multi‑user presence and role‑based interactions.
- **Local UI Layer** – provides overlay elements (sync status, connection info) using Jetpack Compose for 2D panels.

All components communicate via internal APIs and share state through a central **Session Manager**, which also tracks the current device’s capabilities.

---

### 1. System Architecture (Client‑Side)


```

+---------------------------+       +---------------------+

|      Android XR Device           |    TOS Remote Server   |

|   (Headset / Glasses)            |      (Host Machine)    |

|  +-------------------------+     |  +-------------------+ |

|  |     Connection          | <-->|  |  Control Channel | |

|  |      Manager            |     |  |  (WebSocket/TLS) | |

|  +-------------------------+     |  +-------------------+ |

|            |                     |  +-------------------+ |

|  +-------------------------+     |  |   Video Stream    | |

|  |     Rendering Engine    | <-->|  |   (WebRTC/H.264)  | |

|  |     (OpenXR based)      |     |  +-------------------+ |

|  +-------------------------+     |  +-------------------+ |

|            |                     |  |   File Transfer   | |

|  +-------------------------+     |  |   (WebDAV/HTTPS)  | |

|  |     Input Processor     | <-->|  +-------------------+ |

|  | (Gaze/Pinch/Gemini)     |     |                        |

|  +-------------------------+     |                        |

|            |                     |                        |

|  +-------------------------+     |                        |

|  |    File Sync Service    | <-->|                        |

|  | (device‑aware)          |     |                        |

|  +-------------------------+     |                        |

|            |                     |                        |

|  +-------------------------+     |                        |

|  |   Collaboration Module  | <-->|                        |

|  +-------------------------+     |                        |

|            |                     |                        |

|  +-------------------------+     |                        |

|  |    Local UI Overlay     |     |                        |

|  | (Jetpack Compose panels)|     |                        |

|  +-------------------------+     |                        |
+---------------------------+       +------------------------+

```


#### Key Data Flows
- **Connection Manager** establishes a persistent control channel (WebSocket over TLS) for command/event exchange, and negotiates the video and file transfer channels.
- **Rendering Engine** receives a video stream (via WebRTC) and renders it into an OpenXR swapchain. It also manages the spatial positioning of the virtual screen using passthrough and environment understanding.
- **Input events** (gaze, pinch, hand gestures, voice) are captured by the Android XR runtime, translated by the **Input Processor** into TOS semantic events, and sent over the control channel. Voice commands are processed by the Gemini API to extract intent.
- **File synchronization** uses a separate channel (WebDAV over HTTPS). The **File Sync Service** monitors local and remote changes; its behaviour scales based on device storage (full background sync on headsets, on‑demand access on glasses).
- **Collaboration data** (user presence, cursors, role changes) flows over the control channel; the **Collaboration Module** renders remote user avatars/cursors and enforces role permissions.
- **Local UI Overlay** displays connection status, sync progress, and other native elements using Jetpack Compose, composited on top of the OpenXR scene.

---

### 2. Communication with TOS Remote Server

The client communicates with the TOS Remote Server using multiple parallel channels, identical in protocol to the Horizon OS design but with Android‑specific enhancements.

#### a) Control Channel
- **Protocol**: WebSocket over TLS (or a custom TCP‑based protocol).
- **Authentication**: SSH keys (stored in Android Keystore), password, or time‑limited invite tokens. Optionally, leverage Google Sign‑In for simplified authentication if the server supports OAuth2 delegation.
- **Message Format**: JSON‑RPC or MessagePack.
- **Messages include**:
  - Session establishment (handshake, authentication, device capability negotiation).
  - Input events (zoom, select, mode changes, text input).
  - Viewport commands (split, close, resize).
  - Collaboration events (user join/leave, cursor positions, role changes).
  - Heartbeats and keep‑alives.

#### b) Video/Audio Stream
- **Protocol**: WebRTC (for adaptive bitrate, NAT traversal) with hardware‑accelerated decoding via Android’s `MediaCodec` API.
- **Codec**: H.264 or H.265; the client requests a profile that matches the device’s hardware decoder capabilities.
- **Stream Content**: The TOS compositor outputs a dedicated video stream for the requested sector or viewport. Multiple streams can be requested for split viewports.
- **Audio**: Three‑layer audio (ambient, tactical, voice) is mixed on the host and streamed as a separate audio track, played back through the device’s spatial audio system.

#### c) File Transfer Channel
- **Protocol**: WebDAV over HTTPS, or a custom protocol over the same WebSocket (with binary frames). WebDAV provides a standard interface for file operations.
- **Authentication**: Reuses control channel credentials or a separate token.
- **Operations**: Directory listing, file download/upload, move, delete, metadata retrieval.

---

### 3. Display and Rendering Pipeline

The rendering engine is built on **OpenXR 1.1**, the foundation for all Android XR applications. It leverages platform extensions for optimal performance and integration.

- **OpenXR Initialization**: The client creates an `XrInstance` with the `XR_KHR_android_create_instance` extension, enabling seamless integration with the Android activity lifecycle. It uses a standard OpenXR session and creates swapchains for rendering.
- **Video Decoding to Texture**: The incoming video stream is decoded using `MediaCodec`, producing `Surface` buffers. These are converted to OpenXR-compatible textures (e.g., via `AHardwareBuffer` and `XrSwapchainImageAndroidBufferKHR`).
- **Scene Graph and Spatial Anchoring**: The client maintains a 3D scene with a virtual screen (a quad or curved surface) anchored to a real‑world location using OpenXR’s spatial anchor APIs (`XR_FB_spatial_entity` or similar). The user can reposition and resize the screen using hand gestures.
- **Multiple Viewports**: If the remote sector uses split viewports, the client can either:
  - Composite them on the host and receive a single video stream (simpler), or
  - Request separate streams and place each on a separate 3D surface (more flexible but bandwidth‑intensive).
- **Passthrough**: For augmented reality scenarios, the client enables passthrough via OpenXR’s composition layer. The virtual screen is blended with the real environment, and the system’s plane detection can be used to automatically place the screen on a real‑world surface.
- **Performance Optimizations**:
  - **Eye‑Tracked Foveation**: Using OpenXR extensions (`XR_FB_foveation`), the client reduces rendering resolution in the periphery, lowering GPU load and streaming bandwidth.
  - **Space Warp / Asynchronous TimeWarp**: The OpenXR runtime can generate interpolated frames, smoothing the experience even if the video stream drops frames.
  - **Adaptive Bitrate**: The WebRTC stream adjusts quality based on network conditions; the client may also request lower resolution when the user is not directly looking at the screen (gaze‑aware).

---

### 4. Input Handling and Mapping

Android XR defines a standard interaction model: **gaze + pinch** for targeting and selection, supplemented by hand gestures and voice. The Input Processor maps these to TOS semantic events.

#### Primary Input Mappings
| TOS Semantic Event | Android XR Input | Notes |
|--------------------|------------------|-------|
| `select` | **Air Tap** (thumb‑index pinch) while gazing at target | The standard selection gesture. Dwell time can also be used as an alternative. |
| `secondary_select` | **Two‑handed pinch** (pinch with both hands) or **voice command** (“Open context menu”) | Context menu invocation; may also be mapped to a long pinch. |
| `zoom_in` / `zoom_out` | **Two‑handed pinch and drag** (moving hands apart/together) | Scales the virtual screen or zooms the TOS interface. |
| `toggle_bezel_expanded` | **Gaze at bezel + air tap** or **voice** (“Show controls”) | Expands the Tactical Bezel for additional commands. |
| `drag` (move surfaces) | **Hand grab** (pinch and hold while moving hand) | Repositions the virtual screen in space. |
| `cycle_mode` | **Voice command** (“Switch to command mode”) or **gaze at mode toggle + air tap** | Natural language is heavily emphasised on Android XR. |
| `voice_command_start` | **“Hey Gemini”** + query | The system‑wide wake word; the client integrates with the Gemini API to process the query and map it to a command or search. |

#### Gemini AI Integration
- **Voice Processing**: Instead of sending raw audio, the client uses the on‑device Gemini API to transcribe and interpret user intent. The API can return structured data (e.g., “open file X” → `file_open` command) which the client then translates into the appropriate TOS semantic event.
- **Permission Handling**: The user must grant microphone and conversation data permissions. The client requests these at first use and respects the system’s privacy indicators.
- **Fallback**: If Gemini is unavailable or the user opts out, the client can fall back to local speech‑to‑text and a simpler command parser.

#### Gaze and Hand Tracking
- **Gaze**: The system provides eye‑tracking data via OpenXR (`XR_FB_eye_tracking_social`). The client uses gaze for targeting; a small reticle indicates the current focus.
- **Hand Tracking**: The OpenXR hand tracking extension (`XR_EXT_hand_tracking`) provides joint positions. The Input Processor detects pinches, grabs, and other gestures.

---

### 5. File Synchronization

The File Sync Service provides bidirectional synchronization between a user‑specified folder on the remote host and a corresponding folder on the device’s local storage. Its behaviour adapts to the device’s capabilities.

#### Device‑Aware Behaviour
- **Headsets (e.g., Project Moohan)**: Full background sync is enabled. The service uses Android’s `FileObserver` to monitor local changes and polls the remote server (or uses inotify events over the control channel) for remote changes. Conflicts are resolved according to user preference (keep both, remote wins, local wins). Large file transfers are chunked and can be paused/resumed.
- **Smart Glasses (limited storage)**: The service operates in **on‑demand mode**. Files are not automatically synced; instead, the user browses the remote file system directly (via the file transfer channel) and explicitly downloads files they need. Uploads are similarly manual. A lightweight cache may be maintained.

#### Architecture
- **Sync Engine**: A background service that maintains a database of sync pairs and file metadata. It schedules transfers using WorkManager to respect battery and network constraints.
- **Conflict Resolution UI**: When a conflict occurs, the Local UI Overlay displays a notification; the user can resolve it by choosing a version or merging.
- **Integration with TOS Directory Mode**: The client can enhance Directory Mode by adding “Sync this folder” buttons. Alternatively, a native file manager overlay (built with Jetpack Compose) shows both local and remote files with drag‑and‑drop support.

#### Storage Access
- The client uses the **Storage Access Framework** to request permission to read/write files in user‑selected directories. On Android XR, this ensures compliance with scoped storage.
- For headsets, the client may also have access to a dedicated app‑specific directory for automatic sync.

---

### 6. Collaboration Support

The client fully participates in TOS collaboration features as a guest, with Android XR‑specific enhancements.

- **User Presence**: On connection, the client registers as a guest with a specified role. The host broadcasts presence to all participants. The Collaboration Module renders remote user avatars (simple coloured orbs or custom models) positioned near their cursors.
- **Cursor Sharing**: The client sends its gaze‑targeted cursor position (projected onto the virtual screen) to the host. Remote cursors are rendered as coloured pointers with user initials.
- **Following Mode**: The client can request to follow another user’s view. The host sends viewport state (zoom level, splits, focused surface) and the client replicates it, optionally animating the transition.
- **Role Enforcement**: The host enforces permissions; the client disables UI elements not allowed for its role (e.g., a Viewer cannot type in the prompt). The Local UI Overlay may dim or hide restricted controls.
- **Auditory/Haptic Cues**: Collaboration events (user join, role change, hand raise) trigger audio cues via the three‑layer audio model and haptic feedback using the device’s vibration capabilities (OpenXR haptic feedback extensions or Android `Vibrator`).

---

### 7. Performance Considerations

- **Latency Management**:
  - Local echo for input (e.g., showing button presses immediately) while waiting for remote confirmation.
  - Predictive rendering: the client may use the last known state to interpolate cursor movements.
  - WebRTC’s adaptive bitrate and jitter buffers smooth out network variations.
- **Battery Life**:
  - Use hardware decoding via `MediaCodec`.
  - Throttle background sync (WorkManager with battery‑sensitive scheduling).
  - Reduce frame rate when the user is not interacting (gaze away from screen).
- **Memory**:
  - Limit the number of cached video frames.
  - Unload textures for off‑screen viewports.
  - Use Android’s `ActivityManager` to monitor memory pressure and release caches.

---

### 8. Security

- **Authentication**: Credentials stored in Android Keystore; SSH keys generated locally and never transmitted. Optionally support Google Sign‑In for simplified authentication if the server integrates with OAuth2.
- **Biometric Authentication**: For sensitive actions (e.g., enabling deep inspection), the client can prompt for biometric verification using Android’s BiometricPrompt.
- **Transport**: All channels encrypted with TLS; certificates verified against known host keys or CA. Certificate pinning can be enabled for additional security.
- **File Sync**: Files transferred over HTTPS; local storage is encrypted if the device supports file‑based encryption.
- **Permissions**: The client requests only necessary permissions (internet, storage, microphone for voice). All permissions are explained at runtime.
- **Gemini AI**: The user must explicitly grant permission for the client to use the Gemini API. The client respects the system’s privacy indicators (e.g., a dot on the screen when the microphone is active).

---

### 9. Integration with Android XR

- **System UI**: The client appears as a standard Android application in the app library. It can be launched in immersive mode, taking over the entire display, or as a floating panel (if the system supports 2D app windows in XR).
- **Passthrough**: Using OpenXR’s composition layer, the client can enable passthrough to blend the virtual screen with the real environment. The client may also use plane detection to snap the screen to real‑world surfaces.
- **Multitasking**: Android XR may support multiple concurrent OpenXR applications or 2D panels. The client should handle being paused/resumed gracefully and release resources when not in focus.
- **Notifications**: System notifications (e.g., low battery) are displayed by the Android XR shell. The client can also post its own notifications (e.g., sync completed) using `NotificationManager`.
- **Gemini Integration**: The client registers a custom voice command phrase (e.g., “TOS”) to be invoked directly from the system’s always‑listening assistant. When the user says “Hey Gemini, TOS …”, the intent is forwarded to the client.

---

### 10. Summary

The native Android XR client for TOS is a forward‑looking adaptation of the original Horizon OS design, embracing the open standards and advanced capabilities of the Android XR platform. By building on OpenXR, the client gains access to eye‑tracked foveation, space warp, and robust spatial anchoring, while the Gemini AI assistant enables a natural voice‑first interaction model. The architecture is device‑aware, scaling from powerful headsets to lightweight glasses, and maintains the core TOS philosophy of a spatial command environment. With its modular design and adherence to Android security and privacy best practices, the client delivers a seamless, secure, and deeply integrated remote computing experience across the Android XR ecosystem.
