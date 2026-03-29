# TOS Common Foundation

The shared bedrock of the TOS (Terminal On Steroids) ecosystem. This library provides the unified protocol, data structures, and platform abstractions used by the Brain, the Face, and all auxiliary daemons.

## Design Drivers
- **Architecture Spec §3.3**: Shared Protocol Library (`tos-common`) ensures a stable contract between all system processes.
- **Architecture Spec §18**: Defines the Module Manifest system for all plugin types (Sectors, AI Skills, Renderers).
- **Architecture Spec §15**: Platform abstraction layer for Headless, Remote, and Native rendering.

## Core Features
- Unified State (`TosState`) and Command Hub models.
- IPC Dispatcher for standard `prefix:payload` communication.
- Platform traits for OS-level integration.
