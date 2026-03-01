# TOS Alpha-2.1 Brain (Backend) Roadmap

This roadmap tracks the progress of the TOS backend, specifically focusing on the Logic Thread ("Brain"), AI engines, System Services, Sandboxing, and hardware-level compositor integrations.

## AI Engine & Intelligence (§12)
- [x] **AI Mode Implementation:** Develop the `AiService` in the Brain to handle Natural Language Processing.
- [x] **Staged Command Generation:** Enable the AI to propose complex shell scripts for user review before execution.
- [x] **Contextual Awareness:** Feed system state (active sectors, search results, system logs) into the AI context for smarter assistance.
- [ ] **Natural Language Search:** Replace string matching in SEARCH mode with semantic embedding-based retrieval. (Partial: IPC Routing Ready).
- [ ] **Real LLM Integration:** LLM NLP is completely mocked with exact substring matching (`p.contains("list")`). Needs real API integration.

## Backend Compositing & Wayland (§15)
- [x] **DMABUF Native Path:** Optimize the Linux backend for zero-copy frame buffer sharing with the Wayland compositor.
- [x] **Wayland DMABUF Logic:** Surface attachment and composite loop logic resolved.
- [x] **XR/Quest Swapchain:** Swapchain placeholder resolved.
- [ ] **Android NDK/Intents:** Implement real SurfaceControl and Intent pipelines (currently mocked).

## Production-Grade Security & Isolation (§18.4)
- [x] **Kernel-Level Sandboxing:** Move beyond mock permissions to actual Linux namespaces/cgroups for Standard Modules.
- [x] **Deep Inspection (Level 5) Audit:** Implement a cryptographic audit log for all privilege escalations.
- [x] **Manifest Signing:** Require signed `module.toml` for Marketplace installations to ensure supply-chain security.
- [ ] **Signature Forging:** `verify_manifest_signature` returns a hardcoded `true`. Must check real Edwards-curve signatures.
- [ ] **Interception Loophole:** `is_dangerous` uses naïve string matching (`rm -rf`); `rm -r /` bypasses it. Needs abstract semantic analysis.
- [ ] **Trusted Root Key:** `get_trusted_public_key()` hardcodes an Ed25519 hash literal purely to pass tests.

## Services & Transports
- [ ] **Multi-Sensory Audio Pipeline:** `AudioService::play_earcon` pipes to stdout; needs real `cpal`/`rodio` audio sink initialization.
- [ ] **Remote Desktop Protocol (TDP):** Finalize the custom TOS Display Protocol for low-latency (<50ms) remote sector streaming.
- [ ] **Remote WebRTC Auto-Close:** Uses an arbitrary 5-second `tokio::time::sleep` on close without heartbeat negotiation. Needs true ICE teardown.
- [ ] **Default Shell Hardening:** `std::env::var("SHELL")` fallback misses valid binary verification.

## Testing & Infrastructure
- [ ] **Stress Testing:** Validate the <16ms IPC threshold under heavy load (20+ sectors).
- [ ] **stimulator_brain_node IPC Port:** Fix test crash where the `tos-brain` doesn't bind an IPC server port alongside the web-server for testing mocks.
- [ ] **Beta Branch Merge:** Consolidate all backends and services into the main production branch.
- [ ] **Developer SDK Guide:** Finalize documentation for module creators.
