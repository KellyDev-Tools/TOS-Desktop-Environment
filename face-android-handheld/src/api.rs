//! Stub types that stand in for Android NDK / JNI types when compiling on a
//! non-Android host.  These allow `cargo check` to pass without the NDK
//! toolchain installed.  On a real Android build, the actual crate deps
//! (`android-activity`, `ndk`, `jni`) replace these.

// ---------------------------------------------------------------------------
// Window / Surface
// ---------------------------------------------------------------------------

/// Placeholder for an Android NativeWindow handle.
#[derive(Debug)]
pub struct Window;

/// Placeholder for an EGL display handle.
#[derive(Debug)]
pub struct Display;

/// Placeholder for an Android Configuration.
#[derive(Debug, Clone)]
pub struct Config;

// ---------------------------------------------------------------------------
// Activity State
// ---------------------------------------------------------------------------

/// Mirrors the minimal activity state needed by AndroidFace.
#[derive(Debug, Default)]
pub struct ActivityState {
    pub window: Option<Window>,
    pub display: Option<Display>,
    pub config: Option<Config>,
}


// ---------------------------------------------------------------------------
// Lifecycle & Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Lifecycle {
    Resumed,
    Paused,
    ConfigChanged,
    LowMemory,
    ScreenOn,
    ScreenOff,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TouchAction {
    Down,
    Up,
    Move,
}

#[derive(Debug, Clone)]
pub struct TouchMotion {
    pub action: TouchAction,
    pub x: f32,
    pub y: f32,
    pub pointer_id: i32,
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Touch(TouchMotion),
    Key { key_code: i32, action: i32 },
}

#[derive(Debug)]
pub enum Command {
    Lifecycle(Lifecycle),
    InputEvent(InputEvent),
    WindowCreated(Window),
    WindowDestroyed,
    ConfigChanged(Config),
    Finish,
}

// ---------------------------------------------------------------------------
// System Info (stubs)
// ---------------------------------------------------------------------------

pub struct MemoryInfo {
    pub total_memory: u64,
    pub free_memory: u64,
}

pub fn get_memory_info() -> MemoryInfo {
    MemoryInfo {
        total_memory: 8 * 1024 * 1024 * 1024, // 8 GB placeholder
        free_memory: 4 * 1024 * 1024 * 1024,
    }
}
