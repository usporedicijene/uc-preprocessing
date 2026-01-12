use csv::Reader;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use crate::csv_helpers::CsvProcessingStats;
use crate::loaders::types::Store;

pub fn process_stores_file(
    stores_file: &Path,
    unique_cities: &mut HashSet<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(stores_file)?;
    let mut reader = Reader::from_reader(file);
    let mut file_cities = HashSet::new();
    let mut stats = CsvProcessingStats::new();

    for result in reader.deserialize() {
        stats.total_rows += 1;
        match result {
            Ok(store) => {
                let store: Store = store;
                let city = store.city.as_str().trim();
                if !city.is_empty() {
                    file_cities.insert(city.to_string());
                    unique_cities.insert(city.to_string());
                }
            }
            Err(e) => {
                stats.invalid_rows += 1;
                tracing::warn!(
                    "Skipped invalid CSV row {} in {:?}: {}",
                    stats.total_rows,
                    stores_file,
                    e
                );
            }
        }
    }

    // Use the helper to log summary
    stats.log_summary(stores_file);

    Ok(file_cities.len())
}

pub fn process_products_file(
    products_file: &Path,
    unique_categories: &mut HashSet<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(products_file)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut file_categories = HashSet::new();
    let mut stats = CsvProcessingStats::new();

    // Find category column index
    let headers = reader.headers()?.clone();
    let category_col_index = headers.iter().position(|h| h == "category");

    if category_col_index.is_none() {
        tracing::warn!("No 'category' column found in {:?}", products_file);
        return Ok(0);
    }

    let category_idx = category_col_index.unwrap();

    for result in reader.records() {
        stats.total_rows += 1;
        match result {
            Ok(record) => {
                if let Some(category_value) = record.get(category_idx) {
                    let category = category_value.trim();
                    if !category.is_empty() {
                        file_categories.insert(category.to_string());
                        unique_categories.insert(category.to_string());
                    }
                }
            }
            Err(e) => {
                stats.invalid_rows += 1;
                tracing::warn!(
                    "Skipped invalid CSV row {} in {:?}: {}",
                    stats.total_rows,
                    products_file,
                    e
                );
            }
        }
    }

    // Use the helper to log summary
    stats.log_summary(products_file);

    Ok(file_categories.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_stores_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "store_id,type,address,city,zipcode\n\
             001,supermarket,Main St 1,Zagreb,10000\n\
             002,supermarket,Main St 2,Split,21000\n\
             003,supermarket,Main St 3,Rijeka,51000"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_process_stores_file() {
        let stores_file = create_test_stores_csv();
        let mut unique_cities = HashSet::new();

        let result = process_stores_file(stores_file.path(), &mut unique_cities).unwrap();

        assert_eq!(result, 3); // 3 unique cities
        assert_eq!(unique_cities.len(), 3);
        assert!(unique_cities.contains("Zagreb"));
        assert!(unique_cities.contains("Split"));
        assert!(unique_cities.contains("Rijeka"));
    }

    #[test]
    fn test_process_stores_file_empty_cities() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "store_id,type,address,city,zipcode\n\
             001,supermarket,Main St 1,,10000\n\
             002,supermarket,Main St 2,   ,21000\n\
             003,supermarket,Main St 3,Split,51000"
        )
        .unwrap();

        let mut unique_cities = HashSet::new();
        let result = process_stores_file(file.path(), &mut unique_cities).unwrap();

        // Should only count rows with non-empty cities
        assert_eq!(result, 1); // Only Split
        assert_eq!(unique_cities.len(), 1);
        assert!(unique_cities.contains("Split"));
    }

    #[test]
    fn test_process_products_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "product_id,name,category\n\
             123456,Apple Juice,Food\n\
             789012,Orange Juice,Drinks\n\
             111111,Bread,Food"
        )
        .unwrap();

        let mut unique_categories = HashSet::new();
        let result = process_products_file(file.path(), &mut unique_categories).unwrap();

        assert_eq!(result, 2); // Food and Drinks (Food appears twice but should be unique)
        assert_eq!(unique_categories.len(), 2);
        assert!(unique_categories.contains("Food"));
        assert!(unique_categories.contains("Drinks"));
    }
}
