# Android NDK Support

TOS Beta-0 supports handheld (phone/tablet) targets through the `android-handheld` crate.

## 1. Prerequisites
- Android NDK (r25c+)
- `cargo-ndk` helper
- Rust target: `aarch64-linux-android`

## 2. Cross-Compilation

To build the `android-handheld` Face:
```bash
cargo ndk -t arm64-v8a build -p android-handheld --release
```

## 3. Handheld Profile (§3.3.5)

The Android Face MUST register its profile as `handheld`. The Brain automatically reacts by:
- Enabling the `tabs` layout for the Command Hub.
- Activating voice-first mode if the `voice` capability is declared.
- Scaling bezel slots for touch interaction.
