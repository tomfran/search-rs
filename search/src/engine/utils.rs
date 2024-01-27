pub fn get_matching_prefix_len(s1: &str, s2: &str) -> usize {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(char1, char2)| char1 == char2)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_matching_prefix_len() {
        assert_eq!(get_matching_prefix_len("hello", "hell"), 4);
        assert_eq!(get_matching_prefix_len("abc", "xyz"), 0);
        assert_eq!(get_matching_prefix_len("", ""), 0);
        assert_eq!(get_matching_prefix_len("apple", "appetizer"), 3);
        assert_eq!(get_matching_prefix_len("rust", "rust"), 4);
    }
}
