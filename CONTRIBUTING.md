# Contributing to nginx-discovery

Thank you for your interest in contributing! 🎉

## Code of Conduct

Be respectful, inclusive, and constructive.

## How to Contribute

### Reporting Bugs

1. Check if the bug is already reported in [Issues](https://github.com/urwithajit9/nginx-discovery/issues)
2. If not, create a new issue with:
   - Clear title
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version)

### Suggesting Features

1. Check [Issues](https://github.com/urwithajit9/nginx-discovery/issues) for existing suggestions
2. Create a new issue with:
   - Clear use case
   - Proposed API (if applicable)
   - Examples

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests
5. Run tests: `cargo test --all-features`
6. Run formatting: `cargo fmt`
7. Run lints: `cargo clippy --all-features -- -D warnings`
8. Commit with clear message
9. Push and create PR

## Development Setup
```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/nginx-discovery.git
cd nginx-discovery

# Add upstream
git remote add upstream https://github.com/urwithajit9/nginx-discovery.git

# Install tools
rustup component add rustfmt clippy

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench
```

## Code Style

- Follow Rust standard style (enforced by `rustfmt`)
- Write tests for new features
- Document public APIs with rustdoc
- Keep PRs focused and small

## Testing

- Unit tests in the same file as code
- Integration tests in `tests/`
- Add fixtures in `tests/fixtures/`

## Questions?

Open an issue or start a [Discussion](https://github.com/urwithajit9/nginx-discovery/discussions)
