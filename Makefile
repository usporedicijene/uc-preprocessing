.PHONY: all build run clean release run-release test check fmt clippy lint deny ci doc update deps

all: build

build:
	cargo build

run:
	cargo run

clean:
	cargo clean

release:
	cargo build --release

run-release:
	cargo run --release

# use: make test ARGS="-- --nocapture" or make test ARGS="test_name"
test:
	cargo test $(ARGS)

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

lint: fmt clippy

# Dependency policy checks (license, advisories, bans, duplicates)
deny:
	cargo deny check

# Run what CI runs
ci:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --verbose

doc:
	cargo doc --no-deps --open

update:
	cargo update

deps:
	cargo tree
