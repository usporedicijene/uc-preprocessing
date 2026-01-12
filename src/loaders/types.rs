//! Domain types for the preprocessing tool
//!
//! This module contains type-safe wrappers and data structures used throughout the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// =============================================================================
// Type-safe wrappers (prevent mixing up IDs, barcodes, etc.)
// =============================================================================

/// Type-safe wrapper for store identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreId(String);

impl StoreId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for StoreId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StoreId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Type-safe wrapper for product barcodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Barcode(String);

impl Barcode {
    pub fn new(barcode: impl Into<String>) -> Self {
        Self(barcode.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Barcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Barcode {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Barcode {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Type-safe wrapper for market chain names
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MarketChain(String);

impl MarketChain {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MarketChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MarketChain {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MarketChain {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Type-safe wrapper for city names
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CityName(String);

impl CityName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CityName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CityName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Type-safe wrapper for product IDs (internal IDs, different from barcodes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProductId(String);

impl ProductId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ProductId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProductId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// =============================================================================
// Data structures
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct Store {
    #[serde(rename = "store_id")]
    pub _store_id: String,
    #[serde(rename = "type")]
    pub _store_type: String,
    #[serde(rename = "address")]
    pub _address: String,
    pub city: CityName,
    #[serde(rename = "zipcode")]
    pub _zipcode: String,
}

#[derive(Debug, Deserialize)]
pub struct CityMapping {
    pub from: String,
    pub to: String,
}

impl super::generic::Mapping for CityMapping {
    fn from(&self) -> &str {
        &self.from
    }
    fn to(&self) -> &str {
        &self.to
    }
}

#[derive(Debug, Deserialize)]
pub struct CategoryMapping {
    pub from: String,
    pub to: String,
}

impl super::generic::Mapping for CategoryMapping {
    fn from(&self) -> &str {
        &self.from
    }
    fn to(&self) -> &str {
        &self.to
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceChange {
    pub store_id: StoreId,
    pub store_name: String,
    pub market_chain: MarketChain,
    pub barcode: Barcode,
    pub old_price: f64,
    pub new_price: f64,
    pub change_percent: f64,
}

/// Return type for price comparison containing price changes, store count, and chain statistics
pub type PriceComparisonResult = (
    HashMap<String, Vec<PriceChange>>, // price changes by store name
    usize,                             // total stores with price data
    HashMap<String, usize>,            // price changes by market chain
    HashMap<String, usize>,            // total products by market chain
);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_id_type_safety() {
        let store_id = StoreId::new("001");
        let barcode = Barcode::new("123456");

        assert_eq!(store_id.as_str(), "001");
        assert_eq!(barcode.as_str(), "123456");
    }

    #[test]
    fn test_type_equality() {
        let id1 = StoreId::new("001");
        let id2 = StoreId::new("001");
        let id3 = StoreId::new("002");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_type_display() {
        let store_id = StoreId::new("001");
        assert_eq!(format!("{}", store_id), "001");

        let barcode = Barcode::new("123456");
        assert_eq!(format!("{}", barcode), "123456");
    }

    #[test]
    fn test_type_from_conversions() {
        let from_string: StoreId = "001".into();
        let from_str: StoreId = String::from("001").into();

        assert_eq!(from_string, from_str);
    }

    #[test]
    fn test_barcode_methods() {
        let barcode = Barcode::new("123456789");
        assert_eq!(barcode.as_str(), "123456789");
        assert_eq!(format!("{}", barcode), "123456789");

        let from_string: Barcode = String::from("987654321").into();
        let from_str: Barcode = "987654321".into();
        assert_eq!(from_string, from_str);
    }

    #[test]
    fn test_market_chain_methods() {
        let chain = MarketChain::new("Konzum");
        assert_eq!(chain.as_str(), "Konzum");
        assert_eq!(format!("{}", chain), "Konzum");

        let from_string: MarketChain = String::from("Spar").into();
        let from_str: MarketChain = "Spar".into();
        assert_eq!(from_string, from_str);
    }

    #[test]
    fn test_city_name_methods() {
        let city = CityName::new("Zagreb");
        assert_eq!(city.as_str(), "Zagreb");
        assert_eq!(format!("{}", city), "Zagreb");

        let from_string: CityName = String::from("Split").into();
        let from_str: CityName = "Split".into();
        assert_eq!(from_string, from_str);
    }

    #[test]
    fn test_product_id_methods() {
        let product = ProductId::new("P001");
        assert_eq!(product.as_str(), "P001");
        assert_eq!(format!("{}", product), "P001");

        let from_string: ProductId = String::from("P002").into();
        let from_str: ProductId = "P002".into();
        assert_eq!(from_string, from_str);
    }

    #[test]
    fn test_type_ordering() {
        let id1 = StoreId::new("001");
        let id2 = StoreId::new("002");
        assert!(id1 < id2);

        let b1 = Barcode::new("111");
        let b2 = Barcode::new("222");
        assert!(b1 < b2);
    }

    #[test]
    fn test_type_hashing() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(StoreId::new("001"));
        set.insert(StoreId::new("001")); // duplicate
        set.insert(StoreId::new("002"));

        assert_eq!(set.len(), 2);
    }
}
