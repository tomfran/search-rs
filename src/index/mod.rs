mod builder;
mod loader;
mod text_utils;

use std::collections::BTreeMap;
use tokenizers::Tokenizer;

use crate::disk::bits_reader::BitsReader;

pub const POSTINGS_EXTENSION: &str = ".postings";
pub const OFFSETS_EXTENSION: &str = ".offsets";
pub const DOCUMENT_LENGHTS_EXTENSION: &str = ".doc_lengths";
pub const VOCABULARY_ALPHA_EXTENSION: &str = ".alphas";
pub const VOCABULARY_LENGHTS_EXTENSION: &str = ".term_lengths";

pub struct Index {
    postings: BitsReader,
    term_offset_map: BTreeMap<String, u64>,
    doc_lenghts: Vec<u32>,
    tokenizer: Tokenizer,
}

impl Index {
    pub fn build_index(input_path: &str, output_path: &str, tokenizer_path: &str) {
        let tokenizer = text_utils::load_tokenizer(tokenizer_path, false);
        builder::build_index(input_path, output_path, &tokenizer);
    }

    pub fn load_index(input_path: &str, tokenizer_path: &str) -> Index {
        Index {
            postings: loader::build_postings_reader(input_path),
            term_offset_map: loader::load_terms_to_offsets_map(input_path),
            doc_lenghts: loader::load_document_lenghts(input_path),
            tokenizer: text_utils::load_tokenizer(tokenizer_path, false),
        }
    }

    pub fn get_postings(&mut self, term: &str) -> Option<Vec<u32>> {
        let offset = self.term_offset_map.get(term)?;
        Some(self.get_postings_internal(*offset))
    }

    fn get_postings_internal(&mut self, offset: u64) -> Vec<u32> {
        self.postings.seek(offset);
        let mut prev = 0;

        (0..self.postings.read_vbyte())
            .map(|_| {
                prev += self.postings.read_gamma();
                prev
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build() {
        Index::build_index(
            "data/index_unit_test/docs",
            "data/index_unit_test/index/test",
            "data/index_unit_test/test_tokenizer",
        );

        let mut idx = Index::load_index(
            "data/index_unit_test/index/test",
            "data/index_unit_test/test_tokenizer",
        );

        for ele in ["hello", "man", "world"] {
            assert!(idx.term_offset_map.contains_key(ele));
        }

        assert_eq!(idx.get_postings("hello").unwrap(), [0, 1]);
    }
}
