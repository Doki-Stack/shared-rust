.PHONY: build build-all test lint fmt fmt-fix check doc clean

all: check

build:
	cargo build

build-all:
	cargo build --all-features

test:
	cargo test --all-features

lint:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt --check

fmt-fix:
	cargo fmt

check: fmt lint test

doc:
	cargo doc --all-features --no-deps --open

clean:
	cargo clean
