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
For full-system installation using Linux native service endpoints or Homebrew formulas:
```bash
make build-all
```
For localized and portable deployments, refer to the [Packaging & Release Guide](./docs/guides/Packaging-and-Release.md).

### Running the Brain
To start the core Brain and its system services (requires `make build-services` first):
```bash
make run-services
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
