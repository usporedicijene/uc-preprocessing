# Contributing to UsporediCijene Preprocessing Tool

Thank you for your interest in contributing! This document provides guidelines and information for contributors.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/usporedicijene/preprocessing.git uc-preprocessing
   cd uc-preprocessing
   ```
3. **Set up the development environment**:
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install pre-commit hooks
   pip install pre-commit  # or: brew install pre-commit
   pre-commit install

   # (Optional) Install additional tools
   cargo install cargo-audit
   rustup component add rustfmt clippy

   # Build and test
   make build
   make test
   ```

## Development Workflow

### Making Changes

1. Create a new branch for your feature or fix:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. Make your changes following our coding standards

3. Write or update tests as needed

4. Run the test suite:
   ```bash
   make test
   make clippy
   make fmt
   ```

5. Commit your changes with a descriptive message:
   ```bash
   git commit -m "Add feature: description of what you added"
   ```

6. Push to your fork and create a Pull Request

### Coding Standards

- **Format code** with `make fmt`
- **No clippy warnings** - run `make clippy` and fix all warnings
- **Write tests** for new functionality
- **Document public APIs** with doc comments (`///`)
- **Keep commits atomic** - one logical change per commit

### Commit Message Guidelines

Use clear, descriptive commit messages:

```
type: short description

Longer explanation if needed. Wrap at 72 characters.

Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

## Pull Request Process

1. **Update documentation** if you're changing functionality
2. **Add tests** for new features
3. **Ensure CI passes** - all checks must be green
4. **Request review** from maintainers
5. **Address feedback** promptly

## Pre-commit Hooks

Pre-commit hooks automatically run before each commit:

- Format code
- Lint code (Clippy)
- Check compilation
- Run tests
- YAML/TOML validation
- Security audit (on push)

**Usage:**
- **Manual**: `pre-commit run --all-files`
- **Skip**: `git commit --no-verify`
- **Update hooks**: `pre-commit autoupdate`

## Testing

```bash
make test                                # Run all tests
make test ARGS="-- --nocapture"          # With output
make test ARGS="test_merge_prices"       # Specific test
```

The test suite covers CSV processing, city/category mapping, anchor data integration, error handling, and parallel processing. All tests use temporary files for isolation.

## Available Make Commands

| Command | Description |
|---------|-------------|
| `make build` | Build in debug mode |
| `make release` | Build in release mode |
| `make run` | Run in debug mode |
| `make test` | Run tests |
| `make fmt` | Format code |
| `make clippy` | Lint with Clippy |
| `make check` | Check without building |
| `make doc` | Generate and open documentation |
| `make clean` | Clean build artifacts |

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant error messages or logs

### Feature Requests

For feature requests, please describe:

- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Questions?

Feel free to open an issue for questions or discussions. We're happy to help!

## License

By contributing, you agree that your contributions will be licensed under the AGPL-3.0 license.
