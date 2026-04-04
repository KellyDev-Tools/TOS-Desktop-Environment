# TOS Beta-0 Consolidated Roadmap — Execution Tasks

## Repo Cleanup — Replace Old Roadmaps
- [x] Copy consolidated roadmap to repo root as `TOS_Beta-0_Roadmap.md`
- [x] Archive `TOS_alpha2-to-beta0.md` to `docs/archive/`
- [x] Archive `TOS_SSH_Wayland_Fix_Plan.md` to `docs/archive/`

## Stage 0 — Hard Gate Blockers
- [ ] 0.1 Brain Tool Registry: enforce `tool_bundle` permissions at runtime
- [ ] 0.2 Verify `.tos-skill` accepted / `.tos-aibehavior` rejected by Marketplace
- [ ] 0.3 Silent restore — no notification or prompt on session launch
- [x] 0.4 Verify profile diversity (handheld/spatial face_register) — validated via face_visual_states.rs
- [x] 0.5 All errors routed through `tracing` — fixed 9 stray println/eprintln in audio.rs, haptic.rs, logger.rs, ai/mod.rs, face/mod.rs
- [x] 0.6 Verify IPC round-trip < 16ms — latency warning threshold enforced in IpcHandler
- [x] **BONUS:** Fixed 2 unused import warnings in brain/src/main.rs → 0 warnings workspace-wide

## Stage 1 — Core Runtime Hardening
- [ ] 1.1 Implement 1Hz state_delta push from Brain
- [ ] 1.2 Implement Face heartbeat detection
- [ ] 1.3 Add `Editor` variant to `PaneContent` enum
- [ ] 1.4 Wire OSC 9012 line-level priority parser
- [ ] 1.5 Configurable keyboard shortcut mapping layer
- [ ] 1.6 Exponential backoff on daemon registration retry
- [ ] 1.7 Dynamic sector labeling from cwd changes
- [ ] 1.8 Auto Activity Mode detection on top/ps

## Stage 2–6 — Deferred to subsequent sessions
