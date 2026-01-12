mod checks;
mod cleaners;
mod comparators;
mod config;
mod csv_helpers;
mod derived_traits;
mod extractors;
mod loaders;
mod processors;

#[cfg(test)]
mod tests;

use clap::Parser;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

use config::Config;

use loaders::reference_categories::load_reference_categories;
use loaders::reference_cities::load_reference_cities;

use extractors::extract_unique_categories;
use extractors::extract_unique_cities;

use comparators::runner::{run_comparison, run_comparison_only};

use cleaners::processing::create_cleaned_data;

use loaders::csv_loaders::load_category_mappings;
use loaders::csv_loaders::load_city_mappings;

use checks::validation::{validate_categories, validate_cities};

/// Croatian Price Comparison Data Preprocessing Tool
#[derive(Parser, Debug)]
#[command(name = "usporedicijene-preprocessing")]
#[command(author, version)]
#[command(about = "Croatian Price Comparison Data Preprocessing Tool")]
#[command(
    long_about = "This tool processes raw store data, validates cities and categories,\n\
                        generates cleaned data, and optionally compares with previous day's data."
)]
#[command(after_help = "ENVIRONMENT VARIABLES:
    STORES_DIR_PATH                           Path to directory containing store data
    STORES_DIR_PATH_ANCHOR_CLEANED_DATA       (Optional) Path to cleaned anchor price data
    STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA Path to previous day's cleaned data for comparison
    OUTPUT_DATA_DIR                           (Optional) Output directory (default: cleaned_data)
    COMPARISON_REPORTS_OUTPUT_DIR             (Optional) Reports directory (default: .)
    CITIES_MAPPINGS_CSV                       (Optional) Cities mapping file (default: cities_mappings.csv)
    CATEGORIES_MAPPINGS_CSV                   (Optional) Categories mapping file (default: categories_mappings.csv)

EXAMPLES:
    # Normal processing (clean data + compare):
    usporedicijene-preprocessing

    # Only run comparison (skip data cleaning):
    usporedicijene-preprocessing --compare-only")]
struct Cli {
    /// Skip data cleaning and only run comparison with previous day
    #[arg(long)]
    compare_only: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Parse command line arguments using clap
    let cli = Cli::parse();

    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Configuration error: {}", e);
            error!("Please ensure all required environment variables are set.");
            error!("Run with --help to see required environment variables.");
            std::process::exit(1);
        }
    };

    config.print_summary();
    info!("{}", "-".repeat(50));

    // Check if only comparison is requested
    if cli.compare_only {
        match run_comparison_only(&config) {
            Ok(()) => return Ok(()),
            Err(e) => {
                error!("Comparison failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // More than just comparison :)
    // moving forward with cleaning data and comparison
    // ...

    let stores_dir = config.stores_dir();

    // Validate cities
    info!("Validating cities...");
    let reference_cities = load_reference_cities(config.cities_mapping_file())?;
    let found_cities = extract_unique_cities(stores_dir)?;

    if let Err(e) = validate_cities(
        &reference_cities,
        &found_cities,
        config.cities_mapping_file(),
    ) {
        error!("City validation failed: {}", e);
        std::process::exit(1);
    }

    // Validate categories
    info!("Validating categories...");
    let reference_categories = load_reference_categories(config.categories_mapping_file())?;
    let found_categories = extract_unique_categories(stores_dir)?;

    if let Err(e) = validate_categories(
        &reference_categories,
        &found_categories,
        config.categories_mapping_file(),
    ) {
        error!("Category validation failed: {}", e);
        std::process::exit(1);
    }

    // Proceed with cleaning data
    info!("Creating cleaned data...");
    info!("{}", "-".repeat(50));

    let city_mappings = load_city_mappings(config.cities_mapping_file())?;
    let category_mappings = load_category_mappings(config.categories_mapping_file())?;

    let output_dir = config.output_dir();

    // Create output directory if it doesn't exist
    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path)?;
        info!("Created {} directory", output_dir);
    }

    // Create cleaned data with mapped cities, categories and anchor data
    create_cleaned_data(
        stores_dir,
        config.anchor_path(),
        &city_mappings,
        &category_mappings,
        output_dir,
    )?;

    info!("Cleaned data creation completed successfully!");

    // Compare with previous day's data if available
    if let Some(previous_day_path) = config.previous_day_path() {
        // Run comparison in non-required mode (warns on errors instead of failing)
        if let Err(e) = run_comparison(&config, output_dir, previous_day_path, false) {
            warn!("Comparison encountered an error: {}", e);
        }
    } else {
        info!(
            "No previous day data path specified (STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA not set)"
        );
        info!("Skipping comparison with previous day.");
    }

    Ok(())
}
