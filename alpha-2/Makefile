.PHONY: build check test test-tier1 test-tier2 test-all test-system test-component run clean help fmt clippy doc

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
	@echo "  make test-all    - Run Tier 1 + Tier 2 tests"
	@echo "  make test-system - Run Single-process Comprehensive Test"
	@echo "  make test-component - Run Two-process Component Test (Brain + Stimulator)"
	@echo "  make doc         - Generate documentation"
	@echo "  make run         - Run the TOS Brain + Mock Face Demo"
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

test-all: test-tier1 test-tier2

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

clean:
	cargo clean

