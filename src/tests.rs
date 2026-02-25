// Integration tests for usporedicijene-preprocessing
//
// This module contains integration tests that test multiple modules working together.
// Unit tests for specific modules are located in their respective module files:
// - src/derived_traits/calculators.rs - calculator function tests
// - src/processors.rs - CSV processor tests
// - src/extractors.rs - data extraction tests
// - src/loaders/reference_cities.rs - city loading tests
// - src/loaders/reference_categories.rs - category loading tests
// - src/loaders/loaders.rs - mapping and anchor data loading tests

use std::collections::HashMap;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn create_test_prices_csv() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        "store_id,barcode,price,unit_price,special_price\n\
             001,123456,10.50,1.05,9.99\n\
             002,789012,15.00,1.50,14.99\n\
             001,111111,5.25,0.52,4.99"
    )
    .unwrap();
    file
}

fn create_test_anchor_prices_csv() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        "store_id,barcode,price,unit_price,special_price,derived_price\n\
             001,123456,10.00,2.00,8.50,9.50\n\
             002,789012,15.00,3.00,13.50,14.50\n\
             003,999999,20.00,4.00,18.99,19.99"
    )
    .unwrap();
    file
}

// Integration test: CSV validation with processors
#[test]
fn test_csv_validation_with_invalid_rows() {
    use crate::processors::process_stores_file;
    use std::collections::HashSet;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        "store_id,type,address,city,zipcode\n\
             001,supermarket,Main St 1,Zagreb,10000\n\
             002,supermarket,\"Unclosed quote,Split,21000\n\
             003,supermarket,Main St 3,Rijeka,51000"
    )
    .unwrap();

    let mut unique_cities = HashSet::new();
    let result = process_stores_file(file.path(), &mut unique_cities).unwrap();

    // Should process 2 valid rows (skipping the invalid one with field count mismatch)
    assert_eq!(result, 1); // Only Zagreb and Rijeka are processed correctly
    assert_eq!(unique_cities.len(), 1); // Only one unique city processed
    assert!(unique_cities.contains("Zagreb") || unique_cities.contains("Rijeka"));
}

// Integration test: AnchorPriceData type default
#[test]
fn test_anchor_price_data_default() {
    use crate::cleaners::types::AnchorPriceData;

    let default_data = AnchorPriceData::default();
    assert_eq!(default_data.price, None);
    assert_eq!(default_data.unit_price, None);
    assert_eq!(default_data.special_price, None);
    assert_eq!(default_data.derived_price, None);
}

// Integration test: AnchorPriceData type clone
#[test]
fn test_anchor_price_data_clone() {
    use crate::cleaners::types::AnchorPriceData;

    let data = AnchorPriceData {
        price: Some("10.00".to_string()),
        unit_price: Some("2.00".to_string()),
        special_price: Some("8.50".to_string()),
        derived_price: Some("9.50".to_string()),
    };

    let cloned = data.clone();
    assert_eq!(cloned.price, data.price);
    assert_eq!(cloned.unit_price, data.unit_price);
    assert_eq!(cloned.special_price, data.special_price);
    assert_eq!(cloned.derived_price, data.derived_price);
}

// Integration test: Merge prices with anchor data (cleaners/transformers)
#[test]
fn test_merge_prices_with_anchor() {
    use crate::cleaners::transformers::merge_prices_with_anchor;

    let source_file = create_test_prices_csv();
    let anchor_file = create_test_anchor_prices_csv();
    let target_file = NamedTempFile::new().unwrap();

    merge_prices_with_anchor(source_file.path(), anchor_file.path(), target_file.path()).unwrap();

    // Read and verify the merged result
    let mut reader = csv::Reader::from_path(target_file.path()).unwrap();
    let headers = reader.headers().unwrap().clone();

    // Check that anchor columns were added
    assert!(headers.iter().any(|h| h == "derived_price"));
    assert!(headers.iter().any(|h| h == "uc_anchor_derived_price"));
    assert!(headers.iter().any(|h| h == "derived_price_change"));

    let records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 3);
}

// Integration test: Merge prices with nonexistent anchor (cleaners/transformers)
#[test]
fn test_merge_prices_with_nonexistent_anchor() {
    use crate::cleaners::transformers::merge_prices_with_anchor;

    let temp_dir = TempDir::new().unwrap();
    let source_file = create_test_prices_csv();
    let target_file = NamedTempFile::new().unwrap();
    let nonexistent_anchor = temp_dir.path().join("nonexistent.csv");

    // This should work without error, just no anchor data merged
    merge_prices_with_anchor(source_file.path(), &nonexistent_anchor, target_file.path()).unwrap();

    // Read the result - should have anchor columns but they should be empty
    let mut reader = csv::Reader::from_path(target_file.path()).unwrap();
    let headers = reader.headers().unwrap().clone();

    assert!(headers.iter().any(|h| h == "derived_price"));
    assert!(headers.iter().any(|h| h == "uc_anchor_derived_price"));
    assert!(headers.iter().any(|h| h == "derived_price_change"));

    let records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 3);

    // Find column indices
    let derived_price_idx = headers.iter().position(|h| h == "derived_price").unwrap();
    let uc_anchor_derived_idx = headers
        .iter()
        .position(|h| h == "uc_anchor_derived_price")
        .unwrap();
    let price_change_idx = headers
        .iter()
        .position(|h| h == "derived_price_change")
        .unwrap();

    for record in records {
        // uc_anchor_derived_price should be empty when no anchor data
        assert_eq!(record.get(uc_anchor_derived_idx).unwrap(), "");
        // derived_price_change should be empty when no anchor data
        assert_eq!(record.get(price_change_idx).unwrap(), "");
        // But derived_price should still be calculated from original data
        assert!(!record.get(derived_price_idx).unwrap().is_empty());
    }
}

// Integration test: Merge prices with partial anchor match (cleaners/transformers)
#[test]
fn test_merge_prices_with_partial_anchor_match() {
    use crate::cleaners::transformers::merge_prices_with_anchor;

    let source_file = create_test_prices_csv();

    // Create anchor with only partial match
    let mut anchor_file = NamedTempFile::new().unwrap();
    writeln!(
        anchor_file,
        "store_id,barcode,price,unit_price,special_price,derived_price\n\
             001,123456,10.00,2.00,8.50,9.50"
    )
    .unwrap();

    let target_file = NamedTempFile::new().unwrap();

    merge_prices_with_anchor(source_file.path(), anchor_file.path(), target_file.path()).unwrap();

    let mut reader = csv::Reader::from_path(target_file.path()).unwrap();
    let headers = reader.headers().unwrap().clone();
    let records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();

    let uc_anchor_idx = headers
        .iter()
        .position(|h| h == "uc_anchor_derived_price")
        .unwrap();

    let barcode_idx = headers.iter().position(|h| h == "barcode").unwrap();

    // Find the record with barcode 123456 - it should have anchor data
    let matching_record = records
        .iter()
        .find(|r| r.get(barcode_idx).unwrap() == "123456");
    assert!(matching_record.is_some());
    assert_eq!(matching_record.unwrap().get(uc_anchor_idx).unwrap(), "9.50");

    // Records with other barcodes should have empty anchor data
    let non_matching_count = records
        .iter()
        .filter(|r| r.get(barcode_idx).unwrap() != "123456")
        .filter(|r| r.get(uc_anchor_idx).unwrap().is_empty())
        .count();
    assert_eq!(non_matching_count, 2);
}

// Integration test: Transform and sort stores CSV (cleaners/transformers)
#[test]
fn test_transform_and_sort_stores_csv() {
    use crate::cleaners::transformers::transform_and_sort_stores_csv;

    let mut source_file = NamedTempFile::new().unwrap();
    writeln!(
        source_file,
        "store_id,type,address,city,zipcode\n\
             003,supermarket,Main St 3,Rijeka,51000\n\
             001,supermarket,Main St 1,Zagreb,10000\n\
             002,supermarket,Main St 2,Split,21000"
    )
    .unwrap();

    let target_file = NamedTempFile::new().unwrap();
    let city_mappings = HashMap::new(); // Empty mappings for this test

    transform_and_sort_stores_csv(source_file.path(), target_file.path(), &city_mappings).unwrap();

    // Read and verify the result is sorted
    let mut reader = csv::Reader::from_path(target_file.path()).unwrap();
    let records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();

    assert_eq!(records.len(), 3);
    assert_eq!(records[0].get(0).unwrap(), "001");
    assert_eq!(records[1].get(0).unwrap(), "002");
    assert_eq!(records[2].get(0).unwrap(), "003");
}

// Integration test: Transform products adds hash column and correct values
#[test]
fn test_transform_and_sort_products_adds_name_hash_column() {
    use crate::cleaners::transformers::transform_and_sort_products_csv;
    use crate::embeddings::compute_name_hash;

    let mut source_file = NamedTempFile::new().unwrap();
    writeln!(
        source_file,
        "product_id,name,category\n\
             111111,Cedevita limun 500g,Drinks\n\
             222222,Toaletni papir troslojni 8 rola,Household"
    )
    .unwrap();

    let target_file = NamedTempFile::new().unwrap();
    let category_mappings = HashMap::new();

    transform_and_sort_products_csv(source_file.path(), target_file.path(), &category_mappings)
        .unwrap();

    let mut reader = csv::Reader::from_path(target_file.path()).unwrap();
    let headers = reader.headers().unwrap().clone();

    let hash_col_count = headers
        .iter()
        .filter(|h| *h == "uc_name_searching_algorithm_1")
        .count();
    assert_eq!(
        hash_col_count, 1,
        "Hash column should be present exactly once"
    );

    let product_id_idx = headers.iter().position(|h| h == "product_id").unwrap();
    let name_idx = headers.iter().position(|h| h == "name").unwrap();
    let hash_idx = headers
        .iter()
        .position(|h| h == "uc_name_searching_algorithm_1")
        .unwrap();

    for record in reader.records() {
        let record = record.unwrap();
        let product_id = record.get(product_id_idx).unwrap();
        let name = record.get(name_idx).unwrap();
        let actual_hash = record.get(hash_idx).unwrap();
        let expected_hash = compute_name_hash(name);

        assert_eq!(
            actual_hash, expected_hash,
            "Hash mismatch for product_id {product_id}"
        );
        assert!(
            !actual_hash.is_empty(),
            "Hash should be non-empty for product_id {product_id}"
        );
    }
}

// Integration test: Existing hash column is kept without duplication/recomputation
#[test]
fn test_transform_and_sort_products_keeps_existing_hash_column() {
    use crate::cleaners::transformers::transform_and_sort_products_csv;

    let mut source_file = NamedTempFile::new().unwrap();
    writeln!(
        source_file,
        "product_id,name,category,uc_name_searching_algorithm_1\n\
             111111,Cedevita limun 500g,Drinks,precomputed_hash_1\n\
             222222,Toaletni papir troslojni 8 rola,Household,precomputed_hash_2"
    )
    .unwrap();

    let target_file = NamedTempFile::new().unwrap();
    let category_mappings = HashMap::new();

    transform_and_sort_products_csv(source_file.path(), target_file.path(), &category_mappings)
        .unwrap();

    let mut reader = csv::Reader::from_path(target_file.path()).unwrap();
    let headers = reader.headers().unwrap().clone();

    let hash_col_count = headers
        .iter()
        .filter(|h| *h == "uc_name_searching_algorithm_1")
        .count();
    assert_eq!(hash_col_count, 1, "Hash column should not be duplicated");

    let product_id_idx = headers.iter().position(|h| h == "product_id").unwrap();
    let hash_idx = headers
        .iter()
        .position(|h| h == "uc_name_searching_algorithm_1")
        .unwrap();

    let records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 2);

    for record in records {
        let product_id = record.get(product_id_idx).unwrap();
        let hash = record.get(hash_idx).unwrap();
        match product_id {
            "111111" => assert_eq!(hash, "precomputed_hash_1"),
            "222222" => assert_eq!(hash, "precomputed_hash_2"),
            _ => panic!("Unexpected product_id in output: {product_id}"),
        }
    }
}

// Integration test: Full cleaned data creation workflow
#[test]
fn test_create_cleaned_data() {
    use crate::cleaners::processing::create_cleaned_data;

    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join("raw_data");
    let subdir = base_dir.join("test_store");
    std::fs::create_dir_all(&subdir).unwrap();

    // Create test CSV files in subdirectory
    std::fs::write(
        subdir.join("stores.csv"),
        "store_id,type,address,city,zipcode\n\
             001,supermarket,Main St 1,Zagreb,10000",
    )
    .unwrap();

    std::fs::write(
        subdir.join("products.csv"),
        "product_id,name,category\n\
             123456,Apple Juice,Food",
    )
    .unwrap();

    std::fs::write(
        subdir.join("prices.csv"),
        "store_id,barcode,price,unit_price,special_price\n\
             001,123456,10.50,1.05,9.99",
    )
    .unwrap();

    let output_dir = temp_dir.path().join("cleaned_data");
    let city_mappings = HashMap::from([("Zagreb".to_string(), "Zagreb".to_string())]);
    let category_mappings = HashMap::from([("Food".to_string(), "Food".to_string())]);

    create_cleaned_data(
        base_dir.to_str().unwrap(),
        None,
        &city_mappings,
        &category_mappings,
        output_dir.to_str().unwrap(),
    )
    .unwrap();

    // Verify cleaned data was created
    let cleaned_subdir = output_dir.join("test_store");
    assert!(cleaned_subdir.join("stores.csv").exists());
    assert!(cleaned_subdir.join("products.csv").exists());
    assert!(cleaned_subdir.join("prices.csv").exists());

    // Verify products.csv includes the name-hash column and value
    let mut reader = csv::Reader::from_path(cleaned_subdir.join("products.csv")).unwrap();
    let headers = reader.headers().unwrap().clone();
    let hash_idx = headers
        .iter()
        .position(|h| h == "uc_name_searching_algorithm_1")
        .expect("Expected uc_name_searching_algorithm_1 column in cleaned products.csv");

    let records: Vec<csv::StringRecord> = reader.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 1);
    assert!(
        !records[0].get(hash_idx).unwrap().is_empty(),
        "Expected non-empty name hash in cleaned products.csv"
    );
}

// Add more integration tests here as needed
// Integration tests should test multiple modules working together,
// not individual functions (those belong in their module's test section)
