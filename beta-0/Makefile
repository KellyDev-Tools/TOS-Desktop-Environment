# TOS Beta-0 Build System
# High-Fidelity OS Pipeline

.PHONY: help build-all build-brain build-faces build-face-web build-protocol build-services \
        check check-brain fmt lint docs \
        test test-all test-core test-shell test-ai test-sec test-system test-brain-component test-ui-component test-self test-e2e test-health \
        run run-web run-web-dev dev-web clean \
        android-check android-build android-build-release clean-android android-test

# -----------------------------------------------------------------------------
# 1. HELP & DISCOVERY
# -----------------------------------------------------------------------------

help:
	@echo "\033[1;36mTOS BETA-0 BUILD SYSTEM\033[0m"
	@echo ""
	@echo "\033[1;33mIndependent Build Targets:\033[0m"
	@echo "  make build-all       Compile the entire workspace"
	@echo "  make build-brain     Compile the core Brain process only"
	@echo "  make build-faces     Compile all active face implementations"
	@echo "  make build-face-web  Compile the Svelte web-based Face"
	@echo "  make build-protocol  Compile the shared protocol crate"
	@echo "  make build-services   Compile all auxiliary daemons"
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
	@echo "\033[1;33mMaintenance:\033[0m"
	@echo "  make clean           Purge build artifacts and logs"

# -----------------------------------------------------------------------------
# 2. CORE DEVELOPMENT
# -----------------------------------------------------------------------------

build-all:
	cargo build

build-brain:
	cargo build --bin tos-brain

build-faces: build-face-web android-check

build-face-web:
	@echo "[TOS] Building Svelte Face UI..."
	@$(NVM_INIT) && cd svelte_ui && npm run build
	@echo "[TOS] Svelte Face UI: BUILD COMPLETE"

build-protocol:
	cargo build -p tos-protocol

build-services:
	cargo build --bins

check:
	cargo check

check-brain:
	cargo check --bin tos-brain

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
	@target/debug/tos-settingsd > logs/settingsd.log 2>&1 &
	@target/debug/tos-loggerd > logs/loggerd.log 2>&1 &
	@target/debug/tos-marketplaced > logs/marketplaced.log 2>&1 &
	@target/debug/tos-priorityd > logs/priorityd.log 2>&1 &
	@target/debug/tos-sessiond > logs/sessiond.log 2>&1 &
	@target/debug/tos-heuristicd > logs/heuristicd.log 2>&1 &
	@target/debug/tos-searchd > logs/searchd.log 2>&1 &
	@target/debug/tos-brain --headless > logs/tos-brain.log 2>&1 & BR_PID=$$!; \
	echo "[TOS] Waiting for daemons and Discovery Gate to bind (3s)..."; \
	sleep 3; \
	cargo test --test service_orchestration -- --nocapture; TEST_RES=$$?; \
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

run: run-services
	@mkdir -p logs
	@pkill -x tos-brain || true
	cargo run --bin tos-brain | tee logs/tos-brain.log

dev-web:
	@echo "[TOS] Starting Svelte Face Dev Server (HMR)..."
	@$(NVM_INIT) && cd svelte_ui && npm run dev -- --port 8080 --host 0.0.0.0

run-web: run-services build-face-web
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
	@pkill -x tos-marketplace || pkill -x tos-marketplaced || true
	@pkill -x tos-priorityd || true
	@pkill -x tos-sessiond || true
	@pkill -x tos-heuristicd || true
	@pkill -x tos-searchd || true
	@cargo build --bins
	@target/debug/tos-settingsd > logs/settingsd.log 2>&1 &
	@target/debug/tos-loggerd > logs/loggerd.log 2>&1 &
	@target/debug/tos-marketplaced > logs/marketplaced.log 2>&1 &
	@target/debug/tos-priorityd > logs/priorityd.log 2>&1 &
	@target/debug/tos-sessiond > logs/sessiond.log 2>&1 &
	@target/debug/tos-heuristicd > logs/heuristicd.log 2>&1 &
	@target/debug/tos-searchd > logs/searchd.log 2>&1 &
	@echo "[TOS] Auxiliary Constellation: ONLINE"

# -----------------------------------------------------------------------------
# 5. MAINTENANCE
# -----------------------------------------------------------------------------

clean:
	cargo clean
	rm -rf logs/
	rm -rf svelte_ui/build/ svelte_ui/.svelte-kit/

test-e2e:
	@echo "[TOS] Launching Full-Stack E2E Paces (Playwright)..."
	@$(NVM_INIT) && cd svelte_ui && npx playwright test -c playwright.e2e.config.ts

# -----------------------------------------------------------------------------
# 6. ANDROID BUILD (separate crate: tos-android/)
# -----------------------------------------------------------------------------

ANDROID_CRATE := tos-android

android-check:
	@echo "[TOS] Checking Android Face crate (host target)..."
	cargo check -p $(ANDROID_CRATE)
	@echo "[TOS] Android Face: CHECK PASSED"

android-build: android-check
	@echo "[TOS] Android Face: Host check passed."
	@echo "[TOS] To cross-compile for a real device, run:"
	@echo "  cargo ndk -t arm64-v8a build -p $(ANDROID_CRATE) --release"
	@echo "[TOS] Requires: cargo install cargo-ndk && rustup target add aarch64-linux-android"

android-build-release:
	@echo "[TOS] Building Android Face (Release, arm64-v8a)..."
	cargo ndk -t arm64-v8a build -p $(ANDROID_CRATE) --release
	@echo "[TOS] Android Face: RELEASE BUILD COMPLETE"

android-test:
	@echo "[TOS] Running Android Face tests (host target)..."
	cargo test -p $(ANDROID_CRATE)
	@echo "[TOS] Android Face: TESTS PASSED"

clean-android:
	@echo "[TOS] Cleaning Android Face build artifacts..."
	cargo clean -p $(ANDROID_CRATE)
	@echo "[TOS] Android Face: CLEAN"
