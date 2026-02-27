.PHONY: build check test run clean help

# Default target
help:
	@echo "TOS Alpha-2 Build System"
	@echo ""
	@echo "Usage:"
	@echo "  make build    - Compile the project"
	@echo "  make check    - Run cargo check"
	@echo "  make test     - Run all tests (Tier 1, 2, 4)"
	@echo "  make run      - Run the TOS Brain"
	@echo "  make clean    - Remove build artifacts"

build:
	cargo build

check:
	cargo check

test:
	cargo test

run:
	cargo run --bin tos-brain

clean:
	cargo clean
