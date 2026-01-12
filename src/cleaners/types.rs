/// Anchor price data loaded from cleaned anchor data.
/// This replaces the old AnchorPriceData that loaded from raw data.
#[derive(Debug, Clone, Default)]
pub struct AnchorPriceData {
    pub price: Option<String>,
    pub unit_price: Option<String>,
    pub special_price: Option<String>,
    pub derived_price: Option<String>,
}
