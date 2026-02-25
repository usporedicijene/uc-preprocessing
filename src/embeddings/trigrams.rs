/// Generate character trigrams from text on a **per-word** basis.
///
/// Each word is individually padded with sentinel spaces and trigram-ised.
/// This guarantees that a subset of words always produces a subset of trigrams,
/// which is critical for partial-query containment checks.
pub fn generate_trigrams(text: &str) -> Vec<String> {
    text.split_whitespace()
        .flat_map(|word| {
            let padded = format!("  {word}  ");
            let chars: Vec<char> = padded.chars().collect();
            chars
                .windows(3)
                .map(|w| w.iter().collect::<String>())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_word() {
        let trigrams = generate_trigrams("sok");
        assert_eq!(trigrams, vec!["  s", " so", "sok", "ok ", "k  "]);
    }

    #[test]
    fn test_multi_word_generates_per_word() {
        let trigrams = generate_trigrams("ab cd");
        // "ab" padded: "  a", " ab", "ab ", "b  "
        // "cd" padded: "  c", " cd", "cd ", "d  "
        assert_eq!(
            trigrams,
            vec!["  a", " ab", "ab ", "b  ", "  c", " cd", "cd ", "d  "]
        );
    }

    #[test]
    fn test_subset_words_produce_subset_trigrams() {
        let full: std::collections::HashSet<String> = generate_trigrams("500g cedevita limun")
            .into_iter()
            .collect();
        let partial: std::collections::HashSet<String> =
            generate_trigrams("500g cedevita").into_iter().collect();

        assert!(
            partial.is_subset(&full),
            "Trigrams of a word-subset must be a subset of the full trigrams"
        );
    }

    #[test]
    fn test_empty_string() {
        assert!(generate_trigrams("").is_empty());
    }

    #[test]
    fn test_single_char_word() {
        let trigrams = generate_trigrams("a");
        assert_eq!(trigrams, vec!["  a", " a ", "a  "]);
    }

    #[test]
    fn test_trigram_count_single_word() {
        let text = "cedevita";
        let trigrams = generate_trigrams(text);
        assert_eq!(trigrams.len(), text.len() + 2);
    }
}
