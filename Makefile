.PHONY: all clean build

all: clean-all build

LEVEL ?= "info"

build-debug: 
	cargo build

build:
	cargo build --release

clean-all:
	rm -rf cargo-test*
	cargo clean

verify:
	cargo clippy --all-targets --all-features

fmt:
	rustfmt --check src/*.rs --edition 2024

cargo-fmt:
	cargo fmt
