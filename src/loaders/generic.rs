use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

/// Trait for types that represent a mapping with "from" and "to" fields
pub trait Mapping {
    fn from(&self) -> &str;
    fn to(&self) -> &str;
}

/// Generic function to load a HashSet of values from the "from" column of a CSV file
///
/// This is used for loading reference data where we only care about the first column.
///
/// # Arguments
/// - `mapping_file`: Path to the CSV file
/// - `item_type`: Description of what's being loaded (e.g., "cities", "categories") for logging
///
/// # Returns
/// HashSet of unique values from the "from" column
pub fn load_reference_set<T>(
    mapping_file: &str,
    item_type: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de> + Mapping,
{
    let mut reference_set = HashSet::new();

    if !Path::new(mapping_file).exists() {
        return Err(format!("Reference file '{mapping_file}' does not exist").into());
    }

    let file = File::open(mapping_file)?;
    let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);

    let mut invalid_rows = 0;
    let mut total_rows = 0;

    for result in reader.deserialize() {
        total_rows += 1;
        match result {
            Ok(mapping) => {
                let mapping: T = mapping;
                let value = mapping.from().trim();
                if !value.is_empty() {
                    reference_set.insert(value.to_string());
                }
            }
            Err(e) => {
                invalid_rows += 1;
                eprintln!("Warning: Skipped invalid CSV row {total_rows} in {mapping_file}: {e}");
            }
        }
    }

    if invalid_rows > 0 {
        eprintln!("Warning: Skipped {invalid_rows} invalid CSV rows in {mapping_file}");
    }

    if !reference_set.is_empty() {
        println!(
            "Loaded {} reference {item_type} from first column of {mapping_file}",
            reference_set.len()
        );
    }

    Ok(reference_set)
}

/// Generic function to load a HashMap of mappings from a CSV file
///
/// This loads both "from" and "to" columns to create a mapping.
///
/// # Arguments
/// - `mapping_file`: Path to the CSV file
/// - `item_type`: Description of what's being loaded (e.g., "city", "category") for logging
///
/// # Returns
/// HashMap where keys are "from" values and values are "to" values
pub fn load_mapping_map<T>(
    mapping_file: &str,
    item_type: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de> + Mapping,
{
    let mut mapping_map = HashMap::new();

    let file = File::open(mapping_file)?;
    let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);

    let mut invalid_rows = 0;
    let mut total_rows = 0;

    for result in reader.deserialize() {
        total_rows += 1;
        match result {
            Ok(mapping) => {
                let mapping: T = mapping;
                let from = mapping.from().trim().to_string();
                let to = mapping.to().trim().to_string();
                if !from.is_empty() && !to.is_empty() {
                    mapping_map.insert(from, to);
                }
            }
            Err(e) => {
                invalid_rows += 1;
                eprintln!("Warning: Skipped invalid CSV row {total_rows} in {mapping_file}: {e}");
            }
        }
    }

    if invalid_rows > 0 {
        eprintln!("Warning: Skipped {invalid_rows} invalid CSV rows in {mapping_file}");
    }

    println!("Loaded {} {item_type} mappings", mapping_map.len());
    Ok(mapping_map)
}
