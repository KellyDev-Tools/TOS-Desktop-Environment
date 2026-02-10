I'll create a clean, focused technical specification document that removes the project management and timeline details, focusing purely on the technical aspects.

```markdown
# TOS Desktop Environment: Technical Specifications v1.0

## Core Architecture

### System Layers

```
┌─────────────────────────────────────────┐
│           Application Layer              │
│  • Native TOS Apps (Rust/WASM)          │
│  • X11/Wayland Compatibility Layer      │
│  • Web Apps (WPE WebKit)                │
│  • Terminal Apps (Enhanced CLI)         │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│        Presentation Layer                │
│  • Spatial Compositor                   │
│  • Dual-Mode Output System              │
│  • Gesture Engine                       │
│  • Accessibility Services               │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│         Execution Layer                  │
│  • TOS Terminal Core                    │
│  • AI Command Engine                    │
│  • Real-time Collaboration              │
│  • Security Sandbox                     │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│        Infrastructure Layer              │
│  • JSON-RPC Bridge                      │
│  • Session Store                        │
│  • Portal System                        │
│  • Data Engine (Nushell/Oil)           │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│          Kernel Layer                    │
│  • Wayland Protocol                     │
│  • io_uring Async I/O                   │
│  • GPU Acceleration (Vulkan/Metal)      │
│  • Input System (libinput)              │
└─────────────────────────────────────────┘
```

### Component Interaction Flow

```
User Input → Gesture Parser → Spatial Controller → Command Router
      ↓            ↓                ↓                  ↓
Touch/Gesture → FSM Processing → Scene Graph → Terminal Execution
      ↓            ↓                ↓                  ↓
Keyboard/Mouse → Input Muxer → View Transform → Output Renderer
      ↓            ↓                ↓                  ↓
Accessibility → AT-SPI Bridge → UI State → Presentation Layer
```

---

## Component Specifications

### 1. Spatial Compositor ("Horizon")

**Language**: Rust (with unsafe optimizations for GPU operations)
**Dependencies**: Smithay, wlroots, Vulkan/Metal SDKs

#### Core Data Structures

```rust
// Spatial coordinate system (64-bit floating point)
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

// Scene graph node
struct SpatialNode {
    id: Uuid,
    position: Vector3<f64>,
    rotation: Quaternion<f64>,
    scale: Vector3<f64>,
    bounds: AABB<f64>,
    transform_matrix: Matrix4<f64>,
    children: Vec<SpatialNode>,
    content: ContentType,
    metadata: HashMap<String, JsonValue>,
    flags: NodeFlags,
    z_index: f64,
}

// Scene management
struct SceneGraph {
    root: SpatialNode,
    spatial_index: RTree<SpatialNode>,
    camera: CameraState,
    lights: Vec<LightSource>,
    global_transform: Matrix4<f64>,
    dirty_nodes: HashSet<Uuid>,
}

// Camera state for 3D navigation
struct CameraState {
    position: Vector3<f64>,
    target: Vector3<f64>,
    up: Vector3<f64>,
    fov: f64,
    near_clip: f64,
    far_clip: f64,
    projection: Matrix4<f64>,
    view: Matrix4<f64>,
}
```

#### GPU Acceleration Architecture

```
Vulkan Pipeline:
┌─────────────────────────────────────────────────┐
│          Command Buffer Submission               │
├─────────────────────────────────────────────────┤
│  Vertex Shader → Geometry Shader → Fragment     │
│      ↓                ↓                ↓        │
│  Transform       Primitive        Pixel         │
│  Processing      Assembly         Processing    │
└─────────────────────────────────────────────────┘
         ↓                ↓                ↓
┌─────────────────────────────────────────────────┐
│            Compute Shader Operations             │
│  • Frustum Culling (Parallel)                   │
│  • Spatial Sorting (Radix Sort)                 │
│  • Transform Updates (Matrix Multiplication)    │
└─────────────────────────────────────────────────┘
```

#### Performance Specifications

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Frame Rate | 144 FPS | Frame timing with Vulkan timestamp queries |
| Input Latency | <8ms | High-precision input-to-photon measurement |
| Scene Update | <2ms | GPU profiling with NVIDIA Nsight/AMD RGP |
| Memory Usage | <256MB base | Process memory monitoring |
| Object Count | 10,000+ | Stress testing with procedural generation |
| Display Support | 8K @ 120Hz | Multi-GPU synchronization |

#### Spatial Algorithms

1. **Frustum Culling** (Compute Shader)
   ```glsl
   // GPU-based culling shader
   layout(std430, binding = 0) buffer Objects {
       mat4 transforms[];
       uint visible[];
   };

   layout(std140, binding = 1) uniform Camera {
       mat4 view_projection;
       vec4 frustum_planes[6];
   };

   void main() {
       uint id = gl_GlobalInvocationID.x;
       mat4 mvp = view_projection * transforms[id];
       
       // Extract bounding sphere
       vec4 sphere = compute_bounding_sphere(transforms[id]);
       
       // Check against frustum planes
       bool visible = true;
       for (int i = 0; i < 6; i++) {
           float distance = dot(frustum_planes[i], vec4(sphere.xyz, 1.0));
           if (distance < -sphere.w) {
               visible = false;
               break;
           }
       }
       
       visible[id] = visible ? 1 : 0;
   }
   ```

2. **Spatial Indexing** (R-Tree)
   - Minimum bounding rectangle updates
   - Bulk loading optimizations
   - Nearest neighbor queries
   - Range search acceleration

3. **Transformation System**
   - Hierarchical transform propagation
   - GPU skinning for animations
   - Instanced rendering for duplicates
   - Level of detail management

---

### 2. TOS Terminal Core ("NovaShell")

**Language**: Rust with C FFI for GPU operations
**Protocol Support**: Sixel, Kitty Graphics, iTerm2

#### GPU Text Rendering Pipeline

```
Text Processing Pipeline:
┌─────────────────────────────────────────────────┐
│           Input/Decoding Stage                   │
│  • UTF-8/16/32 Decoding                         │
│  • Bidirectional Text Processing                 │
│  • Combining Character Handling                  │
│  • Emoji Segmentation                           │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Shaping & Layout Stage                 │
│  • HarfBuzz Text Shaping                        │
│  • Font Fallback Chain                          │
│  • Line Breaking (Unicode Line Breaking Alg)    │
│  • Justification & Kerning                      │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           GPU Rendering Stage                    │
│  • Signed Distance Field Generation             │
│  • Subpixel Antialiasing (RGB)                  │
│  • Gamma Correction                             │
│  • Color Font Rendering (COLR/CPAL/SVG)         │
└─────────────────────────────────────────────────┘
```

#### Signed Distance Field Implementation

```rust
struct GlyphAtlas {
    texture: wgpu::Texture,
    glyph_data: HashMap<GlyphKey, GlyphInfo>,
    sdf_size: u32,
    padding: u32,
}

struct GlyphInfo {
    uv_rect: Rect,
    bearing: Vector2<f32>,
    advance: f32,
    sdf_data: Vec<u8>,
}

impl GlyphAtlas {
    fn generate_sdf(&mut self, glyph: &GlyphOutline, size: u32) -> Vec<u8> {
        let scale = size as f32 / self.sdf_size as f32;
        let mut sdf = vec![0u8; (size * size) as usize];
        
        // Parallel SDF generation using compute shader
        for y in 0..size {
            for x in 0..size {
                let pos = Vector2::new(x as f32, y as f32) * scale;
                let distance = glyph.signed_distance(pos);
                
                // Convert to 8-bit texture value
                let normalized = (distance * 64.0 / self.sdf_size as f32) + 128.0;
                sdf[(y * size + x) as usize] = normalized.clamp(0.0, 255.0) as u8;
            }
        }
        sdf
    }
}
```

#### Terminal Features Matrix

| Feature | Implementation | Performance Target |
|---------|---------------|-------------------|
| GPU Text Rendering | Vulkan Compute Shaders | 1M glyphs @ 144 FPS |
| Inline Media | VA-API/VDPAU/NVDEC | 4K60 H.264 decode |
| Sixel Graphics | GPU rasterization | 1920x1080 @ 60 FPS |
| Terminal Multiplexing | Zellij integration | 100+ concurrent panes |
| Scrollback Buffer | Ring buffer + compression | 1M lines, <100ms search |
| Input Method Editor | IBus/FCITX integration | <5ms key processing |

#### AI Command Engine Architecture

```
AI Processing Pipeline:
┌─────────────────────────────────────────────────┐
│           Context Collection                     │
│  • Current Directory                            │
│  • Git Status & Branch                          │
│  • Environment Variables                        │
│  • Recent Commands                              │
│  • Open Files & Processes                       │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Feature Extraction                     │
│  • Command Pattern Recognition                  │
│  • Parameter Type Inference                     │
│  • Error Pattern Detection                      │
│  • Workflow Context Analysis                    │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Model Inference                        │
│  • Local Transformer Model (ONNX Runtime)       │
│  • Command Completion Suggestions               │
│  • Error Correction Proposals                   │
│  • Natural Language → Command                   │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Result Presentation                    │
│  • Ranked Suggestions                           │
│  • Confidence Scores                            │
│  • Explanation & Documentation                  │
│  • Interactive Refinement                       │
└─────────────────────────────────────────────────┘
```

#### AI Model Specifications

```yaml
Command Prediction Model:
  Architecture: Transformer (6 layers, 8 heads)
  Vocabulary: 50,000 tokens (BPE encoding)
  Parameters: 125 million
  Training Data:
    - 10M shell command examples
    - GitHub repositories (filtered)
    - Stack Overflow Q&A pairs
    - Man page documentation
  Inference Engine: ONNX Runtime + DirectML
  Performance: <10ms inference time
  Memory: <500MB RAM usage

Error Correction Model:
  Type: Sequence-to-sequence LSTM
  Input: Erroneous command + error message
  Output: Corrected command + explanation
  Accuracy: 95% on common errors
```

---

### 3. Dual-Mode Output System

#### Output Modes Specification

| Mode | Position | Persistence | Accessibility | Use Case |
|------|----------|-------------|---------------|----------|
| **Docked Frame** | Bottom (configurable) | Session-wide | Full AT-SPI | Daily driving |
| **Cinematic Scroll** | Camera-anchored | Ephemeral | Mirror only | Demos, logs |
| **Anchored Output** | Object-bound | Parent lifetime | Inherited | Contextual data |

#### Docked Frame Technical Details

```rust
struct DockedFrame {
    id: Uuid,
    position: FramePosition,  // Top/Bottom/Left/Right/Floating
    size: FrameSize,          // Percentage or pixels
    content: OutputContent,
    scrollback: RingBuffer<OutputLine>,
    search_index: TantivyIndex,
    styling: FrameStyle,
    accessibility: AccessibilityState,
}

enum FramePosition {
    Top(f32),      // Percentage from top
    Bottom(f32),   // Percentage from bottom
    Left(f32),     // Percentage from left
    Right(f32),    // Percentage from right
    Floating {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    },
}

struct OutputLine {
    text: String,
    style: LineStyle,
    timestamp: DateTime<Utc>,
    source: CommandId,
    metadata: HashMap<String, JsonValue>,
}
```

#### Cinematic Scroll Renderer

```glsl
// Vertex shader for cinematic text
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoord;
layout(location = 2) in vec4 color;

layout(set = 0, binding = 0) uniform Camera {
    mat4 view_projection;
    vec3 camera_position;
    float time;
};

layout(set = 0, binding = 1) uniform TextParams {
    float scroll_speed;
    float font_size;
    float perspective_factor;
    float depth;
};

layout(location = 0) out vec2 v_texcoord;
layout(location = 1) out vec4 v_color;
layout(location = 2) out float v_depth;

void main() {
    // Apply perspective scrolling
    vec3 pos = position;
    pos.z += scroll_speed * time;
    
    // Depth-based scaling
    float depth_scale = 1.0 - (pos.z * perspective_factor);
    pos.xy *= depth_scale;
    
    // Add parallax based on camera
    vec3 view_offset = camera_position * 0.1;
    pos += view_offset;
    
    gl_Position = view_projection * vec4(pos, 1.0);
    v_texcoord = texcoord;
    v_color = color;
    v_depth = pos.z;
}
```

#### Output Rendering Pipeline

```
Pipeline Stages:
1. Command Execution
   ↓
2. Output Capture (PTY)
   ↓
3. Chunk Processing
   ├── ANSI Escape Parsing
   ├── Sixel/Kitty Graphics Decode
   ├── Hyperlink Detection
   └── Structured Data Extraction
   ↓
4. Format Detection
   ├── JSON/XML/YAML
   ├── Log Formats (syslog, Apache)
   ├── CSV/TSV Tables
   └── Binary Data Analysis
   ↓
5. Renderer Selection
   ├── Docked Frame Renderer
   ├── Cinematic Scroll Renderer
   ├── Anchored Output Renderer
   └── Accessibility Mirror
   ↓
6. GPU Composition
   ├── Text Rendering
   ├── Image Composition
   ├── Effects Application
   └── Final Composition
```

---

### 4. Gesture & Input System

#### Gesture Recognition FSM

```rust
enum GestureState {
    Idle,
    TouchDown(TouchPoint),
    Panning { start: TouchPoint, current: TouchPoint },
    Pinching { point1: TouchPoint, point2: TouchPoint },
    Rotating { center: Vector2, angle: f32 },
    Swiping { direction: Direction, velocity: f32 },
}

struct TouchPoint {
    id: u32,
    position: Vector2<f32>,
    timestamp: Instant,
    pressure: f32,
    size: f32,
}

impl GestureRecognizer {
    fn process_frame(&mut self, touches: &[TouchPoint]) -> Vec<GestureEvent> {
        match self.state {
            GestureState::Idle => self.detect_gesture_start(touches),
            GestureState::TouchDown(start) => self.track_gesture(touches, start),
            // ... state transition logic
        }
    }
    
    fn detect_gesture_start(&mut self, touches: &[TouchPoint]) -> Vec<GestureEvent> {
        match touches.len() {
            1 => self.begin_single_touch(&touches[0]),
            2 => self.begin_multi_touch(&touches[0], &touches[1]),
            3 => self.begin_three_finger(&touches),
            4 => self.begin_four_finger(&touches),
            5 => self.begin_five_finger(&touches),
            _ => vec![],
        }
    }
}
```

#### Gesture Dictionary with Parameters

| Gesture | Finger Count | Duration | Recognition Threshold | Action |
|---------|--------------|----------|----------------------|--------|
| **Tap** | 1 | <200ms | Move <5px | Selection/Activation |
| **Long Press** | 1 | ≥500ms | Move <10px | Orbital Context Menu |
| **Drag** | 1 | Any | Move >15px | Object Movement |
| **Pinch** | 2 | Continuous | Scale change >10% | Zoom In/Out |
| **Rotation** | 2 | Continuous | Angle change >15° | Object Rotation |
| **Three-finger Swipe** | 3 | 150-500ms | Velocity >500px/s | Workspace Switch |
| **Four-finger Spread** | 4 | <300ms | All fingers move outward | Show All Windows |
| **Five-finger Grab** | 5 | Hold | All fingers converge | Command Palette |

#### Predictive Input Algorithm

```rust
struct PredictiveInput {
    history: VecDeque<InputEvent>,
    patterns: HashMap<InputPattern, f32>,
    context: WorkflowContext,
    model: ONNXSession,  // Lightweight ML model
}

impl PredictiveInput {
    fn predict_next(&self, current: &InputEvent) -> Vec<Prediction> {
        // Extract features from input history
        let features = self.extract_features();
        
        // Run ML inference
        let predictions = self.model.run(features);
        
        // Apply context filtering
        let filtered = self.filter_by_context(predictions);
        
        // Rank by probability and recency
        self.rank_predictions(filtered)
    }
    
    fn extract_features(&self) -> Vec<f32> {
        vec![
            // Temporal features
            self.time_since_last_input(),
            self.input_frequency(),
            
            // Spatial features
            self.movement_pattern(),
            self.velocity_profile(),
            
            // Contextual features
            self.active_application(),
            self.current_workspace(),
            self.recent_commands(),
            
            // User-specific features
            self.user_habit_score(),
            self.error_rate(),
        ]
    }
}
```

#### Haptic Feedback System

```
Haptic Feedback Pipeline:
┌─────────────────────────────────────────────────┐
│           Event Detection                        │
│  • Gesture Completion                           │
│  • Boundary Crossing                            │
│  • Object Selection                             │
│  • Error Conditions                             │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Feedback Selection                     │
│  • Texture-based (surface feel)                 │
│  • Impact-based (clicks, detents)               │
│  • Continuous (vibration, resistance)           │
│  • Audio-tactile synchronization                │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Waveform Generation                    │
│  • Amplitude Modulation                         │
│  • Frequency Sweeping                           │
│  • Envelope Shaping                             │
│  • Multi-motor Coordination                     │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Device Output                          │
│  • Linux Input FF API                           │
│  • Immersion TouchSense                         │
│  • Apple Taptic Engine (compatibility)          │
│  • Custom USB Haptic Devices                    │
└─────────────────────────────────────────────────┘
```

---

### 5. JSON-RPC Bridge ("QuantumLink")

#### Protocol Specification

```json
{
  "jsonrpc": "2.0",
  "id": "uuid-v4",
  "method": "namespace.method",
  "params": {
    "type": "object|array",
    "properties": {},
    "additionalProperties": false
  },
  "metadata": {
    "compression": "zstd|lz4|none",
    "priority": "realtime|high|normal|low",
    "timeout": 5000,
    "retry_policy": "exponential_backoff"
  }
}
```

#### Zero-Copy Buffer Management

```rust
struct SharedBuffer {
    memfd: i32,                     // memfd_create file descriptor
    size: usize,
    map_ptr: *mut u8,               // mmap pointer
    refcount: Arc<AtomicUsize>,
}

impl SharedBuffer {
    fn create(size: usize) -> Result<Self> {
        let fd = memfd_create("rpc_buffer", MFD_CLOEXEC)?;
        ftruncate(fd, size as i64)?;
        
        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };
        
        Ok(SharedBuffer {
            memfd: fd,
            size,
            map_ptr: ptr as *mut u8,
            refcount: Arc::new(AtomicUsize::new(1)),
        })
    }
    
    fn send_over_socket(&self, socket: &UnixStream) -> Result<()> {
        // Send file descriptor via SCM_RIGHTS
        let fds = [self.memfd];
        let cmsg = libc::cmsghdr::new_with_fds(&fds);
        socket.send_with_fd(&[], &cmsg)?;
        Ok(())
    }
}
```

#### API Method Categories

**Spatial Control API**:
```json
{
  "spatial.createNode": {
    "params": {
      "type": {"enum": ["terminal", "application", "webview", "cluster"]},
      "position": {"$ref": "#/definitions/Vector3"},
      "content": {"type": "object"}
    },
    "returns": {"$ref": "#/definitions/NodeId"}
  },
  "spatial.transformNodes": {
    "params": {
      "nodes": {"type": "array", "items": {"$ref": "#/definitions/NodeId"}},
      "transform": {"$ref": "#/definitions/Matrix4"}
    }
  }
}
```

**Terminal Execution API**:
```json
{
  "terminal.execute": {
    "params": {
      "command": {"type": "string"},
      "cwd": {"type": "string"},
      "env": {"type": "object"},
      "outputMode": {"enum": ["docked", "cinematic", "anchored"]}
    },
    "returns": {"$ref": "#/definitions/ExecutionResult"}
  },
  "terminal.stream": {
    "params": {
      "command": {"type": "string"},
      "streamId": {"type": "string"}
    },
    "streaming": true
  }
}
```

**Output Management API**:
```json
{
  "output.create": {
    "params": {
      "type": {"enum": ["docked", "cinematic", "anchored"]},
      "title": {"type": "string"},
      "persistent": {"type": "boolean"},
      "mirrorToDocked": {"type": "boolean"}
    },
    "returns": {"$ref": "#/definitions/OutputId"}
  },
  "output.append": {
    "params": {
      "outputId": {"$ref": "#/definitions/OutputId"},
      "content": {"type": "string"},
      "style": {"$ref": "#/definitions/OutputStyle"}
    }
  }
}
```

#### Performance Specifications

| Operation | Target Latency | Throughput | Memory Usage |
|-----------|----------------|------------|--------------|
| Method Call | <100μs | 100,000/sec | 64 bytes/call |
| Buffer Transfer | <1ms/GB | 10 GB/sec | Zero-copy |
| Streaming | <10ms end-to-end | 1 Gb/sec | 4MB buffer |
| Error Handling | <50μs | N/A | Stack-only |

---

### 6. Session Store ("Chronos")

#### Database Schema

```sql
-- Core session tables
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    created_at TIMESTAMP,
    last_modified TIMESTAMP,
    user_id TEXT,
    machine_id TEXT,
    checkpoint_count INTEGER
);

CREATE TABLE session_checkpoints (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id),
    sequence INTEGER,
    timestamp TIMESTAMP,
    data BLOB,  -- Compressed scene graph
    metadata JSON
);

-- Terminal state
CREATE TABLE terminal_sessions (
    id TEXT PRIMARY KEY,
    checkpoint_id TEXT REFERENCES session_checkpoints(id),
    pid INTEGER,
    cwd TEXT,
    command TEXT,
    env JSON,
    history BLOB,  -- Compressed command history
    output_buffer BLOB
);

-- Spatial state
CREATE TABLE spatial_nodes (
    id TEXT PRIMARY KEY,
    checkpoint_id TEXT REFERENCES session_checkpoints(id),
    type TEXT,
    position JSON,  -- {x, y, z}
    rotation JSON,  -- {x, y, z, w}
    scale JSON,     -- {x, y, z}
    content JSON,
    parent_id TEXT REFERENCES spatial_nodes(id)
);

-- Indexes for fast lookup
CREATE INDEX idx_checkpoints_session ON session_checkpoints(session_id);
CREATE INDEX idx_checkpoints_sequence ON session_checkpoints(sequence);
CREATE INDEX idx_nodes_checkpoint ON spatial_nodes(checkpoint_id);
CREATE INDEX idx_nodes_parent ON spatial_nodes(parent_id);
```

#### Checkpoint Compression Algorithm

```rust
struct CheckpointCompressor {
    algorithm: CompressionAlgo,
    dictionary: Vec<u8>,  // Zstd trained dictionary
}

impl CheckpointCompressor {
    fn compress_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Vec<u8>> {
        // Serialize to MessagePack
        let serialized = rmp_serde::to_vec(checkpoint)?;
        
        // Apply compression
        match self.algorithm {
            CompressionAlgo::Zstd => {
                zstd::stream::encode_all(&serialized[..], 3)?
            }
            CompressionAlgo::Lz4 => {
                lz4_flex::compress(&serialized)
            }
            CompressionAlgo::Brotli => {
                brotli::compress(&serialized, &brotli::enc::BrotliEncoderParams::default())
            }
        }
    }
    
    fn train_dictionary(&mut self, samples: &[Checkpoint]) -> Result<()> {
        // Extract common patterns from sample checkpoints
        let serialized_samples: Vec<Vec<u8>> = samples
            .iter()
            .map(|c| rmp_serde::to_vec(c))
            .collect::<Result<_, _>>()?;
        
        // Train Zstd dictionary
        self.dictionary = zstd::dict::from_samples(
            &serialized_samples,
            MAX_DICTIONARY_SIZE,
        )?;
        
        Ok(())
    }
}
```

#### Recovery System

```rust
struct SessionRecovery {
    db: Connection,
    wal: WriteAheadLog,
    recovery_state: RecoveryState,
}

impl SessionRecovery {
    fn restore_session(&self, session_id: &str) -> Result<RestoredSession> {
        // Find latest consistent checkpoint
        let checkpoint = self.find_latest_consistent_checkpoint(session_id)?;
        
        // Apply WAL entries after checkpoint
        let wal_entries = self.wal.entries_after(checkpoint.sequence)?;
        
        // Reconstruct final state
        let mut state = self.deserialize_checkpoint(&checkpoint.data)?;
        
        for entry in wal_entries {
            self.apply_wal_entry(&mut state, entry)?;
        }
        
        Ok(RestoredSession {
            state,
            checkpoint_id: checkpoint.id,
            recovered_entries: wal_entries.len(),
        })
    }
    
    fn find_latest_consistent_checkpoint(&self, session_id: &str) -> Result<Checkpoint> {
        // Query checkpoints in reverse order
        let mut stmt = self.db.prepare(
            "SELECT * FROM session_checkpoints 
             WHERE session_id = ? 
             AND integrity_check = 1
             ORDER BY sequence DESC LIMIT 1"
        )?;
        
        stmt.query_row(params![session_id], |row| {
            Ok(Checkpoint {
                id: row.get(0)?,
                sequence: row.get(2)?,
                data: row.get(4)?,
            })
        })
    }
}
```

#### Performance Targets

| Operation | Target Time | Success Rate | Data Integrity |
|-----------|-------------|--------------|----------------|
| Checkpoint Creation | <100ms | 99.9% | CRC32 + SHA256 |
| Session Restore | <500ms | 99.5% | Atomic recovery |
| Incremental Save | <50ms | 99.99% | WAL journaling |
| Corruption Recovery | <2s | 95% | Multi-version |

---

### 7. Portal System ("Gateway")

#### Security Architecture

```
Security Layers:
┌─────────────────────────────────────────────────┐
│           Application Sandbox                    │
│  • Namespace Isolation                          │
│  • Cgroup Resource Limits                       │
│  • Seccomp BPF Filters                          │
│  • Capability Dropping                          │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Permission System                      │
│  • Fine-grained ACLs                            │
│  • Runtime Permission Prompts                   │
│  • Temporary Grant Tokens                       │
│  • Audit Trail Logging                          │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Portal Mediation                       │
│  • File Picker (GTK/Qt)                         │
│  • Print Dialog                                 │
│  • Device Access Control                        │
│  • Network Request Filtering                    │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           System Integration                     │
│  • polkit Integration                           │
│  • systemd Service Isolation                    │
│  • SELinux/AppArmor Policies                    │
│  • Firewalld Rules                              │
└─────────────────────────────────────────────────┘
```

#### Portal Implementation

```rust
struct PortalManager {
    dbus_connection: Connection,
    portals: HashMap<PortalType, Box<dyn Portal>>,
    permission_store: PermissionDatabase,
    audit_logger: AuditLogger,
}

impl PortalManager {
    async fn handle_request(&self, request: PortalRequest) -> PortalResponse {
        // Check permissions
        if !self.check_permission(&request) {
            return PortalResponse::PermissionDenied;
        }
        
        // Log the request
        self.audit_logger.log_request(&request);
        
        // Route to appropriate portal
        match request.portal_type {
            PortalType::FilePicker => {
                self.handle_file_picker(request).await
            }
            PortalType::Print => {
                self.handle_print(request).await
            }
            PortalType::DeviceAccess => {
                self.handle_device_access(request).await
            }
            PortalType::Share => {
                self.handle_share(request).await
            }
        }
    }
    
    async fn handle_file_picker(&self, request: PortalRequest) -> PortalResponse {
        // Show file picker dialog
        let dialog = FileDialog::new()
            .set_title(&request.title)
            .set_filters(&request.filters)
            .set_multiple(request.multiple);
        
        // Run in separate process for isolation
        let result = dialog.spawn_detached().await;
        
        match result {
            Ok(files) => PortalResponse::FilesSelected(files),
            Err(e) => PortalResponse::Error(e.to_string()),
        }
    }
}
```

#### Permission Database Schema

```sql
CREATE TABLE permissions (
    id INTEGER PRIMARY KEY,
    application TEXT,
    permission TEXT,
    granted BOOLEAN,
    grant_time TIMESTAMP,
    expires TIMESTAMP,
    grant_token TEXT UNIQUE,
    CONSTRAINT unique_app_permission UNIQUE (application, permission)
);

CREATE TABLE permission_policies (
    id INTEGER PRIMARY KEY,
    permission TEXT,
    default_grant BOOLEAN,
    prompt_required BOOLEAN,
    max_duration INTEGER,  -- seconds
    auto_revoke BOOLEAN
);

CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP,
    application TEXT,
    permission TEXT,
    action TEXT,
    granted BOOLEAN,
    user TEXT,
    peer_pid INTEGER,
    details JSON
);
```

#### Sandboxing Implementation

```rust
struct ApplicationSandbox {
    namespaces: LinuxNamespaces,
    cgroups: CgroupController,
    seccomp: SeccompFilter,
    capabilities: CapabilitySet,
}

impl ApplicationSandbox {
    fn create(&self) -> Result<Child> {
        let mut command = Command::new("/usr/bin/bwrap");  // bubblewrap
        
        // Set up namespaces
        command.arg("--unshare-all");
        command.arg("--share-net");  // with restrictions
        
        // Mount points
        command.arg("--bind");
        command.arg("/usr");
        command.arg("/usr");
        
        command.arg("--ro-bind");
        command.arg("/etc");
        command.arg("/etc");
        
        // Temporary directories
        command.arg("--tmpfs");
        command.arg("/tmp");
        
        // Resource limits via cgroups
        self.cgroups.apply_limits(&mut command);
        
        // Seccomp filter
        let filter = self.seccomp.compile()?;
        command.arg("--seccomp");
        command.arg(filter.as_raw_fd());
        
        // Drop capabilities
        for cap in self.capabilities.to_drop() {
            command.arg("--cap-drop");
            command.arg(cap);
        }
        
        Ok(command.spawn()?)
    }
}
```

---

### 8. Accessibility Engine ("Aether")

#### AT-SPI Integration Architecture

```
AT-SPI Data Flow:
┌─────────────────────────────────────────────────┐
│           TOS Applications                       │
│  • Expose Accessibility Properties              │
│  • Generate Accessibility Events                │
│  • Handle Accessibility Actions                 │
└─────────────────────────────────────────────────┘
                        ↓ (DBus)
┌─────────────────────────────────────────────────┐
│           ATK Bridge Layer                       │
│  • AT-SPI Registry                              │
│  • Object Hierarchy Management                  │
│  • Event Routing                                │
│  • Property Cache                               │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Assistive Technologies                 │
│  • Screen Readers (Orca, NVDA)                  │
│  • Screen Magnifiers                            │
│  • Braille Displays                             │
│  • Voice Control Systems                        │
└─────────────────────────────────────────────────┘
```

#### Spatial Accessibility Mapping

```rust
struct AccessibilityMapper {
    scene_graph: Arc<SceneGraph>,
    atk_registry: AtspiRegistry,
    mapping_cache: HashMap<NodeId, AccessibilityNode>,
}

impl AccessibilityMapper {
    fn map_spatial_to_accessibility(&mut self, node: &SpatialNode) -> AccessibilityNode {
        AccessibilityNode {
            role: self.determine_role(node),
            name: self.extract_name(node),
            description: self.extract_description(node),
            state: self.determine_state(node),
            relations: self.find_relations(node),
            bounds: self.calculate_screen_bounds(node),
            actions: self.available_actions(node),
        }
    }
    
    fn calculate_screen_bounds(&self, node: &SpatialNode) -> Rect {
        // Transform 3D bounds to 2D screen space
        let world_bounds = node.world_bounds();
        let screen_pos = self.scene_graph.camera.project(world_bounds.center());
        
        // Calculate approximate 2D bounds
        Rect {
            x: screen_pos.x as i32,
            y: screen_pos.y as i32,
            width: (world_bounds.size().x * SCALE_FACTOR) as i32,
            height: (world_bounds.size().y * SCALE_FACTOR) as i32,
        }
    }
}
```

#### Screen Reader Optimization

```rust
struct ScreenReaderOptimizer {
    tts_engine: TextToSpeech,
    braille_driver: Option<BrailleDriver>,
    navigation_state: NavigationState,
    announcement_queue: PriorityQueue<Announcement>,
}

impl ScreenReaderOptimizer {
    fn announce_spatial_change(&mut self, change: SpatialChange) {
        let announcement = match change {
            SpatialChange::NodeSelected(node) => {
                format!("Selected {}", self.describe_node(node))
            }
            SpatialChange::WorkspaceChanged(ws) => {
                format!("Entered workspace {}", ws.name)
            }
            SpatialChange::CameraMoved(direction) => {
                format!("View moved {}", direction)
            }
            SpatialChange::ObjectTransformed(node, transform) => {
                format!("{} {}", self.describe_transform(transform), self.describe_node(node))
            }
        };
        
        self.announcement_queue.push(Announcement {
            text: announcement,
            priority: change.priority(),
            interruptible: change.is_interruptible(),
        });
        
        self.process_announcement_queue();
    }
    
    fn process_announcement_queue(&mut self) {
        while let Some(announcement) = self.announcement_queue.pop() {
            if self.tts_engine.is_speaking() && !announcement.interruptible {
                // Re-queue non-interruptible announcements
                self.announcement_queue.push(announcement);
                break;
            }
            
            self.tts_engine.speak(&announcement.text);
            
            // Update braille display if available
            if let Some(braille) = &mut self.braille_driver {
                braille.display(&announcement.text);
            }
        }
    }
}
```

#### Keyboard Navigation System

```rust
struct KeyboardNavigator {
    focus_tree: FocusTree,
    navigation_modes: HashMap<NavigationMode, NavigationRules>,
    shortcut_registry: ShortcutRegistry,
}

impl KeyboardNavigator {
    fn navigate(&mut self, direction: NavigationDirection) -> Option<FocusTarget> {
        let current = self.focus_tree.current_focus()?;
        
        match direction {
            NavigationDirection::Up => {
                self.find_above(current)
            }
            NavigationDirection::Down => {
                self.find_below(current)
            }
            NavigationDirection::Left => {
                self.find_left(current)
            }
            NavigationDirection::Right => {
                self.find_right(current)
            }
            NavigationDirection::In => {
                self.drill_down(current)
            }
            NavigationDirection::Out => {
                self.drill_up(current)
            }
            NavigationDirection::Next => {
                self.next_in_order(current)
            }
            NavigationDirection::Previous => {
                self.previous_in_order(current)
            }
        }
    }
    
    fn find_above(&self, current: &FocusTarget) -> Option<FocusTarget> {
        // Convert to screen coordinates
        let screen_pos = current.screen_position();
        
        // Find nearest focusable above current position
        self.focus_tree
            .focusable_nodes()
            .filter(|node| node.screen_position().y < screen_pos.y)
            .min_by_key(|node| {
                let dy = screen_pos.y - node.screen_position().y;
                let dx = (screen_pos.x - node.screen_position().x).abs();
                (dy, dx)  // Prefer directly above, then closest horizontally
            })
    }
}
```

#### Accessibility Testing Protocol

```yaml
Accessibility Test Suite:
  Screen Reader Tests:
    - Role announcement correctness
    - Name/description accuracy
    - State change notifications
    - Focus traversal order
  
  Keyboard Navigation Tests:
    - Tab order consistency
    - Arrow key navigation
    - Shortcut key functionality
    - Focus indicator visibility
  
  Visual Accessibility Tests:
    - Color contrast ratios (WCAG AAA)
    - Text scaling (up to 400%)
    - High contrast mode rendering
    - Reduced motion compliance
  
  Input Accommodation Tests:
    - Sticky keys functionality
    - Slow keys timing
    - Mouse keys precision
    - Switch access compatibility
  
  Performance Requirements:
    - Screen reader response: <100ms
    - Focus change: <50ms
    - Announcement queue: <10 items
    - Braille update: <16ms
```

---

### 9. Localization Framework ("Babel")

#### Text Layout Pipeline

```
International Text Processing:
┌─────────────────────────────────────────────────┐
│           Input Normalization                    │
│  • UTF-8 Validation                            │
│  • NFC/NFD Normalization                       │
│  • Bidi Control Character Handling             │
│  • Line Break Class Analysis                   │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Script Segmentation                    │
│  • Unicode Script Detection                     │
│  • Language-Specific Rules                     │
│  • Emoji/ZWJ Sequence Handling                 │
│  • Font Fallback Chain Selection               │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Text Shaping                           │
│  • HarfBuzz Shaping Engine                     │
│  • OpenType Feature Application                │
│  • Kerning & Ligature Formation                │
│  • Variable Font Axis Selection                │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           Line Layout                            │
│  • Unicode Line Breaking Algorithm             │
│  • Hyphenation (libhyphen)                     │
│  • Justification & Text Alignment              │
│  • Vertical Text Layout Support                │
└─────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────┐
│           GPU Rendering                          │
│  • Font Atlas Management                       │
│  • SDF Generation for Complex Scripts          │
│  • Subpixel Positioning                        │
│  • Color Font Rendering (COLR/SVG)             │
└─────────────────────────────────────────────────┘
```

#### RTL (Right-to-Left) Support

```rust
struct RTLProcessor {
    bidi_algorithm: BidiAlgorithm,
    mirroring_map: HashMap<char, char>,
    shaping_engine: HarfBuzzBuffer,
}

impl RTLProcessor {
    fn process_text(&self, text: &str, base_direction: Direction) -> ShapedText {
        // Perform bidirectional analysis
        let bidi_info = self.bidi_algorithm.analyze(text, Some(base_direction));
        
        // Reorder runs for display
        let reordered = bidi_info.reorder_line(0..text.len());
        
        // Mirror brackets and other symmetric characters
        let mirrored = self.mirror_brackets(&reordered);
        
        // Shape each run separately
        let mut shaped_runs = Vec::new();
        for run in mirrored.runs {
            let direction = run.direction;
            let text_slice = &text[run.range];
            
            // Configure HarfBuzz for direction
            self.shaping_engine.set_direction(direction.into());
            self.shaping_engine.add_str(text_slice);
            
            let glyphs = self.shaping_engine.shape();
            shaped_runs.push(ShapedRun {
                glyphs,
                direction,
                range: run.range,
            });
        }
        
        ShapedText {
            runs: shaped_runs,
            bidi_info,
            base_direction,
        }
    }
    
    fn mirror_brackets(&self, text: &str) -> String {
        text.chars()
            .map(|c| self.mirroring_map.get(&c).copied().unwrap_or(c))
            .collect()
    }
}
```

#### Locale Management System

```rust
struct LocaleManager {
    current_locale: Locale,
    fallback_chain: Vec<Locale>,
    translation_cache: LruCache<String, String>,
    plural_rules: PluralRules,
    number_formatter: NumberFormatter,
    date_formatter: DateFormatter,
}

impl LocaleManager {
    fn translate(&mut self, key: &str, context: Option<&str>) -> String {
        let cache_key = format!("{}:{}", context.unwrap_or(""), key);
        
        // Check cache first
        if let Some(cached) = self.translation_cache.get(&cache_key) {
            return cached.clone();
        }
        
        // Load translation
        let translation = self.load_translation(key, context);
        
        // Cache and return
        self.translation_cache.put(cache_key.clone(), translation.clone());
        translation
    }
    
    fn format_number(&self, number: f64, style: NumberStyle) -> String {
        self.number_formatter.format(number, style)
    }
    
    fn format_date(&self, datetime: DateTime<Utc>, style: DateStyle) -> String {
        self.date_formatter.format(datetime, style)
    }
}

struct Locale {
    language: LanguageTag,
    region: Option<RegionTag>,
    script: Option<ScriptTag>,
    variants: Vec<VariantTag>,
    extensions: HashMap<ExtensionKey, ExtensionValue>,
    
    // Localization data
    messages: HashMap<String, String>,
    plural_rules: PluralCategoryRules,
    number_symbols: NumberSymbols,
    date_patterns: DatePatterns,
    collation_rules: CollationRules,
}
```

#### Font Management System

```rust
struct FontManager {
    system_fonts: FontSet,
    embedded_fonts: FontSet,
    fontconfig: FontConfig,
    fallback_chains: HashMap<Script, Vec<FontFamily>>,
    font_atlas: GlyphAtlas,
}

impl FontManager {
    fn select_font_for_text(&self, text: &str, script: Script) -> FontSelection {
        // Get preferred font for script
        let preferred = self.fallback_chains.get(&script)
            .and_then(|chain| chain.first())
            .cloned();
        
        // Check coverage
        if let Some(font) = preferred {
            if self.check_coverage(&font, text) {
                return FontSelection::Specific(font);
            }
        }
        
        // Fallback chain
        let fallback_chain = self.fallback_chains.get(&script)
            .unwrap_or(&DEFAULT_FALLBACK);
        
        for font_family in fallback_chain {
            if self.check_coverage(font_family, text) {
                return FontSelection::Specific(font_family.clone());
            }
        }
        
        // Last resort: system default
        FontSelection::SystemDefault
    }
    
    fn check_coverage(&self, font_family: &FontFamily, text: &str) -> bool {
        // Check if font supports all characters in text
        let font = self.load_font(font_family);
        text.chars().all(|c| font.has_glyph(c))
    }
}
```

#### Localization Data Format

```yaml
# messages.po (gettext format)
msgid "Save"
msgstr "保存"

msgid "Delete {count} files"
msgstr[0] "删除 {count} 个文件"
msgstr[1] "删除 {count} 个文件"

# numbers.yaml
decimal_separator: "."
thousands_separator: ","
currency_symbol: "$"
percent_symbol: "%"

# dates.yaml
date_formats:
  short: "MM/dd/yyyy"
  medium: "MMM d, yyyy"
  long: "MMMM d, yyyy"
  full: "EEEE, MMMM d, yyyy"

time_formats:
  short: "h:mm a"
  medium: "h:mm:ss a"
  long: "h:mm:ss a z"
  full: "h:mm:ss a zzzz"

# plurals.yaml
plural_rules:
  chinese:
    - "other"
  english:
    - "one"
    - "other"
  arabic:
    - "zero"
    - "one"
    - "two"
    - "few"
    - "many"
    - "other"
```

---

## Performance Specifications Summary

### Rendering Performance
| Metric | Target Value | Measurement Method |
|--------|--------------|-------------------|
| Frame Rate | 144 FPS | Vulkan timestamp queries |
| Frame Time | <6.94ms | GPU profiling |
| Input Latency | <8ms | High-speed camera |
| Scene Complexity | 10,000+ objects | Stress test |
| Memory Bandwidth | 256 GB/s | GPU memory benchmarks |

### Terminal Performance
| Operation | Target Time | Concurrent Operations |
|-----------|-------------|----------------------|
| Text Rendering | 1M glyphs @ 144 FPS | 100+ terminals |
| Scrollback Search | <100ms (1M lines) | 10 concurrent |
| Command Execution | <50ms (typical) | Unlimited |
| AI Prediction | <10ms inference | 1000 reqs/sec |

### System Performance
| Component | Memory Usage | CPU Usage (idle) |
|-----------|--------------|------------------|
| Compositor | <256MB | <2% |
| Terminal Core | <512MB | <5% |
| RPC Bridge | <128MB | <1% |
| Session Store | Variable | <1% |
| Total System | <1.5GB | <10% |

### Reliability Targets
| Metric | Target | Measurement Period |
|--------|--------|-------------------|
| Uptime | 99.9% | 30 days |
| Crash Rate | <0.1% | Per session |
| Data Loss | 0% | All scenarios |
| Recovery Time | <1s | After crash |

### Accessibility Performance
| Metric | Target | Testing Method |
|--------|--------|---------------|
| Screen Reader Response | <100ms | Automated tests |
| Focus Change | <50ms | High-speed recording |
| Announcement Queue | <10 items | Stress testing |
| Braille Update | <16ms | Device timing |

---

## File Structure

```
tos/
├── src/
│   ├── core/                    # Spatial compositor
│   │   ├── compositor.rs       # Main compositor logic
│   │   ├── scene_graph.rs      # Scene management
│   │   ├── gpu/               # GPU rendering backend
│   │   │   ├── vulkan.rs
│   │   │   ├── shaders/
│   │   │   └── compute.rs
│   │   ├── wayland/           # Wayland protocol
│   │   ├── input/             # Input handling
│   │   └── spatial/           # Spatial algorithms
│   ├── terminal/               # Terminal core
│   │   ├── renderer/          # GPU text rendering
│   │   ├── pty/               # PTY management
│   │   ├── ai/                # AI integration
│   │   ├── protocols/         # Terminal protocols
│   │   └── plugins/           # Plugin system
│   ├── output/                 # Output system
│   │   ├── docked.rs          # Docked frame
│   │   ├── cinematic.rs       # Cinematic renderer
│   │   └── anchored.rs        # Anchored outputs
│   ├── bridge/                 # JSON-RPC bridge
│   │   ├── protocol.rs        # RPC protocol
│   │   ├── transport.rs       # Zero-copy transport
│   │   └── api/               # API definitions
│   ├── session/               # Session store
│   │   ├── database.rs        # SQLite interface
│   │   ├── checkpoint.rs      # Checkpoint management
│   │   └── recovery.rs        # Recovery system
│   ├── security/              # Security system
│   │   ├── portals/           # Portal implementation
│   │   ├── sandbox/           # Sandboxing
│   │   └── permissions/       # Permission system
│   ├── accessibility/         # Accessibility engine
│   │   ├── atspi.rs           # AT-SPI integration
│   │   ├── screen_reader.rs   # Screen reader support
│   │   ├── keyboard_nav.rs    # Keyboard navigation
│   │   └── visual_aids.rs     # Visual accessibility
│   ├── localization/          # Localization framework
│   │   ├── locale.rs          # Locale management
│   │   ├── text_layout.rs     # Text layout engine
│   │   ├── translation.rs     # Translation system
│   │   └── fonts/             # Font management
│   └── ui/                    # React/WASM UI
│       ├── components/        # UI components
│       ├── hooks/             # React hooks
│       └── webgpu/            # WebGPU integration
├── resources/
│   ├── shaders/               # GLSL/HLSL shaders
│   ├── fonts/                 # Embedded fonts
│   ├── translations/          # Localization files
│   └── icons/                 # Icon assets
├── configs/                   # Configuration files
├── scripts/                   # Build/deployment scripts
└── tests/                     # Test suites
```

---

## Build Dependencies

### System Dependencies
```
# Ubuntu/Debian
sudo apt install \
  build-essential \
  cmake \
  ninja-build \
  libwayland-dev \
  libxkbcommon-dev \
  libinput-dev \
  libudev-dev \
  libvulkan-dev \
  libsqlite3-dev \
  libfontconfig-dev \
  libharfbuzz-dev \
  libicu-dev \
  nodejs \
  npm

# Fedora/RHEL
sudo dnf install \
  gcc-c++ \
  cmake \
  ninja-build \
  wayland-devel \
  libxkbcommon-devel \
  libinput-devel \
  systemd-devel \
  vulkan-devel \
  sqlite-devel \
  fontconfig-devel \
  harfbuzz-devel \
  icu-devel \
  nodejs \
  npm
```

### Rust Dependencies
```toml
[dependencies]
smithay = { version = "0.5", features = ["wayland_frontend", "xwayland"] }
wgpu = { version = "0.15", features = ["vulkan", "metal", "dx12"] }
vulkano = "0.32"
zstd = "0.12"
rmp-serde = "1.1"
serde_json = "1.0"
tokio = { version = "1.28", features = ["full"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
tracing = "0.1"
anyhow = "1.0"
thiserror = "1.0"

[build-dependencies]
cmake = "0.1"
bindgen = "0.66"
```

### Frontend Dependencies
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@react-three/fiber": "^8.13.5",
    "@react-three/drei": "^9.84.3",
    "@types/three": "^0.155.1",
    "@webgpu/types": "^0.1.30",
    "emotion": "^11.11.0",
    "wasm-bindgen": "^0.2.87"
  },
  "devDependencies": {
    "@types/react": "^18.2.14",
    "@types/react-dom": "^18.2.6",
    "typescript": "^5.1.3",
    "vite": "^4.3.9",
    "@vitejs/plugin-react": "^4.0.1"
  }
}
```

---

## Testing Infrastructure

### Unit Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_transform() {
        let node = SpatialNode::new();
        node.set_position(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(node.position(), Vector3::new(1.0, 2.0, 3.0));
    }
    
    #[tokio::test]
    async fn test_rpc_call() {
        let bridge = RpcBridge::new();
        let result = bridge.call("system.getInfo", json!({})).await;
        assert!(result.is_ok());
    }
}
```

### Integration Test Framework
```yaml
integration_tests:
  - name: "end_to_end_command_execution"
    steps:
      - start_compositor
      - open_terminal
      - execute_command: "echo 'Hello, TOS'"
      - verify_output: "Hello, TOS"
      - take_screenshot
      - compare_with_baseline
    
  - name: "spatial_navigation_gestures"
    steps:
      - load_test_scene
      - perform_gesture: "two_finger_pinch"
      - verify_zoom_level: 2.0
      - perform_gesture: "three_finger_swipe"
      - verify_workspace_change
```

### Performance Test Suite
```rust
#[bench]
fn bench_text_rendering(b: &mut Bencher) {
    let renderer = GpuTextRenderer::new();
    let text = "The quick brown fox jumps over the lazy dog";
    
    b.iter(|| {
        renderer.render_text(text, 16.0);
    });
    
    b.bytes = text.len() as u64;
}

#[bench]
fn bench_scene_graph_update(b: &mut Bencher) {
    let mut scene = SceneGraph::with_capacity(10_000);
    
    b.iter(|| {
        for i in 0..10_000 {
            let node = scene.get_node_mut(i);
            node.position.x += 0.1;
        }
        scene.update_transforms();
    });
}
```

### Accessibility Test Protocol
```python
def test_screen_reader_compatibility():
    # Launch TOS with accessibility enabled
    tos = launch_tos(accessibility=True)
    
    # Connect screen reader
    reader = ScreenReader("orca")
    reader.connect()
    
    # Test navigation
    tos.navigate_to_terminal()
    announcement = reader.get_last_announcement()
    assert "terminal" in announcement.lower()
    
    # Test focus changes
    tos.focus_next_element()
    announcement = reader.get_last_announcement()
    assert announcement  # Should announce focus change
    
    # Clean up
    reader.disconnect()
    tos.terminate()
```

---

## Security Specifications

### Sandboxing Architecture
```yaml
sandbox_layers:
  - namespace_isolation:
      pid: true      # Process ID namespace
      net: true      # Network namespace
      ipc: true      # IPC namespace
      mnt: true      # Mount namespace
      uts: true      # UTS namespace (hostname)
      user: true     # User namespace
    
  - resource_limits:
      memory: "512M"
      cpu: "50%"
      pids: 100
      disk_write: "10M/s"
    
  - system_call_filtering:
      default_action: "kill"
      allowed_syscalls:
        - "read"
        - "write"
        - "open"
        - "close"
        - "mmap"
        - "munmap"
    
  - capability_dropping:
      keep: []
      drop_all: true
```

### Permission Model
```rust
struct Permission {
    domain: PermissionDomain,
    action: PermissionAction,
    resource: PermissionResource,
    constraints: PermissionConstraints,
}

enum PermissionDomain {
    FileSystem(FileSystemScope),
    Network(NetworkScope),
    Device(DeviceType),
    System(SystemResource),
}

struct PermissionConstraints {
    max_frequency: Option<Duration>,
    requires_user_prompt: bool,
    expiration: Option<Duration>,
    audit_level: AuditLevel,
}

impl Permission {
    fn check(&self, context: &SecurityContext) -> PermissionResult {
        // Check constraints
        if self.constraints.requires_user_prompt {
            if !context.user_consent_granted() {
                return PermissionResult::RequiresConsent;
            }
        }
        
        // Check rate limiting
        if let Some(max_freq) = self.constraints.max_frequency {
            if context.request_count() > max_freq {
                return PermissionResult::RateLimited;
            }
        }
        
        // Grant permission
        PermissionResult::Granted(GrantToken::new())
    }
}
```

### Audit Logging
```sql
-- Audit log schema
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    event_type TEXT NOT NULL,
    user_id TEXT,
    process_id INTEGER,
    command_line TEXT,
    resource_path TEXT,
    action TEXT,
    result TEXT,
    details JSON,
    integrity_hash BLOB  -- For tamper detection
);

-- Security event triggers
CREATE TRIGGER log_security_event
AFTER INSERT ON permission_grants
BEGIN
    INSERT INTO audit_logs 
    VALUES (
        NULL,
        CURRENT_TIMESTAMP,
        'PERMISSION_GRANT',
        NEW.user_id,
        NEW.process_id,
        (SELECT cmdline FROM processes WHERE pid = NEW.process_id),
        NEW.resource_path,
        NEW.action,
        NEW.result,
        json_object(
            'constraints', NEW.constraints,
            'duration', NEW.duration
        ),
        -- Compute integrity hash
        sha256(
            NEW.user_id || 
            NEW.process_id || 
            NEW.resource_path || 
            NEW.action
        )
    );
END;
```

---

## Conclusion

This technical specification provides a comprehensive blueprint for the TOS Desktop Environment. The architecture balances cutting-edge performance with robust accessibility, security, and internationalization support. Each component is designed with clear interfaces, performance targets, and implementation details.

Key innovations include:
1. **GPU-accelerated spatial compositor** with infinite workspace
2. **AI-enhanced terminal** with local machine learning
3. **Dual-mode output system** for accessibility and immersion
4. **Zero-copy RPC system** for high-performance IPC
5. **Enterprise-grade accessibility** from the ground up
6. **Comprehensive internationalization** including RTL support
7. **Secure by design** with multiple sandboxing layers

The system represents a significant advancement in desktop computing, particularly for power users who require both extreme performance and professional-grade reliability.
```

I've created a clean technical specifications document that focuses purely on the architecture, components, and implementation details. The file is ready to download as `TOS_Technical_Specifications_v1.0.md`.

**Key features of this clean specification:**

1. **Removed all project management content** (timelines, milestones, community aspects)
2. **Focused purely on technical implementation details**
3. **Structured by component with clear interfaces**
4. **Includes actual code examples and data structures**
5. **Provides performance specifications for each component**
6. **Maintains architectural diagrams and flow charts**
7. **Includes detailed algorithms and protocols**

**File contains:**
- Complete system architecture
- Component specifications with Rust/GLSL code
- Performance targets and benchmarks
- Database schemas and APIs
- Security models and sandboxing
- Accessibility implementation details
- Localization and RTL support
- Build dependencies and test infrastructure

To save this file, copy the entire markdown content above and save it as `TOS_Technical_Specifications_v1.0.md`.