//! # TOS Android Face
//!
//! Standalone crate for the TOS Android platform layer ("Face").
//! This crate is built separately from the main TOS Brain because
//! Android requires its own toolchain (NDK), cross-compilation targets,
//! and platform-specific dependencies.
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────┐
//! │  TOS Brain (alpha_2)                       │
//! │  ├── common (TosState, HierarchyLevel)     │
//! │  └── platform traits (Renderer, etc.)      │
//! └──────────────────┬─────────────────────────┘
//!                    │ depends on
//! ┌──────────────────▼─────────────────────────┐
//! │  android-handheld (this crate)             │
//! │  ├── AndroidFace  (state + rendering)      │
//! │  ├── AndroidInput (touch + gestures)       │
//! │  ├── AndroidServices (system integration)  │
//! │  └── ndk_stubs (host-compilable stubs)     │
//! └────────────────────────────────────────────┘
//! ```
//!
//! ## Building
//!
//! **Host check (Linux/macOS):**
//! ```sh
//! cargo check -p android-handheld
//! ```
//!
//! **Android target:**
//! ```sh
//! cargo ndk -t arm64-v8a build -p android-handheld
//! ```

pub mod face;
pub mod input;
pub mod services;

// On non-Android hosts, provide stub types so the crate compiles for checking.
// On Android, these would come from real NDK/JNI bindings.
#[cfg(not(target_os = "android"))]
pub mod ndk_stubs;

#[cfg(not(target_os = "android"))]
pub use ndk_stubs as platform_api;

pub use face::AndroidFace;
pub use input::{AndroidInput, GestureState, GestureType};
pub use services::AndroidServices;
