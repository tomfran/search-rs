use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

pub struct Preprocessor {
    stemmer: Stemmer,
    regex: Regex,
}

impl Preprocessor {
    pub fn new() -> Preprocessor {
        Preprocessor {
            stemmer: Stemmer::create(Algorithm::English),
            regex: Regex::new(r"[^a-zA-Z0-9\s]+").expect("error while building regex"),
        }
    }

    pub fn tokenize_and_stem(&self, text: &str) -> Vec<String> {
        self.regex
            .replace_all(text, " ")
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .map(|t| self.stemmer.stem(&t).to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_and_stem() {
        let preprocessor = Preprocessor::new();

        let text1 = "The quick brown, fox jumps over the lazy dog!!!";
        let result1 = preprocessor.tokenize_and_stem(text1);
        assert_eq!(
            result1,
            vec!["the", "quick", "brown", "fox", "jump", "over", "the", "lazi", "dog"]
        );
    }
}
