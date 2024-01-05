use std::{cmp::min, collections::HashMap};

use crate::index::Index;

use self::document_selector::DocumentSelector;

mod document_selector;

const WINDOW_MULTIPLIER: f32 = 100.0;
pub struct QueryProcessor {
    index: Index,
    num_documents: u32,
}

#[derive(Default)]
struct DocumentScore {
    tf_idf: f32,
    term_positions: HashMap<u32, Vec<u32>>,
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

    pub fn query(&mut self, query: &str, num_results: usize) -> Vec<u32> {
        let mut scores: HashMap<u32, DocumentScore> = HashMap::new();

        let tokens = self.index.tokenize_and_stem_query(query);

        for (id, token) in tokens.iter().enumerate() {
            if let Some(postings) = self.index.get_term(token) {
                let idf = (self.num_documents as f32 / postings.collection_frequency as f32).log2();

                for doc_posting in &postings.documents {
                    let td_idf_score = doc_posting.document_frequency as f32 * idf;

                    let doc_score = scores
                        .entry(doc_posting.document_id)
                        .or_insert(DocumentScore::default());

                    doc_score.tf_idf += td_idf_score;
                    let positions = doc_score
                        .term_positions
                        .entry(id as u32)
                        .or_insert(Vec::new());

                    doc_posting
                        .positions
                        .iter()
                        .for_each(|p| positions.push(*p));
                }
            }
        }

        let mut selector = DocumentSelector::new(num_results);
        let num_tokens = tokens.len();
        scores.iter().for_each(|(id, score)| {
            selector.push(*id, QueryProcessor::compute_score(score, num_tokens));
        });

        selector.get_sorted_ids()
    }

    fn compute_score(document_score: &DocumentScore, num_tokens: usize) -> f32 {
        let mut window = u32::MAX;

        let mut arr: Vec<(u32, u32)> = document_score
            .term_positions
            .iter()
            .flat_map(|(id, positions)| positions.iter().map(|p| (*p, *id)))
            .collect();

        arr.sort();

        let mut j = 0;
        let mut seen: HashMap<u32, u32> = HashMap::new();
        for (pos, id) in arr.iter().cloned() {
            seen.entry(id).and_modify(|c| *c += 1).or_insert(1);

            while seen.len() == num_tokens && j < arr.len() {
                let (j_pos, j_id) = arr[j];
                window = min(window, pos - j_pos);

                seen.entry(j_id).and_modify(|c| *c -= 1);
                if *seen.get(&j_id).unwrap() == 0 {
                    seen.remove(&j_id);
                }

                j += 1;
            }
        }

        WINDOW_MULTIPLIER * (1.0 / window as f32) + document_score.tf_idf
    }
}
