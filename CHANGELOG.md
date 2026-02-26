# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.1] - 2026-02-22

### Changed
- Replace deprecated `actions-rs/toolchain` with `dtolnay/rust-toolchain` and add `Swatinem/rust-cache` in CI
- Pin Rust 1.93 via `rust-toolchain.toml`
- Simplify pre-commit hooks: remove unused external repo, consolidate all Rust hooks as local
- Improve Makefile: fix `.PHONY`, add `lint`, `deny`, and `ci` targets
- Expand `.gitignore` with OS, editor, and log patterns

### Added
- `deny.toml` for dependency license, advisory, and source policy checks via `cargo-deny`
- `license` field in `Cargo.toml` package metadata

## [2.0.0] - 2026-02-22

### Added
- Product-name fingerprinting pipeline that writes `uc_name_searching_algorithm_1` into cleaned `products.csv` files.
- New embeddings components for Croatian normalization, per-word trigram generation, 256-bit bit-vector hashing, and hex encoding.
- Search comparison utilities (containment, overlap ratio, Jaccard) and focused tests for name-search behavior.
- New `docs/NAME_SEARCH.md` documentation with algorithm details, backend usage guidance, and benchmark examples.

### Changed
- `products.csv` now includes a new `uc_name_searching_algorithm_1` column. This can break consumers that rely on strict CSV schemas or positional parsing; update parsers/mappings accordingly.
- Documentation refreshed to reflect preprocessing and backend search strategy (`README.md`, `docs/DATA_PROCESSING.md`, `docs/NAME_SEARCH.md`).

## [1.0.0] - 2026-01-12

### Added
- Initial public release

[2.0.1]: https://github.com/usporedicijene/uc-preprocessing/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/usporedicijene/uc-preprocessing/compare/v1.0.0...v2.0.0
[1.0.0]: https://github.com/usporedicijene/uc-preprocessing/releases/tag/v1.0.0
