.PHONY: build check test test-tier1 test-tier2 run clean help

# Default target
help:
	@echo "TOS Alpha-2 Build System"
	@echo ""
	@echo "Usage:"
	@echo "  make build       - Compile the project"
	@echo "  make check       - Run cargo check"
	@echo "  make test        - Run all tests (Tier 1 & 2)"
	@echo "  make test-tier1  - Run Unit tests (Brain Logic)"
	@echo "  make test-tier2  - Run Integration tests (Shell/PTY)"
	@echo "  make run         - Run the TOS Brain + Mock Face Demo"
	@echo "  make clean       - Remove build artifacts"

build:
	cargo build

check:
	cargo check

test:
	cargo test

test-tier1:
	cargo test --test tier1_brain

test-tier2:
	cargo test --test tier2_integration

run:
	cargo run --bin tos-brain

clean:
	cargo clean
