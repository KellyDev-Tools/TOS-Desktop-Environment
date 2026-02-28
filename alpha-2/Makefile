# TOS Alpha-2 Build System
# High-Fidelity OS Pipeline

.PHONY: help build check fmt lint docs test test-all test-core test-shell test-ai test-sec test-system test-brain-component test-ui-component test-self run run-web clean

# -----------------------------------------------------------------------------
# 1. HELP & DISCOVERY
# -----------------------------------------------------------------------------

help:
	@echo "\033[1;36mTOS ALPHA-2.1 BUILD SYSTEM\033[0m"
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
	@echo ""
	@echo "\033[1;33mExecution Targets:\033[0m"
	@echo "  make run             Direct launch of TOS Brain + Terminal Face"
	@echo "  make run-web         Orchestrate full stack (Brain + Web UI Server)"
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

test-all: test-core test-shell test-ai test-sec

test-core:
	cargo test --test tier1_brain

test-shell:
	cargo test --test tier2_integration

test-ai:
	cargo test --test tier3_ai

test-sec:
	cargo test --test tier4_security
	cargo test --test tier4_security_manifest

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

# -----------------------------------------------------------------------------
# 4. EXECUTION
# -----------------------------------------------------------------------------

run:
	@mkdir -p logs
	@pkill -f tos-brain || true
	cargo run --bin tos-brain | tee logs/tos-brain.log

run-web:
	@mkdir -p logs
	@pkill -f tos-brain || true
	@pkill -f "http.server 8080" || true
	@echo "[TOS] Initializing Web UI Server (8080)..."
	@python3 -m http.server 8080 -d web_ui > logs/web_ui.log 2>&1 & WEB_PID=$$!; \
	echo "[TOS] Synchronizing Brain Core (7000/7001)..."; \
	trap "kill $$WEB_PID; pkill -f tos-brain; exit" EXIT INT TERM; \
	cargo run --bin tos-brain | tee logs/tos-brain.log

# -----------------------------------------------------------------------------
# 5. MAINTENANCE
# -----------------------------------------------------------------------------

clean:
	cargo clean
	rm -rf logs/
