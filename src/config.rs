//! Application configuration module
//!
//! This module handles loading and managing all configuration from environment variables.
//! By centralizing configuration, we make the code more testable and easier to understand.

use std::env;
use std::path::PathBuf;
use tracing::info;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to directory containing raw store data
    pub stores_dir_path: PathBuf,

    /// Path to cleaned anchor price data (optional)
    pub anchor_cleaned_data_path: Option<PathBuf>,

    /// Path to previous day's cleaned data for comparison (optional)
    pub previous_day_cleaned_data_path: Option<PathBuf>,

    /// Path to output directory for cleaned data
    pub output_data_dir: PathBuf,

    /// Path to comparison reports output directory
    pub comparison_reports_output_dir: PathBuf,

    /// Path to cities mapping CSV file
    pub cities_mappings_csv: PathBuf,

    /// Path to categories mapping CSV file
    pub categories_mappings_csv: PathBuf,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingRequired(String),
    PathDoesNotExist(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingRequired(var) => {
                write!(f, "Required environment variable '{}' is not set", var)
            }
            ConfigError::PathDoesNotExist(msg) => write!(f, "Path does not exist: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Required Environment Variables
    /// - `STORES_DIR_PATH`: Path to raw store data
    /// - `OUTPUT_DATA_DIR`: Output directory for cleaned data
    /// - `COMPARISON_REPORTS_OUTPUT_DIR`: Reports directory
    /// - `CITIES_MAPPINGS_CSV`: Path to cities mapping CSV file
    /// - `CATEGORIES_MAPPINGS_CSV`: Path to categories mapping CSV file
    ///
    /// # Optional Environment Variables
    /// - `STORES_DIR_PATH_ANCHOR_CLEANED_DATA`: Anchor price data path
    /// - `STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA`: Previous day data path
    ///
    /// # Errors
    /// Returns `ConfigError::MissingRequired` if required variables are not set
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists (for development convenience)
        // Skip during tests to avoid interference
        #[cfg(not(test))]
        {
            let _ = dotenvy::dotenv();
        }

        // Load required variables
        let stores_dir_path = env::var("STORES_DIR_PATH")
            .map_err(|_| ConfigError::MissingRequired("STORES_DIR_PATH".to_string()))?;

        let output_data_dir = env::var("OUTPUT_DATA_DIR")
            .map_err(|_| ConfigError::MissingRequired("OUTPUT_DATA_DIR".to_string()))?;

        let comparison_reports_output_dir =
            env::var("COMPARISON_REPORTS_OUTPUT_DIR").map_err(|_| {
                ConfigError::MissingRequired("COMPARISON_REPORTS_OUTPUT_DIR".to_string())
            })?;

        let cities_mappings_csv = env::var("CITIES_MAPPINGS_CSV")
            .map_err(|_| ConfigError::MissingRequired("CITIES_MAPPINGS_CSV".to_string()))?;

        let categories_mappings_csv = env::var("CATEGORIES_MAPPINGS_CSV")
            .map_err(|_| ConfigError::MissingRequired("CATEGORIES_MAPPINGS_CSV".to_string()))?;

        // Load optional variables
        let anchor_cleaned_data_path = env::var("STORES_DIR_PATH_ANCHOR_CLEANED_DATA")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

        let previous_day_cleaned_data_path = env::var("STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

        // Validate that stores_dir_path exists
        let stores_path = PathBuf::from(&stores_dir_path);
        if !stores_path.exists() {
            return Err(ConfigError::PathDoesNotExist(format!(
                "STORES_DIR_PATH: {} (please ensure this directory exists)",
                stores_dir_path
            )));
        }

        Ok(Config {
            stores_dir_path: stores_path,
            anchor_cleaned_data_path,
            previous_day_cleaned_data_path,
            output_data_dir: PathBuf::from(output_data_dir),
            comparison_reports_output_dir: PathBuf::from(comparison_reports_output_dir),
            cities_mappings_csv: PathBuf::from(cities_mappings_csv),
            categories_mappings_csv: PathBuf::from(categories_mappings_csv),
        })
    }

    /// Get the stores directory path as a string
    pub fn stores_dir(&self) -> &str {
        self.stores_dir_path
            .to_str()
            .expect("Invalid UTF-8 in STORES_DIR_PATH")
    }

    /// Get the output directory path as a string
    pub fn output_dir(&self) -> &str {
        self.output_data_dir
            .to_str()
            .expect("Invalid UTF-8 in output_data_dir path")
    }

    /// Get the comparison reports directory path as a string
    pub fn reports_dir(&self) -> &str {
        self.comparison_reports_output_dir
            .to_str()
            .expect("Invalid UTF-8 in comparison_reports_output_dir path")
    }

    /// Get the cities mapping file path as a string
    pub fn cities_mapping_file(&self) -> &str {
        self.cities_mappings_csv
            .to_str()
            .expect("Invalid UTF-8 in cities_mappings_csv path")
    }

    /// Get the categories mapping file path as a string
    pub fn categories_mapping_file(&self) -> &str {
        self.categories_mappings_csv
            .to_str()
            .expect("Invalid UTF-8 in categories_mappings_csv path")
    }

    /// Get the anchor cleaned data path as an option of string
    pub fn anchor_path(&self) -> Option<&str> {
        self.anchor_cleaned_data_path
            .as_ref()
            .and_then(|p| p.to_str())
    }

    /// Get the previous day cleaned data path as an option of string
    pub fn previous_day_path(&self) -> Option<&str> {
        self.previous_day_cleaned_data_path
            .as_ref()
            .and_then(|p| p.to_str())
    }

    /// Print configuration summary
    #[allow(dead_code)]
    pub fn print_summary(&self) {
        info!("Configuration:");
        info!("  Stores directory: {}", self.stores_dir());
        info!("  Output directory: {}", self.output_dir());
        info!("  Cities mapping: {}", self.cities_mapping_file());
        info!("  Categories mapping: {}", self.categories_mapping_file());

        if let Some(anchor) = self.anchor_path() {
            info!("  Anchor data: {}", anchor);
        }

        if let Some(prev) = self.previous_day_path() {
            info!("  Previous day data: {}", prev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_config_from_env_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Set all required env vars
        unsafe {
            env::set_var("STORES_DIR_PATH", temp_path);
            env::set_var("OUTPUT_DATA_DIR", "/tmp/output");
            env::set_var("COMPARISON_REPORTS_OUTPUT_DIR", "/tmp/reports");
            env::set_var("CITIES_MAPPINGS_CSV", "cities.csv");
            env::set_var("CATEGORIES_MAPPINGS_CSV", "categories.csv");
        }

        let config = Config::from_env().unwrap();

        assert_eq!(config.stores_dir(), temp_path);
        assert_eq!(config.output_dir(), "/tmp/output");
        assert_eq!(config.reports_dir(), "/tmp/reports");
        assert_eq!(config.cities_mapping_file(), "cities.csv");
        assert_eq!(config.categories_mapping_file(), "categories.csv");
        assert!(config.previous_day_path().is_none());

        // Cleanup
        unsafe {
            env::remove_var("STORES_DIR_PATH");
            env::remove_var("OUTPUT_DATA_DIR");
            env::remove_var("COMPARISON_REPORTS_OUTPUT_DIR");
            env::remove_var("CITIES_MAPPINGS_CSV");
            env::remove_var("CATEGORIES_MAPPINGS_CSV");
        }
    }

    #[test]
    #[serial]
    fn test_config_from_env_missing_required() {
        // Ensure all required vars are not set
        unsafe {
            env::remove_var("STORES_DIR_PATH");
            env::remove_var("OUTPUT_DATA_DIR");
            env::remove_var("COMPARISON_REPORTS_OUTPUT_DIR");
            env::remove_var("CITIES_MAPPINGS_CSV");
            env::remove_var("CATEGORIES_MAPPINGS_CSV");
        }

        let result = Config::from_env();

        assert!(result.is_err());
        // Should fail on the first missing required var (STORES_DIR_PATH)
        match result {
            Err(ConfigError::MissingRequired(var)) => {
                assert_eq!(var, "STORES_DIR_PATH");
            }
            _ => panic!("Expected MissingRequired error"),
        }
    }

    #[test]
    #[serial]
    fn test_config_with_all_vars() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Set all env vars
        unsafe {
            env::set_var("STORES_DIR_PATH", temp_path);
            env::set_var("OUTPUT_DATA_DIR", "/output");
            env::set_var("STORES_DIR_PATH_ANCHOR_CLEANED_DATA", "/anchor");
            env::set_var("STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA", "/previous");
            env::set_var("COMPARISON_REPORTS_OUTPUT_DIR", "/reports");
            env::set_var("CITIES_MAPPINGS_CSV", "custom_cities.csv");
            env::set_var("CATEGORIES_MAPPINGS_CSV", "custom_categories.csv");
        }

        let config = Config::from_env().unwrap();

        assert_eq!(config.stores_dir(), temp_path);
        assert_eq!(config.output_dir(), "/output");
        assert_eq!(config.anchor_path(), Some("/anchor"));
        assert_eq!(config.previous_day_path(), Some("/previous"));
        assert_eq!(config.reports_dir(), "/reports");
        assert_eq!(config.cities_mapping_file(), "custom_cities.csv");
        assert_eq!(config.categories_mapping_file(), "custom_categories.csv");

        // Cleanup
        unsafe {
            env::remove_var("STORES_DIR_PATH");
            env::remove_var("OUTPUT_DATA_DIR");
            env::remove_var("STORES_DIR_PATH_ANCHOR_CLEANED_DATA");
            env::remove_var("STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA");
            env::remove_var("COMPARISON_REPORTS_OUTPUT_DIR");
            env::remove_var("CITIES_MAPPINGS_CSV");
            env::remove_var("CATEGORIES_MAPPINGS_CSV");
        }
    }

    #[test]
    #[serial]
    fn test_config_empty_strings_treated_as_none() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        unsafe {
            env::set_var("STORES_DIR_PATH", temp_path);
            env::set_var("OUTPUT_DATA_DIR", "/output");
            env::set_var("COMPARISON_REPORTS_OUTPUT_DIR", "/reports");
            env::set_var("CITIES_MAPPINGS_CSV", "cities.csv");
            env::set_var("CATEGORIES_MAPPINGS_CSV", "categories.csv");
            env::set_var("STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA", "   "); // Empty after trim
        }

        let config = Config::from_env().unwrap();

        assert!(config.previous_day_path().is_none());

        // Cleanup
        unsafe {
            env::remove_var("STORES_DIR_PATH");
            env::remove_var("OUTPUT_DATA_DIR");
            env::remove_var("COMPARISON_REPORTS_OUTPUT_DIR");
            env::remove_var("CITIES_MAPPINGS_CSV");
            env::remove_var("CATEGORIES_MAPPINGS_CSV");
            env::remove_var("STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA");
        }
    }

    #[test]
    #[serial]
    fn test_config_path_validation() {
        // Test that non-existent path is rejected
        unsafe {
            env::set_var(
                "STORES_DIR_PATH",
                "/path/that/absolutely/does/not/exist/xyz123",
            );
            env::set_var("OUTPUT_DATA_DIR", "/output");
            env::set_var("COMPARISON_REPORTS_OUTPUT_DIR", "/reports");
            env::set_var("CITIES_MAPPINGS_CSV", "cities.csv");
            env::set_var("CATEGORIES_MAPPINGS_CSV", "categories.csv");
        }

        let result = Config::from_env();
        assert!(result.is_err());

        match result {
            Err(ConfigError::PathDoesNotExist(msg)) => {
                assert!(msg.contains("/path/that/absolutely/does/not/exist/xyz123"));
            }
            _ => panic!("Expected PathDoesNotExist error"),
        }

        // Cleanup
        unsafe {
            env::remove_var("STORES_DIR_PATH");
            env::remove_var("OUTPUT_DATA_DIR");
            env::remove_var("COMPARISON_REPORTS_OUTPUT_DIR");
            env::remove_var("CITIES_MAPPINGS_CSV");
            env::remove_var("CATEGORIES_MAPPINGS_CSV");
        }
    }
}
