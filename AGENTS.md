# AI Agent Guidelines

## Project Overview

Rust CLI tool that processes Croatian retail store data (from [cijene.dev](https://cijene.dev/)): validates, cleans, and compares CSV datasets. Licensed under AGPL-3.0-or-later.

## Build & Test Commands

```bash
make build          # Debug build
make test           # Run all tests
make lint           # cargo fmt + cargo clippy
make ci             # Format check + clippy + tests
make deny           # Dependency policy checks (license, advisories)
```

## Code Conventions

- Format with `cargo fmt` (style edition 2024, see `rustfmt.toml`)
- Zero clippy warnings: `cargo clippy -- -D warnings`
- Prefer `///` doc comments on public APIs
- Tests go in `#[cfg(test)]` modules within each file; `src/tests.rs` has integration tests. Use `tempfile`/`assert_fs` for file isolation
- Prefer `.expect("reason")` over `.unwrap()` in non-test code

## Project Structure

```
src/
├── main.rs              # Entry point, CLI args (clap)
├── config.rs            # Environment config (dotenvy)
├── processors.rs        # Orchestrates per-store processing
├── loaders/             # CSV loading and reference data
├── cleaners/            # Data cleaning and transformation
├── checks/              # Validation (cities, categories)
├── comparators/         # Day-to-day data comparison and reports
├── derived_traits/      # Derived column calculators
├── embeddings/          # Name search fingerprinting (trigrams, normalization)
├── extractors.rs        # Field extraction helpers
├── csv_helpers.rs       # CSV reading/writing utilities
└── tests.rs             # Integration tests
```

## Versioning & Release

- Version lives in `Cargo.toml` (`version = "x.y.z"`)
- The `version_guard.yml` CI workflow enforces a version bump on PRs with release-relevant changes
- The `tag_release.yml` CI workflow auto-creates a git tag from `Cargo.toml` version on merge to main
- Update `CHANGELOG.md` (Keep a Changelog format) for every version bump
- Update the Rust version badge in `README.md` if `rust-toolchain.toml` changes

## Pull Requests

- Follow the template in `.github/PULL_REQUEST_TEMPLATE.md`
- Commit messages: short imperative descriptions (see `CONTRIBUTING.md` for optional conventional prefixes)
- Run `make ci` and `make deny` before submitting

## Tooling

- Rust version pinned in `rust-toolchain.toml`
- Pre-commit hooks defined in `.pre-commit-config.yaml` (all Rust hooks are local)
- Dependency policy enforced by `cargo-deny` via `deny.toml`
- CI runs on GitHub Actions (`.github/workflows/test.yml`)

## Key Dependencies

- `csv` / `serde` / `serde_json` — CSV and JSON serialization
- `walkdir` — Recursive directory traversal
- `rayon` — Parallel processing
- `clap` — CLI argument parsing (derive + env features)
- `dotenvy` — `.env` file loading
- `tracing` / `tracing-subscriber` — Structured logging
- `chrono` — Date/time handling
