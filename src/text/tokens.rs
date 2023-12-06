use regex::Regex;

pub fn tokenize(s: &str, re: Regex) -> Vec<String> {
    let vec: Vec<String> = re
        .replace_all(s, "")
        .to_lowercase()
        .split_whitespace()
        .map(|t| t.to_string())
        .collect();
    vec
}

pub fn build_tokenization_regex() -> Regex {
    Regex::new(r"[^a-zA-Z\s]").unwrap()
}

#[cfg(test)]
mod test {

    use super::tokenize;
    use crate::text::tokens::build_tokenization_regex;

    #[test]
    fn test_tokenization() {
        let r = build_tokenization_regex();
        let mut t = tokenize("123#Hello, __World!", r);
        t.sort();

        assert_eq!(t, ["hello", "world"]);
    }
}
