# --- Variables ---
CARGO        := cargo
NPM          := npm
CORE_DIR     := ./core
UI_DIR       := ./plugins/lcars-theme
BUILD_DIR    := ./target
WPE_BACKEND  := fdo

# --- Multi-Distro Dependency Logic ---
define INSTALL_DEPS
	@if [ -f /etc/arch-release ]; then \
		echo "Detected Arch Linux..."; \
		sudo pacman -S --needed rustup nodejs npm wpewebkit-fdo nushell libinput mesa wayland wayland-protocols; \
	elif [ -f /etc/fedora-release ]; then \
		echo "Detected Fedora..."; \
		sudo dnf install rustup nodejs npm wpewebkit-devel nushell libinput-devel mesa-libgbm-devel wayland-devel; \
	elif [ -f /etc/debian_version ]; then \
		echo "Detected Debian/Ubuntu..."; \
		sudo apt-get update && sudo apt-get install -y rustup nodejs npm libwpewebkit-1.0-dev nushell libinput-dev libgbm-dev libwayland-dev; \
	else \
		echo "Unsupported distribution. Please install dependencies manually."; \
	fi
endef

.PHONY: all setup-deps core ui run clean

all: ui core ## Build the entire SDE environment

setup-deps: ## Install system-level dependencies across Arch, Fedora, and Debian
	$(INSTALL_DEPS)
	rustup default stable
	rustup component add rust-src
	$(NPM) install -g wasm-pack

# --- Component Builds ---

ui: ## Build the LCARS Theme Plugin (React + WASM)
	@echo "Building UI Shell..."
	cd $(UI_DIR) && $(NPM) install && $(NPM) run build:wasm

core: ## Build the Micro-Shell Core (Rust + Smithay)
	@echo "Building Rust Spatial Compositor..."
	cd $(CORE_DIR) && $(CARGO) build --release

# --- Execution ---

run: all ## Launch the SDE with the WPE host active
	WPE_BACKEND=$(WPE_BACKEND) ./$(BUILD_DIR)/release/sde-core --plugin $(UI_DIR)/dist

clean:
	rm -rf $(BUILD_DIR)
	cd $(UI_DIR) && rm -rf dist node_modules
	