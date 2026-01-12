use std::fs;
use std::path::Path;

use crate::comparators::types::ComparisonSummary;

pub fn generate_json_comparison_report(
    summary: &ComparisonSummary,
    reports_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Sort arrays alphabetically
    let mut new_stores_sorted = summary.new_stores.clone();
    new_stores_sorted.sort();

    let mut missing_stores_sorted = summary.missing_stores.clone();
    missing_stores_sorted.sort();

    let mut new_products_sorted: Vec<_> = summary.new_products.iter().take(50).collect();
    new_products_sorted.sort();

    let mut missing_products_sorted: Vec<_> = summary.missing_products.iter().take(50).collect();
    missing_products_sorted.sort();

    // Sort price changes by market chain - convert HashMap to sorted Vec of tuples
    let mut price_changes_by_chain_sorted: Vec<_> = summary.price_changes_by_chain.iter().collect();
    price_changes_by_chain_sorted.sort_by(|a, b| a.0.cmp(b.0));

    let mut price_changes_by_chain_with_percentages_sorted: Vec<_> = summary
        .price_changes_by_chain_with_percentages
        .iter()
        .collect();
    price_changes_by_chain_with_percentages_sorted.sort_by(|a, b| a.0.cmp(b.0));

    // Sort significant price changes (they're already sorted by percentage, but ensure consistent order)
    let mut significant_above_100: Vec<_> = summary
        .significant_price_changes
        .iter()
        .filter(|pc| pc.change_percent.abs() > 100.0)
        .collect();
    significant_above_100.sort_by(|a, b| {
        // Sort by absolute change percentage (descending), then by barcode for consistency
        let cmp = b
            .change_percent
            .abs()
            .partial_cmp(&a.change_percent.abs())
            .unwrap_or(std::cmp::Ordering::Equal);
        if cmp == std::cmp::Ordering::Equal {
            a.barcode.cmp(&b.barcode)
        } else {
            cmp
        }
    });

    let mut significant_between_10_and_100: Vec<_> = summary
        .significant_price_changes
        .iter()
        .filter(|pc| pc.change_percent.abs() <= 100.0 && pc.change_percent.abs() > 10.0)
        .take(50)
        .collect();
    significant_between_10_and_100.sort_by(|a, b| {
        // Sort by absolute change percentage (descending), then by barcode for consistency
        let cmp = b
            .change_percent
            .abs()
            .partial_cmp(&a.change_percent.abs())
            .unwrap_or(std::cmp::Ordering::Equal);
        if cmp == std::cmp::Ordering::Equal {
            a.barcode.cmp(&b.barcode)
        } else {
            cmp
        }
    });

    // Create a JSON-serializable version of the report
    let json_report = serde_json::json!({
        "generated_on": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "price_changes_by_market_chain": {
            "by_count": price_changes_by_chain_sorted.into_iter().collect::<std::collections::BTreeMap<_, _>>(),
            "by_percentage": price_changes_by_chain_with_percentages_sorted.into_iter().collect::<std::collections::BTreeMap<_, _>>()
        },
        "price_comparison": {
            "average_price_change_percent": summary.average_price_change_percent,
            "price_changes_percentage": summary.price_changes_percentage,
            "products_with_price_changes": summary.price_changes,
            "significant_price_changes_count": summary.significant_price_changes.len(),
            "significant_price_changes_percentage": summary.significant_price_changes_percentage,
            "stores_with_price_changes": summary.stores_with_price_changes,
            "total_product_store_combinations": summary.total_product_store_combinations,
            "total_stores_compared": summary.total_price_comparisons
        },
        "product_comparison": {
            "current_products": summary.total_products_current,
            "missing_products": missing_products_sorted,
            "missing_products_count": summary.missing_products.len(),
            "new_products": new_products_sorted,
            "new_products_count": summary.new_products.len(),
            "previous_products": summary.total_products_previous
        },
        "significant_price_changes": {
            "above_100_percent": significant_above_100,
            "between_10_and_100_percent": significant_between_10_and_100
        },
        "store_comparison": {
            "current_stores": summary.total_stores_current,
            "missing_stores": missing_stores_sorted,
            "missing_stores_count": summary.missing_stores.len(),
            "new_stores": new_stores_sorted,
            "new_stores_count": summary.new_stores.len(),
            "previous_stores": summary.total_stores_previous
        },
        "summary": {
            "price_change_rate": summary.price_changes_percentage,
            "product_change_rate": if summary.total_products_previous > 0 {
                ((summary.new_products.len() + summary.missing_products.len()) as f64
                    / summary.total_products_previous as f64) * 100.0
            } else {
                0.0
            },
            "store_change_rate": if summary.total_stores_previous > 0 {
                ((summary.new_stores.len() + summary.missing_stores.len()) as f64
                    / summary.total_stores_previous as f64) * 100.0
            } else {
                0.0
            }
        }
    });

    // Write JSON report to file
    let reports_path = Path::new(reports_dir);
    if !reports_path.exists() {
        fs::create_dir_all(reports_path)?;
    }

    let json_report_path = reports_path.join("data_comparison_report.json");
    fs::write(
        &json_report_path,
        serde_json::to_string_pretty(&json_report)?,
    )?;
    println!(
        "JSON comparison report saved to: {}",
        json_report_path.display()
    );

    Ok(())
}

pub fn generate_comparison_report(
    summary: &ComparisonSummary,
    reports_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate JSON report
    generate_json_comparison_report(summary, reports_dir)?;

    // Generate markdown report (existing functionality)
    let report_content = format!(
        "# Data Comparison Report\n\
        Generated on: {}\n\n\
        ## Store Comparison\n\
        - Current stores: {}\n\
        - Previous stores: {}\n\
        - New stores: {}\n\
        - Missing stores: {}\n\n\
        ## Product Comparison\n\
        - Current products: {}\n\
        - Previous products: {}\n\
        - New products: {}\n\
        - Missing products: {}\n\n\
        ## Price Comparison\n\
        - Total stores compared (have price data in both days): {}\n\
        - Stores with price changes: {}\n\
        - Products with price changes: {} ({:.1}% of {} product-store combinations)\n\
        - Average price change: {:.2}%\n\
        - Significant price changes (>10%): {} ({:.1}% of product-store combinations)\n\n\
        ## Price Changes by Market Chain\n{}\n\n\
        ## New Stores\n{}\n\n\
        ## Missing Stores\n{}\n\n\
        ## New Products (first 50)\n{}\n\n\
        ## Missing Products (first 50)\n{}\n\n\
        ## Significant Price Changes\n{}\n\n\
        ## Summary\n\
        - Store change rate: {:.2}%\n\
        - Product change rate: {:.2}%\n\
        - Price change rate: {:.2}%\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        summary.total_stores_current,
        summary.total_stores_previous,
        summary.new_stores.len(),
        summary.missing_stores.len(),
        summary.total_products_current,
        summary.total_products_previous,
        summary.new_products.len(),
        summary.missing_products.len(),
        summary.total_price_comparisons,
        summary.stores_with_price_changes,
        summary.price_changes,
        summary.price_changes_percentage,
        summary.total_product_store_combinations,
        summary.average_price_change_percent,
        summary.significant_price_changes.len(),
        summary.significant_price_changes_percentage,
        if summary.price_changes_by_chain_with_percentages.is_empty() {
            "None".to_string()
        } else {
            let mut chain_breakdown: Vec<_> = summary
                .price_changes_by_chain_with_percentages
                .iter()
                .collect();
            chain_breakdown.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Sort by count descending
            chain_breakdown
                .iter()
                .map(|(chain, (count, percentage))| {
                    format!("- {chain}: {count} products ({percentage:.1}%)")
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
        if summary.new_stores.is_empty() {
            "None".to_string()
        } else {
            summary
                .new_stores
                .iter()
                .map(|s| format!("- {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if summary.missing_stores.is_empty() {
            "None".to_string()
        } else {
            summary
                .missing_stores
                .iter()
                .map(|s| format!("- {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if summary.new_products.is_empty() {
            "None".to_string()
        } else {
            summary
                .new_products
                .iter()
                .take(50)
                .map(|p| format!("- {p}"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if summary.missing_products.is_empty() {
            "None".to_string()
        } else {
            summary
                .missing_products
                .iter()
                .take(50)
                .map(|p| format!("- {p}"))
                .collect::<Vec<_>>()
                .join("\n")
        },
        if summary.significant_price_changes.is_empty() {
            "None".to_string()
        } else {
            // Filter for changes above 100% and show all of them
            let major_changes: Vec<_> = summary
                .significant_price_changes
                .iter()
                .filter(|pc| pc.change_percent.abs() > 100.0)
                .collect();

            if major_changes.is_empty() {
                format!(
                    "No price changes above 100%\n\nTop 50 significant changes (>10%):\n{}",
                    summary
                        .significant_price_changes
                        .iter()
                        .take(50)
                        .map(|pc| {
                            format!(
                                "- {}: Store {} ({}), Barcode {}: {:.2} → {:.2} ({:+.1}%)",
                                pc.market_chain,
                                pc.store_id,
                                pc.store_name,
                                pc.barcode,
                                pc.old_price,
                                pc.new_price,
                                pc.change_percent
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else {
                format!(
                    "Price changes above 100% ({} items):\n{}\n\nOther significant changes (>10%, first 50):\n{}",
                    major_changes.len(),
                    major_changes
                        .iter()
                        .map(|pc| {
                            format!(
                                "- {}: Store {} ({}), Barcode {}: {:.2} → {:.2} ({:+.1}%)",
                                pc.market_chain,
                                pc.store_id,
                                pc.store_name,
                                pc.barcode,
                                pc.old_price,
                                pc.new_price,
                                pc.change_percent
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                    summary
                        .significant_price_changes
                        .iter()
                        .filter(|pc| pc.change_percent.abs() <= 100.0)
                        .take(50)
                        .map(|pc| {
                            format!(
                                "- {}: Store {} ({}), Barcode {}: {:.2} → {:.2} ({:+.1}%)",
                                pc.market_chain,
                                pc.store_id,
                                pc.store_name,
                                pc.barcode,
                                pc.old_price,
                                pc.new_price,
                                pc.change_percent
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        },
        if summary.total_stores_previous > 0 {
            ((summary.new_stores.len() + summary.missing_stores.len()) as f64
                / summary.total_stores_previous as f64)
                * 100.0
        } else {
            0.0
        },
        if summary.total_products_previous > 0 {
            ((summary.new_products.len() + summary.missing_products.len()) as f64
                / summary.total_products_previous as f64)
                * 100.0
        } else {
            0.0
        },
        summary.price_changes_percentage,
    );

    let reports_path = Path::new(reports_dir);
    if !reports_path.exists() {
        fs::create_dir_all(reports_path)?;
    }

    let md_report_path = reports_path.join("data_comparison_report.md");
    fs::write(&md_report_path, report_content)?;
    println!("Comparison report saved to: {}", md_report_path.display());

    Ok(())
}
