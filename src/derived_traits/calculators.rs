pub fn calculate_derived_price(price: Option<&str>, special_price: Option<&str>) -> String {
    // Use special_price if it exists and is not empty/zero/na, otherwise use price
    let special = special_price.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("na") {
            return None;
        }
        // Try to parse as f64 - if invalid or zero, fall back to regular price
        match trimmed.parse::<f64>() {
            Ok(val) if val != 0.0 => Some(trimmed),
            _ => None,
        }
    });

    if let Some(special) = special {
        special.to_string()
    } else if let Some(price) = price {
        price.trim().to_string()
    }
    // This would happen in real data when a product row in the CSV has
    // missing/empty values for both the price and special_price columns.
    else {
        String::new()
    }
}

pub fn calculate_price_change(derived_price: &str, anchor_derived_price: &str) -> String {
    // Parse both prices as f64
    let current = match derived_price.trim().parse::<f64>() {
        Ok(val) if val > 0.0 => val,
        _ => return String::new(), // Return empty if current price is invalid or zero
    };

    let anchor = match anchor_derived_price.trim().parse::<f64>() {
        Ok(val) if val > 0.0 => val,
        _ => return String::new(), // Return empty if anchor price is invalid or zero
    };

    // Calculate percentage change
    let change = ((current - anchor) / anchor) * 100.0;
    format!("{change:.2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_derived_price_function() {
        // Test with valid special price
        assert_eq!(calculate_derived_price(Some("10.00"), Some("8.50")), "8.50");

        // Test with empty special price - should use regular price
        assert_eq!(calculate_derived_price(Some("10.00"), Some("")), "10.00");

        // Test with zero special price - should use regular price
        assert_eq!(calculate_derived_price(Some("10.00"), Some("0")), "10.00");
        assert_eq!(calculate_derived_price(Some("10.00"), Some("0.0")), "10.00");

        // Test with NA special price - should use regular price
        assert_eq!(calculate_derived_price(Some("10.00"), Some("NA")), "10.00");
        assert_eq!(calculate_derived_price(Some("10.00"), Some("na")), "10.00");

        // Test with no special price - should use regular price
        assert_eq!(calculate_derived_price(Some("10.00"), None), "10.00");

        // Test with no prices - should return empty
        assert_eq!(calculate_derived_price(None, None), "");
    }

    #[test]
    fn test_calculate_derived_price_edge_cases() {
        // Test with whitespace
        assert_eq!(
            calculate_derived_price(Some("  10.00  "), Some("  8.50  ")),
            "8.50"
        );

        // Test with "0.0" special price
        assert_eq!(calculate_derived_price(Some("10.00"), Some("0.0")), "10.00");

        // Test with mixed case NA
        assert_eq!(calculate_derived_price(Some("10.00"), Some("Na")), "10.00");
        assert_eq!(calculate_derived_price(Some("10.00"), Some("nA")), "10.00");

        // Test with empty price but valid special price
        assert_eq!(calculate_derived_price(None, Some("8.50")), "8.50");
    }

    #[test]
    fn test_calculate_price_change_function() {
        // Test normal price change
        assert_eq!(calculate_price_change("10.00", "8.00"), "25.00"); // 25% increase
        assert_eq!(calculate_price_change("8.00", "10.00"), "-20.00"); // 20% decrease

        // Test with zero anchor price - should return empty
        assert_eq!(calculate_price_change("10.00", "0"), "");
        assert_eq!(calculate_price_change("10.00", "0.0"), "");

        // Test with zero current price - should return empty
        assert_eq!(calculate_price_change("0", "10.00"), "");

        // Test with invalid values - should return empty
        assert_eq!(calculate_price_change("invalid", "10.00"), "");
        assert_eq!(calculate_price_change("10.00", "invalid"), "");
        assert_eq!(calculate_price_change("", "10.00"), "");
    }

    #[test]
    fn test_calculate_price_change_edge_cases() {
        // Test with negative prices (should return empty)
        assert_eq!(calculate_price_change("-10.00", "8.00"), "");
        assert_eq!(calculate_price_change("10.00", "-8.00"), "");

        // Test with very small price change
        assert_eq!(calculate_price_change("10.01", "10.00"), "0.10");

        // Test with large price change
        assert_eq!(calculate_price_change("20.00", "10.00"), "100.00");

        // Test with whitespace
        assert_eq!(calculate_price_change("  10.00  ", "  8.00  "), "25.00");
    }
}
