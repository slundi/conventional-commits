# Makefile for conventional-commits

.PHONY: help build test lint fmt clean install example setup-hook bench doc

# Default target
help:
	@echo "Available targets:"
	@echo "  build      - Build the project"
	@echo "  test       - Run all tests"
	@echo "  lint       - Run clippy linter"
	@echo "  fmt        - Format code"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install the CLI tool"
	@echo "  example    - Run example usage"
	@echo "  setup-hook - Setup git commit-msg hook"
	@echo "  bench      - Run benchmarks"
	@echo "  doc        - Generate documentation"
	@echo "  check      - Run all checks (test, lint, fmt)"

# Build the project
build:
	@echo "🔨 Building project..."
	cargo build

# Build release version
build-release:
	@echo "🔨 Building release version..."
	cargo build --release

# Run all tests
test:
	@echo "🧪 Running comprehensive tests..."
	./scripts/test.sh

# Run only unit tests
test-unit:
	@echo "🧪 Running unit tests only..."
	cargo test --lib --verbose

# Run tests with build step
test-safe:
	@echo "🧪 Building and testing safely..."
	cargo build
	cargo build --release  
	cargo test --verbose

# Run tests with coverage
test-coverage:
	@echo "🧪 Running tests with coverage..."
	cargo tarpaulin --verbose --all-features --workspace --timeout 120

# Run clippy linter
lint:
	@echo "📋 Running clippy..."
	cargo clippy -- -D warnings

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt

# Check formatting
fmt-check:
	@echo "🎨 Checking code formatting..."
	cargo fmt -- --check

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Install the CLI tool locally
install:
	@echo "📦 Installing commit-check..."
	cargo install --path .

# Run example
example:
	@echo "🚀 Running basic usage example..."
	cargo run --example basic_usage

# Setup git hook
setup-hook:
	@echo "🪝 Setting up git commit-msg hook..."
	./scripts/setup-git-hook.sh

# Run benchmarks (if any)
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench

# Generate documentation
doc:
	@echo "📚 Generating documentation..."
	cargo doc --open

# Security audit
audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Run all checks
check: fmt-check lint test
	@echo "✅ All checks passed!"

# Development setup
dev-setup:
	@echo "🛠️ Setting up development environment..."
	cargo install cargo-tarpaulin cargo-audit
	rustup component add clippy rustfmt

# Test the CLI with various inputs
test-cli:
	@echo "🧪 Testing CLI with various inputs..."
	@echo "Valid commit:"
	cargo run -- --message "feat: add new feature"
	@echo ""
	@echo "Invalid commit:"
	-cargo run -- --message "invalid commit message"
	@echo ""
	@echo "Examples command:"
	cargo run -- examples
	@echo ""
	@echo "Types command:"
	cargo run -- types

# Publish to crates.io (use with caution)
publish:
	@echo "📦 Publishing to crates.io..."
	cargo publish

# Pre-commit checks
pre-commit: fmt lint test
	@echo "🚀 Pre-commit checks completed!"

# CI simulation (run what CI would run)
ci: fmt-check lint test audit
	@echo "🤖 CI checks completed!"
