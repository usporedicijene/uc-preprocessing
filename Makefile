.PHONY: build run clean release test check all

# Default target
all: build

# Build the project in debug mode
build:
	cargo build

# Run the project in debug mode
run:
	cargo run

# Clean build artifacts
clean:
	cargo clean

# Build in release mode
release:
	cargo build --release

# Run in release mode
run-release:
	cargo run --release

# Run tests (use: make test ARGS="-- --nocapture" or make test ARGS="test_name")
test:
	cargo test $(ARGS)

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Lint code with Clippy
clippy:
	cargo clippy -- -D warnings

# Generate documentation
doc:
	cargo doc --no-deps --open

# Update dependencies
update:
	cargo update

# Show dependencies
deps:
	cargo tree
