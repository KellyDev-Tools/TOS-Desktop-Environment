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
//! │  TOS Common                                │
//! │  ├── shared types (TosState, etc.)         │
//! │  └── platform traits (Renderer, etc.)      │
//! └──────────────────┬─────────────────────────┘
//!                    │ depends on
//! ┌──────────────────▼─────────────────────────┐
//! │  face-android-handheld (this crate)        │
//! │  ├── AndroidFace  (state + rendering)      │
//! │  ├── AndroidInput (touch + gestures)       │
//! │  ├── AndroidServices (system integration)  │
//! │  └── api (NDK/JNI stubs)                   │
//! └────────────────────────────────────────────┘
//! ```
//!
//! ## Building
//!
//! **Host check (Linux/macOS):**
//! ```sh
//! cargo check -p face-android-handheld
//! ```
//!
//! **Android target:**
//! ```sh
//! cargo ndk -t arm64-v8a build -p face-android-handheld
//! ```

pub mod face;
pub mod input;
pub mod services;
pub mod api;

pub use face::AndroidFace;
pub use input::{AndroidInput, GestureState, GestureType};
pub use services::AndroidServices;
