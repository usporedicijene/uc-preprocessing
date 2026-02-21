use super::normalization::normalize_croatian_text;
use super::trigrams::generate_trigrams;

const BITVEC_BITS: usize = 256;
const BITVEC_WORDS: usize = BITVEC_BITS / 64;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// A 256-bit vector stored as 4 x u64.
///
/// Each trigram from the product name is hashed to a bit position and that bit
/// is set. This acts as a compact Bloom-filter-style fingerprint where:
/// - Identical or reordered names produce identical bit vectors.
/// - A partial query's bits are a **subset** of the full product's bits,
///   enabling containment checks via `query & product == query`.
/// - Similarity between any two names can be measured with Jaccard on bits:
///   `popcount(a & b) / popcount(a | b)`.
type BitVec256 = [u64; BITVEC_WORDS];

/// FNV-1a 64-bit hash. Chosen for cross-language determinism: the same
/// five-line function produces identical results in Rust, Python, Go, JS, etc.
fn fnv1a_64(data: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Compute a 256-bit trigram bit vector from a slice of trigrams.
pub fn compute_bitvec(trigrams: &[String]) -> BitVec256 {
    let mut bv: BitVec256 = [0u64; BITVEC_WORDS];

    for trigram in trigrams {
        let hash = fnv1a_64(trigram.as_bytes());
        let bit_pos = (hash as usize) % BITVEC_BITS;
        let word_idx = bit_pos / 64;
        let bit_idx = bit_pos % 64;
        bv[word_idx] |= 1u64 << bit_idx;
    }

    bv
}

/// Encode a 256-bit vector as a 64-character zero-padded hex string (big-endian word order).
pub fn bitvec_to_hex(bv: &BitVec256) -> String {
    bv.iter()
        .rev()
        .map(|w| format!("{w:016x}"))
        .collect::<String>()
}

/// Decode a 64-character hex string back into a 256-bit vector.
pub fn hex_to_bitvec(hex: &str) -> Option<BitVec256> {
    if hex.len() != BITVEC_WORDS * 16 {
        return None;
    }
    let mut bv = [0u64; BITVEC_WORDS];
    for (i, chunk) in hex.as_bytes().chunks(16).enumerate() {
        let s = std::str::from_utf8(chunk).ok()?;
        bv[BITVEC_WORDS - 1 - i] = u64::from_str_radix(s, 16).ok()?;
    }
    Some(bv)
}

/// Compute the full name-search fingerprint: normalize, trigram, bitvec, hex.
///
/// This is the top-level function called by the CSV transformer to produce the
/// `uc_name_searching_algorithm_1` column value.
pub fn compute_name_hash(name: &str) -> String {
    let normalized = normalize_croatian_text(name);
    let trigrams = generate_trigrams(&normalized);
    if trigrams.is_empty() {
        return String::new();
    }
    bitvec_to_hex(&compute_bitvec(&trigrams))
}

/// Check whether all bits in `query` are also set in `product` (containment).
///
/// Returns true when the query trigrams are a subset of the product trigrams
/// (modulo hash collisions).
pub fn contains(product: &BitVec256, query: &BitVec256) -> bool {
    for i in 0..BITVEC_WORDS {
        if query[i] & product[i] != query[i] {
            return false;
        }
    }
    true
}

/// Overlap ratio: fraction of query bits that are set in the product.
///
/// Returns a value in 0.0..=1.0.  A value of 1.0 means full containment.
pub fn overlap_ratio(product: &BitVec256, query: &BitVec256) -> f64 {
    let mut matched: u32 = 0;
    let mut total: u32 = 0;
    for i in 0..BITVEC_WORDS {
        matched += (query[i] & product[i]).count_ones();
        total += query[i].count_ones();
    }
    if total == 0 {
        return 0.0;
    }
    matched as f64 / total as f64
}

/// Jaccard similarity on the bit vectors: `|A & B| / |A | B|`.
pub fn jaccard(a: &BitVec256, b: &BitVec256) -> f64 {
    let mut intersection: u32 = 0;
    let mut union: u32 = 0;
    for i in 0..BITVEC_WORDS {
        intersection += (a[i] & b[i]).count_ones();
        union += (a[i] | b[i]).count_ones();
    }
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_names_produce_identical_hashes() {
        let h1 = compute_name_hash("Cedevita naranča 500g");
        let h2 = compute_name_hash("Cedevita naranča 500g");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_case_and_diacritics_invariance() {
        let h1 = compute_name_hash("Cedevita naranča 500g");
        let h2 = compute_name_hash("CEDEVITA NARANCA 500G");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_word_order_invariance() {
        let h1 = compute_name_hash("Cedevita limun 500g");
        let h2 = compute_name_hash("Cedevita 500g limun");
        let h3 = compute_name_hash("500g limun Cedevita");
        assert_eq!(h1, h2, "word reorder should produce identical hash");
        assert_eq!(h1, h3, "word reorder should produce identical hash");
    }

    #[test]
    fn test_partial_match_containment() {
        let full_hex = compute_name_hash("Cedevita limun 500g");
        let partial_hex = compute_name_hash("500g cedevita");

        let full = hex_to_bitvec(&full_hex).unwrap();
        let partial = hex_to_bitvec(&partial_hex).unwrap();

        assert!(
            contains(&full, &partial),
            "Product should contain all partial-query trigram bits"
        );

        let ratio = overlap_ratio(&full, &partial);
        assert!(
            ratio > 0.99,
            "Overlap ratio for contained partial should be ~1.0, got {ratio}"
        );
    }

    #[test]
    fn test_shorter_product_ranks_higher_with_jaccard() {
        let query_hex = compute_name_hash("voda");
        let short_hex = compute_name_hash("voda one");
        let long_hex = compute_name_hash("voda one two");

        let query = hex_to_bitvec(&query_hex).unwrap();
        let short = hex_to_bitvec(&short_hex).unwrap();
        let long = hex_to_bitvec(&long_hex).unwrap();

        let jaccard_short = jaccard(&query, &short);
        let jaccard_long = jaccard(&query, &long);

        assert!(
            jaccard_short > jaccard_long,
            "Closer match should have higher Jaccard ({jaccard_short}) than longer match ({jaccard_long})"
        );
    }

    #[test]
    fn test_partial_match_overlap_vs_unrelated() {
        let product_hex = compute_name_hash("Cedevita limun 500g");
        let partial_hex = compute_name_hash("cedevita 500g");
        let unrelated_hex = compute_name_hash("Toaletni papir troslojni 8 rola");

        let product = hex_to_bitvec(&product_hex).unwrap();
        let partial = hex_to_bitvec(&partial_hex).unwrap();
        let unrelated = hex_to_bitvec(&unrelated_hex).unwrap();

        let overlap_partial = overlap_ratio(&product, &partial);
        let overlap_unrelated = overlap_ratio(&product, &unrelated);

        assert!(
            overlap_partial > overlap_unrelated,
            "Partial match overlap ({overlap_partial}) should exceed unrelated overlap ({overlap_unrelated})"
        );
    }

    #[test]
    fn test_similar_names_have_high_jaccard() {
        let h1 = compute_name_hash("Cedevita naranča 500g");
        let h2 = compute_name_hash("Cedevita limun 500g");

        let bv1 = hex_to_bitvec(&h1).unwrap();
        let bv2 = hex_to_bitvec(&h2).unwrap();
        let j = jaccard(&bv1, &bv2);

        assert!(
            j > 0.3,
            "Similar product names should have meaningful Jaccard overlap, got {j}"
        );
    }

    #[test]
    fn test_dissimilar_names_have_lower_jaccard() {
        let h1 = compute_name_hash("Cedevita naranča 500g");
        let h_similar = compute_name_hash("Cedevita limun 500g");
        let h_different = compute_name_hash("Toaletni papir troslojni 8 rola");

        let bv1 = hex_to_bitvec(&h1).unwrap();
        let bv_similar = hex_to_bitvec(&h_similar).unwrap();
        let bv_different = hex_to_bitvec(&h_different).unwrap();

        let j_similar = jaccard(&bv1, &bv_similar);
        let j_different = jaccard(&bv1, &bv_different);

        assert!(
            j_similar > j_different,
            "Similar Jaccard ({j_similar}) should exceed dissimilar Jaccard ({j_different})"
        );
    }

    #[test]
    fn test_hex_format() {
        let hex = compute_name_hash("test");
        assert_eq!(hex.len(), 64, "Hex string should be 64 characters");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "Should contain only hex digits"
        );
    }

    #[test]
    fn test_hex_roundtrip() {
        let bv: BitVec256 = [0x1234_5678_9abc_def0, 0x0fed_cba9_8765_4321, 0, u64::MAX];
        let hex = bitvec_to_hex(&bv);
        let decoded = hex_to_bitvec(&hex).unwrap();
        assert_eq!(bv, decoded);
    }

    #[test]
    fn test_empty_name() {
        assert_eq!(compute_name_hash(""), "");
    }

    #[test]
    fn test_only_punctuation() {
        assert_eq!(compute_name_hash("---!!!"), "");
    }

    #[test]
    fn test_all_requirements() {
        let product = compute_name_hash("Cedevita limun 500g");
        let bv_product = hex_to_bitvec(&product).unwrap();

        // 1) Word reorder
        let reorder = compute_name_hash("500g limun Cedevita");
        let bv_reorder = hex_to_bitvec(&reorder).unwrap();
        let j_reorder = jaccard(&bv_product, &bv_reorder);
        eprintln!("[word reorder]   jaccard={j_reorder:.3}  contains={}",
            contains(&bv_product, &bv_reorder));

        // 2) Word subset (partial query)
        let subset = compute_name_hash("cedevita 500g");
        let bv_subset = hex_to_bitvec(&subset).unwrap();
        let o_subset = overlap_ratio(&bv_product, &bv_subset);
        eprintln!("[word subset]    overlap={o_subset:.3}  contains={}",
            contains(&bv_product, &bv_subset));

        // 3) Single typo
        let typo = compute_name_hash("Cedevota limun 500g");
        let bv_typo = hex_to_bitvec(&typo).unwrap();
        let j_typo = jaccard(&bv_product, &bv_typo);
        let o_typo = overlap_ratio(&bv_product, &bv_typo);
        eprintln!("[1 typo]         jaccard={j_typo:.3}  overlap={o_typo:.3}");

        // 4) Subset + typo
        let sub_typo = compute_name_hash("cedevota 500g");
        let bv_sub_typo = hex_to_bitvec(&sub_typo).unwrap();
        let o_sub_typo = overlap_ratio(&bv_product, &bv_sub_typo);
        eprintln!("[subset+typo]    overlap={o_sub_typo:.3}");

        // 5) Unrelated product (baseline)
        let unrelated = compute_name_hash("Toaletni papir troslojni 8 rola");
        let bv_unrelated = hex_to_bitvec(&unrelated).unwrap();
        let j_unrelated = jaccard(&bv_product, &bv_unrelated);
        let o_unrelated = overlap_ratio(&bv_product, &bv_unrelated);
        eprintln!("[unrelated]      jaccard={j_unrelated:.3}  overlap={o_unrelated:.3}");
    }

    #[test]
    fn test_bitvec_to_hex_zero() {
        assert_eq!(
            bitvec_to_hex(&[0u64; BITVEC_WORDS]),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
