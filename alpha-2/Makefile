.PHONY: build check test test-tier1 test-tier2 test-all run clean help fmt clippy doc

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

test:
	cargo test

test-tier1:
	cargo test --test tier1_brain

test-tier2:
	cargo test --test tier2_integration

test-all: test-tier1 test-tier2

doc:
	cargo doc --no-deps

run:
	cargo run --bin tos-brain

clean:
	cargo clean
