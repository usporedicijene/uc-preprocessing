use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;

use super::processors::process_products_file;
use super::processors::process_stores_file;

pub fn extract_unique_cities(
    base_path: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let base_dir = Path::new(base_path);

    if !base_dir.exists() {
        eprintln!("Error: Directory '{base_path}' does not exist.");
        return Ok(HashSet::new());
    }

    // Find all stores.csv files in subdirectories
    let mut stores_files = Vec::new();
    for entry in WalkDir::new(base_dir)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "stores.csv" {
            stores_files.push(entry.path().to_path_buf());
        }
    }

    if stores_files.is_empty() {
        eprintln!("Warning: No stores.csv files found in subdirectories of '{base_path}'");
        return Ok(HashSet::new());
    }

    println!("Found {} stores.csv files", stores_files.len());

    // Use thread-safe mutex for collecting unique cities
    let unique_cities = Mutex::new(HashSet::new());

    // Process files in parallel
    stores_files.par_iter().for_each(|stores_file| {
        let mut local_cities = HashSet::new();
        match process_stores_file(stores_file, &mut local_cities) {
            Ok(file_cities_count) => {
                let parent_name = stores_file
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                println!("  {parent_name}/stores.csv: {file_cities_count} unique cities");

                // Merge local cities into global set
                let mut global_cities = unique_cities.lock().unwrap();
                global_cities.extend(local_cities);
            }
            Err(e) => {
                eprintln!("Error reading {stores_file:?}: {e}");
            }
        }
    });

    let unique_cities = unique_cities.into_inner().unwrap();
    Ok(unique_cities)
}

pub fn extract_unique_categories(
    base_path: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let base_dir = Path::new(base_path);

    if !base_dir.exists() {
        eprintln!("Error: Directory '{base_path}' does not exist.");
        return Ok(HashSet::new());
    }

    // Find all products.csv files in subdirectories
    let mut products_files = Vec::new();
    for entry in WalkDir::new(base_dir)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "products.csv" {
            products_files.push(entry.path().to_path_buf());
        }
    }

    if products_files.is_empty() {
        eprintln!("Warning: No products.csv files found in subdirectories of '{base_path}'");
        return Ok(HashSet::new());
    }

    println!("Found {} products.csv files", products_files.len());

    // Use thread-safe mutex for collecting unique categories
    let unique_categories = Mutex::new(HashSet::new());

    // Process files in parallel
    products_files.par_iter().for_each(|products_file| {
        let mut local_categories = HashSet::new();
        match process_products_file(products_file, &mut local_categories) {
            Ok(file_categories_count) => {
                let parent_name = products_file
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                println!(
                    "  {parent_name}/products.csv: {file_categories_count} unique categories",

                );

                // Merge local categories into global set
                let mut global_categories = unique_categories.lock().unwrap();
                global_categories.extend(local_categories);
            }
            Err(e) => {
                eprintln!("Error reading {products_file:?}: {e}");
            }
        }
    });

    let unique_categories = unique_categories.into_inner().unwrap();
    Ok(unique_categories)
}

pub fn extract_store_ids(data_path: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut store_ids = HashSet::new();

    for entry in WalkDir::new(data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("stores.csv") {
            let file = File::open(entry.path())?;
            let mut reader = csv::ReaderBuilder::new().from_reader(file);

            // Get market chain from path to create unique store identifiers
            let market_chain = extract_market_chain_from_path(entry.path(), data_path);

            for result in reader.records() {
                let record = result?;
                if let Some(store_id) = record.get(0) {
                    // Create unique store identifier: market_chain + store_id
                    store_ids.insert(format!("{market_chain}_{store_id}"));
                }
            }
        }
    }

    Ok(store_ids)
}

pub fn extract_product_barcodes(
    data_path: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut barcodes = HashSet::new();

    for entry in WalkDir::new(data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("products.csv") {
            let file = File::open(entry.path())?;
            let mut reader = csv::ReaderBuilder::new().from_reader(file);

            // Get market chain from path to create unique product identifiers
            let market_chain = extract_market_chain_from_path(entry.path(), data_path);

            for result in reader.records() {
                let record = result?;
                if let Some(barcode) = record.get(1) {
                    // Column 1 is the barcode
                    // Create unique product identifier: market_chain + barcode
                    barcodes.insert(format!("{market_chain}_{barcode}"));
                }
            }
        }
    }

    Ok(barcodes)
}

pub fn extract_market_chain_from_path(file_path: &Path, base_path: &str) -> String {
    // Extract market chain from directory structure
    if let Ok(relative_path) = file_path.strip_prefix(base_path)
        && let Some(first_component) = relative_path.components().next()
        && let Some(chain_name) = first_component.as_os_str().to_str()
    {
        return chain_name.to_string();
    }
    "Unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_unique_cities_integration() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("test_store");
        std::fs::create_dir_all(&subdir).unwrap();

        // Create a stores.csv file in the subdirectory
        let stores_path = subdir.join("stores.csv");
        std::fs::write(
            &stores_path,
            "store_id,type,address,city,zipcode\n\
             001,supermarket,Main St 1,Zagreb,10000\n\
             002,supermarket,Main St 2,Split,21000\n\
             003,supermarket,Main St 3,Zagreb,10000",
        )
        .unwrap();

        let result = extract_unique_cities(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 2); // Zagreb and Split (Zagreb appears twice but should be unique)
        assert!(result.contains("Zagreb"));
        assert!(result.contains("Split"));
    }

    #[test]
    fn test_extract_unique_cities_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let result = extract_unique_cities(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_unique_cities_nonexistent_directory() {
        let result = extract_unique_cities("/path/that/does/not/exist").unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_unique_categories() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("test_store");
        std::fs::create_dir_all(&subdir).unwrap();

        // Create a products.csv file in the subdirectory
        let products_path = subdir.join("products.csv");
        std::fs::write(
            &products_path,
            "product_id,name,category\n\
             123456,Apple Juice,Food\n\
             789012,Orange Juice,Drinks\n\
             111111,Bread,Food\n\
             222222,Smartphone,Electronics",
        )
        .unwrap();

        let result = extract_unique_categories(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 3); // Food, Drinks, Electronics
        assert!(result.contains("Food"));
        assert!(result.contains("Drinks"));
        assert!(result.contains("Electronics"));
    }
}
