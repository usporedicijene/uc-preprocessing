# Data Comparison and Change Rate Analysis

The program includes a powerful **day-to-day comparison feature** that analyzes changes between current and previous day data when `STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA` environment variable is set.

## Comparison Reports Generated

1. **`data_comparison_report.md`** - Human-readable markdown report with detailed analysis
2. **`data_comparison_report.json`** - Machine-readable JSON report for programmatic consumption

**Note**: Report output location can be configured with the `COMPARISON_REPORTS_OUTPUT_DIR` environment variable.

## Running Comparison

To enable comparison analysis, set the previous day data path:

```bash
export STORES_DIR_PATH_PREVIOUS_DAY_CLEANED_DATA=/path/to/previous/cleaned_data
```

The comparison runs automatically after data cleaning, or you can run comparison-only mode:

```bash
cargo run -- --compare-only
```

## Change Rate Metrics

The comparison generates three key change rate metrics:

### Store Change Rate

**Definition**: Percentage of stores that were added or removed between comparison periods.

**Formula**: `((New Stores + Missing Stores) / Previous Day Stores) × 100`

| Rate | Interpretation |
|------|----------------|
| 0-2% | Stable store network |
| 2-5% | Moderate network changes |
| >5% | Significant network disruption |

### Product Change Rate

**Definition**: Percentage of products that were added or removed from the catalog.

**Formula**: `((New Products + Missing Products) / Previous Day Products) × 100`

| Rate | Interpretation |
|------|----------------|
| 0-1% | Stable product assortment |
| 1-3% | Regular product rotation |
| >3% | Major catalog updates |

### Price Change Rate

**Definition**: Percentage of product-store combinations that experienced price changes.

**Formula**: `(Products with Price Changes / Total Product-Store Combinations) × 100`

| Rate | Interpretation |
|------|----------------|
| 0-1% | Price stability |
| 1-3% | Regular price management |
| >3% | Active pricing strategy or market volatility |

## Understanding Product-Store Combinations

Price change rate uses "product-store combinations" as the denominator, not unique products.

**Example**:
- 1,000 unique products across 10 stores = 10,000 possible combinations
- If 56 combinations have price changes, the rate is 56/10,000 = 0.56%

## Comparison Analysis Features

The comparison system provides:

- **Store Analysis**: New/missing stores with full details
- **Product Analysis**: New/missing products with market chain breakdown
- **Price Analysis**:
  - Products with price changes by market chain
  - Significant price changes (>10% and >100% categories)
  - Average price change percentage

## Interpreting Change Rates

**Healthy Patterns**:
- Store change rate: 0-2%
- Product change rate: 0.1-1%
- Price change rate: 0.2-2%

**Concern Indicators** (>5% in any rate):
- Data collection issues
- Major operational changes
- Market disruptions

**Seasonal Considerations**:
- Holiday periods → higher product change rates
- Back-to-school → higher price change rates
- End-of-month → pricing cycle effects
