.PHONY: build check test test-tier1 test-tier2 test-tier3 test-tier4 test-all test-system test-component run run-web clean help fmt clippy doc

# Default target
help:
	@echo "TOS Alpha-2 Build System"
	@echo ""
	@echo "Usage:"
	@echo "  make build       - Compile the project"
	@echo "  make check       - Run cargo check"
	@echo "  make fmt         - Format all code (cargo fmt)"
	@echo "  make clippy      - Run static analysis (cargo clippy)"
	@echo "  make test        - Run all standard tests"
	@echo "  make test-tier1  - Run Unit tests (Brain Logic)"
	@echo "  make test-tier2  - Run Integration tests (Shell/PTY)"
	@echo "  make test-tier3  - Run Contextual AI Engine tests"
	@echo "  make test-tier4  - Run Infrastructure/Security tests"
	@echo "  make test-all    - Run all test tiers"
	@echo "  make test-system - Run Single-process Comprehensive Test"
	@echo "  make test-component - Run Two-process Component Test (Brain + Stimulator)"
	@echo "  make doc         - Generate documentation"
	@echo "  make run         - Run the TOS Brain + Mock Face Demo"
	@echo "  make run-web     - Run the TOS Brain + Web UI Server concurrently"
	@echo "  make clean       - Remove build artifacts"

build:
	cargo build

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

test: test-all

test-tier1:
	cargo test --test tier1_brain

test-tier2:
	cargo test --test tier2_integration

test-tier3:
	cargo test --test tier3_ai

test-tier4:
	cargo test --test tier4_security
	cargo test --test tier4_security_manifest

test-all: test-tier1 test-tier2 test-tier3 test-tier4

test-system:
	@mkdir -p logs
	cargo run --bin system_test | tee logs/system_test.log

test-component:
	@echo "Starting Brain Node in background..."
	@mkdir -p logs
	@rm -f logs/brain_node.log
	cargo run --bin brain_node > logs/brain_node.log 2>&1 & BR_PID=$$!; \
	echo "Waiting for Brain Node to initialize..."; \
	sleep 5; \
	echo "Starting stimulator_brain_node..."; \
	cargo test --test stimulator_brain_node -- --nocapture; \
	echo "Shutting down Brain Node..."; \
	kill $$BR_PID; \
	echo "Component Test Complete. Review 'logs/brain_node.log' for UI output."

doc:
	cargo doc --no-deps

run:
	@mkdir -p logs
	cargo run --bin tos-brain | tee logs/tos-brain.log

run-web:
	@mkdir -p logs
	@echo "Starting Web UI Server on http://localhost:8080..."
	@python3 -m http.server 8080 -d web_ui > logs/web_ui.log 2>&1 & WEB_PID=$$!; \
	trap "kill $$WEB_PID" EXIT INT TERM; \
	cargo run --bin tos-brain | tee logs/tos-brain.log

clean:
	cargo clean

