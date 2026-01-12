use super::generic::load_reference_set;
use super::types::CategoryMapping;
use std::collections::HashSet;

pub fn load_reference_categories(
    mapping_file: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    load_reference_set::<CategoryMapping>(mapping_file, "categories")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_reference_categories() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            "from;to\n\
             Food;Food\n\
             Drinks;Drinks\n\
             Electronics;Electronics",
        )
        .unwrap();

        let result = load_reference_categories(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains("Food"));
        assert!(result.contains("Drinks"));
        assert!(result.contains("Electronics"));
    }

    #[test]
    fn test_load_reference_categories_missing_file() {
        let result = load_reference_categories("nonexistent_categories.csv");
        assert!(result.is_err()); // Should return error if file doesn't exist
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
