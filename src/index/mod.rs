mod builder;
mod documents;
mod postings;
mod text;
mod vocabulary;

use rust_stemmers::Stemmer;
use std::collections::BTreeMap;
use std::fmt::Display;
use tokenizers::Tokenizer;

use crate::disk::bits_reader::BitsReader;

use self::postings::PostingList;

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

pub struct InMemoryIndex {
    term_index_map: BTreeMap<String, usize>,
    postings: Vec<PostingList>,
    document_lengths: Vec<u32>,
}

impl Index {
    pub fn build_index(input_path: &str, output_path: &str, tokenizer_path: &str) {
        let tokenizer = text::load_tokenizer(tokenizer_path, false);
        let stemmer = text::load_stemmer();
        builder::build_index(input_path, output_path, &tokenizer, &stemmer);
    }

    pub fn load_index(input_path: &str, tokenizer_path: &str) -> Index {
        Index {
            postings: postings::build_postings_reader(input_path),
            term_offset_map: vocabulary::load_vocabulary(input_path),
            doc_lenghts: documents::load_document_lenghts(input_path),
            tokenizer: text::load_tokenizer(tokenizer_path, false),
            stemmer: text::load_stemmer(),
        }
    }

    pub fn get_num_documents(&self) -> u32 {
        self.doc_lenghts.len() as u32
    }

    pub fn get_term(&mut self, term: &str) -> Option<postings::PostingList> {
        let offset = self.term_offset_map.get(term)?;
        Some(postings::load_postings_list(&mut self.postings, *offset))
    }

    pub fn tokenize_and_stem_query(&self, query: &str) -> Vec<String> {
        text::tokenize_and_stem(&self.tokenizer, &self.stemmer, query)
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

        let pl = idx.get_term("world").unwrap();
        assert_eq!(pl.documents[0].positions, [1]);
    }
}
