.PHONY: help
help:
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@printf '\nAvailable variables:\n'
	@grep -E '^[a-zA-Z_-]+\?=.* ## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = "?=.* ## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

run: ## Run locally for development
	@cargo run

fmt: ## Auto-format code locally
	@cargo fmt

test: ## Run tests
	@cargo test

build: ## Build with release optimization
	@cargo build --release

install: build ## Install the binary into the PATH. Requires sudo.
	@sudo cp target/release/marmotte /usr/local/bin/marmotte

uninstall: ## Removes the binary from the PATH. Requires sudo.
	@sudo rm /usr/local/bin/marmotte || echo "It seems marmotte wasn't installed"

release_linux: ## Build for all targets
	for target in i686-unknown-linux-musl x86_64-unknown-linux-musl x86_64-unknown-linux-gnu ; do \
		rustup target add $$target ; \
		cargo build --release --target $$target ; \
		cp target/$${target}/release/marmotte marmotte-$${target} ; \
	done
