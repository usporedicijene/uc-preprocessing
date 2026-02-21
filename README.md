# UsporediCijene Preprocessing Tool

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

A Rust tool that processes Croatian retail store data - validates, cleans, and prepares it for analysis. Uses parallel processing for high performance.

## Features

- **Validates** cities and categories against reference mapping files
- **Filters** excluded products and invalid prices
- **Merges** anchor price data for comparison analysis
- **Compares** day-to-day data changes with detailed reports
- **Name search fingerprinting** for fast, typo-tolerant product name matching
- **Parallel processing** for fast execution on large datasets

## Quick Start

1. **Configure** environment variables:
   ```bash
   cp env.example .env
   # Edit .env with your paths
   ```

2. **Run**:
   ```bash
   cargo run
   ```

## Configuration

### Required Environment Variables

| Variable | Description |
|----------|-------------|
| `STORES_DIR_PATH` | Path to raw store data directory |
| `OUTPUT_DATA_DIR` | Output directory for cleaned data |
| `COMPARISON_REPORTS_OUTPUT_DIR` | Reports output directory |
| `CITIES_MAPPINGS_CSV` | Path to cities mapping file |
| `CATEGORIES_MAPPINGS_CSV` | Path to categories mapping file |

### Optional Environment Variables

| Variable | Description |
|----------|-------------|
| `STORES_DIR_PATH_ANCHOR_CLEANED_DATA` | Anchor price data for comparisons |
| `STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA` | Previous day data for change analysis |

## Usage

```bash
# Normal processing (clean + compare)
cargo run

# Only run comparison (skip cleaning)
cargo run -- --compare-only
```

## Input Structure

This tool processes data from [cijene.dev](https://cijene.dev/) - a project providing current and historical prices from Croatian retail chains.

```
raw-data/
├── konzum/
│   ├── stores.csv
│   ├── products.csv
│   └── prices.csv
├── spar/
│   └── ...
└── ...
```

## Output Structure

```
cleaned_data/
├── konzum/
│   ├── stores.csv      # City names mapped
│   ├── products.csv    # Categories mapped, filtered, name search fingerprint added
│   └── prices.csv      # Invalid prices filtered, derived columns added
└── ...
```

## Exit Codes

- **0**: Success
- **1**: New unmapped cities/categories found (update mapping files)

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Dependencies

- `csv`, `serde`, `serde_json` - CSV/JSON processing
- `rayon` - Parallel processing
- `clap` - CLI argument parsing
- `dotenvy` - Environment configuration
- `tracing` - Structured logging
- `chrono` - Date/time handling

## Documentation

- [Data Processing Details](docs/DATA_PROCESSING.md) - Validation, filtering, anchor data
- [Name Search Fingerprinting](docs/NAME_SEARCH.md) - Trigram-based product name matching algorithm
- [Comparison Analysis](docs/COMPARISON.md) - Change rate metrics and reports
- [Logging Guide](docs/LOGGING.md) - Log levels and configuration

## Contributing

Contributions are welcome! Please read:

- [Contributing Guidelines](CONTRIBUTING.md) - How to contribute
- [Code of Conduct](CODE_OF_CONDUCT.md) - Community standards
- [Changelog](CHANGELOG.md) - Version history

## License

AGPL-3.0 - See [LICENSE.txt](LICENSE.txt) for details.
