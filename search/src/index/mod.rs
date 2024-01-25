mod builder;
mod documents;
mod postings;
mod preprocessor;
mod utils;
mod vocabulary;

use self::documents::{Document, Documents};
use self::postings::{PostingList, Postings};
use self::preprocessor::Preprocessor;
use self::vocabulary::Vocabulary;
use std::collections::BTreeMap;

pub const POSTINGS_EXTENSION: &str = ".postings";
pub const OFFSETS_EXTENSION: &str = ".offsets";
pub const DOCUMENTS_EXTENSION: &str = ".docs";
pub const VOCABULARY_ALPHA_EXTENSION: &str = ".alphas";

pub struct Index {
    vocabulary: Vocabulary,
    postings: Postings,
    documents: Documents,
    preprocessor: Preprocessor,
}

pub struct InMemoryIndex {
    term_index_map: BTreeMap<String, usize>,
    postings: Vec<PostingList>,
    documents: Vec<Document>,
}

impl Index {
    pub fn build_index(input_path: &str, output_path: &str) {
        builder::build_index(input_path, output_path, &Preprocessor::new());
    }

    pub fn load_index(input_path: &str) -> Index {
        Index {
            vocabulary: Vocabulary::load_vocabulary(input_path),
            postings: Postings::load_postings_reader(input_path),
            documents: Documents::load_documents(input_path),
            preprocessor: Preprocessor::new(),
        }
    }

    pub fn get_term_postings(&mut self, term: &str) -> Option<postings::PostingList> {
        self.vocabulary
            .get_term_index(term)
            .map(|i| self.postings.load_postings_list(i))
    }

    pub fn get_query_tokens(&self, query: &str) -> Vec<String> {
        self.preprocessor.tokenize_and_stem(query)
    }

    pub fn get_num_documents(&self) -> u32 {
        self.documents.get_num_documents()
    }

    pub fn get_document_len(&self, doc_id: u32) -> u32 {
        self.documents.get_doc_len(doc_id)
    }

    pub fn get_document_path(&self, doc_id: u32) -> String {
        self.documents.get_doc_path(doc_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::utils::create_temporary_dir_path;

    #[test]
    fn test_build() {
        let index_path = &create_temporary_dir_path();

        Index::build_index("test_data/docs", index_path);

        let mut idx = Index::load_index(index_path);

        for ele in ["hello", "man", "world"] {
            assert!(idx.vocabulary.get_term_index(ele).is_some());
        }

        let pl = idx.get_term_postings("hello").unwrap();

        let mut hello_docs = pl
            .documents
            .iter()
            .map(|d| idx.get_document_path(d.document_id))
            .collect::<Vec<String>>();

        hello_docs.sort();

        assert_eq!(hello_docs, ["test_data/docs/1.txt", "test_data/docs/2.txt"]);

        assert_eq!(pl.collection_frequency, 2);

        let pl = idx.get_term_postings("world").unwrap();
        assert_eq!(pl.documents[0].positions, [1]);
    }
}
