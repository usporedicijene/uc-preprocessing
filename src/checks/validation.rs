use std::collections::HashSet;
use tracing::{debug, info};

/// Validates that all found items are in the reference set
///
/// This function compares items found in data files against a reference set,
/// and reports any new items that need to be added to the mapping file.
///
/// # Arguments
/// - `reference_set`: Set of items from the reference mapping file
/// - `found_set`: Set of items found in the data files
/// - `item_type`: Name of item type for logging (e.g., "cities", "categories")
/// - `mapping_file`: Path to the mapping file for error messages
/// - `debug_env_var`: Optional environment variable name to enable debug output
///
/// # Returns
/// - `Ok(())` if all found items are in the reference set
/// - `Err(String)` with formatted error message if new items are found
pub fn validate_against_reference(
    reference_set: &HashSet<String>,
    found_set: &HashSet<String>,
    item_type: &str,
    mapping_file: &str,
    debug_env_var: Option<&str>,
) -> Result<(), String> {
    info!(
        "Found {} unique {} in data files",
        found_set.len(),
        item_type
    );
    info!("{}", "-".repeat(50));

    // Debug output if requested
    if let Some(debug_var) = debug_env_var
        && std::env::var(debug_var).is_ok()
    {
        debug!("Reference {} from mapping file:", item_type);
        let mut ref_sorted: Vec<_> = reference_set.iter().collect();
        ref_sorted.sort();
        for item in &ref_sorted {
            debug!("  '{}'", item);
        }

        debug!("Found {} in data:", item_type);
        let mut found_sorted: Vec<_> = found_set.iter().collect();
        found_sorted.sort();
        for item in &found_sorted {
            debug!("  '{}'", item);
        }
    }

    // Find new items (in data but not in reference)
    let new_items: HashSet<_> = found_set.difference(reference_set).collect();

    if !new_items.is_empty() {
        let new_count = new_items.len();
        info!("New {} found ({} items):", item_type, new_count);
        info!("{}", "-".repeat(30));
        let mut new_sorted: Vec<_> = new_items.into_iter().collect();
        new_sorted.sort();
        for item in new_sorted {
            info!("{};{}", item, item);
        }

        return Err(format!(
            "Found {} new {} that are not in {}\n\
             Please update {} with the {} listed above.",
            new_count, item_type, mapping_file, mapping_file, item_type
        ));
    } else if found_set.is_empty() {
        return Err(format!(
            "No {} found in data files\n\
             Please check if the data files are empty or if the {} are not in the reference file.",
            item_type, item_type
        ));
    } else {
        info!("All {} found in data files are already mapped.", item_type);
    }

    Ok(())
}

/// Validates cities found in stores.csv files against reference cities
///
/// # Arguments
/// - `reference_cities`: Set of cities from cities mapping file
/// - `found_cities`: Set of cities found in stores.csv files
/// - `cities_mapping_file`: Path to cities mapping file
///
/// # Returns
/// - `Ok(())` if validation succeeds
/// - `Err(String)` if validation fails
pub fn validate_cities(
    reference_cities: &HashSet<String>,
    found_cities: &HashSet<String>,
    cities_mapping_file: &str,
) -> Result<(), String> {
    validate_against_reference(
        reference_cities,
        found_cities,
        "cities",
        cities_mapping_file,
        Some("DEBUG_CITIES"),
    )
}

/// Validates categories found in products.csv files against reference categories
///
/// # Arguments
/// - `reference_categories`: Set of categories from categories mapping file
/// - `found_categories`: Set of categories found in products.csv files
/// - `categories_mapping_file`: Path to categories mapping file
///
/// # Returns
/// - `Ok(())` if validation succeeds
/// - `Err(String)` if validation fails
pub fn validate_categories(
    reference_categories: &HashSet<String>,
    found_categories: &HashSet<String>,
    categories_mapping_file: &str,
) -> Result<(), String> {
    validate_against_reference(
        reference_categories,
        found_categories,
        "categories",
        categories_mapping_file,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_against_reference_all_found() {
        let reference: HashSet<String> = ["Zagreb", "Split", "Rijeka"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let found: HashSet<String> = ["Zagreb", "Split"].iter().map(|s| s.to_string()).collect();

        let result = validate_against_reference(&reference, &found, "cities", "mapping.csv", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_against_reference_new_items_found() {
        let reference: HashSet<String> =
            ["Zagreb", "Split"].iter().map(|s| s.to_string()).collect();
        let found: HashSet<String> = ["Zagreb", "Split", "Osijek"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let result = validate_against_reference(&reference, &found, "cities", "mapping.csv", None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("1 new cities"));
        assert!(err.contains("mapping.csv"));
    }

    #[test]
    fn test_validate_against_reference_empty_found() {
        let reference: HashSet<String> =
            ["Zagreb", "Split"].iter().map(|s| s.to_string()).collect();
        let found: HashSet<String> = HashSet::new();

        let result = validate_against_reference(&reference, &found, "cities", "mapping.csv", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No cities found"));
    }

    #[test]
    fn test_validate_cities() {
        let reference: HashSet<String> =
            ["Zagreb", "Split"].iter().map(|s| s.to_string()).collect();
        let found: HashSet<String> = ["Zagreb"].iter().map(|s| s.to_string()).collect();

        let result = validate_cities(&reference, &found, "cities_mapping.csv");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_categories() {
        let reference: HashSet<String> = ["Food", "Drinks"].iter().map(|s| s.to_string()).collect();
        let found: HashSet<String> = ["Food", "Electronics"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let result = validate_categories(&reference, &found, "categories_mapping.csv");
        assert!(result.is_err());
    }
}
