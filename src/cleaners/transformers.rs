use crate::cleaners::types::AnchorPriceData;
use crate::derived_traits::calculators::calculate_derived_price;
use crate::derived_traits::calculators::calculate_price_change;
use crate::embeddings::compute_name_hash;
use crate::loaders::csv_loaders::load_product_categories;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::loaders::csv_loaders::load_anchor_price_data;

pub fn sort_and_add_derived_price_csv(
    source_file: &Path,
    target_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(source_file)?;
    let mut reader = csv::Reader::from_reader(input_file);

    let output_file = File::create(target_file)?;
    let mut writer = csv::Writer::from_writer(output_file);

    // Read headers and add derived_price column if it doesn't exist
    let original_headers = reader.headers()?.clone();
    let mut new_headers = original_headers
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>();

    // Add derived_price column if it doesn't already exist
    if !new_headers.iter().any(|h| h == "derived_price") {
        new_headers.push("derived_price".to_string());
    }

    writer.write_record(&new_headers)?;

    // Find price-related column indices
    let price_idx = original_headers.iter().position(|h| h == "price");
    let special_price_idx = original_headers.iter().position(|h| h == "special_price");

    if price_idx.is_none() {
        println!("Warning: No 'price' column found in {source_file:?}");
    }
    if special_price_idx.is_none() {
        println!("Warning: No 'special_price' column found in {source_file:?}");
    }

    let mut records = Vec::new();
    let mut invalid_rows = 0;
    let mut total_rows = 0;
    let mut filtered_rows = 0;

    // Process and collect all records
    for result in reader.records() {
        total_rows += 1;
        match result {
            Ok(record) => {
                // Calculate derived price first to check if row should be filtered
                let derived_price = calculate_derived_price(
                    price_idx.and_then(|idx| record.get(idx)),
                    special_price_idx.and_then(|idx| record.get(idx)),
                );

                // Skip rows with invalid or non-positive derived prices
                if let Ok(price_value) = derived_price.trim().parse::<f64>() {
                    if price_value <= 0.0 {
                        filtered_rows += 1;
                        continue; // Skip this row
                    }
                } else if !derived_price.trim().is_empty() {
                    // If derived_price is not empty but can't be parsed as a number, skip it
                    filtered_rows += 1;
                    continue;
                } else {
                    // If derived_price is empty (no valid price data), skip it
                    filtered_rows += 1;
                    continue;
                }

                let mut new_record = record.iter().map(|f| f.to_string()).collect::<Vec<_>>();

                // Add derived_price column if it doesn't already exist in the original data
                if !original_headers.iter().any(|h| h == "derived_price") {
                    new_record.push(derived_price);
                }

                records.push(csv::StringRecord::from(new_record));
            }
            Err(e) => {
                invalid_rows += 1;
                eprintln!("Warning: Skipped invalid CSV row {total_rows} in {source_file:?}: {e}");
            }
        }
    }

    // Sort records and remove duplicates
    let duplicate_count = sort_and_deduplicate_records(&mut records, source_file);

    // Write sorted records
    for record in records {
        writer.write_record(&record)?;
    }

    writer.flush()?;

    if invalid_rows > 0 {
        eprintln!(
            "Warning: Skipped {invalid_rows} invalid CSV rows when processing {source_file:?}"
        );
    }

    if filtered_rows > 0 {
        eprintln!(
            "Info: Filtered out {filtered_rows} rows with invalid/zero prices in {source_file:?}"
        );
    }

    if duplicate_count > 0 {
        eprintln!("Info: Removed {duplicate_count} duplicate rows when processing {source_file:?}");
    }

    Ok(())
}

/// Helper function to sort records and remove duplicates
pub fn sort_and_deduplicate_records(
    records: &mut Vec<csv::StringRecord>,
    source_file: &Path,
) -> usize {
    // Sort records by all columns (lexicographically) using parallel sort
    records.par_sort_by(|a, b| {
        for i in 0..a.len().min(b.len()) {
            let cmp = a[i].cmp(&b[i]);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }
        }
        a.len().cmp(&b.len())
    });

    let original_len = records.len();

    // Check if this is a price file to apply special deduplication logic
    let is_price_file = source_file
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.contains("prices"))
        .unwrap_or(false);

    if is_price_file {
        // For price files, remove ALL entries where store_id + product_id appears more than once
        let mut key_counts = std::collections::HashMap::new();

        // First pass: count occurrences of each store_id + product_id combination
        for record in records.iter() {
            if record.len() >= 2 {
                let key = format!("{}_{}", &record[0], &record[1]);
                *key_counts.entry(key).or_insert(0) += 1;
            }
        }

        // Second pass: keep only records with unique store_id + product_id combinations
        records.retain(|record| {
            if record.len() >= 2 {
                let key = format!("{}_{}", &record[0], &record[1]);
                key_counts.get(&key).is_none_or(|&count| count == 1)
            } else {
                true // Keep malformed records, they'll be handled elsewhere
            }
        });
    } else {
        // For non-price files, use standard deduplication (remove consecutive duplicates)
        records.dedup();
    }

    let duplicate_count = original_len - records.len();

    if duplicate_count > 0 {
        if is_price_file {
            eprintln!(
                "Info: Removed {duplicate_count} rows with duplicate store_id+product_id combinations in {source_file:?}"
            );
        } else {
            eprintln!("Info: Removed {duplicate_count} duplicate rows in {source_file:?}");
        }
    }

    duplicate_count
}

pub fn transform_and_merge_prices_with_anchor(
    source_prices: &Path,
    anchor_prices: &Path,
    target_file: &Path,
    category_mappings: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // First, merge prices with anchor normally
    merge_prices_with_anchor(source_prices, anchor_prices, target_file)?;

    // Then, filter out products with categories ending in "~~"
    let products_file = source_prices
        .parent()
        .map(|p| p.join("products.csv"))
        .unwrap_or_else(|| Path::new("products.csv").to_path_buf());
    let product_categories = load_product_categories(&products_file, category_mappings)?;

    // Create a temporary file for the filtered result
    let temp_file = target_file.with_extension("tmp");

    // Read the merged file and filter out excluded categories
    let input_file = File::open(target_file)?;
    let mut reader = csv::Reader::from_reader(input_file);

    let output_file = File::create(&temp_file)?;
    let mut writer = csv::Writer::from_writer(output_file);

    // Write headers
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    // Find barcode/product_id column
    let barcode_idx = headers
        .iter()
        .position(|h| h == "barcode")
        .or_else(|| headers.iter().position(|h| h == "product_id"));

    let mut filtered_count = 0;
    let mut records = Vec::new();

    // Collect all records that should be included
    for result in reader.records() {
        match result {
            Ok(record) => {
                let mut should_include = true;

                // Check if product category ends with "~~"
                if let Some(barcode_column) = barcode_idx
                    && let Some(barcode) = record.get(barcode_column)
                    && let Some(category) = product_categories.get(barcode.trim())
                    && category.ends_with("~~")
                {
                    should_include = false;
                    filtered_count += 1;
                }

                if should_include {
                    records.push(record);
                }
            }
            Err(e) => {
                eprintln!("Warning: Error reading record in {target_file:?}: {e}");
            }
        }
    }

    // Sort records and remove duplicates
    let duplicate_count = sort_and_deduplicate_records(&mut records, target_file);

    // Write sorted records
    for record in records {
        writer.write_record(&record)?;
    }

    writer.flush()?;

    // Replace the original file with the filtered one
    std::fs::rename(&temp_file, target_file)?;

    if filtered_count > 0 {
        eprintln!(
            "Info: Filtered out {filtered_count} price records for products with excluded categories in {target_file:?}"
        );
    }

    if duplicate_count > 0 {
        eprintln!("Info: Removed {duplicate_count} duplicate rows when filtering {target_file:?}");
    }

    Ok(())
}

// Transform and sort products.csv file
// This function transforms the category column by mapping the category to the new category
// and sorts the records by the category column
pub fn transform_and_sort_products_csv(
    source_file: &Path,
    target_file: &Path,
    category_mappings: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(source_file)?;
    let mut reader = csv::Reader::from_reader(input_file);

    let output_file = File::create(target_file)?;
    let mut writer = csv::Writer::from_writer(output_file);

    let headers = reader.headers()?.clone();

    let mut new_headers = headers.iter().map(|h| h.to_string()).collect::<Vec<_>>();
    let has_hash_col = new_headers
        .iter()
        .any(|h| h == "uc_name_searching_algorithm_1");
    if !has_hash_col {
        new_headers.push("uc_name_searching_algorithm_1".to_string());
    }
    writer.write_record(&new_headers)?;

    // Find category column index
    let category_col_index = headers
        .iter()
        .position(|h| h == "category")
        .ok_or("Category column not found in products.csv")?;

    let name_col_index = headers.iter().position(|h| h == "name");

    let mut invalid_rows = 0;
    let mut total_rows = 0;
    let mut filtered_rows = 0; // Track rows filtered out due to categories ending with "~~"
    let mut records = Vec::new();

    // Process and collect all records
    for result in reader.records() {
        total_rows += 1;
        match result {
            Ok(mut record) => {
                // Replace category value with mapping if it exists
                let mut final_category = String::new();
                if let Some(category_value) = record.get(category_col_index) {
                    if let Some(mapped_category) = category_mappings.get(category_value.trim()) {
                        final_category = mapped_category.clone();
                        record = csv::StringRecord::from(
                            record
                                .iter()
                                .enumerate()
                                .map(|(i, field)| {
                                    if i == category_col_index {
                                        mapped_category.as_str()
                                    } else {
                                        field
                                    }
                                })
                                .collect::<Vec<_>>(),
                        );
                    } else {
                        final_category = category_value.trim().to_string();
                    }
                }

                // Skip products whose mapped categories end with "~~"
                if final_category.ends_with("~~") {
                    filtered_rows += 1;
                    continue; // Skip this row
                }

                if !has_hash_col {
                    let name_value = name_col_index
                        .and_then(|idx| record.get(idx))
                        .unwrap_or("");
                    let hash = compute_name_hash(name_value);
                    let mut fields: Vec<String> =
                        record.iter().map(|f| f.to_string()).collect();
                    fields.push(hash);
                    records.push(csv::StringRecord::from(fields));
                } else {
                    records.push(record);
                }
            }
            Err(e) => {
                invalid_rows += 1;
                eprintln!("Warning: Skipped invalid CSV row {total_rows} in {source_file:?}: {e}");
            }
        }
    }

    // Sort records and remove duplicates
    let duplicate_count = sort_and_deduplicate_records(&mut records, source_file);

    // Store count before moving the vector
    let processed_rows = records.len();

    // Write sorted records
    for record in records {
        if let Err(e) = writer.write_record(&record) {
            eprintln!("Warning: Failed to write row in {target_file:?}: {e}");
            invalid_rows += 1;
        }
    }

    writer.flush()?;

    if invalid_rows > 0 {
        eprintln!(
            "Warning: Skipped {invalid_rows} invalid CSV rows when transforming {source_file:?} (processed {processed_rows} valid rows)"
        );
    }

    if filtered_rows > 0 {
        eprintln!(
            "Info: Filtered out {filtered_rows} products with categories ending in '~~' in {source_file:?}"
        );
    }

    if duplicate_count > 0 {
        eprintln!(
            "Info: Removed {duplicate_count} duplicate rows when transforming {source_file:?}"
        );
    }

    Ok(())
}

pub fn transform_and_sort_stores_csv(
    source_file: &Path,
    target_file: &Path,
    city_mappings: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(source_file)?;
    let mut reader = csv::Reader::from_reader(input_file);

    let output_file = File::create(target_file)?;
    let mut writer = csv::Writer::from_writer(output_file);

    // Write header
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    // Find city column index
    let city_col_index = headers
        .iter()
        .position(|h| h == "city")
        .ok_or("City column not found in stores.csv")?;

    let mut invalid_rows = 0;
    let mut total_rows = 0;
    let mut records = Vec::new();

    // Process and collect all records
    for result in reader.records() {
        total_rows += 1;
        match result {
            Ok(mut record) => {
                // Replace city value with mapping if it exists
                if let Some(city_value) = record.get(city_col_index)
                    && let Some(mapped_city) = city_mappings.get(city_value.trim())
                {
                    record = csv::StringRecord::from(
                        record
                            .iter()
                            .enumerate()
                            .map(|(i, field)| {
                                if i == city_col_index {
                                    mapped_city.as_str()
                                } else {
                                    field
                                }
                            })
                            .collect::<Vec<_>>(),
                    );
                }

                records.push(record);
            }
            Err(e) => {
                invalid_rows += 1;
                eprintln!("Warning: Skipped invalid CSV row {total_rows} in {source_file:?}: {e}");
            }
        }
    }

    // Sort records and remove duplicates
    let duplicate_count = sort_and_deduplicate_records(&mut records, source_file);

    // Store count before moving the vector
    let processed_rows = records.len();

    // Write sorted records
    for record in records {
        if let Err(e) = writer.write_record(&record) {
            eprintln!("Warning: Failed to write row in {target_file:?}: {e}");
            invalid_rows += 1;
        }
    }

    writer.flush()?;

    if invalid_rows > 0 {
        eprintln!(
            "Warning: Skipped {invalid_rows} invalid CSV rows when transforming {source_file:?} (processed {processed_rows} valid rows)"
        );
    }

    if duplicate_count > 0 {
        eprintln!(
            "Info: Removed {duplicate_count} duplicate rows when transforming {source_file:?}"
        );
    }

    Ok(())
}

pub fn merge_prices_with_anchor(
    source_prices: &Path,
    anchor_prices: &Path,
    target_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read anchor data first
    let anchor_data = load_anchor_price_data(anchor_prices)?;

    let input_file = File::open(source_prices)?;
    let mut reader = csv::Reader::from_reader(input_file);

    let output_file = File::create(target_file)?;
    let mut writer = csv::Writer::from_writer(output_file);

    // Read headers and add anchor columns if they don't exist
    let original_headers = reader.headers()?.clone();
    let mut new_headers = original_headers
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>();

    // Add derived columns for anchor comparison
    let derived_columns = [
        "uc_anchor_price",
        "uc_anchor_unit_price",
        "uc_anchor_special_price",
        "derived_price",
        "uc_anchor_derived_price",
        "derived_price_change",
    ];
    for col in &derived_columns {
        // Always add uc_anchor_* columns and derived_price_change, only check for derived_price
        let should_add = if *col == "derived_price" {
            !new_headers.iter().any(|h| h == *col)
        } else if col.starts_with("uc_anchor_") || *col == "derived_price_change" {
            // Always add uc_anchor_* and derived_price_change columns
            true
        } else {
            !new_headers.iter().any(|h| h == *col)
        };

        if should_add {
            new_headers.push(col.to_string());
        }
    }

    writer.write_record(&new_headers)?;

    // Find key column indices - support both barcode and product_id
    let store_id_idx = original_headers.iter().position(|h| h == "store_id");
    let barcode_idx = original_headers
        .iter()
        .position(|h| h == "barcode")
        .or_else(|| original_headers.iter().position(|h| h == "product_id"));

    // Find price-related column indices
    let price_idx = original_headers.iter().position(|h| h == "price");
    let special_price_idx = original_headers.iter().position(|h| h == "special_price");

    let mut records = Vec::new();
    let mut invalid_rows = 0;
    let mut total_rows = 0;
    let mut filtered_rows = 0;
    let mut matched_rows = 0;

    // Process and collect all records
    for result in reader.records() {
        total_rows += 1;
        match result {
            Ok(record) => {
                // Calculate derived price first to check if row should be filtered
                let derived_price = calculate_derived_price(
                    price_idx.and_then(|idx| record.get(idx)),
                    special_price_idx.and_then(|idx| record.get(idx)),
                );

                // Skip rows with invalid or non-positive derived prices
                if let Ok(price_value) = derived_price.trim().parse::<f64>() {
                    if price_value <= 0.0 {
                        filtered_rows += 1;
                        continue; // Skip this row
                    }
                } else if !derived_price.trim().is_empty() {
                    // If derived_price is not empty but can't be parsed as a number, skip it
                    filtered_rows += 1;
                    continue;
                } else {
                    // If derived_price is empty (no valid price data), skip it
                    filtered_rows += 1;
                    continue;
                }

                let mut new_record = record.iter().map(|f| f.to_string()).collect::<Vec<_>>();

                // Try to find matching anchor data
                let anchor_key =
                    if let (Some(store_idx), Some(bar_idx)) = (store_id_idx, barcode_idx) {
                        if let (Some(store_id), Some(barcode)) =
                            (record.get(store_idx), record.get(bar_idx))
                        {
                            Some(format!("{}|{}", store_id.trim(), barcode.trim()))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                let anchor_data = if let Some(key) = &anchor_key {
                    if let Some(data) = anchor_data.get(key) {
                        matched_rows += 1;
                        data.clone()
                    } else {
                        AnchorPriceData::default()
                    }
                } else {
                    AnchorPriceData::default()
                };

                // Calculate and add derived columns
                for col in &derived_columns {
                    // Always add uc_anchor_* columns and derived_price_change, only check for derived_price
                    let should_add = if *col == "derived_price" {
                        !original_headers.iter().any(|h| h == *col)
                    } else if col.starts_with("uc_anchor_") || *col == "derived_price_change" {
                        // Always add uc_anchor_* and derived_price_change columns
                        true
                    } else {
                        !original_headers.iter().any(|h| h == *col)
                    };

                    if should_add {
                        let derived_value = if *col == "derived_price" {
                            // We already calculated and validated this above
                            derived_price.clone()
                        } else if *col == "uc_anchor_price" {
                            // Use anchor price if available
                            anchor_data.price.clone().unwrap_or_default()
                        } else if *col == "uc_anchor_unit_price" {
                            // Use anchor unit_price if available
                            anchor_data.unit_price.clone().unwrap_or_default()
                        } else if *col == "uc_anchor_special_price" {
                            // Use anchor special_price if available
                            anchor_data.special_price.clone().unwrap_or_default()
                        } else if *col == "uc_anchor_derived_price" {
                            // Use anchor derived_price if available
                            anchor_data.derived_price.clone().unwrap_or_default()
                        } else if *col == "derived_price_change" {
                            // Calculate price change if we have both current and anchor derived prices
                            if let Some(anchor_derived) = &anchor_data.derived_price {
                                if !anchor_derived.trim().is_empty() {
                                    calculate_price_change(&derived_price, anchor_derived)
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };
                        new_record.push(derived_value);
                    }
                }

                records.push(csv::StringRecord::from(new_record));
            }
            Err(e) => {
                invalid_rows += 1;
                eprintln!(
                    "Warning: Skipped invalid CSV row {total_rows} in {source_prices:?}: {e}"
                );
            }
        }
    }

    // Sort records and remove duplicates
    let duplicate_count = sort_and_deduplicate_records(&mut records, source_prices);

    // Store count before moving the vector
    let processed_rows = records.len();

    // Write sorted records
    for record in records {
        if let Err(e) = writer.write_record(&record) {
            eprintln!("Warning: Failed to write row in {target_file:?}: {e}");
            invalid_rows += 1;
        }
    }

    writer.flush()?;

    if invalid_rows > 0 {
        eprintln!(
            "Warning: Skipped {invalid_rows} invalid CSV rows when merging {source_prices:?}"
        );
    }

    if filtered_rows > 0 {
        eprintln!(
            "Info: Filtered out {filtered_rows} rows (invalid prices and excluded categories) in {source_prices:?}"
        );
    }

    if duplicate_count > 0 {
        eprintln!("Info: Removed {duplicate_count} duplicate rows when merging {source_prices:?}");
    }

    // Extract directory name and filename for better logging
    let file_info = source_prices
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|dir| format!("{dir}/prices.csv"))
        .unwrap_or_else(|| "prices.csv".to_string());

    // Calculate percentage - use records.len() instead of total_rows for valid processed rows
    let percentage = if processed_rows > 0 {
        (matched_rows as f64 / processed_rows as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "  Anchor matching: {matched_rows}/{processed_rows} ({percentage:.2}%) rows matched in {file_info} (filtered {filtered_rows} invalid price rows)"
    );

    Ok(())
}
