# Contributing to Lux MCP

Thank you for your interest in contributing to Lux MCP! We welcome contributions from the community.

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Provide a clear description
- Include steps to reproduce
- Share error messages and logs
- Mention your environment (OS, Rust version, etc.)

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Format code (`cargo fmt`)
6. Check linting (`cargo clippy`)
7. Commit with clear messages
8. Push to your fork
9. Open a Pull Request

### Development Setup

```bash
# Clone your fork
git clone https://github.com/yourusername/lux-mcp.git
cd lux-mcp

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Code Style

- Follow Rust conventions
- Use `cargo fmt` before committing
- Address `cargo clippy` warnings
- Write descriptive commit messages
- Add tests for new features
- Update documentation as needed

## Testing

- Write unit tests for new functions
- Add integration tests for new tools
- Test with different model providers
- Verify MCP protocol compliance

## Documentation

- Update README.md for user-facing changes
- Add inline documentation for public APIs
- Update tool documentation in docs/
- Include examples for new features

## Review Process

1. All submissions require review
2. CI must pass (when available)
3. Changes must be tested
4. Documentation must be updated
5. Code must follow style guidelines

## Community

- Be respectful and inclusive
- Follow our Code of Conduct
- Help others when possible
- Share knowledge and learn together

## Questions?

Feel free to open an issue for any questions about contributing.

Thank you for helping make Lux MCP better!