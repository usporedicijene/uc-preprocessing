# Data Processing Details

## Data Validation Process

The program performs comprehensive validation before processing:

### City Validation
- Extracts unique city values from `stores.csv` files
- Compares with reference cities from `cities_mapping.csv`
- Shows any new cities that need mapping

### Category Validation
- Extracts unique category values from `products.csv` files
- Compares with reference categories from `categories_mapping.csv`
- Shows any new categories that need mapping

### Data Filtering
- **Filters out products** whose mapped categories end with "~~"
- **Filters out corresponding price records** for excluded products
- **Filters out invalid prices** (≤ 0, empty, or non-numeric values)

## Data Cleaning Process

When all cities and categories match their references, the program:

1. **Creates output directory** with the same subdirectory structure
2. **Transforms `products.csv`**: Apply category mappings, filter excluded categories, compute name search fingerprint (`uc_name_searching_algorithm_1`), sort
3. **Processes `prices.csv`**: Filter invalid prices/excluded products, merge anchor data, add derived columns, sort
4. **Transforms `stores.csv`**: Apply city mappings, sort

## Anchor Data Integration

When `STORES_DIR_PATH_ANCHOR_CLEANED_DATA` is set:

- **Match records** by `store_id` + `barcode` combination
- **Add derived columns**: `derived_price`, `uc_anchor_derived_price`, `derived_price_change`
- **Handle missing data gracefully**: Empty values for non-matching records

### Name Search Column (products.csv)

| Column | Description |
|--------|-------------|
| `uc_name_searching_algorithm_1` | 256-bit trigram fingerprint for fuzzy name matching (see [Name Search Fingerprinting](NAME_SEARCH.md)) |

### Derived Price Columns

| Column | Description |
|--------|-------------|
| `derived_price` | Uses `special_price` if valid, otherwise `price` |
| `uc_anchor_derived_price` | Pre-calculated `derived_price` from anchor data |
| `derived_price_change` | `((derived_price - anchor) / anchor) × 100` |

## CSV Format Requirements

### stores.csv
```csv
store_id,type,address,city,zipcode
0720,supermarket,Zagrebačka 10,Varaždin,42000
```

### Mapping files (semicolon-delimited)

**cities_mapping.csv**:
```csv
from;to
Zagreb;Zagreb
Biograd Na Moru;Biograd na Moru
```

**categories_mapping.csv**:
```csv
from;to
Meso i mesni proizvodi;Meso i mesni proizvodi
Neželjena kategorija;Neželjena kategorija~~
```

**Note**: Categories ending with "~~" in the `to` column will be excluded from output.

## CSV Validation

The program handles malformed CSV data:
- **Detects invalid rows** (malformed quotes, improper escaping)
- **Skips invalid rows** and continues processing
- **Logs all issues** with row numbers and error details

## Performance

Uses **Rayon** for parallel processing:
- Parallel file and directory processing
- Parallel sorting of large CSV files
- Thread-safe operations
