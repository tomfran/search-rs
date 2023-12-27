mod builder;
mod loader;
mod text_utils;

use rust_stemmers::Stemmer;
use std::collections::BTreeMap;
use std::fmt::Display;
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
    stemmer: Stemmer,
}

#[derive(Debug)]
pub struct PostingList {
    pub documents: Vec<PostingEntry>,
    pub collection_frequency: u32,
}

#[derive(Debug)]
pub struct PostingEntry {
    pub document_id: u32,
    pub document_frequency: u32,
}

impl Index {
    pub fn build_index(input_path: &str, output_path: &str, tokenizer_path: &str) {
        let tokenizer = text_utils::load_tokenizer(tokenizer_path, false);
        let stemmer = text_utils::load_stemmer();
        builder::build_index(input_path, output_path, &tokenizer, &stemmer);
    }

    pub fn load_index(input_path: &str, tokenizer_path: &str) -> Index {
        Index {
            postings: loader::build_postings_reader(input_path),
            term_offset_map: loader::load_terms_to_offsets_map(input_path),
            doc_lenghts: loader::load_document_lenghts(input_path),
            tokenizer: text_utils::load_tokenizer(tokenizer_path, false),
            stemmer: text_utils::load_stemmer(),
        }
    }

    pub fn get_num_documents(&self) -> u32 {
        self.doc_lenghts.len() as u32
    }

    pub fn get_term(&mut self, term: &str) -> Option<PostingList> {
        let offset = self.term_offset_map.get(term)?;

        self.postings.seek(*offset);
        let mut document_id = 0;

        let documents: Vec<PostingEntry> = (0..self.postings.read_vbyte())
            .map(|_| {
                let doc_id_delta = self.postings.read_gamma();
                let document_frequency = self.postings.read_gamma();

                document_id += doc_id_delta;

                PostingEntry {
                    document_id,
                    document_frequency,
                }
            })
            .collect();

        let collection_frequency = documents.len() as u32;

        Some(PostingList {
            documents,
            collection_frequency,
        })
    }

    pub fn tokenize_and_stem_query(&self, query: &str) -> Vec<String> {
        text_utils::tokenize_and_stem(&self.tokenizer, &self.stemmer, query)
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Index:\n- vocab size: {}\n- num. documents: {})",
            self.term_offset_map.len(),
            self.get_num_documents()
        )
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

        let pl = idx.get_term("hello").unwrap();
        assert_eq!(
            pl.documents
                .iter()
                .map(|d| d.document_id)
                .collect::<Vec<u32>>(),
            [0, 1]
        );

        assert_eq!(pl.collection_frequency, 2);
    }
}
