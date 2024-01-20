use fxhash::FxHashMap;

use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

use super::{InMemoryIndex, VOCABULARY_ALPHA_EXTENSION};

pub fn write_vocabulary(index: &InMemoryIndex, output_path: &str) {
    let terms_path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_writer = BitsWriter::new(&terms_path);

    let vocab = &index.term_index_map;

    terms_writer.write_vbyte(vocab.len() as u32);

    let mut prev = "";

    vocab.keys().for_each(|s| {
        let p_len = get_matching_prefix_len(prev, s);
        terms_writer.write_gamma(p_len as u32);
        let remaining: String = s.chars().skip(p_len).collect();
        terms_writer.write_str(&remaining);
        prev = s;
    });

    terms_writer.flush();
}

fn get_matching_prefix_len(s1: &str, s2: &str) -> usize {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(char1, char2)| char1 == char2)
        .count()
}

pub fn load_vocabulary(input_path: &str) -> FxHashMap<String, usize> {
    let terms_path = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_reader = BitsReader::new(&terms_path);

    let num_terms: u32 = terms_reader.read_vbyte();

    let mut prev: String = "".to_string();
    (0..num_terms)
        .map(|i| {
            let p_len = terms_reader.read_gamma();
            let prefix: String = prev.chars().take(p_len as usize).collect();
            let s = prefix + &terms_reader.read_str();
            prev = s.clone();
            (s, i as usize)
        })
        .collect()
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
