use super::generic::load_reference_set;
use super::types::CityMapping;
use std::collections::HashSet;

pub fn load_reference_cities(
    mapping_file: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    load_reference_set::<CityMapping>(mapping_file, "cities")
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
    fn test_load_reference_cities() {
        let cities_file = create_test_cities_mapping_csv();
        let result = load_reference_cities(cities_file.path().to_str().unwrap()).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result.contains("Zagreb"));
        assert!(result.contains("Split"));
        assert!(result.contains("Rijeka"));
        assert!(result.contains("Osijek"));
    }

    #[test]
    fn test_load_reference_cities_missing_file() {
        let result = load_reference_cities("nonexistent_file.csv");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
