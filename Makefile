# Define variables for the targets and output binary name
TARGETS := aarch64-unknown-linux-musl x86_64-unknown-linux-musl

# Setup phase to install necessary Rust targets
.PHONY: setup
setup:
	@for target in $(TARGETS); do \
		rustup target add $$target; \
	done


# Build the binary for all targets
.PHONY: build
build: setup
	cargo install cargo-binstall
	cargo binstall cargo-lambda --no-confirm
	@for target in $(TARGETS); do \
		cargo lambda build --release --extension --target $$target; \
	done

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Deploy the Lambda function to AWS
.PHONY: deploy
deploy: build
	cargo lambda deploy --extension