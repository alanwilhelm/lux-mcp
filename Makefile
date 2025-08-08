# Lux MCP Makefile
# Common development tasks

.PHONY: help build release check fmt clippy test clean run install

# Default target
help:
	@echo "Lux MCP Development Commands"
	@echo "============================"
	@echo "  make build    - Build debug version"
	@echo "  make release  - Build optimized release version"
	@echo "  make check    - Run all quality checks (fmt, clippy, test)"
	@echo "  make fmt      - Format code"
	@echo "  make clippy   - Run clippy lints"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make run      - Build and run the server"
	@echo "  make install  - Install to ~/.cargo/bin"
	@echo ""
	@echo "Quick Commands:"
	@echo "  make fix      - Auto-fix formatting and clippy issues"
	@echo "  make ci       - Run full CI checks"
	@echo "  make dev      - Build and run with debug logging"

# Build debug version
build:
	@echo "Building debug version..."
	@cargo build

# Build release version
release:
	@echo "Building release version..."
	@cargo build --release

# Run all checks
check:
	@./check.sh

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Run clippy
clippy:
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -f test_threading_*.log

# Build and run
run: build
	@echo "Starting Lux MCP server..."
	@RUST_LOG=info ./target/debug/lux-mcp

# Install to cargo bin
install: release
	@echo "Installing to ~/.cargo/bin..."
	@cp target/release/lux-mcp ~/.cargo/bin/
	@echo "Installed! You can now run 'lux-mcp' from anywhere"

# Auto-fix issues
fix:
	@echo "Auto-fixing issues..."
	@cargo fmt
	@cargo fix --allow-dirty --allow-staged
	@cargo clippy --fix --allow-dirty --allow-staged
	@echo "✓ Auto-fix complete. Please review changes."

# CI checks (strict)
ci:
	@echo "Running CI checks..."
	@cargo fmt -- --check
	@cargo check --all-targets
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo test --quiet
	@cargo doc --no-deps --quiet
	@echo "✅ All CI checks passed!"

# Development mode with debug logging
dev:
	@echo "Starting in development mode..."
	@RUST_LOG=debug cargo run

# Quick rebuild and test
quick: fmt build test
	@echo "✅ Quick check complete!"

# Check for unused dependencies
audit:
	@echo "Checking dependencies..."
	@cargo audit || echo "Run 'cargo install cargo-audit' to enable security audits"
	@cargo outdated || echo "Run 'cargo install cargo-outdated' to check for updates"

# Generate documentation
docs:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open

# Database setup (optional)
db-setup:
	@echo "Setting up database..."
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "⚠️  DATABASE_URL not set. Database is optional."; \
		echo "   To enable: export DATABASE_URL='postgresql://localhost/lux_mcp'"; \
	else \
		sea-orm-cli migrate up || echo "Run 'cargo install sea-orm-cli' first"; \
	fi

# Show current configuration
config:
	@echo "Current Configuration:"
	@echo "====================="
	@echo "OPENAI_API_KEY:     $${OPENAI_API_KEY:+[SET]}"
	@echo "OPENROUTER_API_KEY: $${OPENROUTER_API_KEY:+[SET]}"
	@echo "DATABASE_URL:       $${DATABASE_URL:-[NOT SET - Running in-memory]}"
	@echo "RUST_LOG:           $${RUST_LOG:-info}"
	@echo ""
	@echo "Default Models:"
	@echo "  Chat:      $${LUX_DEFAULT_CHAT_MODEL:-gpt-4o}"
	@echo "  Reasoning: $${LUX_DEFAULT_REASONING_MODEL:-o3-pro}"
	@echo "  Bias:      $${LUX_DEFAULT_BIAS_CHECKER_MODEL:-o4-mini}"