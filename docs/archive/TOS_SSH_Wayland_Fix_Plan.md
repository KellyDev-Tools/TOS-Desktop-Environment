# TOS SSH/Wayland Buffer Streaming Fix Plan

**Issue:** Brain cannot initialize Renderer and stream buffers to Face when started over SSH without a local Wayland compositor.

**Expected Behavior (per Architecture §15.2–§15.5):**
- Brain starts in any environment (SSH, headless, direct)
- Defers buffer creation until Face requests a surface
- Streams buffers on-demand (local direct or remote via WebRTC)

**Actual Behavior:**
- Brain panics or hangs at eager Wayland initialization
- No fallback for headless/remote environments

---

## Phase 1: Code Archaeology (Find the Blocker)

### 1.1 Investigate Renderer Initialization

**Suspected Problem Locations:**

```
src/
├── brain/
│   ├── main.rs              ← Check: Renderer initialized at startup?
│   ├── platform_manager.rs  ← Check: Wayland binding happens here?
│   └── session.rs           ← Check: Surface creation deferred or eager?
├── platform/
│   ├── mod.rs               ← Core trait definitions
│   ├── linux/
│   │   └── wayland.rs       ← Check: dmabuf init (likely culprit)
│   ├── headless.rs          ← Check: Does headless mode exist?
│   └── remote.rs            ← Check: Remote rendering fallback
└── system/
    └── render.rs            ← Check: Buffer pool & on-demand creation
```

### 1.2 Investigation Checklist

**Task 1: Check `src/brain/main.rs`**

- Search for: `Renderer::new()`, `Renderer::init()`, `renderer.connect()` at the top level
- Question: Does this execute unconditionally at Brain startup?
- Action: If yes, **defer initialization to lazy evaluation**
- Expected: Renderer should be optional or initialized on first Face request

---

**Task 2: Check `src/platform/linux/wayland.rs`**

- Search for: `wl_display_connect()`, `wayland_client::*` usage
- Question: Does it try to connect to `WAYLAND_DISPLAY` immediately in `new()` or `init()`?
- Action: Make connection lazy; return `Ok(Renderer { ... })` even if compositor is unreachable
- Expected: Function completes quickly without blocking on compositor

---

**Task 3: Check `src/system/render.rs` (if exists)**

- Search for: GPU initialization, `dmabuf` allocation, `SurfacePool`
- Question: Are buffers allocated globally at startup, or per-application on-demand?
- Action: Refactor to allocate buffers only when `create_surface()` is called
- Expected: No GPU resources locked until a surface is explicitly created

---

**Task 4: Check for headless mode**

- Search for: `--headless` flag, feature gates, conditional compilation
- Question: Does the codebase have a code path for headless operation?
- Action: If missing, create `HeadlessRenderer` implementation
- Expected: `RUST_LOG=debug tos-brain --headless` starts without error

---

### 1.3 Run SSH Diagnostic

Execute on a remote Linux box **without a running Wayland compositor**:

```bash
# SSH to remote Linux box
ssh user@linux-box

# Clear Wayland environment
export WAYLAND_DISPLAY=
export DISPLAY=

# Start Brain in verbose mode
cd ~/path/to/tos
RUST_LOG=debug cargo run --bin tos-brain 2>&1 | tee /tmp/tos-brain-debug.log

# Monitor output for errors involving:
# - "wayland", "WAYLAND_DISPLAY", "wl_display"
# - "renderer", "buffer", "dmabuf"
# - panic/thread traces
```

**Document the output:**

```
[COPY FULL ERROR LOG HERE]
```

**Expected outcomes:**
- ✅ **Success:** Brain starts, logs show `[INFO] Brain initialized without local Renderer`
- ❌ **Failure:** Panic or hang with Wayland-related error

---

## Phase 2: Root Cause Analysis

Based on diagnostic output, categorize the blocker:

| Symptom | Root Cause | Next Action |
|---------|-----------|-------------|
| Panic: `WAYLAND_DISPLAY not found` | Eager env var check | Proceed to Phase 3 |
| Hang at `wl_display_connect()` | Blocking connection attempt | Proceed to Phase 3 |
| Panic: `No GPU device found` | Eager GPU init | Proceed to Phase 3 |
| Brain starts, no error | Renderer optional already | Skip to Phase 5 |

---

## Phase 3: Architecture Audit

Verify the Renderer trait supports three implementations per Architecture §15:

### 3.1 Check Core Trait Definition

**File:** `src/platform/mod.rs`

```rust
pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> Result<SurfaceHandle>;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn composite(&mut self);
}
```

**Verification:**
- [ ] `create_surface()` is called **only on demand** (lazy), not at `Renderer::new()`
- [ ] `update_surface()` and `composite()` do not assume GPU availability
- [ ] Trait is object-safe (`Box<dyn Renderer>` compiles)

---

### 3.2 Check Available Implementations

**File:** `src/platform/` directory listing

```bash
ls -la src/platform/
# Expected:
# - linux/wayland.rs     (local Wayland)
# - headless.rs or similar  (CPU buffers, no GPU)
# - remote.rs or similar (WebRTC streaming fallback)
```

**Verification:**
- [ ] `impl Renderer for WaylandRenderer` exists
- [ ] `impl Renderer for HeadlessRenderer` exists (or needs to be created)
- [ ] `impl Renderer for RemoteRenderer` exists (or needs to be created)
- [ ] All three can be compiled with feature flags or runtime selection

---

### 3.3 Check Runtime Selection Logic

**File:** `src/brain/main.rs` or `src/brain/platform_manager.rs`

Search for logic that selects which Renderer to use:

```rust
// Expected pattern (may be different):
let renderer: Box<dyn Renderer> = if wayland_available {
    Box::new(WaylandRenderer::new()?)
} else if headless_mode {
    Box::new(HeadlessRenderer::new())
} else {
    Box::new(RemoteRenderer::new())
};
```

**Verification:**
- [ ] Selection logic exists
- [ ] All three branches are reachable
- [ ] None of the branches panic or hang indefinitely

---

## Phase 4: Fix Strategy

### 4.1 Create RendererManager (New File)

**File:** `src/brain/renderer_manager.rs`

```rust
//! Renderer mode detection and initialization.
//! 
//! Per Architecture §15.2–§15.5, the Brain must support three rendering modes:
//! - LocalWayland: Direct Wayland compositor available
//! - Headless: No GPU/compositor; buffers in CPU RAM
//! - Remote: No local render; stream to remote Face via WebRTC

use std::env;
use crate::platform::{Renderer, WaylandRenderer, HeadlessRenderer, RemoteRenderer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RendererMode {
    LocalWayland,
    Headless,
    Remote,
}

pub struct RendererManager;

impl RendererManager {
    /// Detect the appropriate renderer mode for the current environment.
    pub fn detect() -> RendererMode {
        // Priority order: explicit flag > environment detection > default
        
        // Check for explicit headless flag
        if env::var("TOS_HEADLESS").is_ok() {
            return RendererMode::Headless;
        }
        
        // Check for Wayland availability
        if env::var("WAYLAND_DISPLAY").is_ok() {
            // Verify Wayland compositor is actually reachable
            // (don't just check env var — do a quick non-blocking connect test)
            if WaylandRenderer::can_connect() {
                return RendererMode::LocalWayland;
            }
        }
        
        // Default to remote/streaming fallback
        RendererMode::Remote
    }
    
    /// Initialize renderer for the detected mode.
    /// 
    /// **Critical:** This function must NEVER panic or block indefinitely.
    /// It should succeed even if hardware/compositor is unavailable.
    pub fn init(mode: RendererMode) -> Result<Box<dyn Renderer>, String> {
        match mode {
            RendererMode::LocalWayland => {
                WaylandRenderer::connect()
                    .map(|r| Box::new(r) as Box<dyn Renderer>)
                    .map_err(|e| format!("Wayland init failed: {}", e))
            }
            RendererMode::Headless => {
                Ok(Box::new(HeadlessRenderer::new()))
            }
            RendererMode::Remote => {
                Ok(Box::new(RemoteRenderer::new()))
            }
        }
    }
}
```

**What this does:**
- Detects environment (explicit flag > Wayland > default remote)
- Returns a valid Renderer for ANY environment
- Never panics on missing hardware

---

### 4.2 Update WaylandRenderer

**File:** `src/platform/linux/wayland.rs`

**Changes:**

1. Add a non-blocking connectivity check:

```rust
impl WaylandRenderer {
    /// Check if Wayland compositor is reachable without blocking.
    /// Returns true only if connection can be established immediately.
    pub fn can_connect() -> bool {
        match std::env::var("WAYLAND_DISPLAY") {
            Ok(_) => {
                // Attempt non-blocking connection with short timeout
                match wayland_client::Connection::connect_to_env() {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
    
    /// Connect to Wayland compositor.
    /// If compositor is unavailable, return Err rather than panic/hang.
    pub fn connect() -> Result<Self, String> {
        let conn = wayland_client::Connection::connect_to_env()
            .map_err(|e| format!("Wayland connection failed: {}", e))?;
        
        // ... rest of initialization
        Ok(WaylandRenderer { /* ... */ })
    }
}
```

**Important:** Replace any `expect()` or `unwrap()` calls with `Result` returns.

---

### 4.3 Create HeadlessRenderer (New File)

**File:** `src/platform/headless.rs`

```rust
//! Headless renderer for environments without GPU or compositor.
//! 
//! Buffers are stored in CPU RAM. Per Architecture §15.3, this supports
//! testing, SSH environments, and remote streaming scenarios.

use std::collections::HashMap;
use crate::platform::{Renderer, SurfaceHandle, SurfaceConfig, SurfaceContent};

pub struct HeadlessRenderer {
    surfaces: HashMap<SurfaceHandle, HeadlessSurface>,
    next_handle: u64,
}

struct HeadlessSurface {
    config: SurfaceConfig,
    buffer: Vec<u8>, // RGBA or similar format
}

impl HeadlessRenderer {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            next_handle: 1,
        }
    }
}

impl Renderer for HeadlessRenderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> Result<SurfaceHandle, String> {
        let handle = SurfaceHandle(self.next_handle);
        self.next_handle += 1;
        
        let buffer_size = (config.width * config.height * 4) as usize; // RGBA
        let surface = HeadlessSurface {
            config,
            buffer: vec![0u8; buffer_size],
        };
        
        self.surfaces.insert(handle, surface);
        Ok(handle)
    }
    
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent) -> Result<(), String> {
        if let Some(surface) = self.surfaces.get_mut(&handle) {
            // Write content to CPU buffer
            // (Implementation depends on SurfaceContent trait)
            Ok(())
        } else {
            Err(format!("Surface {} not found", handle.0))
        }
    }
    
    fn composite(&mut self) -> Result<(), String> {
        // No GPU composite needed — surfaces are already in RAM
        Ok(())
    }
}
```

---

### 4.4 Update Brain Initialization

**File:** `src/brain/main.rs`

**Before:**
```rust
fn main() {
    let renderer = Renderer::new()?;  // ← Panics if Wayland unavailable
    let brain = Brain::new(renderer)?;
    brain.run()?;
}
```

**After:**
```rust
use crate::brain::renderer_manager::RendererManager;

fn main() {
    // Detect and initialize renderer for current environment
    let mode = RendererManager::detect();
    println!("[INFO] Detected renderer mode: {:?}", mode);
    
    let renderer = RendererManager::init(mode)?;
    println!("[INFO] Renderer initialized successfully");
    
    let brain = Brain::new(renderer)?;
    brain.run()?;
}
```

---

### 4.5 Update Module Exports

**File:** `src/platform/mod.rs`

Add to `pub mod` section:

```rust
mod headless;
mod remote;  // if not already present

pub use headless::HeadlessRenderer;
pub use remote::RemoteRenderer;  // if not already present
```

---

## Phase 5: Testing Strategy (TDD)

### 5.1 Unit Test: Headless Initialization

**File:** `tests/unit_renderer_headless.rs`

```rust
#[cfg(test)]
mod tests {
    use std::env;
    use tos::platform::{HeadlessRenderer, Renderer};
    
    #[test]
    fn test_headless_renderer_new_succeeds() {
        let renderer = HeadlessRenderer::new();
        // Should not panic or hang
        assert_eq!(renderer.surfaces.len(), 0);
    }
    
    #[test]
    fn test_headless_create_surface() {
        let mut renderer = HeadlessRenderer::new();
        let config = SurfaceConfig {
            width: 1920,
            height: 1080,
            app_id: "test_app".to_string(),
        };
        
        let handle = renderer.create_surface(config);
        assert!(handle.is_ok());
        assert_eq!(renderer.surfaces.len(), 1);
    }
}
```

**Run:**
```bash
cargo test --lib test_headless_renderer_new_succeeds
```

---

### 5.2 Unit Test: Mode Detection

**File:** `tests/unit_renderer_manager.rs`

```rust
#[cfg(test)]
mod tests {
    use std::env;
    use tos::brain::renderer_manager::{RendererManager, RendererMode};
    
    #[test]
    fn test_detect_headless_explicit_flag() {
        env::set_var("TOS_HEADLESS", "1");
        env::remove_var("WAYLAND_DISPLAY");
        
        let mode = RendererManager::detect();
        assert_eq!(mode, RendererMode::Headless);
    }
    
    #[test]
    fn test_detect_remote_fallback() {
        env::remove_var("TOS_HEADLESS");
        env::remove_var("WAYLAND_DISPLAY");
        
        let mode = RendererManager::detect();
        assert_eq!(mode, RendererMode::Remote);
    }
}
```

**Run:**
```bash
cargo test --lib test_detect_headless_explicit_flag
cargo test --lib test_detect_remote_fallback
```

---

### 5.3 Integration Test: SSH Scenario

**File:** `tests/integration_ssh_headless.rs`

```rust
#[test]
fn test_brain_starts_without_wayland() {
    // Simulate SSH environment: no WAYLAND_DISPLAY, no GPU
    std::env::set_var("TOS_HEADLESS", "1");
    std::env::remove_var("WAYLAND_DISPLAY");
    
    // Brain should initialize without error
    let renderer = tos::brain::renderer_manager::RendererManager::init(
        tos::brain::renderer_manager::RendererMode::Headless
    );
    assert!(renderer.is_ok(), "Renderer should initialize in headless mode");
    
    // Brain should reach ready state
    let brain = tos::brain::Brain::new(renderer.unwrap());
    assert!(brain.is_ok(), "Brain should initialize with headless renderer");
    
    let state = brain.unwrap().get_state();
    assert_eq!(state.status, tos::brain::BrainStatus::Ready);
}
```

**Run:**
```bash
cargo test --test integration_ssh_headless -- --nocapture
```

---

### 5.4 Component Test: On-Demand Surface Creation

**File:** `tests/component_surface_creation.rs`

```rust
#[test]
fn test_surface_creation_deferred() {
    let mut renderer = tos::platform::HeadlessRenderer::new();
    
    // Brain starts — no surfaces allocated yet
    assert_eq!(renderer.surfaces.len(), 0);
    
    // Surface created ONLY when Face requests it
    let config = SurfaceConfig { width: 1920, height: 1080, app_id: "app_1".to_string() };
    let handle = renderer.create_surface(config);
    
    assert!(handle.is_ok());
    assert_eq!(renderer.surfaces.len(), 1);
    
    // Multiple applications each get their own surface
    let config2 = SurfaceConfig { width: 1920, height: 1080, app_id: "app_2".to_string() };
    let handle2 = renderer.create_surface(config2);
    
    assert!(handle2.is_ok());
    assert_eq!(renderer.surfaces.len(), 2);
}
```

**Run:**
```bash
cargo test --test component_surface_creation -- --nocapture
```

---

### 5.5 Test Execution Checklist

Run all tests in sequence:

```bash
# Unit tests
cargo test --lib test_headless_renderer_new_succeeds
cargo test --lib test_detect_headless_explicit_flag
cargo test --lib test_detect_remote_fallback

# Integration test
cargo test --test integration_ssh_headless -- --nocapture

# Component test
cargo test --test component_surface_creation -- --nocapture

# Full test suite
cargo test
```

**Expected output:**
```
test test_headless_renderer_new_succeeds ... ok
test test_detect_headless_explicit_flag ... ok
test test_detect_remote_fallback ... ok
test integration_ssh_headless ... ok
test component_surface_creation ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

---

## Phase 6: Implementation Roadmap

| Step | Task | File(s) | Tests | Approx. Time |
|------|------|---------|-------|--------------|
| 1 | Create `RendererManager` | `src/brain/renderer_manager.rs` | `test_detect_*` | 30 min |
| 2 | Update `WaylandRenderer` for non-blocking | `src/platform/linux/wayland.rs` | N/A | 45 min |
| 3 | Create `HeadlessRenderer` | `src/platform/headless.rs` | `test_headless_*` | 1 hour |
| 4 | Update `Brain::new()` to use `RendererManager` | `src/brain/main.rs` | `test_brain_starts_*` | 30 min |
| 5 | Update module exports | `src/platform/mod.rs` | All | 15 min |
| 6 | Write and run all tests | `tests/` | All 5 tests | 1 hour |
| 7 | Validate SSH scenario | Remote Linux SSH | Manual test | 30 min |
| 8 | Update documentation | `docs/` | N/A | 1 hour |

**Total estimated time:** 5–6 hours

---

## Phase 7: Validation

### 7.1 Compile Check

```bash
cargo check
# Expected: no errors
```

### 7.2 Full Test Suite

```bash
cargo test --all
# Expected: all tests pass, including new ones
```

### 7.3 SSH Scenario Validation

**On remote Linux (no compositor):**

```bash
# SSH to Linux box
ssh user@linux-box
cd ~/path/to/tos

# Clear any Wayland env
export WAYLAND_DISPLAY=
export DISPLAY=

# Start Brain with explicit headless flag
TOS_HEADLESS=1 RUST_LOG=debug cargo run --bin tos-brain 2>&1 | head -30
```

**Expected log output:**
```
[INFO] Detected renderer mode: Headless
[INFO] Renderer initialized successfully
[INFO] Brain initialized without local Renderer
[INFO] Waiting for Face connection...
[INFO] Brain ready and listening on port 7000
```

### 7.4 Remote Face Connection

**On Windows (or another machine):**

```bash
tos-face --remote linux-box:7000

# Expected: Face connects, begins receiving state updates
# Log: [INFO] Face connected from 192.168.1.x
```

**Verify:**
- [ ] Face displays Brain state
- [ ] Commands can be staged in prompt
- [ ] No video stream errors (buffers transmitted over WebRTC)

---

## Phase 8: Documentation Updates

### 8.1 Update Architecture Specification

**File:** `TOS_beta-0_Architecture.md`

**Section to update:** §15 (Platform Abstraction & Rendering)

**Changes:**

1. Add subsection §15.6: "Renderer Mode Detection & Fallback"

```markdown
### 15.6 Renderer Mode Detection & Fallback

The Brain implements automatic detection of the rendering environment and selects an appropriate Renderer implementation at runtime. This ensures TOS can operate in any context: local Wayland, headless (SSH), or remote streaming.

**Mode Selection (Priority Order):**
1. **Explicit Flag:** `TOS_HEADLESS=1` environment variable
2. **Wayland Detection:** Check `WAYLAND_DISPLAY` and verify compositor connectivity
3. **Remote Fallback:** Default to streaming buffers to a remote Face via WebRTC

**Renderer Implementations:**
- `WaylandRenderer`: Local Wayland compositor (Architecture §15.2)
- `HeadlessRenderer`: CPU-based buffers, no GPU (NEW — for SSH/headless)
- `RemoteRenderer`: Stream buffers to remote Face (Architecture §12)

**Key Principle:** Brain initialization must never block or panic due to missing hardware.
```

2. Update §15.2 (Wayland) to note:

```markdown
### 15.2 Linux Wayland Implementation

- **Layer Shell:** The Face renders as a `wlr-layer-shell` on the `TOP` layer...
- **Connection Fallback:** If the Wayland compositor is unavailable (e.g., SSH session), 
  the RendererManager falls back to HeadlessRenderer (§15.6) automatically.
```

---

### 8.2 Update Developer Reference

**File:** `TOS_beta-0_Developer.md`

**Section to update:** §2.1 (Starting the Full Stack)

**Add new subsection:** "2.2 SSH Remote Scenario"

```markdown
### 2.2 SSH Remote Scenario

When starting TOS over SSH (no local Wayland compositor), use the headless flag:

\`\`\`bash
# On remote Linux box
ssh user@linux-box
cd ~/path/to/tos

# Start Brain in headless mode
TOS_HEADLESS=1 cargo run --bin tos-brain

# In another window on Windows/local machine:
tos-face --remote linux-box:7000
\`\`\`

**What happens:**
1. Brain detects `TOS_HEADLESS=1` and initializes HeadlessRenderer
2. Brain binds to anchor port 7000 and advertises via mDNS
3. Face connects remotely and receives buffer streams via WebRTC
4. Commands flow bidirectionally over the control channel (WebSocket)

See Architecture §15.6 (Renderer Mode Detection) for technical details.
```

---

### 8.3 Add Implementation Notes

**File:** `TOS_beta-0_Developer.md`

**Section to update:** §3 (Third-Party Module SDK)

**Add note about HeadlessRenderer:**

```markdown
### 3.X HeadlessRenderer API

For modules that need to render in headless contexts (testing, CI, SSH):

- Buffers are stored in CPU RAM (Vec<u8>)
- No GPU calls — all operations succeed even without hardware
- Useful for unit testing without a running compositor
- Remote streaming extracts buffers and encodes to H.264 for WebRTC

\`\`\`rust
let renderer = HeadlessRenderer::new();
let handle = renderer.create_surface(config)?;
// Buffer is now allocated in memory; ready for updates
\`\`\`
```

---

### 8.4 Update README

**File:** `README.md` (if it exists)

**Add section:** "Running TOS Over SSH"

```markdown
## Running TOS Over SSH

TOS can be started on a remote Linux system and controlled from any device via remote connection.

### Quick Start

```bash
# On remote Linux box (no compositor needed)
TOS_HEADLESS=1 tos-brain &

# On your local machine
tos-face --remote <linux-ip>:7000
```

See [Architecture §15.6](./docs/TOS_beta-0_Architecture.md) for technical details.
```

---

### 8.5 Documentation Checklist

- [ ] Update Architecture §15 (Platform Abstraction) with §15.6
- [ ] Update Architecture §15.2 (Wayland) with fallback note
- [ ] Update Developer Reference §2 with SSH scenario subsection
- [ ] Add Developer Reference §3.X (HeadlessRenderer API)
- [ ] Update README with SSH section
- [ ] Verify all cross-references use `§X.Y` notation
- [ ] Run Markdown linter: `markdownlint docs/*.md`
- [ ] Verify no broken internal links

---

## Handoff Checklist

**Before handing to worker, verify:**

- [ ] All investigation tasks in Phase 1 completed
- [ ] Root cause identified and documented
- [ ] Phase 3 (Architecture Audit) confirms three Renderer paths are needed
- [ ] All Phase 4 files created (RendererManager, HeadlessRenderer, updates)
- [ ] All Phase 5 tests written and passing
- [ ] Phase 7 validation completed (SSH scenario works)
- [ ] Phase 8 documentation updated
- [ ] Code compiles: `cargo check`
- [ ] All tests pass: `cargo test`
- [ ] No compiler warnings: `cargo clippy`

---

## Worker Instructions

1. **Start with Phase 1:** Run the SSH diagnostic and document the error output.
2. **Complete Phases 2–3:** Identify the blocker in the codebase.
3. **Implement Phase 4:** Create new files and modify existing ones as specified.
4. **Test Phase 5:** Write tests first (TDD), then implement until tests pass.
5. **Validate Phase 7:** Test on actual SSH environment.
6. **Document Phase 8:** Update all specification files.
7. **Final QA:** Ensure all tests pass and no warnings appear.
8. **Commit & PR:** Submit with commit message:

```
Fix: SSH/Wayland buffer streaming initialization

- Implement RendererManager for runtime mode detection
- Add HeadlessRenderer for CPU-based buffers (no GPU required)
- Make Wayland connection non-blocking with fallback
- Add comprehensive tests for headless and SSH scenarios
- Update documentation (Architecture §15.6, Developer §2.2)

Fixes: Brain panics/hangs when started over SSH without Wayland

See: Phase 1–8 in TOS_SSH_Wayland_Fix_Plan.md
```

---

## Success Criteria

✅ **Brain starts successfully over SSH** (no Wayland compositor)
✅ **Face connects remotely and receives buffered frames**
✅ **All tests pass** (unit, integration, component)
✅ **Documentation updated** (Architecture, Developer, README)
✅ **No compiler warnings or clippy violations**
✅ **Manual validation** on actual SSH environment succeeds

---

**Date Prepared:** 2026-03-28
**TOS Version:** Beta-0
**Assigned Worker:** [TO BE FILLED]
**Expected Completion:** [5–6 hours from start]
