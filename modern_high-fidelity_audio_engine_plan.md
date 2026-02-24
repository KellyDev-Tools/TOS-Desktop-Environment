# 🎵 Implementation Plan: TOS Audio Evolution (Rodio → Kira)

This plan outlines the transition to a modern, high-fidelity audio engine capable of supporting the "Tactical" micro-animations and rich auditory feedback required for the TOS Desktop Environment.

---

## Phase 1: Infrastructure & Dependencies

Foundational changes to support the new audio engine and resolve current system-level missing links.

### 1.1 Dependency Update
Update `tos-dream/Cargo.toml` to swap the engines and add necessary features for `kira`.
- **Action**: Replace `rodio` with `kira` (v0.8+).
- **Features to enable**: `cpal`, `wav`, `mp3`, `ogg` (for sound pack support).

### 1.2 System-Level Fixes
Update `install_deps.sh` to include ALSA development headers, which are required for `cpal` (the backend for Kira) to function on Linux.
- **Action**: Add `libasound2-dev` (Debian), `alsa-lib-devel` (Fedora), or `alsa-lib` (Arch) to the installation script.

---

## Phase 2: Core Audio Refactoring

Rebuilding the `AudioManager` to leverage Kira's bus-based mixing architecture.

### 2.1 Refactor `src/system/audio.rs`
- **Manager Initialization**: Initialize the `kira::manager::AudioManager`.
- **Bus Architecture**: Create a dedicated hierarchy of mixer buses:
  - `Master Bus`
    - `Ambience Bus` (for background hums/chirps)
    - `UI Bus` (for Bezel and Prompt feedback)
    - `Voice/TTS Bus` (for high-priority communication)
- **Tween Integration**: Implement `play_event` to use Kira's `Tween` for smooth volume/pitch transitions instead of rodio's abrupt starts.

### 2.2 Refactor `src/system/audio/earcons.rs`
- **Arrangements**: Replace manual sine wave construction with `Arrangements`. This allows complex sequences (like the "Zoom In" ascending chime) to be defined as data rather than procedural code.
- **Dynamic Filtering**: Add support for real-time effects (e.g., a low-pass filter on the `Ambience Bus` that triggers during a Level 3 Application Focus).

---

## Phase 3: Tactical Bezel Integration

Enhancing the user experience at the application level.

### 3.1 Audio-UI Linking
- **Slide-to-Sound**: Link the `BezelExpand` and `BezelCollapse` events to sounds that dynamically scale their pitch or volume based on the bezel's expansion percentage.
- **Ducking Logic**: Implement "Side-chaining" where the `Ambience Bus` volume is automatically lowered (ducked) when a critical `TacticalAlert` or `CommandError` is played.

### 3.2 Spatial Audio Support
- **Recursive Spatialization**: Map the `SpatialPosition` coordinates in `earcons.rs` to Kira's `SpatialEmitter` for 3D sound positioning in the recursive zoom hierarchy.

---

## Phase 4: Verification & Polish

### 4.1 Accessibility Sync
Ensure `src/accessibility/audio.rs` is updated to use the new `Kira`-backed `AudioManager` so that screen reader cues and earcons remain consistent.

### 4.2 Testing & Validation
- **Hardware Check**: Verify playback on physical hardware via the updated `install_deps.sh`.
- **Headless Fallback**: Ensure the system handles missing audio devices gracefully (critical for CI/CD and remote server environments).
