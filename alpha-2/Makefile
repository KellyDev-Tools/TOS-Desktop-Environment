# TOS Alpha-2 Build System
# High-Fidelity OS Pipeline

.PHONY: help build check fmt lint docs test test-all test-core test-shell test-ai test-sec test-system test-brain-component test-ui-component test-self run run-web run-web-dev build-web dev-web clean electron-setup electron-build electron-dev build-electron-win build-electron-macos build-electron-linux build-electron-all test-electron

# -----------------------------------------------------------------------------
# 1. HELP & DISCOVERY
# -----------------------------------------------------------------------------

help:
	@echo "\033[1;36mTOS ALPHA-2.2 BUILD SYSTEM\033[0m"
	@echo ""
	@echo "\033[1;33mDevelopment Targets:\033[0m"
	@echo "  make build           Compile the core Brain processes"
	@echo "  make check           Fast project verification (cargo check)"
	@echo "  make fmt             Initialize code formatting"
	@echo "  make lint            Run static analysis (Clippy)"
	@echo "  make docs            Generate local developer documentation"
	@echo ""
	@echo "\033[1;33mTesting Tier (Unit & Logic):\033[0m"
	@echo "  make test            Run the default test suite (test-all)"
	@echo "  make test-core       Logic tests for the Brain core state machine"
	@echo "  make test-shell      PTY and Shell integration tests"
	@echo "  make test-ai         AI Engine and Contextual intent tests"
	@echo "  make test-sec        Security manifest and privilege tests"
	@echo ""
	@echo "\033[1;33mTesting Tier (Integration & UI):\033[0m"
	@echo "  make test-system          Single-process comprehensive system test"
	@echo "  make test-brain-component  Two-process Stimulator/Node test"
	@echo "  make test-ui-component     Playwright-based UI Component verification"
	@echo "  make test-self            Internal Brain Self-Test Sequence"
	@echo "  make test-health          Verify orchestration reachability (Tier 5)"
	@echo ""
	@echo "\033[1;33mExecution Targets:\033[0m"
	@echo "  make run             Direct launch of TOS Brain + Terminal Face"
	@echo "  make run-web         Orchestrate full stack (Brain + Web UI Server)"
	@echo "  make run-services    Spawn auxiliary background daemons"
	@echo ""
	@echo "\033[1;33mElectron Platform:\033[0m"
	@echo "  make electron-setup        Install Electron dependencies"
	@echo "  make electron-build        Compile Electron main process + Svelte UI"
	@echo "  make electron-dev          Launch Electron in dev mode"
	@echo "  make build-electron-win    Package for Windows (NSIS + portable)"
	@echo "  make build-electron-macos  Package for macOS (DMG + zip)"
	@echo "  make build-electron-linux  Package for Linux (AppImage + deb + rpm)"
	@echo "  make build-electron-all    Package for all platforms"
	@echo ""
	@echo "\033[1;33mMaintenance:\033[0m"
	@echo "  make clean           Purge build artifacts and logs"

# -----------------------------------------------------------------------------
# 2. CORE DEVELOPMENT
# -----------------------------------------------------------------------------

build:
	cargo build

check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

docs:
	cargo doc --no-deps --open

# -----------------------------------------------------------------------------
# 3. TEST SUITE
# -----------------------------------------------------------------------------

test: test-all

test-all: test-core test-shell test-ai test-sec test-health

test-core:
	cargo test --test brain_core

test-shell:
	cargo test --test shell_integration

test-ai:
	cargo test --test ai_integration

test-sec:
	cargo test --test sandbox
	cargo test --test security_manifest

test-system:
	@mkdir -p logs
	cargo run --bin system_test | tee logs/system_test.log

test-brain-component:
	@echo "[TOS] Orchestrating Component Test..."
	@mkdir -p logs
	@rm -f logs/brain_node.log
	cargo run --bin brain_node > logs/brain_node.log 2>&1 & BR_PID=$$!; \
	echo "[TOS] Waiting for Brain Node boot..."; \
	sleep 5; \
	echo "[TOS] Triggering Stimulator..."; \
	cargo test --test stimulator_brain_node -- --nocapture; \
	echo "[TOS] Terminating simulation..."; \
	kill $$BR_PID; \
	echo "Component Test Complete. Analysis: 'logs/brain_node.log'"

test-ui-component:
	@echo "[TOS] Launching UI Component Paces (Playwright)..."
	@npm install @playwright/test
	@npx playwright install chromium
	@npx playwright test tests/ui_component.spec.js

test-self:
	cargo run -- --self-test

test-health:
	@echo "[TOS] Running Orchestration Health Check..."
	cargo test --test service_orchestration -- --nocapture

# -----------------------------------------------------------------------------
# 4. EXECUTION
# -----------------------------------------------------------------------------

# --- NVM Helper (Node v20 required for Svelte) ---
NVM_INIT = export NVM_DIR="$$HOME/.nvm" && [ -s "$$NVM_DIR/nvm.sh" ] && . "$$NVM_DIR/nvm.sh" && nvm use 20 --silent

run: run-services
	@mkdir -p logs
	@pkill -x tos-brain || true
	cargo run --bin tos-brain | tee logs/tos-brain.log

build-web:
	@echo "[TOS] Building Svelte Face UI..."
	@$(NVM_INIT) && cd svelte_ui && npm run build
	@echo "[TOS] Svelte Face UI: BUILD COMPLETE → svelte_ui/build/"

dev-web:
	@echo "[TOS] Starting Svelte Face Dev Server (HMR)..."
	@$(NVM_INIT) && cd svelte_ui && npm run dev -- --port 8080 --host 0.0.0.0

run-web: run-services build-web
	@mkdir -p logs
	@pkill -x tos-brain || true
	@pkill -f "[h]ttp.server 8080" || true
	@echo "[TOS] Initializing Svelte Face Server (8080)..."
	@python3 -m http.server 8080 -d svelte_ui/build > logs/web_ui.log 2>&1 & WEB_PID=$$!; \
	echo "[TOS] Synchronizing Brain Core (7000/7001)..."; \
	trap "kill $$WEB_PID; pkill -x tos-brain; exit" EXIT INT TERM; \
	cargo run --bin tos-brain -- --headless 2>&1 | tee logs/tos-brain.log

run-web-dev: run-services
	@mkdir -p logs
	@pkill -x tos-brain || true
	@echo "[TOS] Starting Svelte Dev Server + Brain Core..."
	@($(NVM_INIT) && cd svelte_ui && npm run dev -- --port 8080 --host 0.0.0.0) > logs/svelte_dev.log 2>&1 & SVELTE_PID=$$!; \
	echo "[TOS] Synchronizing Brain Core (7000/7001)..."; \
	trap "kill $$SVELTE_PID; pkill -x tos-brain; exit" EXIT INT TERM; \
	cargo run --bin tos-brain -- --headless 2>&1 | tee logs/tos-brain.log

run-services:
	@echo "[TOS] Initializing Auxiliary Daemons..."
	@mkdir -p logs
	@pkill -x tos-settingsd || true
	@pkill -x tos-loggerd || true
	@pkill -x tos-marketplaced || true
	@pkill -x tos-priorityd || true
	@pkill -x tos-sessiond || true
	@pkill -x tos-heuristicd || true
	@pkill -x tos-searchd || true
	@cargo run --bin tos-settingsd > logs/settingsd.log 2>&1 &
	@cargo run --bin tos-loggerd > logs/loggerd.log 2>&1 &
	@cargo run --bin tos-marketplaced > logs/marketplaced.log 2>&1 &
	@cargo run --bin tos-priorityd > logs/priorityd.log 2>&1 &
	@cargo run --bin tos-sessiond > logs/sessiond.log 2>&1 &
	@cargo run --bin tos-heuristicd > logs/heuristicd.log 2>&1 &
	@cargo run --bin tos-searchd > logs/searchd.log 2>&1 &
	@echo "[TOS] Auxiliary Constellation: ONLINE"

# -----------------------------------------------------------------------------
# 5. ELECTRON PLATFORM
# -----------------------------------------------------------------------------

ELECTRON_DIR = src/platform/electron

electron-setup:
	@echo "[TOS] Installing Electron dependencies..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm install
	@echo "[TOS] Electron dependencies: INSTALLED"

electron-build: electron-setup build-web
	@echo "[TOS] Building Electron main process..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm run build
	@echo "[TOS] Electron main process: BUILT"

electron-dev: electron-setup
	@echo "[TOS] Starting Electron in dev mode..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm run dev

build-electron-win: electron-build
	@echo "[TOS] Packaging Electron for Windows..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm run pack:win
	@echo "[TOS] Windows package: COMPLETE → $(ELECTRON_DIR)/release/"

build-electron-macos: electron-build
	@echo "[TOS] Packaging Electron for macOS..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm run pack:mac
	@echo "[TOS] macOS package: COMPLETE → $(ELECTRON_DIR)/release/"

build-electron-linux: electron-build
	@echo "[TOS] Packaging Electron for Linux..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm run pack:linux
	@echo "[TOS] Linux package: COMPLETE → $(ELECTRON_DIR)/release/"

build-electron-all: electron-build
	@echo "[TOS] Packaging Electron for ALL platforms..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm run pack:all
	@echo "[TOS] All-platform packages: COMPLETE → $(ELECTRON_DIR)/release/"

test-electron: electron-setup
	@echo "[TOS] Running Electron test suite..."
	@$(NVM_INIT) && cd $(ELECTRON_DIR) && npm test
	@echo "[TOS] Electron tests: COMPLETE"

# -----------------------------------------------------------------------------
# 6. MAINTENANCE
# -----------------------------------------------------------------------------

clean:
	cargo clean
	rm -rf logs/
	rm -rf svelte_ui/build/ svelte_ui/.svelte-kit/
	rm -rf $(ELECTRON_DIR)/dist/ $(ELECTRON_DIR)/release/ $(ELECTRON_DIR)/node_modules/

