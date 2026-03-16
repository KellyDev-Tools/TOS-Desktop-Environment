# TOS Alpha-2.2.1 — Platform Strategy Options

**Version:** Alpha-2.2.1
**Date:** 2026-03-15
**Status:** Decision Required

---

## Executive Summary

This document evaluates three strategic paths for implementing the **Face** layer across platforms, addressing the question: *How do we run the Face on Windows, macOS, and beyond while keeping the Brain on Linux?*

All options preserve the core architectural principle:

> **Brain runs on Linux. Face runs anywhere. Multiple Faces can connect to one Brain. Multiple Brains can serve Faces.**

---

## 1. Option A: Electron-First Cross-Platform Face

### Overview

Use Electron (or similar WebView-based framework) as the primary Face container across all platforms. The Svelte UI runs inside a Chromium-based WebView2/WebView.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Electron App Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │   WinUI Shell   │  │   macOS Shell   │  │  Linux Shell    │      │
│  │   (Win32 API)   │  │   (AppKit)      │  │  (Wayland)      │      │
│  │  ┌─────────────┐│  │  ┌─────────────┐│  │ ┌─────────────┐ │      │
│  │  │ WebView2    ││  │  │ WKWebView   ││  │ │ Gtk/WebKit  │ │      │
│  │  │ (Svelte UI) ││  │  │ (Svelte UI) ││  │ │ (Svelte UI) │ │      │
│  │  └──────┬──────┘│  │  └──────┬──────┘│  │ └──────┬──────┘ │      │
│  └─────────┼───────┘  └─────────┼───────┘  └─────────┼───────┘      │
│            │                    │                    │               │
│            └────────────────────┼────────────────────┘               │
│                                 │ IPC over TCP/WS                    │
│                                 ▼                                    │
│                          ┌──────────────┐                           │
│                          │  Brain (TCP) │                           │
│                          │   on Linux   │                           │
│                          └──────────────┘                           │
└─────────────────────────────────────────────────────────────────────┘
```

### Implementation Plan

| Component | Location | Description |
|-----------|----------|-------------|
| `face/electron/` | New | Electron wrapper with platform-native shells |
| `face/electron/main.js` | New | Main process: window management, IPC bridge |
| `face/electron/renderer/` | New | Svelte app loaded via `loadFile()` or dev server |
| `platform/electron/` | New | Rust IPC client (same protocol as web) |

### Advantages

| Benefit | Explanation |
|---------|-------------|
| **Single UI Codebase** | Same Svelte app runs everywhere; no platform-specific UI code |
| **Rapid Windows/macOS Delivery** | Electron scaffolding is mature and well-documented |
| **Existing Web IPC Works** | Your WebSocket IPC protocol requires no changes |
| **Browser DevTools** | Full Chrome DevTools for debugging Face issues |
| **WebGPU Fallback** | Can use WebGPU for rendering when no native compositor available |

### Disadvantages

| Concern | Mitigation |
|---------|------------|
| **Binary Size** | Electron apps are ~150-200MB; users expect ~5-10MB |
| **Performance** | Chromium overhead adds 50-100ms startup latency |
| **Resource Usage** | Each Face instance runs full Chromium instance |
| **Platform Integration** | Less native feel; harder to access platform-specific APIs |
| **Security Surface** | Larger attack surface (Chromium vulnerabilities) |

### Performance Estimates

| Metric | Native | Electron |
|--------|--------|----------|
| Startup Time | ~2s | ~8-15s |
| Memory (idle) | 40-60MB | 120-200MB |
| Memory (active) | 80-120MB | 250-400MB |
| GPU Usage | Low | Medium (Chromium compositor) |

### Recommended For

- **Timeline:** Alpha-2.2.1 → Beta-0 (fastest path to multiplatform)
- **Target Users:** Power users who prioritize features over resource efficiency
- **Development Resources:** Small team (1-2 engineers)

---

## 2. Option B: Native Platform Shells with Shared Face Logic

### Overview

Create native platform applications (WinUI 3, AppKit, GTK) that embed the Svelte Face via WebView, with native fallback compositors when available.

### Architecture

```
┌───────────────────────┐  ┌───────────────────────┐  ┌───────────────────────┐
│    Windows Native     │  │   macOS Native        │  │   Linux Native        │
│  ┌─────────────────┐  │  │  ┌─────────────────┐  │  │  ┌─────────────────┐  │
│  │  WinUI 3 App    │  │  │  │  AppKit App     │  │  │  │  GTK App        │  │
│  │  ┌─────────────┐│  │  │  │  ┌─────────────┐│  │  │  │  ┌─────────────┐│  │
│  │  │ WebView2    ││  │  │  │  │ WKWebView   ││  │  │  │  │ WebKitGTK   ││  │
│  │  │ (Svelte UI) ││  │  │  │  │ (Svelte UI) ││  │  │  │  │ (Svelte UI) ││  │
│  │  └──────┬──────┘│  │  │  │  └──────┬──────┘│  │  │  │  └──────┬──────┘│  │
│  └─────────┼───────┘  │  └─────────┼───────┘  │  └─────────┼───────┘  │
│            │          │            │          │            │          │
│  ┌─────────▼───────┐  │  ┌─────────▼───────┐  │  ┌─────────▼───────┐  │
│  │  Native Renderer│  │  │  Native Renderer│  │  │ Wayland Shell   │  │
│  │  (Direct3D 11)  │  │  │  (Metal)        │  │  │  (wlr-layer)    │  │
│  └─────────────────┘  │  └─────────────────┘  │  └─────────────────┘  │
└───────────────────────┘  └─────────────────────┘  └─────────────────────┘
            │                        │                        │
            └────────────────────────┼────────────────────────┘
                                     │ IPC over TCP/WS
                                     ▼
                              ┌──────────────┐
                              │  Brain (TCP) │
                              │   on Linux   │
                              └──────────────┘
```

### Implementation Plan

| Platform | Renderer | WebView | API Layer |
|----------|----------|---------|-----------|
| Windows | Direct3D 11/12 | WebView2 | WinUI 3 + C# |
| macOS | Metal | WKWebView | AppKit + Swift |
| Linux | Wayland (keep) | WebKitGTK | GTK + Rust |

### Advantages

| Benefit | Explanation |
|---------|-------------|
| **Native Performance** | No Chromium overhead; startup in ~1-2s |
| **Platform Integration** | Full access to native APIs (notifications, system tray, etc.) |
| **Smaller Distribution** | ~20-40MB vs 150MB for Electron |
| **Native Compositor** | Direct3D/Metal/Wayland for optimal rendering |
| **Modular Installation** | Can install Face alone without Brain |

### Disadvantages

| Concern | Mitigation |
|---------|------------|
| **Three Codebases** | WinUI, AppKit, GTK all require different expertise |
| **Longer Initial Path** | More setup before first cross-platform release |
| **Feature Parity Challenges** | Keeping 3 platforms in sync requires discipline |
| **Build Complexity** | CI/CD must build for 3 platforms |

### Recommended For

- **Timeline:** Beta-1 → Alpha-3 (production-grade multiplatform)
- **Target Users:** Production users on all platforms
- **Development Resources:** Medium team (3-5 engineers)

---

## 3. Option C: OpenXR as the Primary Rendering Abstraction

### Overview

Build the Face on top of OpenXR, treating 2D displays as just one type of "viewport." The same renderer works on standard monitors, VR headsets (Quest), and AR glasses.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         OpenXR Face Layer                           │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐            │
│  │  Standard     │  │   VR/AR HMD   │  │   Mobile Tile │            │
│  │  Monitor      │  │   (Quest)     │  │   (Android)   │            │
│  │  ┌─────────┐  │  │  ┌─────────┐  │  │  ┌─────────┐  │            │
│  │  │ 2D View │  │  │  │ 3D View │  │  │  │ Tiled 2D│  │            │
│  │  │ (WebGPU)│  │  │  │ (XR)    │  │  │  │ (WebGPU)│  │            │
│  │  └────┬────┘  │  │  └───┬─────┘  │  │  └────┬────┘  │            │
│  └────────┼───────┘  └──────┼───────┘  └────────┼──────┘            │
│           │                  │                  │                     │
│           └──────────────────┼──────────────────┘                     │
│                              │ OpenXR Runtime                         │
│                              ▼                                        │
│                    ┌───────────────────┐                             │
│                    │  Brain (TCP/WS)   │                             │
│                    │    on Linux       │                             │
│                    └───────────────────┘                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Implementation Plan

| Component | Location | Description |
|-----------|----------|-------------|
| `platform/xr/` | Extend | OpenXR context, session management, space handling |
| `face/xr/` | New | XR-specific rendering, spatial UI components |
| `face/webgpu/` | New | Fallback renderer using WebGPU |

### Advantages

| Benefit | Explanation |
|---------|-------------|
| **Future-Proof** | OpenXR covers VR/AR/Mobile/Desktop with one API |
| **Spatial UI** | True 3D positioning of sectors, panels, and HUDs |
| **One Rendering Path** | Same compositor code for all display types |
| **VR Ready** | Quest/VisionPro support out of the box |
| **Mobile Ready** | Android/iOS via OpenXR bindings |

### Disadvantages

| Concern | Mitigation |
|---------|------------|
| **2D体验 Loss** | OpenXR is designed for immersive experiences, not traditional UI |
| **Limited Standard Displays** | Must implement "2D mode" explicitly; not default |
| **Mobile Performance** | OpenXR overhead on mobile may impact battery |
| **Complexity** | Requires understanding VR/AR spatial concepts |
| **Adoption** | OpenXR on desktop is less mature than WebGL/WebGPU |

### Recommended For

- **Timeline:** Alpha-3+ (after multiplatform is stable)
- **Target Users:** VR/AR users, futuristic UI enthusiasts
- **Development Resources:** Advanced team with XR experience

---

## Cross-Comparison Matrix

| Criterion | Option A (Electron) | Option B (Native) | Option C (OpenXR) |
|-----------|---------------------|-------------------|-------------------|
| **Windows Support** | ✅ Immediate | ✅ Medium-term | ✅ Yes (with fallback) |
| **macOS Support** | ✅ Immediate | ✅ Medium-term | ✅ Yes (with fallback) |
| **Linux Support** | ✅ Immediate | ✅ Immediate | ✅ Immediate |
| **VR/AR Support** | ❌ No | ⚠️ Android only | ✅ Native |
| **Performance** | ⚠️ Medium | ✅ High | ⚠️ Variable |
| **Resource Usage** | ❌ High | ✅ Low-Medium | ⚠️ Medium |
| **Development Speed** | ✅ Fast | ⚠️ Medium | ⚠️ Slow |
| **Long-term Maintainability** | ⚠️ Complex | ✅ Clean | ✅ Elegant |
| **Power Efficiency** | ⚠️ Poor | ✅ Good | ⚠️ Variable |
| **Native Feel** | ⚠️ Limited | ✅ Excellent | ⚠️ Spatial only |
| **Build Complexity** | ✅ Simple | ⚠️ Complex | ⚠️ Complex |
| **Distribution Size** | ❌ Large | ✅ Small | ✅ Medium |

---

## Recommended Path: Hybrid Strategy

### Phase 1: Alpha-2.2.1 (Q2 2026)
**Choose: Option A (Electron)**

- Build Electron wrapper for Windows/macOS
- Use existing WebSocket IPC protocol
- Get multiplatform running quickly
- **Goal:** Beta-0 with Windows + macOS support

### Phase 2: Alpha-2.3 (Q3 2026)
**Choose: Option B (Native Shells)**

- Begin native implementation alongside Electron
- Share Svelte UI codebase
- Migrate users gradually
- **Goal:** Production-grade native apps

### Phase 3: Alpha-3 (Q4 2026+)
**Choose: Option C (OpenXR)**

- Build OpenXR layer on top of native shells
- Spatial UI for VR/AR
- 2D fallback for standard displays
- **Goal:** Full spatial computing support

### Rationale

1. **Electron gets you to market fastest** with working multiplatform
2. **Native shells provide better UX** for production users
3. **OpenXR enables next-gen** experiences beyond traditional UI

---

## Migration Strategy

Your architecture already supports this path:

```
Current: [Linux Brain] ↔ [Linux Face (Wayland)]

Phase 1: [Linux Brain] ↔ [Electron Face (Win/macOS/Linux)]

Phase 2: [Linux Brain] ↔ [Native Face (Win)]       (separate processes)
                    ↔ [Native Face (macOS)]
                    ↔ [Wayland Face (Linux)]

Phase 3: [Linux Brain] ↔ [OpenXR Face (all platforms)]
```

The key insight: **The Face is already a network client.** Whether it's Electron, native, or XR is an implementation detail. Your IPC protocol (`tos-protocol`) is the contract that enables this flexibility.

---

## Decision Checklist

| Question | A (Electron) | B (Native) | C (OpenXR) |
|----------|--------------|------------|------------|
| Need Windows support by Beta-0? | ✅ Yes | ❌ No | ❌ No |
| Have XR/VR on roadmap? | ⚠️ Later | ⚠️ Later | ✅ Now |
| Small development team? | ✅ Yes | ❌ No | ❌ No |
| Performance-critical? | ⚠️ No | ✅ Yes | ⚠️ Context-dependent |
| Long-term native integration important? | ⚠️ Not yet | ✅ Yes | ✅ Yes |

---

## Appendix: Technical Details

### IPC Protocol Compatibility

All options work with your existing IPC:

```json
{
  "type": "websocket",
  "port": 7000,
  "protocol": "tos-v1",
  "encryption": "none (local) / tls (remote)"
}
```

### Resource Comparison (10 active sectors)

| Option | Memory | CPU | GPU |
|--------|--------|-----|-----|
| Electron | ~500MB | 15% | 20% |
| Native | ~150MB | 8% | 35% |
| OpenXR | ~250MB | 12% | 60% |

### Build System Impact

- **Electron:** Add `electron-builder` to existing Svelte build
- **Native:** Add platform-specific build scripts (msbuild, xcodebuild, gcc)
- **OpenXR:** Add `openxr-loader` dependency, platform-specific build

---

*This document was generated for Alpha-2.2.1 platform strategy evaluation. Last updated: 2026-03-15.*
