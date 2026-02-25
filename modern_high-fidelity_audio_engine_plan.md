# 🎵 Implementation Plan: TOS Audio Evolution (Rodio → Kira)

This plan outlines the transition to a modern, high-fidelity audio engine capable of supporting the "Tactical" micro-animations and rich auditory feedback required for the TOS Desktop Environment.

**Last validated:** 2026-02-25 (Phase 2 + Phase 3 completed)

---

## Phase 1: Infrastructure & Dependencies ✅ COMPLETE

Foundational changes to support the new audio engine and resolve current system-level missing links.

### 1.1 Dependency Update ✅ DONE
`tos-dream/Cargo.toml` has been updated. Kira is fully configured.
- **Implemented**: `kira = { version = "0.9", features = ["cpal", "wav", "mp3", "ogg"], optional = true }`
- **Also added**: `glam = { version = "0.27", optional = true }` and `mint = { version = "0.5", optional = true }` — math types used by Kira's spatial API.
- **Feature gate**: All three deps are enabled via the `accessibility` feature flag.
- **Note**: The plan referenced `v0.8+`; the codebase is on **v0.9** (minor API differences already accounted for).

### 1.2 System-Level Fixes ✅ DONE
`install_deps.sh` includes ALSA development headers for all three major distros.
- **Debian**: `libasound2-dev`
- **Fedora**: `alsa-lib-devel`
- **Arch**: `alsa-lib`

---

## Phase 2: Core Audio Refactoring ✅ COMPLETE

### 2.1 Three-Bus `AudioManager` Architecture ✅ DONE
`src/system/audio.rs` fully refactored with a **three-bus Kira hierarchy**:

```
Master
  ├── Ambience Bus  (looping backgrounds)
  │     ├── Low-pass FilterHandle  (muted at ~800 Hz during L3 ApplicationFocus)
  │     └── VolumeControlHandle   (ducked to 20% during critical alerts)
  ├── UI Bus         (earcon one-shots)
  └── Voice/TTS Bus  (high-priority speech / TTS one-shots)
```

- ✅ `manager.add_sub_track(TrackBuilder::default())` × 3 buses
- ✅ `TrackBuilder::add_effect(FilterBuilder)` → `FilterHandle` on Ambience Bus
- ✅ `TrackBuilder::add_effect(VolumeControlBuilder)` → `VolumeControlHandle` on Ambience Bus
- ✅ `play_voice_event()` for dedicated TTS/voice routing through the Voice Bus
- ✅ `duck_ambience()` / `unduck_ambience()` — smooth side-chain ducking with `Tween`
- ✅ `apply_focus_filter()` / `remove_focus_filter()` — L3 low-pass filter with `Tween`
- ✅ Graceful `None` fallback on Kira initialization failure (headless environments)

### 2.2 `EarconPlayer` with True 3D Spatial Audio ✅ DONE
`src/system/audio/earcons.rs` fully upgraded:

- ✅ `KiraManager::add_spatial_scene()` — persistent Kira `SpatialScene` initialized
- ✅ `SpatialSceneHandle::add_listener(mint::Vector3, mint::Quaternion, ...)` — centre listener at origin
- ✅ `play_spatial()` routes **spatial earcons** through a per-call `SpatialEmitter` (true 3D, distance attenuated by Kira)
- ✅ Non-spatial earcons continue via the flat `ui_track` sub-track
- ✅ `play_through_ui_track()` internal helper isolates the flat routing path
- ✅ All earcon features retained: debouncing, priority levels, polyphony cap, category volumes

> ⚠️ **Kira `Arrangements` NOT implemented**: Arrangements are not part of the Kira 0.9 public API surface (`kira::arrangement` was removed in 0.8). Complex sound sequences remain single-shot file playback. This item is **closed as N/A**.

---

## Phase 3: Tactical Bezel Integration ✅ COMPLETE

### 3.1 Audio-UI Linking ✅ DONE

**Bezel earcons wired** in `src/lib.rs` `TosState::toggle_bezel()`:
```rust
if was_expanded {
    self.earcon_player.bezel_collapse();
} else {
    self.earcon_player.bezel_expand();
}
```
Every `toggle_bezel` IPC call (from `ipc.rs` `"toggle_bezel"` handler and `"open_settings"`) now fires the appropriate sound.

**Ambience ducking wired** in `src/lib.rs`:
- `TosState::tactical_reset()` → `self.audio_manager.duck_ambience()`
- `TosState::play_critical_earcon()` → new helper that simultaneously plays a high-priority earcon AND ducks ambience. Used for:
  - `CommandError` on isolation policy violations (IPC `connect_remote`, `invite_participant`)
  - `CommandError` on deep-inspection policy denial

**L3 focus filter wired** in `src/lib.rs`:
- `TosState::zoom_in()` → `audio_manager.apply_focus_filter()` when entering `ApplicationFocus`
- `TosState::zoom_out()` → `audio_manager.remove_focus_filter()` when leaving `ApplicationFocus`

### 3.2 Spatial Audio Support ✅ DONE (True 3D via SpatialEmitter)
- `SpatialScene` + `Listener` initialized in `EarconPlayer::new()`
- Per-call `SpatialEmitter` created at `SpatialPosition { x, y, z }` for spatial earcons
- `sound.output_destination(&emitter)` routes audio through Kira's 3D pipeline
- Fallback to flat UI sub-track if spatial scene is unavailable

---

## Phase 4: Verification & Polish ✅ COMPLETE

### 4.1 Accessibility Sync ✅ DONE
`src/accessibility/audio.rs` (`AuditoryInterface`) uses Kira with `fade_in_tween`. 
> ⚠️ **No dedicated bus**: Accessibility audio still plays directly to the main output (not the Voice/TTS bus). This is acceptable for now — routing the `AuditoryInterface` through the `AudioManager`'s Voice bus would require a shared Kira manager, which needs an `Arc<Mutex<>>` refactor.

### 4.2 Testing & Validation ✅ DONE (Headless)
- ✅ All Kira initialization wrapped in `Option<...>` — headless graceful fallback
- ✅ **Integration Tests**: All `cargo test --features accessibility` pass (zero failures)
- ✅ Stale test assertion fixed: `test_ipc_navigation_integration` updated to check `hub-content` instead of removed `tactical-header` class
- ⚠️ **Hardware playback check**: No automated test for actual hardware audio output — remains a manual verification step

---

## 🗂️ Remaining Work Summary

| Item | Priority | Complexity | Status |
|---|---|---|---|
| Share Kira manager between `AudioManager` and `EarconPlayer` via `Arc<Mutex<>>` | Low | High | Open |
| Route `AuditoryInterface` through AudioManager Voice/TTS Bus | Low | Medium | Open |
| Hardware audio playback smoke test in CI | Medium | Low | Open |
| Kira `Arrangement`-based synthesized fallbacks | N/A | — | Closed (API removed in Kira 0.8+) |
