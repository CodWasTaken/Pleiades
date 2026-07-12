.PHONY: help setup build run test lint clean release docs

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## Install development dependencies
	@echo "Installing development tools..."
	cargo install cargo-audit cargo-llvm-cov cargo-criterion cargo-outdated 2>/dev/null || true
	@echo "Setting up git hooks..."
	rustup component add rustfmt clippy 2>/dev/null || true

build: ## Build the project in release mode
	cargo build --release --workspace --all-features

run: ## Run in debug mode
	cargo run

test: ## Run all tests
	cargo test --workspace --all-features

test-ci: ## Run tests with CI configuration
	cargo test --workspace --all-features --verbose

lint: ## Run clippy lints
	cargo clippy --workspace --all-targets --all-features -- -D warnings

format: ## Check formatting
	cargo fmt --all -- --check

format-fix: ## Apply formatting
	cargo fmt --all

audit: ## Run security audit
	cargo audit

coverage: ## Generate coverage report
	cargo llvm-cov --workspace --all-features --html

bench: ## Run benchmarks
	cargo criterion --workspace --all-features

clean: ## Clean build artifacts
	cargo clean
	rm -rf target/

docs: ## Build documentation
	cargo doc --workspace --no-deps --document-private-items --all-features
	@echo "Documentation available at target/doc/pleiades/index.html"

release: ## Create a release build
	cargo build --release --workspace --all-features
	@echo "Release binary at target/release/pleiades"

check-all: lint test build ## Run all checks (lint, test, build)
	@echo "All checks passed!"
