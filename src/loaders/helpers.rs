use crate::loaders::types::StoreId;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

pub fn load_product_id_to_barcode_mapping(
    data_path: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut product_mapping = HashMap::new();

    for entry in WalkDir::new(data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("products.csv") {
            let file = File::open(entry.path())?;
            let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(file);

            for result in reader.records() {
                let record = match result {
                    Ok(r) => r,
                    Err(_) => continue, // Skip malformed records
                };

                if record.len() >= 2 {
                    let product_id = record.get(0).unwrap_or("").to_string();
                    let barcode = record.get(1).unwrap_or("").to_string();
                    if !product_id.is_empty() && !barcode.is_empty() {
                        product_mapping.insert(product_id, barcode);
                    }
                }
            }
        }
    }

    Ok(product_mapping)
}

pub fn load_store_names(
    data_path: &str,
) -> Result<HashMap<StoreId, String>, Box<dyn std::error::Error>> {
    let mut store_names = HashMap::new();

    for entry in WalkDir::new(data_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("stores.csv") {
            let file = File::open(entry.path())?;
            let mut reader = csv::ReaderBuilder::new().from_reader(file);

            // Get headers to find columns
            let headers = reader.headers()?.clone();
            let store_id_idx = headers.iter().position(|h| h == "store_id").unwrap_or(0);
            let store_type_idx = headers.iter().position(|h| h == "type");
            let address_idx = headers.iter().position(|h| h == "address");
            let city_idx = headers.iter().position(|h| h == "city");

            for result in reader.records() {
                let record = result?;
                if let Some(store_id) = record.get(store_id_idx) {
                    let store_type = store_type_idx.and_then(|idx| record.get(idx)).unwrap_or("");
                    let address = address_idx.and_then(|idx| record.get(idx)).unwrap_or("");
                    let city = city_idx.and_then(|idx| record.get(idx)).unwrap_or("");

                    // Create a readable store name with market chain, address and city
                    let store_name =
                        if !store_type.is_empty() && !address.is_empty() && !city.is_empty() {
                            format!("{store_type} - {address}, {city}")
                        } else if !store_type.is_empty() && !city.is_empty() {
                            format!("{store_type} - Store in {city}")
                        } else if !store_type.is_empty() && !address.is_empty() {
                            format!("{store_type} - {address}")
                        } else if !address.is_empty() && !city.is_empty() {
                            format!("{address}, {city}")
                        } else if !store_type.is_empty() {
                            format!("{store_type} Store {store_id}")
                        } else if !address.is_empty() {
                            address.to_string()
                        } else if !city.is_empty() {
                            format!("Store in {city}")
                        } else {
                            format!("Store {store_id}")
                        };

                    store_names.insert(StoreId::new(store_id), store_name);
                }
            }
        }
    }

    Ok(store_names)
}

pub fn count_total_product_instances_in_file(
    price_file: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(price_file)
        .map_err(|e| format!("Failed to open price file '{}': {e}", price_file.display()))?;
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true) // Allow records with varying number of fields
        .from_reader(file);

    // Count unique store_id + barcode combinations (product instances)
    let mut unique_product_instances = HashSet::new();

    for (line_num, result) in reader.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Warning: Skipping malformed record {} in price file '{}': {e}",
                    line_num + 2,
                    price_file.display()
                );
                continue;
            }
        };

        if record.len() >= 2 {
            let store_id = record.get(0).unwrap_or("");
            let product_id = record.get(1).unwrap_or("");
            // Create unique product instance identifier: store_id + product_id
            unique_product_instances.insert(format!("{store_id}_{product_id}"));
        } else {
            eprintln!(
                "Warning: Skipping record {} in '{}' - insufficient fields (has {}, needs at least 2)",
                line_num + 2,
                price_file.display(),
                record.len()
            );
        }
    }

    Ok(unique_product_instances.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_load_store_names() {
        let temp_dir = TempDir::new().unwrap();
        let stores_path = temp_dir.path().join("stores.csv");
        std::fs::write(
            &stores_path,
            "store_id,type,address,city,zipcode\n\
             001,supermarket,Main St 1,Zagreb,10000\n\
             002,hypermarket,Second St 2,Split,21000",
        )
        .unwrap();

        let result = load_store_names(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&StoreId::new("001")));
        assert!(result.contains_key(&StoreId::new("002")));
    }

    #[test]
    fn test_load_store_names_partial_data() {
        let temp_dir = TempDir::new().unwrap();
        let stores_path = temp_dir.path().join("stores.csv");
        std::fs::write(
            &stores_path,
            "store_id,type,address,city,zipcode\n\
             001,supermarket,,,10000\n\
             002,,Second St 2,Split,",
        )
        .unwrap();

        let result = load_store_names(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 2);
        // Store with only type should have format "type Store store_id"
        let store1_name = result.get(&StoreId::new("001")).unwrap();
        assert!(store1_name.contains("supermarket"));
        // Store with address and city should have format "address, city"
        let store2_name = result.get(&StoreId::new("002")).unwrap();
        assert!(store2_name.contains("Split"));
    }

    #[test]
    fn test_load_product_id_to_barcode_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let products_path = temp_dir.path().join("products.csv");
        std::fs::write(
            &products_path,
            "product_id,barcode,name,category\n\
             P001,123456,Apple,Food\n\
             P002,789012,Orange,Food",
        )
        .unwrap();

        let result = load_product_id_to_barcode_mapping(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("P001"), Some(&"123456".to_string()));
        assert_eq!(result.get("P002"), Some(&"789012".to_string()));
    }

    #[test]
    fn test_count_total_product_instances_in_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "store_id,product_id,price\n\
             001,P001,10.00\n\
             001,P002,15.00\n\
             002,P001,10.50"
        )
        .unwrap();

        let result = count_total_product_instances_in_file(file.path()).unwrap();
        assert_eq!(result, 3); // 3 unique store_id + product_id combinations
    }

    #[test]
    fn test_count_total_product_instances_with_duplicates() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "store_id,product_id,price\n\
             001,P001,10.00\n\
             001,P001,10.50\n\
             002,P001,10.50"
        )
        .unwrap();

        let result = count_total_product_instances_in_file(file.path()).unwrap();
        assert_eq!(result, 2); // Only 2 unique combinations (001_P001 appears twice)
    }
}
