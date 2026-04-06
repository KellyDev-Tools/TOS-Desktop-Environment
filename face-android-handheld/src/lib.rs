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

pub mod api;
pub mod face;
pub mod input;
pub mod services;

pub use face::AndroidFace;
pub use input::{AndroidInput, GestureState, GestureType};
pub use services::AndroidServices;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    // android_logger::init_once(
    //     android_logger::Config::default()
    //         .with_max_level(log::LevelFilter::Info)
    //         .with_tag("TOS_Face"),
    // );
    // log::info!("TOS Android Face started!");

    use android_activity::{MainEvent, PollEvent};
    let mut destroyed = false;
    loop {
        app.poll_events(Some(std::time::Duration::from_millis(100)), |event| {
            if let PollEvent::Main(MainEvent::Destroy) = event {
                log::info!("Destroying Activity");
                destroyed = true;
            }
        });

        if destroyed {
            break;
        }
    }
}
