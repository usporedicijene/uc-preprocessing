use crate::comparators::basics::compare_with_previous_day;
use crate::comparators::reports::generate_comparison_report;
use crate::config::Config;

/// Runs the comparison-only mode
///
/// This function orchestrates the entire comparison process:
/// 1. Uses config for paths
/// 2. Runs comparison between current and previous day data
/// 3. Generates comparison reports (JSON and Markdown)
/// 4. Prints summary to console
///
/// # Arguments
/// - `config`: Application configuration
///
/// # Returns
/// - `Ok(())` if comparison succeeds
/// - `Err(String)` if comparison fails with error message
pub fn run_comparison_only(config: &Config) -> Result<(), String> {
    println!("🔍 COMPARISON-ONLY MODE");
    println!("Skipping cleaned_data generation and running comparison only.");
    println!("{}", "=".repeat(60));

    // Get previous day's data path (required for comparison-only mode)
    let previous_day_path = config.previous_day_path().ok_or_else(|| {
        "STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA environment variable not set.\n\
         This is required for comparison-only mode."
            .to_string()
    })?;

    // Run comparison with required mode
    run_comparison(config, config.output_dir(), previous_day_path, true)
        .map_err(|e| format!("Failed to compare with previous day data: {e}"))?;

    Ok(())
}

/// Runs comparison and prints results (can be called from normal or compare-only mode)
///
/// # Arguments
/// - `config`: Application configuration
/// - `current_path`: Path to current day's cleaned data
/// - `previous_path`: Path to previous day's cleaned data
/// - `required`: If true, treats errors as fatal; if false, just warns
///
/// # Returns
/// - `Ok(())` if comparison succeeds or is skipped
/// - `Err(Box<dyn std::error::Error>)` if comparison fails and is required
pub fn run_comparison(
    config: &Config,
    current_path: &str,
    previous_path: &str,
    required: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(60));
    println!("COMPARING WITH PREVIOUS DAY DATA");
    println!("{}", "=".repeat(60));

    // Run comparison
    let summary = match compare_with_previous_day(current_path, previous_path) {
        Ok(s) => s,
        Err(e) => {
            if required {
                return Err(e);
            } else {
                eprintln!("Warning: Failed to compare with previous day data: {e}");
                return Ok(());
            }
        }
    };

    // Generate reports
    if let Err(e) = generate_comparison_report(&summary, config.reports_dir()) {
        eprintln!("Warning: Failed to generate comparison report: {e}");
    }

    // Print summary to console
    print_comparison_summary(&summary, config.reports_dir());

    Ok(())
}

/// Prints a formatted comparison summary to the console
fn print_comparison_summary(
    summary: &crate::comparators::types::ComparisonSummary,
    _reports_dir: &str,
) {
    println!("\n📊 COMPARISON SUMMARY:");

    // Stores summary
    println!(
        "Stores: {} current, {} previous ({:+} new, {:+} missing)",
        summary.total_stores_current,
        summary.total_stores_previous,
        summary.new_stores.len() as i32,
        -(summary.missing_stores.len() as i32)
    );

    // Products summary
    println!(
        "Products: {} current, {} previous ({:+} new, {:+} missing)",
        summary.total_products_current,
        summary.total_products_previous,
        summary.new_products.len() as i32,
        -(summary.missing_products.len() as i32)
    );

    // Price changes summary
    println!(
        "Price changes: {} products changed prices ({:.1}% of total, avg: {:.2}%)",
        summary.price_changes,
        summary.price_changes_percentage,
        summary.average_price_change_percent
    );

    println!(
        "Significant price changes (>10%): {} ({:.1}% of total)",
        summary.significant_price_changes.len(),
        summary.significant_price_changes_percentage
    );

    // Price changes by market chain
    if !summary.price_changes_by_chain_with_percentages.is_empty() {
        println!("\n📊 Price changes by market chain:");
        let mut chain_changes: Vec<_> = summary
            .price_changes_by_chain_with_percentages
            .iter()
            .collect();
        chain_changes.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Sort by count descending
        for (chain, (count, percentage)) in chain_changes {
            println!("  {chain}: {count} products changed ({percentage:.1}%)");
        }
    }

    // New and missing stores
    if !summary.new_stores.is_empty() {
        println!("\n🆕 New stores: {}", summary.new_stores.join(", "));
    }
    if !summary.missing_stores.is_empty() {
        println!("\n❌ Missing stores: {}", summary.missing_stores.join(", "));
    }

    // Top significant price changes
    if !summary.significant_price_changes.is_empty() {
        println!("\n💰 Top 5 significant price changes:");
        for (i, change) in summary.significant_price_changes.iter().take(5).enumerate() {
            println!(
                "  {}. {} - Store {} ({}), Barcode {}: {:.2} → {:.2} ({:+.1}%)",
                i + 1,
                change.market_chain.as_str(),
                change.store_id.as_str(),
                change.store_name,
                change.barcode.as_str(),
                change.old_price,
                change.new_price,
                change.change_percent
            );
        }
    }

    // Report location
    println!("\n✅ Full comparison report saved to '{_reports_dir}/data_comparison_report.md'");
}
