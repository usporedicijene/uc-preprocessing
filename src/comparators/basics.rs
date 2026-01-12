use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

use crate::comparators::types::ComparisonSummary;
use crate::extractors::{
    extract_market_chain_from_path, extract_product_barcodes, extract_store_ids,
};
use crate::loaders::helpers::{
    count_total_product_instances_in_file, load_product_id_to_barcode_mapping, load_store_names,
};
use crate::loaders::types::{Barcode, MarketChain, StoreId};
use crate::loaders::types::{PriceChange, PriceComparisonResult};

pub fn compare_stores(
    current_path: &str,
    previous_path: &str,
) -> Result<(HashSet<String>, HashSet<String>), Box<dyn std::error::Error>> {
    let current_stores = extract_store_ids(current_path)?;
    let previous_stores = extract_store_ids(previous_path)?;
    Ok((current_stores, previous_stores))
}

pub fn compare_products(
    current_path: &str,
    previous_path: &str,
) -> Result<(HashSet<String>, HashSet<String>), Box<dyn std::error::Error>> {
    let current_products = extract_product_barcodes(current_path)?;
    let previous_products = extract_product_barcodes(previous_path)?;
    Ok((current_products, previous_products))
}

pub fn compare_prices(
    current_path: &str,
    previous_path: &str,
) -> Result<PriceComparisonResult, Box<dyn std::error::Error>> {
    let mut price_changes = HashMap::new();
    let mut total_stores_with_price_data = 0;
    let mut price_changes_by_chain = HashMap::new();
    let mut total_products_by_chain = HashMap::new();

    // Load store names once for efficiency
    let store_names = load_store_names(current_path)?;

    for entry in WalkDir::new(current_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name().to_str() == Some("prices.csv") {
            let current_prices_file = entry.path();
            let relative_path = current_prices_file.strip_prefix(current_path)?;
            let previous_prices_file = Path::new(previous_path).join(relative_path);

            if previous_prices_file.exists() {
                total_stores_with_price_data += 1;

                // Extract market chain from directory path for counting
                let market_chain =
                    extract_market_chain_from_path(current_prices_file, current_path);

                // Count total product-store combinations in this price file for the market chain (ALL product instances, not just changed ones)
                let total_products_in_file =
                    count_total_product_instances_in_file(current_prices_file)?;
                *total_products_by_chain
                    .entry(market_chain.clone())
                    .or_insert(0) += total_products_in_file;

                let store_changes = compare_price_files(
                    current_prices_file,
                    &previous_prices_file,
                    &store_names,
                    current_path,
                )?;
                if !store_changes.is_empty() {
                    let store_name = relative_path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Count price changes by market chain
                    let change_count = store_changes.len();
                    *price_changes_by_chain.entry(market_chain).or_insert(0) += change_count;

                    price_changes.insert(store_name, store_changes);
                }
            }
        }
    }

    Ok((
        price_changes,
        total_stores_with_price_data,
        price_changes_by_chain,
        total_products_by_chain,
    ))
}

pub fn compare_price_files(
    current_file: &Path,
    previous_file: &Path,
    store_names: &HashMap<StoreId, String>,
    base_path: &str,
) -> Result<Vec<PriceChange>, Box<dyn std::error::Error>> {
    // Load product_id to barcode mapping
    let product_mapping = load_product_id_to_barcode_mapping(base_path)?;
    let mut price_changes = Vec::new();

    // Load previous prices using derived_price
    let mut previous_prices = HashMap::new();
    let prev_file = File::open(previous_file).map_err(|e| {
        format!(
            "Failed to open previous price file '{}': {e}",
            previous_file.display()
        )
    })?;
    let mut prev_reader = csv::ReaderBuilder::new()
        .flexible(true) // Allow records with varying number of fields
        .from_reader(prev_file);

    // Get headers to find derived_price column
    let prev_headers = prev_reader.headers()?.clone();
    let derived_price_idx = prev_headers
        .iter()
        .position(|h| h == "derived_price")
        .or_else(|| prev_headers.iter().position(|h| h == "price"));

    if let Some(price_idx) = derived_price_idx {
        for (line_num, result) in prev_reader.records().enumerate() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "Warning: Skipping malformed record {} in previous price file '{}': {e}",
                        line_num + 2,
                        previous_file.display()
                    );
                    continue;
                }
            };

            // Check if record has enough fields
            if record.len() <= price_idx || record.len() < 2 {
                eprintln!(
                    "Warning: Skipping record {} in '{}' - insufficient fields (has {}, needs at least {})",
                    line_num + 2,
                    previous_file.display(),
                    record.len(),
                    price_idx + 1
                );
                continue;
            }

            let store_id = record.get(0).unwrap_or("").to_string();
            let product_id = record.get(1).unwrap_or("").to_string();
            let price_str = record.get(price_idx).unwrap_or("0.0");

            if let Ok(price) = price_str.parse::<f64>() {
                // Map product_id to barcode
                let barcode = product_mapping
                    .get(&product_id)
                    .cloned()
                    .unwrap_or(product_id.clone());
                previous_prices.insert(
                    format!("{store_id}_{product_id}"),
                    (store_id, product_id, barcode, price),
                );
            }
        }
    }

    // Compare with current prices using derived_price
    let curr_file = File::open(current_file).map_err(|e| {
        format!(
            "Failed to open current price file '{}': {e}",
            current_file.display()
        )
    })?;
    let mut curr_reader = csv::ReaderBuilder::new()
        .flexible(true) // Allow records with varying number of fields
        .from_reader(curr_file);

    // Get headers to find derived_price column
    let curr_headers = curr_reader.headers()?.clone();
    let curr_derived_price_idx = curr_headers
        .iter()
        .position(|h| h == "derived_price")
        .or_else(|| curr_headers.iter().position(|h| h == "price"));

    if let Some(price_idx) = curr_derived_price_idx {
        for (line_num, result) in curr_reader.records().enumerate() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "Warning: Skipping malformed record {} in current price file '{}': {e}",
                        line_num + 2,
                        current_file.display()
                    );
                    continue;
                }
            };

            // Check if record has enough fields
            if record.len() <= price_idx || record.len() < 2 {
                eprintln!(
                    "Warning: Skipping record {} in '{}' - insufficient fields (has {}, needs at least {})",
                    line_num + 2,
                    current_file.display(),
                    record.len(),
                    price_idx + 1
                );
                continue;
            }

            let store_id = record.get(0).unwrap_or("").to_string();
            let product_id = record.get(1).unwrap_or("").to_string();
            let price_str = record.get(price_idx).unwrap_or("0.0");

            if let Ok(current_price) = price_str.parse::<f64>() {
                let key = format!("{store_id}_{product_id}");
                if let Some((_, _, barcode, previous_price)) = previous_prices.get(&key)
                    && (current_price - previous_price).abs() > 0.01
                    && *previous_price > 0.0
                {
                    let change_percent =
                        ((current_price - previous_price) / previous_price) * 100.0;
                    let store_name = store_names
                        .get(&StoreId::new(&store_id))
                        .cloned()
                        .unwrap_or_else(|| format!("Store {}", store_id));
                    let market_chain_str = extract_market_chain_from_path(current_file, base_path);
                    price_changes.push(PriceChange {
                        store_id: StoreId::new(store_id.clone()),
                        store_name,
                        market_chain: MarketChain::new(market_chain_str),
                        barcode: Barcode::new(barcode.clone()),
                        old_price: *previous_price,
                        new_price: current_price,
                        change_percent,
                    });
                }
            }
        }
    }

    Ok(price_changes)
}

/// Compares current day data with previous day data.
///
/// This function performs three types of comparisons:
/// 1. **Store comparison**: Compares store_ids from stores.csv files
/// 2. **Product comparison**: Compares unique products (market_chain + barcode) from products.csv files
/// 3. **Price comparison**: Compares prices for product-store combinations from prices.csv files
///
/// **Important distinction**:
/// - `total_products_current/previous`: Unique products (one per barcode per market chain) from products.csv
/// - `total_product_store_combinations`: Product instances (same product can appear at multiple stores) from prices.csv
///
/// The product-store combination count is typically higher than unique product count because:
/// - products.csv: Contains catalog of all products (1 entry per unique product per market chain)
/// - prices.csv: Contains price data (1 entry per product per store, so same product appears multiple times)
///
/// Example: If Konzum has 1000 unique products and 50 stores, there could be up to 50,000 product-store
/// combinations in prices.csv, but only 1000 unique products in products.csv.
pub fn compare_with_previous_day(
    current_data_path: &str,
    previous_data_path: &str,
) -> Result<ComparisonSummary, Box<dyn std::error::Error>> {
    println!("Comparing current data with previous day...");
    println!("Current: {current_data_path}");
    println!("Previous: {previous_data_path}");

    let mut summary = ComparisonSummary {
        total_stores_current: 0,
        total_stores_previous: 0,
        new_stores: Vec::new(),
        missing_stores: Vec::new(),
        total_products_current: 0,
        total_products_previous: 0,
        new_products: Vec::new(),
        missing_products: Vec::new(),
        price_changes: 0,
        total_price_comparisons: 0,
        stores_with_price_changes: 0,
        price_changes_by_chain: HashMap::new(),
        price_changes_by_chain_with_percentages: HashMap::new(),
        average_price_change_percent: 0.0,
        significant_price_changes: Vec::new(),
        total_product_store_combinations: 0,
        price_changes_percentage: 0.0,
        significant_price_changes_percentage: 0.0,
    };

    // Check if previous data exists
    if !Path::new(previous_data_path).exists() {
        println!("Warning: Previous day data not found at {previous_data_path}");
        return Ok(summary);
    }

    // Compare stores
    let (current_stores, previous_stores) = compare_stores(current_data_path, previous_data_path)?;
    summary.total_stores_current = current_stores.len();
    summary.total_stores_previous = previous_stores.len();

    for store in &current_stores {
        if !previous_stores.contains(store) {
            summary.new_stores.push(store.clone());
        }
    }

    for store in &previous_stores {
        if !current_stores.contains(store) {
            summary.missing_stores.push(store.clone());
        }
    }

    // Compare products (unique barcodes from products.csv files)
    let (current_products, previous_products) =
        compare_products(current_data_path, previous_data_path)?;
    summary.total_products_current = current_products.len();
    summary.total_products_previous = previous_products.len();

    for product in &current_products {
        if !previous_products.contains(product) {
            summary.new_products.push(product.clone());
        }
    }

    for product in &previous_products {
        if !current_products.contains(product) {
            summary.missing_products.push(product.clone());
        }
    }

    // Compare prices
    let (
        price_comparison,
        total_stores_with_price_data,
        price_changes_by_chain,
        total_products_by_chain,
    ) = compare_prices(current_data_path, previous_data_path)?;
    summary.price_changes = price_comparison.values().map(|pc| pc.len()).sum();
    summary.total_price_comparisons = total_stores_with_price_data;
    summary.stores_with_price_changes = price_comparison.len();

    // Calculate total product-store combinations available for comparison (the sum of all product instances by chain)
    let total_product_store_combinations: usize = total_products_by_chain.values().sum();

    // Calculate price changes by chain with percentages
    let mut price_changes_with_percentages = HashMap::new();
    for (chain, changes) in price_changes_by_chain.iter() {
        let total = total_products_by_chain.get(chain).unwrap_or(&1).max(&1); // Avoid division by zero
        let percentage = (*changes as f64 / *total as f64) * 100.0;
        price_changes_with_percentages.insert(chain.clone(), (*changes, percentage));
    }
    summary.price_changes_by_chain = price_changes_by_chain;
    summary.price_changes_by_chain_with_percentages = price_changes_with_percentages;

    let mut all_price_changes = Vec::new();
    let mut total_change_percent = 0.0;
    let mut change_count = 0;

    for store_changes in price_comparison.values() {
        for change in store_changes {
            all_price_changes.push(change.clone());
            total_change_percent += change.change_percent.abs();
            change_count += 1;

            // Market chain will be added from the directory structure in compare_prices function
        }
    }

    if change_count > 0 {
        summary.average_price_change_percent = total_change_percent / change_count as f64;
    }

    // Keep only significant price changes (>10% change) and sort by percentage change (descending)
    let mut significant_changes: Vec<PriceChange> = all_price_changes
        .into_iter()
        .filter(|pc| pc.change_percent.abs() > 10.0)
        .collect();

    // Sort by absolute value of percentage change in descending order
    significant_changes.sort_by(|a, b| {
        b.change_percent
            .abs()
            .partial_cmp(&a.change_percent.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    summary.significant_price_changes = significant_changes;

    // Store total product-store combinations for comparison and calculate percentages
    summary.total_product_store_combinations = total_product_store_combinations;

    if total_product_store_combinations > 0 {
        summary.price_changes_percentage =
            (summary.price_changes as f64 / total_product_store_combinations as f64) * 100.0;
        summary.significant_price_changes_percentage = (summary.significant_price_changes.len()
            as f64
            / total_product_store_combinations as f64)
            * 100.0;
    }

    Ok(summary)
}
