//! Common CSV processing helpers to reduce code duplication
//!
//! This module provides reusable utilities for CSV operations with
//! consistent error handling and logging.

use std::path::Path;
use tracing::warn;

/// Result of CSV processing that tracks errors and filtering
#[derive(Debug, Default)]
pub struct CsvProcessingStats {
    pub total_rows: usize,
    pub invalid_rows: usize,
    pub filtered_rows: usize,
}

impl CsvProcessingStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log_summary(&self, file_path: &Path) {
        if self.invalid_rows > 0 {
            warn!(
                "Skipped {} invalid CSV rows in {:?}",
                self.invalid_rows, file_path
            );
        }
        if self.filtered_rows > 0 {
            warn!("Filtered {} rows in {:?}", self.filtered_rows, file_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_processing_stats_new() {
        let stats = CsvProcessingStats::new();
        assert_eq!(stats.total_rows, 0);
        assert_eq!(stats.invalid_rows, 0);
        assert_eq!(stats.filtered_rows, 0);
    }

    #[test]
    fn test_csv_processing_stats_values() {
        let stats = CsvProcessingStats {
            total_rows: 100,
            invalid_rows: 5,
            filtered_rows: 10,
        };

        assert_eq!(stats.total_rows, 100);
        assert_eq!(stats.invalid_rows, 5);
        assert_eq!(stats.filtered_rows, 10);
    }
}
