This plan outlines a phased approach to developing the Tactical Operating System (TOS) within a WSL2 environment, focusing on the specific architectural requirements of your Rust-based Wayland compositor.
🛠️ Phase 1: Environment Orchestration
The foundation of the build process relies on maximizing the performance of the WSL2 kernel for intensive Rust compilation.
 * Install WSL2: Ensure you are running the latest version by executing wsl --install in PowerShell to enable WSLg (essential for the TOS GUI).
 * Linux Distribution: Install Ubuntu 24.04 LTS or Fedora from the Microsoft Store.
 * Hardware Acceleration: Install the latest Windows GPU drivers (NVIDIA/AMD/Intel) to enable vGPU passthrough, allowing TOS to access hardware acceleration for its LCARS-style rendering.
 * System Optimization: Create a .wslconfig file in your Windows user profile to allocate at least 50% of system RAM and specify CPU cores for the Linux kernel.
🦀 Phase 2: Toolchain & Dependency Injection
Set up the Rust environment and the necessary libraries for Wayland development.
 * Rust Installation: Use curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh to install the Rust toolchain.
 * Wayland Dependencies: Install development headers for Wayland, Vulkan, and OpenGL (e.g., libwayland-dev, libegl1-mesa-dev, and libxkbcommon-dev).
 * Integrated Development: Use VS Code with the WSL Extension to bridge your Windows UI with the Linux-native toolchain.
🏗️ Phase 3: TOS Architecture Setup
Initialize the project structure based on the v1.2 Platform Abstraction.
 * Crate Structure: Organize your Rust workspace into a core library (platform-agnostic) and a linux-wayland backend.
 * Trait Implementation: Begin implementing the Renderer, InputSource, and SystemServices traits for the Linux backend.
 * Wayland Socket Routing: Configure your environment to point to the WSLg socket by setting export XDG_RUNTIME_DIR=/tmp/tos-runtime and ensuring your compositor can bind to it.
🚀 Phase 4: Iterative Development & Validation
Focus on the unique "Three-Level Hierarchy" and input-agnostic philosophy.
 * Command Hub Prototype: Develop Level 2 (Command Hub) with a focus on the Persistent Unified Prompt.
 * AI Backend Integration: Install Ollama within WSL2 to test the AI Mode of the prompt.
 * Tactical Bezel Testing: Render the immutable system overlay (Level 3) using Wayland's subsurfaces to ensure it stays above applications.
 * Performance Monitoring: Utilize the Tactical Alert system to monitor frame drops within the WSLg environment, adjusting the .wslconfig if resource contention occurs.
📦 Phase 5: Distribution & Logging
Finalize the system for local testing and debugging.
 * TOS Log Implementation: Set up the SQLite-based event history to record commands and lifecycle events in ~/.local/share/tos/logs/.
 * Deployment: Package your binaries for the Debian/Ubuntu system using the APT interface.
Would you like me to export this entire plan into a .md file format that you can download?
