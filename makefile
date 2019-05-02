# The use of `${MAKE} target` is to allow the reuse of targets and also ensure
# explicit ordering

SHELL := /usr/bin/env bash

# Vars
TEMP_BUILD_DIR = .build
RELEASE_DIR = release

# Prepare build and release dirs
.PHONY: prepare
prepare:
	mkdir -p $(TEMP_BUILD_DIR) $(RELEASE_DIR)

# Remove build dir
.PHONY: clean
clean:
	rm -rf $(TEMP_BUILD_DIR)

# Remove build and release dirs
.PHONY: wipe
wipe:
	${MAKE} clean
	rm -rf $(RELEASE_DIR)

# Build for Linux and zip into release dir
.PHONY: build-linux-x64
build-linux-x64:
	${MAKE} prepare
	cd binary && cargo build --release --target=x86_64-unknown-linux-gnu
	mv binary/target/x86_64-unknown-linux-gnu/release/bukubrow $(TEMP_BUILD_DIR)/bukubrow-linux-x64
	cd $(TEMP_BUILD_DIR) && zip -r '../$(RELEASE_DIR)/binary-linux-x64' ./*
	${MAKE} clean

# Build for macOS and zip into release dir
.PHONY: build-darwin-x64
build-darwin-x64:
	${MAKE} prepare
	cd binary && cargo build --release --target=x86_64-apple-darwin
	mv binary/target/x86_64-apple-darwin/release/bukubrow $(TEMP_BUILD_DIR)/bukubrow-darwin-x64
	cd $(TEMP_BUILD_DIR) && zip -r '../$(RELEASE_DIR)/binary-darwin-x64' ./*
	${MAKE} clean
