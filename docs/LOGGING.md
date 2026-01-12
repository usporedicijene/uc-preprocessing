# Logging Guide

This application uses structured logging via the `tracing` crate for better observability and debugging.

## Quick Start

### Default Logging (Info Level)

```bash
cargo run
```

This shows informational messages about the application's progress.

### Debug Logging

To see detailed debug output including reference data comparisons:

```bash
RUST_LOG=debug cargo run
```

### Only Warnings and Errors

To reduce noise and only see warnings and errors:

```bash
RUST_LOG=warn cargo run
```

### Specific Module Logging

You can control logging per module:

```bash
# Debug only for validation module
RUST_LOG=usporedicijene_preprocessing::checks::validation=debug cargo run

# Info for everything except one module
RUST_LOG=info,usporedicijene_preprocessing::processors=warn cargo run
```

## Log Levels

The application uses the following log levels:

- **`error`**: Critical errors that prevent processing
- **`warn`**: Warnings about issues that don't stop processing (e.g., invalid CSV rows)
- **`info`**: General information about processing progress
- **`debug`**: Detailed debugging information (e.g., all reference cities/categories)
- **`trace`**: Very detailed trace information (currently not used)

## Examples

### Normal Processing

```bash
$ cargo run
2024-11-10T12:00:00.000Z  INFO usporedicijene_preprocessing: Configuration:
2024-11-10T12:00:00.001Z  INFO usporedicijene_preprocessing:   Stores directory: ../data/raw
2024-11-10T12:00:00.001Z  INFO usporedicijene_preprocessing:   Output directory: cleaned_data
2024-11-10T12:00:00.002Z  INFO usporedicijene_preprocessing: Validating cities...
2024-11-10T12:00:00.100Z  INFO checks::validation: Found 45 unique cities in data files
2024-11-10T12:00:00.101Z  INFO checks::validation: All cities found in data files are already mapped.
```

### Debug Mode

```bash
$ RUST_LOG=debug cargo run
2024-11-10T12:00:00.100Z DEBUG checks::validation: Reference cities from mapping file:
2024-11-10T12:00:00.100Z DEBUG checks::validation:   'Zagreb'
2024-11-10T12:00:00.100Z DEBUG checks::validation:   'Split'
...
```

### With Warnings

```bash
$ cargo run
2024-11-10T12:00:00.500Z  WARN processors: Skipped invalid CSV row 42 in "data/konzum/stores.csv": CSV error: ...
2024-11-10T12:00:00.600Z  WARN processors: Skipped 3 invalid CSV rows in "data/konzum/stores.csv"
```

## Environment Variable Reference

Set these before running the application:

```bash
# Log level for entire application
export RUST_LOG=info

# Log level with timestamp format
export RUST_LOG=debug

# Complex filter
export RUST_LOG="warn,usporedicijene_preprocessing::checks=debug"
```

Or use them inline:

```bash
RUST_LOG=debug cargo run
```

## Logging in Different Environments

### Development

Use debug level for detailed output:

```bash
RUST_LOG=debug cargo run
```

### Production

Use info level and redirect to a file:

```bash
RUST_LOG=info ./usporedicijene-preprocessing > /var/log/preprocessing.log 2>&1
```

### CI/CD

Use warn level to reduce noise in CI logs:

```bash
RUST_LOG=warn cargo test
```

## Advanced Configuration

The tracing subscriber supports advanced configuration via environment variables:

### Custom Format

The default format includes timestamp, level, and message. This is configured in `src/main.rs`:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    )
    .init();
```

### JSON Output (Future Enhancement)

For structured logging in production, you can modify the initialization to output JSON:

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(...)
    .init();
```

## Troubleshooting

### No Logs Appearing

If you don't see any logs, ensure `RUST_LOG` is not set to a level that filters everything out:

```bash
# Ensure you see something
RUST_LOG=trace cargo run
```

### Too Much Output

Reduce the log level:

```bash
RUST_LOG=warn cargo run
```

### Specific Module is Too Verbose

Filter out specific modules:

```bash
RUST_LOG="info,usporedicijene_preprocessing::comparators=warn" cargo run
```

## Migration from println!/eprintln!

The application has been migrated from `println!` and `eprintln!` to structured logging:

- `println!` → `info!()` (for informational messages)
- `eprintln!("Warning: ...")` → `warn!()` (for warnings)
- `eprintln!("Error: ...")` → `error!()` (for errors)
- Debug output → `debug!()` (for debug information)

## Benefits

Using structured logging provides:

1. **Filtering**: Control what you see via environment variables
2. **Levels**: Different severity levels for different situations
3. **Performance**: Debug logging has zero cost when disabled
4. **Consistency**: All logs follow the same format
5. **Extensibility**: Easy to add structured fields (key-value pairs)
6. **Integration**: Works well with log aggregation tools

## Further Reading

- [tracing documentation](https://docs.rs/tracing/)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/)
- [RUST_LOG environment variable](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)

