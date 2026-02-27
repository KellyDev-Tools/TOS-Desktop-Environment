# TOS Alpha-2 Comprehensive Implementation Plan

This plan tracks the granular tasks required to fulfill the TOS Alpha-2 Architectural Specification.

## Phase 1: Testing (TDD Tiered Plan)
- [x] Execute Tier 1 Unit Tests (Brain Core)
- [x] Execute Tier 2 Integration Tests (Shell/PTY)
- [ ] Execute Tier 4 Security Tests (Sandbox/Permissions)
- [ ] Stress Test IPC Round-trip (<16ms) §31.5

## Phase 2: Core Foundation & Infrastructure
- [x] Initialize Alpha-2 Project Structure §30.1
- [x] Standardize Build System (Makefile & Cargo.toml)
- [x] Define Common Data Types (Sectors, Hubs, Levels) §5, §10
- [x] Implement Basic PTY Integration (portable-pty) §30.1
- [x] Implement Standardized IPC Format (`prefix:arg1;arg2`) §3.3.1
- [x] Basic Line-Level Priority Support (OSC 9012) §27.5

## Phase 3: Brain (Logic & State Management)
- [x] Implement Hierarchy Level Transition Logic (Levels 1-5) §5
- [x] Implement Sector Lifecycle (Create, Clone, Close, Freeze) §6.5
- [x] Implement Persistent Command Dispatcher (ls/cd Sniffing) §28.1
- [x] Implement Directory Mode Overlay Logic (Local FS) §27.3
- [x] Implement Activity Mode Overlay Logic (Process Tracking) §7.3
- [x] Implement Global Search Logic §7.3

## Phase 4: Shell API & Enhanced Integration
- [x] Implement CWD Tracking (OSC 9003) §27
- [x] Implement Command Result Reporting (OSC 9002) §27.1
- [x] Implement Directory Listing IPC (OSC 9001) §27
- [x] Implement Remote Session Management (Disconnection Banner/Auto-close) §27.3
- [x] Implement Shell Integration Script Generation (Fish/Bash/Zsh) §18.7

## Phase 5: Face (UI & Presentation Layer)
- [ ] Define Renderer Traits (SystemServices §15.1)
- [ ] Implement Level 1: Global Overview (Sector Tiles + System Log Layer) §6
- [ ] Implement Level 2: Interactive Command Hub (Bezels + Persistent Prompt) §7
- [ ] Implement Tactical Mini-Map §22
- [ ] Implement Priority-Weighted Visual Indicators §21
- [ ] Implement Level 3/4 Detail & Application Focus Views §8, §9

## Phase 6: Security & Modular Runtime
- [ ] Implement Dual-Tier Trust Model (Standard vs. Trusted) §18
- [ ] Implement Modular Sandbox Runtime (Linux: Bubblewrap/AppImage) §17.2
- [ ] Implement Tactile Confirmation Slider Logic §17.3
- [ ] Implement Voice Confirmation Fallback §17.3
- [ ] Implement Deep Inspection (Level 5) Privilege Escalation §17.4

## Phase 7: Auxiliary Services
- [ ] Implement Unified TOS Log Service (Merging Local/System/Remote) §19.1
- [ ] Implement Settings Daemon (Persistence + Cascading Defaults) §26
- [ ] Implement Audio/Haptic Engine (Earcons & Haptic Feedback) §23, §24
- [ ] Implement AI Engine (NLE Transition to Staged Commands) §18.3, §30.12

## Phase 8: Platform & Remote Support
- [ ] Implement TOS Remote Server (File Service & Stream Management) §12.1
- [ ] Implement Collaboration Manager (Multi-user Sync) §13
- [ ] Implement Linux Wayland Backend §15
- [ ] Implement Android Backend §15
- [ ] Implement OpenXR/Meta Quest Backend §15

## Phase 9: System Polish
- [ ] Visual Transition Refinement
- [ ] Performance Optimization
- [ ] Documentation Finalization
