# TOS Alpha-2.0 Comprehensive Implementation Plan

This plan tracks the granular tasks required to fulfill the TOS Alpha-2 Architectural Specification.

## Phase 1: Testing (TDD Tiered Plan)
- [x] Execute Tier 1 Unit Tests (Brain Core)
- [x] Execute Tier 2 Integration Tests (Shell/PTY)
- [x] Execute Tier 4 Security Tests (Sandbox/Permissions) §18
- [x] Stress Test IPC Round-trip (<16ms) §31.5

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
- [x] Define Renderer Traits (SystemServices §15.1)
- [x] Implement Bezel IPC Bridge (Face -> Brain) §28.2
- [x] Implement State Sync Protocol (Brain -> Face) §19.1
- [x] Mock Level 1 (Global Overview) UI Components
- [x] Mock Level 2 (Command Hub) UI Components
- [x] Implement Tactical Mini-Map §22
- [x] Implement Priority-Weighted Visual Indicators §21
- [x] Implement Level 3/4 Detail & Application Focus Views (Mocks) §8, §9

## Phase 6: Security & Trust
- [x] Implement Dual-Tier Trust Model §18
- [x] Implement Dangerous Command Interception §17.3
- [x] Implement Tactile/Voice Confirmation State Machine §17.3
- [x] Implement Standard Tier Sandbox (Mock Permissions) §18.4
- [x] Implement Deep Inspection (Level 5) Privilege Escalation §17.4

## Phase 7: Auxiliary Services
- [x] Implement Unified TOS Log Service (Merging Local/System/Remote) §19.1
- [x] Implement Settings Daemon (Cascading Persistence) §11.1/§26
- [x] Implement Audio Engine Service (Earcons placeholder) §21.2/§23
- [x] Implement Marketplace Discovery (module.toml Parsing) §18.1

## Phase 8: Platform & Remote Support
- [x] Implement TOS Remote Server (File Service & Stream Management) §12.1
- [x] Implement Collaboration Manager (Multi-user Sync) §13
- [x] Implement Linux Backend (PTY/FS) §15
- [x] Implement Android & OpenXR Backends (Mocks) §15
- [x] Implement SSH Fallback Logic for Remote Sectors §12.1

## Phase 9: System Polish
- [x] Visual Transition Refinement §30.15
- [x] Performance Optimization (IPC Latency Tracking) §31.5
- [x] Documentation Finalization
