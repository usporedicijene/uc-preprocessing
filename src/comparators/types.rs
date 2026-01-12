use crate::loaders::types::PriceChange;
use serde::Serialize;
use std::collections::HashMap;

/// Summary of comparison between current and previous day data.
///
/// **Data Counting Distinction**:
/// - `total_products_*`: Unique products from products.csv (market_chain + barcode combinations)
/// - `total_product_store_combinations`: Product instances from prices.csv (store_id + product_id combinations)
///
/// The product-store combination count is typically higher because the same product can appear
/// at multiple store locations, so prices.csv has more entries than products.csv.
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonSummary {
    pub total_stores_current: usize,
    pub total_stores_previous: usize,
    pub new_stores: Vec<String>,
    pub missing_stores: Vec<String>,
    /// Unique products from products.csv files (market_chain + barcode combinations)
    pub total_products_current: usize,
    /// Unique products from products.csv files (market_chain + barcode combinations)
    pub total_products_previous: usize,
    pub new_products: Vec<String>,
    pub missing_products: Vec<String>,
    pub price_changes: usize,
    pub total_price_comparisons: usize,
    pub stores_with_price_changes: usize,
    pub price_changes_by_chain: HashMap<String, usize>,
    pub price_changes_by_chain_with_percentages: HashMap<String, (usize, f64)>,
    pub average_price_change_percent: f64,
    pub significant_price_changes: Vec<PriceChange>,
    /// Total product-store combinations from prices.csv files (store_id + product_id combinations)
    /// This is typically higher than total_products_* because same product appears at multiple stores
    pub total_product_store_combinations: usize,
    pub price_changes_percentage: f64,
    pub significant_price_changes_percentage: f64,
}
