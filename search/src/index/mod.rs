mod builder;
mod documents;
mod postings;
mod text;
mod vocabulary;

use fxhash::FxHashMap;
use rust_stemmers::Stemmer;
use std::collections::BTreeMap;
use std::fmt::Display;
use tokenizers::Tokenizer;

use crate::disk::bits_reader::BitsReader;

use self::documents::Document;
use self::postings::PostingList;

pub const POSTINGS_EXTENSION: &str = ".postings";
pub const OFFSETS_EXTENSION: &str = ".offsets";
pub const DOCUMENTS_EXTENSION: &str = ".docs";
pub const VOCABULARY_ALPHA_EXTENSION: &str = ".alphas";

pub struct Index {
    term_to_index: FxHashMap<String, usize>,
    postings: BitsReader,
    term_offsets: Vec<u64>,
    documents: Vec<Document>,
    tokenizer: Tokenizer,
    stemmer: Stemmer,
}

pub struct InMemoryIndex {
    term_index_map: BTreeMap<String, usize>,
    postings: Vec<PostingList>,
    documents: Vec<Document>,
}

impl Index {
    pub fn build_index(input_path: &str, output_path: &str, tokenizer_path: &str) {
        let tokenizer = text::load_tokenizer(tokenizer_path, false);
        let stemmer = text::load_stemmer();
        builder::build_index(input_path, output_path, &tokenizer, &stemmer);
    }

    pub fn load_index(input_path: &str, tokenizer_path: &str) -> Index {
        Index {
            term_to_index: vocabulary::load_vocabulary(input_path),
            postings: postings::build_postings_reader(input_path),
            term_offsets: postings::load_offsets(input_path),
            documents: documents::load_documents(input_path),
            tokenizer: text::load_tokenizer(tokenizer_path, false),
            stemmer: text::load_stemmer(),
        }
    }

    pub fn get_num_documents(&self) -> u32 {
        self.documents.len() as u32
    }

    pub fn get_document_len(&self, doc_id: u32) -> u32 {
        self.documents[doc_id as usize].lenght
    }

    pub fn get_document_path(&self, doc_id: u32) -> String {
        self.documents[doc_id as usize].path.clone()
    }

    pub fn get_term(&mut self, term: &str) -> Option<postings::PostingList> {
        self.term_to_index
            .get(term)
            .map(|i| self.term_offsets[*i])
            .map(|o| postings::load_postings_list(&mut self.postings, o))
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
            self.term_to_index.len(),
            self.get_num_documents()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::utils::create_temporary_dir_path;

    #[test]
    fn test_build() {
        let index_path = &create_temporary_dir_path();

        Index::build_index("test_data/docs", index_path, "test_data/test_tokenizer");

        let mut idx = Index::load_index(index_path, "test_data/test_tokenizer");

        for ele in ["hello", "man", "world"] {
            assert!(idx.term_to_index.contains_key(ele));
        }

        let pl = idx.get_term("hello").unwrap();

        let mut hello_docs = pl
            .documents
            .iter()
            .map(|d| idx.get_document_path(d.document_id))
            .collect::<Vec<String>>();

        hello_docs.sort();

        assert_eq!(hello_docs, ["test_data/docs/1.txt", "test_data/docs/2.txt"]);

        assert_eq!(pl.collection_frequency, 2);

        let pl = idx.get_term("world").unwrap();
        assert_eq!(pl.documents[0].positions, [1]);
    }
}
