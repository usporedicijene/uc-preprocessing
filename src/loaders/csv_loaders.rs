use crate::cleaners::types::AnchorPriceData;
use crate::loaders::generic::load_mapping_map;
use crate::loaders::types::{CategoryMapping, CityMapping};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub fn load_city_mappings(
    mapping_file: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    load_mapping_map::<CityMapping>(mapping_file, "city")
}

pub fn load_category_mappings(
    mapping_file: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    load_mapping_map::<CategoryMapping>(mapping_file, "category")
}

pub fn load_anchor_price_data(
    anchor_prices: &Path,
) -> Result<HashMap<String, AnchorPriceData>, Box<dyn std::error::Error>> {
    let mut anchor_data = HashMap::new();

    if !anchor_prices.exists() {
        return Ok(anchor_data); // Return empty if anchor file doesn't exist
    }

    let file = File::open(anchor_prices)?;
    let mut reader = csv::Reader::from_reader(file);

    let headers = reader.headers()?.clone();
    let store_id_idx = headers.iter().position(|h| h == "store_id");
    let barcode_idx = headers
        .iter()
        .position(|h| h == "barcode")
        .or_else(|| headers.iter().position(|h| h == "product_id"));
    let price_idx = headers.iter().position(|h| h == "price");
    let unit_price_idx = headers.iter().position(|h| h == "unit_price");
    let special_price_idx = headers.iter().position(|h| h == "special_price");
    let derived_price_idx = headers.iter().position(|h| h == "derived_price");

    for record in reader.records().flatten() {
        if let (Some(store_idx), Some(bar_idx)) = (store_id_idx, barcode_idx)
            && let (Some(store_id), Some(barcode)) = (record.get(store_idx), record.get(bar_idx))
        {
            let key = format!("{}|{}", store_id.trim(), barcode.trim());

            let price_data = AnchorPriceData {
                price: price_idx
                    .and_then(|idx| record.get(idx))
                    .map(|s| s.to_string()),
                unit_price: unit_price_idx
                    .and_then(|idx| record.get(idx))
                    .map(|s| s.to_string()),
                special_price: special_price_idx
                    .and_then(|idx| record.get(idx))
                    .map(|s| s.to_string()),
                derived_price: derived_price_idx
                    .and_then(|idx| record.get(idx))
                    .map(|s| s.to_string()),
            };

            anchor_data.insert(key, price_data);
        }
    }

    Ok(anchor_data)
}

pub fn load_product_categories(
    products_file: &Path,
    category_mappings: &HashMap<String, String>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut product_categories = HashMap::new();

    if !products_file.exists() {
        return Ok(product_categories); // Return empty if products file doesn't exist
    }

    let file = File::open(products_file)?;
    let mut reader = csv::Reader::from_reader(file);

    let headers = reader.headers()?.clone();
    let product_id_idx = headers
        .iter()
        .position(|h| h == "product_id")
        .or_else(|| headers.iter().position(|h| h == "barcode"));
    let category_idx = headers.iter().position(|h| h == "category");

    if let (Some(prod_idx), Some(cat_idx)) = (product_id_idx, category_idx) {
        for record in reader.records().flatten() {
            if let (Some(product_id), Some(category)) = (record.get(prod_idx), record.get(cat_idx))
            {
                let final_category =
                    if let Some(mapped_category) = category_mappings.get(category.trim()) {
                        mapped_category.clone()
                    } else {
                        category.trim().to_string()
                    };

                product_categories.insert(product_id.trim().to_string(), final_category);
            }
        }
    }

    Ok(product_categories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_cities_mapping_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "from;to\n\
             Zagreb;Zagreb\n\
             Split;Split\n\
             Rijeka;Rijeka\n\
             Osijek;Osijek"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_city_mappings() {
        let cities_file = create_test_cities_mapping_csv();
        let result = load_city_mappings(cities_file.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result.get("Zagreb"), Some(&"Zagreb".to_string()));
        assert_eq!(result.get("Split"), Some(&"Split".to_string()));
    }

    #[test]
    fn test_load_city_mappings_malformed_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "from;to\n\
             Zagreb;Zagreb\n\
             \"Unclosed;quote;Split\n\
             Rijeka;Rijeka"
        )
        .unwrap();

        let result = load_city_mappings(file.path().to_str().unwrap()).unwrap();

        // Should still load valid rows and skip the malformed one
        // The unclosed quote causes the CSV parser to treat the rest as one field
        assert!(!result.is_empty());
        assert!(result.contains_key("Zagreb"));
    }

    #[test]
    fn test_load_category_mappings() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            "from;to\n\
             Food;Food Items\n\
             Drinks;Beverages\n\
             Electronics;Tech Products",
        )
        .unwrap();

        let result = load_category_mappings(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("Food"), Some(&"Food Items".to_string()));
        assert_eq!(result.get("Drinks"), Some(&"Beverages".to_string()));
        assert_eq!(
            result.get("Electronics"),
            Some(&"Tech Products".to_string())
        );
    }

    #[test]
    fn test_load_anchor_price_data() {
        let mut anchor_file = NamedTempFile::new().unwrap();
        writeln!(
            anchor_file,
            "store_id,barcode,price,unit_price,special_price,derived_price\n\
             001,123456,10.00,2.00,8.50,9.50\n\
             002,789012,15.00,3.00,13.50,14.50\n\
             003,999999,20.00,4.00,18.99,19.99"
        )
        .unwrap();

        let result = load_anchor_price_data(anchor_file.path()).unwrap();

        assert_eq!(result.len(), 3);
        let key = "001|123456";
        assert!(result.contains_key(key));
        let data = result.get(key).unwrap();
        assert_eq!(data.price, Some("10.00".to_string()));
        assert_eq!(data.derived_price, Some("9.50".to_string()));
    }

    #[test]
    fn test_load_anchor_price_data_with_product_id_column() {
        let mut anchor_file = NamedTempFile::new().unwrap();
        writeln!(
            anchor_file,
            "store_id,product_id,price,unit_price,special_price,derived_price\n\
             001,123456,10.00,2.00,8.50,9.50\n\
             002,789012,15.00,3.00,13.50,14.50"
        )
        .unwrap();

        let result = load_anchor_price_data(anchor_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("001|123456"));
        assert!(result.contains_key("002|789012"));
    }
}
