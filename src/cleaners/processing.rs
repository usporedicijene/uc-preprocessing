use crate::cleaners::transformers::sort_and_add_derived_price_csv;
use crate::cleaners::transformers::transform_and_merge_prices_with_anchor;
use crate::cleaners::transformers::transform_and_sort_products_csv;
use crate::cleaners::transformers::transform_and_sort_stores_csv;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tracing::{error, info};
use walkdir::WalkDir;

pub fn create_cleaned_data(
    base_path: &str,
    anchor_path: Option<&str>,
    city_mappings: &HashMap<String, String>,
    category_mappings: &HashMap<String, String>,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(base_path);
    let anchor_dir = anchor_path.map(Path::new);
    let cleaned_dir = Path::new(output_dir);

    // Find all subdirectories with stores.csv files
    let subdirs: Vec<_> = WalkDir::new(base_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_dir())
        .collect();

    // Process subdirectories in parallel
    let processed_count = Mutex::new(0);

    subdirs.par_iter().for_each(|entry| {
        let subdir_name = entry.file_name().to_string_lossy();
        let source_subdir = entry.path();
        let target_subdir = cleaned_dir.join(&*subdir_name);

        // Create target subdirectory
        if let Err(e) = fs::create_dir_all(&target_subdir) {
            error!("Error creating directory {:?}: {}", target_subdir, e);
            return;
        }

        // Handle products.csv (transform categories and sort)
        let source_products = source_subdir.join("products.csv");
        let target_products = target_subdir.join("products.csv");
        if source_products.exists() {
            if let Err(e) = transform_and_sort_products_csv(
                &source_products,
                &target_products,
                category_mappings,
            ) {
                error!("Error processing {}/products.csv: {}", subdir_name, e);
            } else {
                info!("  Transformed and sorted {}/products.csv", subdir_name);
            }
        }

        // Handle prices.csv (merge with anchor if available, then sort)
        let source_prices = source_subdir.join("prices.csv");
        let target_prices = target_subdir.join("prices.csv");
        if source_prices.exists() {
            let anchor_prices = anchor_dir.map(|dir| dir.join(&*subdir_name).join("prices.csv"));

            match anchor_prices {
                Some(anchor_path) if anchor_path.exists() => {
                    if let Err(e) = transform_and_merge_prices_with_anchor(
                        &source_prices,
                        &anchor_path,
                        &target_prices,
                        category_mappings,
                    ) {
                        error!(
                            "Error merging {}/prices.csv with anchor: {}",
                            subdir_name, e
                        );
                    } else {
                        info!(
                            "  Merged and sorted {}/prices.csv with anchor data",
                            subdir_name
                        );
                    }
                }
                _ => {
                    // No anchor data available, add derived_price and sort
                    if let Err(e) = sort_and_add_derived_price_csv(&source_prices, &target_prices) {
                        error!("Error processing {}/prices.csv: {}", subdir_name, e);
                    } else {
                        info!(
                            "  Sorted and added derived_price to {}/prices.csv (no anchor data)",
                            subdir_name
                        );
                    }
                }
            }
        }

        // Transform and sort stores.csv if it exists
        let source_stores = source_subdir.join("stores.csv");
        if source_stores.exists() {
            if let Err(e) = transform_and_sort_stores_csv(
                &source_stores,
                &target_subdir.join("stores.csv"),
                city_mappings,
            ) {
                error!("Error processing {}/stores.csv: {}", subdir_name, e);
            } else {
                info!("  Transformed and sorted {}/stores.csv", subdir_name);
            }
        }

        let mut count = processed_count.lock().unwrap();
        *count += 1;
    });

    let final_count = *processed_count.lock().unwrap();
    info!("Processed {} directories in {}", final_count, output_dir);
    Ok(())
}
