# TOS // v0.1

**Terminal On Steroids** for the spatial and tabletop computing era.

v0.1 marks the transition to a dynamic, registry-driven architecture for the TOS Brain and its ecosystem of satellite daemons.

## Architecture Highlights
- **Modular Face Architecture**: Platform-specific layers are decoupled into independent `face-*` crates (Wayland, Android, Svelte, Electron).
- **Consolidated Common Core**: The `tos-common` crate defines the unified contract for both the Face and Brain.
- **Headless Brain Daemon**: The core `tos-brain` functions as a headless service hub, with optional native output via `face-wayland-linux`.
- **Recursive Zoom Hierarchy**: Navigate between system-wide contexts using vertical spatial zooming.
- **AI-Managed Workflow**: Native integration for AI skills, including Chat Companion and Passive Observer.

## Getting Started
### Build Requirements
- Rust stable
- Node.js 20+ (for Svelte Face)
- Android NDK (optional, for handheld builds)

### Installation
For full-system installation including GDM session support:
```bash
make build-all
make install # will ask for sudo password
make restart-gdm
```
After installation, you can select **TOS** from your display manager (GDM/SDDM) login screen.

### Running the Brain
To start the core Brain in a standalone terminal:
```bash
# Start brain only
./target/debug/tos-brain

# Start brain and automatically spawn all auxiliary services (Orchestration)
./target/debug/tos-brain --orchestrate
```

### Running the Face (Svelte-based Web Interface)
```bash
make run-web
```

## Documentation
- [Architecture Specification](./docs/spec/TOS_beta-0_Architecture.md)
- [Developer SDK Guide](./docs/spec/TOS_beta-0_Developer.md)
- [Features & User Experience](./docs/spec/TOS_beta-0_Features.md)
- [Packaging & Release Guide](./docs/guides/Packaging-and-Release.md)
- [Beta-0 Migration Roadmap](../TOS_alpha2-to-beta0.md)

## Status
- **Phase 1: Selective Pull & Code Standard** — COMPLETED ✅
- **Phase 2: Registry & Daemon Refactor** — COMPLETED ✅
- **Phase 3: Versioning & Release Prep** — COMPLETED ✅
- **Phase 4: Production Readiness (Security/Perf)** — COMPLETED ✅
- **Phase 5: Native Platform & Feature Validation** — IN PROGRESS 🔧
- **Phase 6: Release Artifacts & Packaging** — COMPLETED ✅
