use std::collections::HashMap;

use crate::index::Index;

use self::document_selector::DocumentSelector;

mod document_selector;
pub struct QueryProcessor {
    index: Index,
    num_documents: u32,
}

impl QueryProcessor {
    pub fn build_query_processor(
        index_input_path: &str,
        index_tokenizer_path: &str,
    ) -> QueryProcessor {
        let index = Index::load_index(index_input_path, index_tokenizer_path);
        let num_documents = index.get_num_documents();

        QueryProcessor {
            index,
            num_documents,
        }
    }

    pub fn query(&mut self, query: &str) -> Vec<u32> {
        println!("\nQuery: {:?}", query);

        let mut scores: HashMap<u32, f32> = HashMap::new();

        for token in self.index.tokenize_query(query) {
            if let Some(postings) = self.index.get_term(&token) {
                let idf = (self.num_documents as f32 / postings.collection_frequency as f32).log2();

                for doc_posting in &postings.documents {
                    let doc_score = doc_posting.document_frequency as f32 * idf;
                    scores
                        .entry(doc_posting.document_id)
                        .and_modify(|s| *s += doc_score)
                        .or_insert(doc_score);
                }
            }
        }

        let mut selector = DocumentSelector::new(3);
        scores.iter().for_each(|(id, score)| {
            println!("- document: {:?}, score: {:?}", id, score);
            selector.push(*id, *score)
        });

        selector.get_sorted_ids()
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_build() {
//         let mut q = QueryProcessor::build_query_processor(
//             "data/small/index/small",
//             "data/small/bert-base-uncased",
//         );
//         q.query("google");
//         q.query("apple");
//         q.query("microsoft");
//     }
// }
