# Define variables for the targets and output binary name
TARGETS := x86_64-unknown-linux-musl aarch64-unknown-linux-musl

# Setup phase to install necessary Rust targets
.PHONY: setup
setup:
	@for target in $(TARGETS); do \
		rustup target add $$target; \
	done

# Build the binary for all targets
.PHONY: build
build: setup
	@for target in $(TARGETS); do \
		cargo lambda build --release --extension --target $$target; \
	done

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean