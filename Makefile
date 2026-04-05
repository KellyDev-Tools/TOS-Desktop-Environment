# TOS Beta-0 Build System
# High-Fidelity OS Pipeline

.PHONY: help build-all build-brain build-faces build-face-web build-face-electron build-protocol build-services \
        check check-brain fmt lint docs \
        test test-all test-core test-shell test-ai test-sec test-system test-brain-component test-ui-component test-self test-e2e test-health \
        run run-web run-web-dev dev-web clean \
        android-check android-build android-release clean-android android-test \
        pack-face-electron-win pack-face-electron-linux pack-face-electron-mac

# -----------------------------------------------------------------------------
# 0. INFRASTRUCTURE & HOOKS
# -----------------------------------------------------------------------------

PRE_COMMIT_HOOK := .git/hooks/pre-commit

$(PRE_COMMIT_HOOK): scripts/pre-commit.sh
	@mkdir -p .git/hooks
	@cp scripts/pre-commit.sh $(PRE_COMMIT_HOOK)
	@chmod +x $(PRE_COMMIT_HOOK)
	@echo "[TOS] Pre-commit hooks updated."

install-hooks: $(PRE_COMMIT_HOOK)

# -----------------------------------------------------------------------------
# 1. HELP & DISCOVERY
# -----------------------------------------------------------------------------

help:
	@echo "\033[1;36mTOS BETA-0 BUILD SYSTEM\033[0m"
	@echo ""
	@echo "\033[1;33mIndependent Build Targets:\033[0m"
	@echo "  make build-all       Compile the entire workspace (Auto-installs hooks)"
	@echo "  make build-brain     Compile the core Brain process only"
	@echo "  make build-faces     Compile all active face implementations"
	@echo "  make build-face-web      Compile the Svelte web-based Face"
	@echo "  make build-face-electron Compile the Electron desktop Face"
	@echo "  make build-common        Compile the shared common crate"
	@echo "  make build-services      Compile all auxiliary daemons"
	@echo ""
	@echo "\033[1;33mDevelopment Targets:\033[0m"
	@echo "  make check           Fast workspace verification (cargo check)"
	@echo "  make check-brain     Fast check of Brain core only"
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
	@echo "  make test-self test-e2e            Internal Brain Self-Test Sequence"
	@echo "  make test-health          Verify orchestration reachability (Tier 5)"
	@echo ""
	@echo "\033[1;33mExecution Targets:\033[0m"
	@echo "  make run             Direct launch of TOS Brain + Terminal Face"
	@echo "  make run-web         Orchestrate full stack (Brain + Web UI Server)"
	@echo "  make run-services    Spawn auxiliary background daemons"
	@echo ""
	@echo "\033[1;33mHandheld / Spatial Platforms:\033[0m"
	@echo "  make android-check   Check Android Face crate (host target)"
	@echo "  make android-build   Check + instructions for cross-compile"
	@echo "  make android-release Cross-compile release APK"
	@echo "  make android-test    Run Android Face tests"
	@echo ""
	@echo "\033[1;33mElectron Packaging:\033[0m"
	@echo "  make pack-face-electron-win   Bundle Electron Face for Windows (.exe)"
	@echo "  make pack-face-electron-linux Bundle Electron Face for Linux (.AppImage)"
	@echo "  make pack-face-electron-mac   Bundle Electron Face for macOS (.dmg)"
	@echo ""
	@echo "\033[1;33mMaintenance:\033[0m"
	@echo "  make clean           Purge build artifacts and logs"

# -----------------------------------------------------------------------------
# 2. CORE DEVELOPMENT
# -----------------------------------------------------------------------------

build-all: $(PRE_COMMIT_HOOK) build-common build-brain build-services
	cd face-wayland-linux && cargo build
	cd face-android-handheld && cargo build

build-brain:
	cd brain && cargo build --bin tos-brain

build-faces: build-face-web build-face-electron android-check

build-face-web:
	@echo "[TOS] Building Svelte Face UI..."
	@$(NVM_INIT) && cd face-svelte-ui && npm run build
	@echo "[TOS] Svelte Face UI: BUILD COMPLETE"

build-face-electron: build-face-web
	@echo "[TOS] Building Electron Face Container..."
	cd face-electron-any && npm install && npm run build

pack-face-electron-win: build-face-electron
	@echo "[TOS] Packaging Electron Face for Windows..."
	cd face-electron-any && npm run pack:win

pack-face-electron-linux: build-face-electron
	@echo "[TOS] Packaging Electron Face for Linux..."
	cd face-electron-any && npm run pack:linux

pack-face-electron-mac: build-face-electron
	@echo "[TOS] Packaging Electron Face for macOS..."
	cd face-electron-any && npm run pack:mac

build-common:
	cd tos-common && cargo build

build-services:
	cd tos-settingsd && cargo build
	cd tos-loggerd && cargo build
	cd tos-marketplaced && cargo build
	cd tos-priorityd && cargo build
	cd tos-sessiond && cargo build
	cd tos-heuristicd && cargo build
	cd tos-searchd && cargo build

check: $(PRE_COMMIT_HOOK)
	cd tos-common && cargo check
	cd brain && cargo check
	cd tos-settingsd && cargo check
	cd tos-loggerd && cargo check
	cd tos-marketplaced && cargo check
	cd tos-priorityd && cargo check
	cd tos-sessiond && cargo check
	cd tos-heuristicd && cargo check
	cd tos-searchd && cargo check
	cd face-wayland-linux && cargo check
	cd face-android-handheld && cargo check

check-brain:
	cd brain && cargo check --bin tos-brain

fmt:
	cd tos-common && cargo fmt
	cd brain && cargo fmt
	cd face-wayland-linux && cargo fmt
	cd face-android-handheld && cargo fmt

lint:
	cd tos-common && cargo clippy -- -D warnings
	cd brain && cargo clippy -- -D warnings
	cd face-wayland-linux && cargo clippy -- -D warnings
	cd face-android-handheld && cargo clippy -- -D warnings

docs:
	cd tos-common && cargo doc --no-deps
	cd brain && cargo doc --no-deps
	cd face-wayland-linux && cargo doc --no-deps
	cd face-android-handheld && cargo doc --no-deps

# -----------------------------------------------------------------------------
# 3. TEST SUITE
# -----------------------------------------------------------------------------

test: test-all

test-all: $(PRE_COMMIT_HOOK) test-common test-core test-shell test-search

test-common:
	cd tos-common && cargo test

test-core:
	cd brain && cargo test --test brain_core

test-shell:
	cd brain && cargo test --test shell_integration

test-search:
	cd tos-searchd && cargo test --test search_integration

test-system:
	@mkdir -p logs
	cd brain && cargo run --bin system_test | tee ../logs/system_test.log

test-brain-component:
	@echo "[TOS] Orchestrating Component Test..."
	@mkdir -p logs
	@rm -f logs/brain_node.log
	cd brain && cargo run --bin brain_node > ../logs/brain_node.log 2>&1 & BR_PID=$$!; \
	echo "[TOS] Waiting for Brain Node boot..."; \
	sleep 5; \
	echo "[TOS] Triggering Stimulator..."; \
	cd brain && cargo test --test stimulator_brain_node -- --nocapture; \
	echo "[TOS] Terminating simulation..."; \
	kill $$BR_PID; \
	echo "Component Test Complete. Analysis: 'logs/brain_node.log'"

test-ui-component:
	@echo "[TOS] Launching UI Component Paces (Playwright)..."
	@npm install @playwright/test
	@npx playwright install chromium
	@npx playwright test tests/ui_component.spec.ts

test-health:
	@echo "[TOS] Running Orchestration Health Check..."
	@$(MAKE) build-services build-brain
	@echo "[TOS] Booting Auxiliary Daemons and Brain Core..."
	@pkill -x tos-brain || true
	@pkill -x tos-settingsd || true
	@pkill -x tos-loggerd || true
	@pkill -x tos-marketplace || pkill -x tos-marketplaced || true
	@pkill -x tos-priorityd || true
	@pkill -x tos-sessiond || true
	@pkill -x tos-heuristicd || true
	@pkill -x tos-searchd || true
	@mkdir -p logs
	@tos-settingsd/target/debug/tos-settingsd > logs/settingsd.log 2>&1 &
	@tos-loggerd/target/debug/tos-loggerd > logs/loggerd.log 2>&1 &
	@tos-marketplaced/target/debug/tos-marketplaced > logs/marketplaced.log 2>&1 &
	@tos-priorityd/target/debug/tos-priorityd > logs/priorityd.log 2>&1 &
	@tos-sessiond/target/debug/tos-sessiond > logs/sessiond.log 2>&1 &
	@tos-heuristicd/target/debug/tos-heuristicd > logs/heuristicd.log 2>&1 &
	@tos-searchd/target/debug/tos-searchd > logs/searchd.log 2>&1 || true &
	@brain/target/debug/tos-brain --headless > logs/tos-brain.log 2>&1 & BR_PID=$$!; \
	echo "[TOS] Waiting for daemons and Discovery Gate to bind (3s)..."; \
	sleep 3; \
	cd tests && cargo test --test service_orchestration -- --nocapture; TEST_RES=$$?; \
	echo "[TOS] Cleaning up Orchestration Environment..."; \
	kill $$BR_PID 2>/dev/null || true; \
	pkill -x tos-settingsd || true; \
	pkill -x tos-loggerd || true; \
	pkill -x tos-marketplace || pkill -x tos-marketplaced || true; \
	pkill -x tos-priorityd || true; \
	pkill -x tos-sessiond || true; \
	pkill -x tos-heuristicd || true; \
	pkill -x tos-searchd || true; \
	exit $$TEST_RES

# -----------------------------------------------------------------------------
# 4. EXECUTION
# -----------------------------------------------------------------------------

# --- NVM Helper (Node v20 required for Svelte) ---
NVM_INIT = export NVM_DIR="$$HOME/.nvm" && [ -s "$$NVM_DIR/nvm.sh" ] && . "$$NVM_DIR/nvm.sh" && nvm use 20 --silent

run: $(PRE_COMMIT_HOOK) run-services
	@mkdir -p logs
	@pkill -x tos-brain || true
	cd brain && cargo run --bin tos-brain | tee ../logs/tos-brain.log

dev-web:
	@echo "[TOS] Starting Svelte Face Dev Server (HMR)..."
	@$(NVM_INIT) && cd face-svelte-ui && npm run dev -- --port 8080 --host 0.0.0.0

run-web: run-services build-face-web
	@mkdir -p logs
	@pkill -x tos-brain || true
	@pkill -f "[h]ttp.server 8080" || true
	@echo "[TOS] Initializing Svelte Face Server (8080)..."
	@python3 -m http.server 8080 -d face-svelte-ui/build > logs/web_ui.log 2>&1 & WEB_PID=$$!; \
	echo "[TOS] Synchronizing Brain Core (7000/7001)..."; \
	trap "kill $$WEB_PID; pkill -x tos-brain; exit" EXIT INT TERM; \
	cd brain && cargo run --bin tos-brain -- --headless 2>&1 | tee ../logs/tos-brain.log

run-web-dev: run-services
	@mkdir -p logs
	@pkill -x tos-brain || true
	@echo "[TOS] Starting Svelte Dev Server + Brain Core..."
	@($(NVM_INIT) && cd face-svelte-ui && npm run dev -- --port 8080 --host 0.0.0.0) > logs/svelte_dev.log 2>&1 & SVELTE_PID=$$!; \
	echo "[TOS] Synchronizing Brain Core (7000/7001)..."; \
	trap "kill $$SVELTE_PID; pkill -x tos-brain; exit" EXIT INT TERM; \
	cd brain && cargo run --bin tos-brain -- --headless 2>&1 | tee ../logs/tos-brain.log

run-services:
	@echo "[TOS] Initializing Auxiliary Daemons..."
	@mkdir -p logs
	@pkill -x tos-settingsd || true
	@pkill -x tos-loggerd || true
	@pkill -x tos-marketplace || pkill -x tos-marketplaced || true
	@pkill -x tos-priorityd || true
	@pkill -x tos-sessiond || true
	@pkill -x tos-heuristicd || true
	@pkill -x tos-searchd || true
	cd tos-settingsd && cargo build
	cd tos-loggerd && cargo build
	cd tos-marketplaced && cargo build
	cd tos-priorityd && cargo build
	cd tos-sessiond && cargo build
	cd tos-heuristicd && cargo build
	cd tos-searchd && cargo build
	@tos-settingsd/target/debug/tos-settingsd > logs/settingsd.log 2>&1 &
	@tos-loggerd/target/debug/tos-loggerd > logs/loggerd.log 2>&1 &
	@tos-marketplaced/target/debug/tos-marketplaced > logs/marketplaced.log 2>&1 &
	@tos-priorityd/target/debug/tos-priorityd > logs/priorityd.log 2>&1 &
	@tos-sessiond/target/debug/tos-sessiond > logs/sessiond.log 2>&1 &
	@tos-heuristicd/target/debug/tos-heuristicd > logs/heuristicd.log 2>&1 &
	@tos-searchd/target/debug/tos-searchd > logs/searchd.log 2>&1 || true &
	@echo "[TOS] Auxiliary Constellation: ONLINE"

# -----------------------------------------------------------------------------
# 5. MAINTENANCE
# -----------------------------------------------------------------------------

clean:
	cd tos-common && cargo clean
	cd brain && cargo clean
	cd tos-settingsd && cargo clean
	cd tos-loggerd && cargo clean
	cd tos-marketplaced && cargo clean
	cd tos-priorityd && cargo clean
	cd tos-sessiond && cargo clean
	cd tos-heuristicd && cargo clean
	cd tos-searchd && cargo clean
	cd face-wayland-linux && cargo clean
	cd face-android-handheld && cargo clean
	cd tests && cargo clean
	rm -rf logs/
	rm -rf face-svelte-ui/build/ face-svelte-ui/.svelte-kit/

test-e2e:
	@echo "[TOS] Launching Full-Stack E2E Paces (Playwright)..."
	@$(NVM_INIT) && cd face-svelte-ui && npx playwright test -c playwright.e2e.config.ts

# -----------------------------------------------------------------------------
# 6. ANDROID BUILD (separate crate: android-handheld/)
# -----------------------------------------------------------------------------

ANDROID_CRATE := face-android-handheld

android-check:
	@echo "[TOS] Checking Android Face crate (host target)..."
	cd face-android-handheld && cargo check
	@echo "[TOS] Android Face: CHECK PASSED"

android-build: android-check
	@echo "[TOS] Android Face: Host check passed."
	@echo "[TOS] To cross-compile for a real device, run:"
	@echo "  cd face-android-handheld && cargo ndk -t arm64-v8a build --release"
	@echo "[TOS] Requires: cargo install cargo-ndk && rustup target add aarch64-linux-android"

android-release:
	@echo "[TOS] Building Android Face (Release, arm64-v8a)..."
	cd face-android-handheld && /home/tim/gradle/gradle-8.5/bin/gradle assembleRelease
	@echo "[TOS] Android Face: RELEASE BUILD COMPLETE"
	@echo "[TOS] APK located at: face-android-handheld/build/outputs/apk/release/face-android-handheld-release-unsigned.apk"

android-test:
	@echo "[TOS] Running Android Face tests (host target)..."
	cd face-android-handheld && cargo test
	@echo "[TOS] Android Face: TESTS PASSED"

clean-android:
	@echo "[TOS] Cleaning Android Face build artifacts..."
	cd face-android-handheld && cargo clean
	@echo "[TOS] Android Face: CLEAN"

# -----------------------------------------------------------------------------
# 7. RELEASE ORCHESTRATION (§4.2)
# -----------------------------------------------------------------------------

release:
	@echo "[TOS] Orchestrating Canonical Release (Tarball)..."
	bash scripts/release.sh

release-all:
	@echo "[TOS] Orchestrating Multi-Platform Release (Tar, Deb, Arch, Android)..."
	bash scripts/release.sh --all
