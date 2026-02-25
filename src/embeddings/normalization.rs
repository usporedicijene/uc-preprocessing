/// Normalize a Croatian product name for trigram fingerprinting.
///
/// Applies: lowercase, diacritics stripping, punctuation removal, whitespace collapse.
pub fn normalize_croatian_text(name: &str) -> String {
    let lowered = name.to_lowercase();

    let mut result = String::with_capacity(lowered.len());
    for ch in lowered.chars() {
        match ch {
            'č' | 'ć' => result.push('c'),
            'š' => result.push('s'),
            'ž' => result.push('z'),
            'đ' => result.push('d'),
            c if c.is_alphanumeric() || c == ' ' => result.push(c),
            _ => result.push(' '),
        }
    }

    let mut words: Vec<&str> = result.split_whitespace().collect();
    words.sort_unstable();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase_and_diacritics() {
        assert_eq!(
            normalize_croatian_text("Čokolada Šumska Žetva"),
            "cokolada sumska zetva"
        );
    }

    #[test]
    fn test_dj_diacritics() {
        assert_eq!(normalize_croatian_text("Đumbir"), "dumbir");
    }

    #[test]
    fn test_punctuation_removed() {
        assert_eq!(
            normalize_croatian_text("Mlijeko, 2.8% m.m."),
            "2 8 m m mlijeko"
        );
    }

    #[test]
    fn test_whitespace_collapsed() {
        assert_eq!(
            normalize_croatian_text("  Sok   od   naranče  "),
            "narance od sok"
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(normalize_croatian_text(""), "");
    }

    #[test]
    fn test_only_punctuation() {
        assert_eq!(normalize_croatian_text("---...!!!"), "");
    }

    #[test]
    fn test_mixed_case_and_symbols() {
        assert_eq!(
            normalize_croatian_text("CEDEVITA® Naranča 500g"),
            "500g cedevita naranca"
        );
    }

    #[test]
    fn test_word_order_invariant() {
        let a = normalize_croatian_text("Cedevita limun 500g");
        let b = normalize_croatian_text("500g limun Cedevita");
        let c = normalize_croatian_text("limun Cedevita 500g");
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
