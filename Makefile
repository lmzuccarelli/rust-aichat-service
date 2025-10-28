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
