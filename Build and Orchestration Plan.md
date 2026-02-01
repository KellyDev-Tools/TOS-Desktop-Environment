Building a high-performance spatial desktop environment requires coordinating multiple domains: Rust (system/compositor), Node.js/WASM (UI frontend), and Shell scripts (integration).
Based on the architectural specifications, the project is structured as a Hybrid Pluggable Micro-Shell. Below is the make_definition.md which outlines a centralized Makefile strategy to manage the lifecycle of the LCARS SDE.
make_definition.md: Build & Orchestration Plan
1. Directory Context
The Makefile will reside at the root of the /sde-workspace. It targets three primary sub-modules:
Core: The Rust-based Smithay compositor.
Plugins: The React + WASM LCARS interface.
Bridge: The Unix socket and JSON-RPC layer.
2. Makefile specifications
'''
# Variables
CARGO := cargo
NPM := npm
CORE_DIR := ./core
UI_DIR := ./plugins/lcars-theme
BUILD_DIR := ./target
WPE_BACKEND := fdo

.PHONY: all core ui clean run setup-deps

all: ui core ## Build the entire SDE environment

# --- Setup & Dependencies ---
setup-deps: ## Install system-level dependencies (Arch/Fedora)
	sudo dnf install smithay-devel wpewebkit-devel nushell libinput-devel

# --- UI Layer (React + WASM) ---
# Renders the "Face" of the OS
ui:
	@echo "Building LCARS UI Shell..."
	cd $(UI_DIR) && $(NPM) install && $(NPM) run build:wasm

# --- Core Layer (Rust + Smithay) ---
# Manages the Spatial Matrix and Wayland protocols
core:
	@echo "Building Rust Micro-Shell Core..."
	cd $(CORE_DIR) && $(CARGO) build --release

# --- Orchestration ---
run: all ## Launch the SDE in a nested Wayland window for testing
	# Launches the compositor which then triggers WPE WebKit
	WPE_BACKEND=$(WPE_BACKEND) ./$(BUILD_DIR)/release/sde-core --plugin $(UI_DIR)/dist

clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	cd $(UI_DIR) && rm -rf dist node_modules
'''
3. Build & Integration Strategy
A. The Dependency Chain
To achieve the "Prezi-style" zooming, the build must be strictly sequenced:
UI/WASM Asset Generation: The LCARS elbows and HUD elements must be compiled into optimized assets that the WPE WebKit host can render at 60fps.
Rust Compilation: The compositor links the sde-executor (Nushell logic) and the sde-bridge (JSON-RPC).
WPE Linking: The fdo backend is initialized to allow the Rust compositor to "grab" UI frames as textures.
B. Development vs. Production Modes
Development: The Makefile should support a dev-ui target that runs a hot-reloading React server, allowing for real-time LCARS CSS adjustments without restarting the Rust core.
Production: The all target performs LTO (Link Time Optimization) on the Rust core and minifies the UI to ensure the "Infinite Canvas" remains fluid during z-axis scaling.
4. Automation Logic for the "Four Pillars"
The Makefile coordinates the Four Pillars identified in your architecture:
Pillar Build Task Verification
Kernel/Input Link libinput Check for gesture parsing capability.
Compositor Compile Smithay/Rust Validate Global Scene Graph (x, y, z) initialization.
UI Renderer Build React/WPE Ensure "Layer Shell" HUD transparency is functional.
Logic Bundle Nushell scripts Confirm JSON-RPC bridge responds to system queries.
