mod builder;
mod documents;
mod heap;
mod postings;
mod preprocessor;
mod utils;
mod vocabulary;

use self::documents::{Document, Documents};
use self::heap::FixedMinHeap;
use self::postings::{DocumentIdsList, Postings, PostingsList};
use self::preprocessor::Preprocessor;
use self::vocabulary::Vocabulary;
use std::cmp::min;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

pub const POSTINGS_EXTENSION: &str = ".postings";
pub const OFFSETS_EXTENSION: &str = ".offsets";
pub const DOCUMENTS_EXTENSION: &str = ".docs";
pub const VOCABULARY_ALPHA_EXTENSION: &str = ".alphas";

const WINDOW_SCORE_MULTIPLIER: f64 = 0.0;
const BM25_SCORE_MULTIPLIER: f64 = 1.0;

const BM25_KL: f64 = 1.2;
const BM25_B: f64 = 0.75;

pub struct Engine {
    vocabulary: Vocabulary,
    postings: Postings,
    documents: Documents,
    preprocessor: Preprocessor,
}

pub struct InMemory {
    term_index_map: BTreeMap<String, usize>,
    postings: Vec<PostingsList>,
    documents: Vec<Document>,
}

pub struct BooleanQueryResult {
    pub postfix_query: Vec<String>,
    pub documents_ids: DocumentIdsList,
    pub time_ms: u128,
}

pub struct RankedQueryResult {
    pub tokens: Vec<String>,
    pub documents: Vec<RankedDocumentResult>,
    pub time_ms: u128,
}

pub struct RankedDocumentResult {
    pub id: u32,
    pub path: String,
    pub score: f64,
}

#[derive(Default)]
struct DocumentScore {
    tf_idf: f64,
    term_positions: HashMap<u32, Vec<u32>>,
}

impl Engine {
    pub fn build_engine(
        input_path: &str,
        output_path: &str,
        max_freq_percentage_threshold: f64,
        min_freq_threshold: u32,
    ) {
        builder::build_engine(
            input_path,
            output_path,
            &Preprocessor::new(),
            max_freq_percentage_threshold,
            min_freq_threshold,
        );
    }

    pub fn load_index(input_path: &str) -> Engine {
        Engine {
            vocabulary: Vocabulary::load_vocabulary(input_path),
            postings: Postings::load_postings_reader(input_path),
            documents: Documents::load_documents(input_path),
            preprocessor: Preprocessor::new(),
        }
    }

    pub fn boolean_query(&mut self, postfix_expression: Vec<&str>) -> BooleanQueryResult {
        let start_time = Instant::now();

        let mut stack = Vec::new();
        let mut intermediate_result;
        let num_docs = self.documents.get_num_documents();

        for p in postfix_expression.clone() {
            match p {
                "AND" => {
                    intermediate_result =
                        Postings::and_operator(stack.pop().unwrap(), stack.pop().unwrap());
                }
                "OR" => {
                    intermediate_result =
                        Postings::or_operator(stack.pop().unwrap(), stack.pop().unwrap());
                }
                "NOT" => {
                    intermediate_result = Postings::not_operator(stack.pop().unwrap(), num_docs);
                }
                _ => {
                    intermediate_result = self
                        .vocabulary
                        .spellcheck_term(p)
                        .and_then(|t| self.get_term_doc_ids(&t))
                        .unwrap_or_default();
                }
            }

            stack.push(intermediate_result);
        }

        let time_ms = start_time.elapsed().as_millis();

        BooleanQueryResult {
            postfix_query: postfix_expression.iter().map(|s| s.to_string()).collect(),
            documents_ids: stack.pop().unwrap(),
            time_ms,
        }
    }

    pub fn free_query(&mut self, query: &str, num_results: usize) -> RankedQueryResult {
        let start_time = Instant::now();

        let tokens: Vec<String> = self
            .preprocessor
            .tokenize_and_stem(query)
            .iter()
            .filter_map(|t| self.vocabulary.spellcheck_term(t))
            .collect();

        let mut scores: HashMap<u32, DocumentScore> = HashMap::new();

        let n = self.documents.get_num_documents() as f64;
        let avgdl = self.documents.get_avg_doc_len();

        for (id, token) in tokens.iter().enumerate() {
            if let Some(postings) = self.get_term_postings(token) {
                // compute idf where n is the number of documents and
                // nq the number of documents containing query term

                let nq = self.vocabulary.get_term_frequency(token).unwrap() as f64;
                let idf = ((n - nq + 0.5) / (nq + 0.5) + 1.0).ln();

                for doc_posting in &postings {
                    // compute B25 score, where fq is the frequency of term in this documents
                    // dl is the document len, and avgdl is the average document len accross the collection

                    let fq = doc_posting.document_frequency as f64;
                    let dl = self.documents.get_doc_len(doc_posting.document_id) as f64;

                    let bm_score = idf * (fq * (BM25_KL + 1.0))
                        / (fq + BM25_KL * (1.0 - BM25_B + BM25_B * (dl / avgdl)));

                    let doc_score = scores.entry(doc_posting.document_id).or_default();
                    doc_score.tf_idf += bm_score;
                    let positions = doc_score.term_positions.entry(id as u32).or_default();

                    doc_posting
                        .positions
                        .iter()
                        .for_each(|p| positions.push(*p));
                }
            }
        }

        let mut selector = FixedMinHeap::new(num_results);
        let num_tokens = tokens.len();
        for (id, score) in &mut scores {
            score.tf_idf /= self.documents.get_doc_len(*id) as f64;
            selector.push(*id, Self::compute_score(score, num_tokens));
        }

        let documents = selector
            .get_sorted_id_priority_pairs()
            .iter()
            .map(|(id, score)| RankedDocumentResult {
                id: *id,
                score: *score,
                path: self.documents.get_doc_path(*id),
            })
            .collect();

        let time_ms = start_time.elapsed().as_millis();

        RankedQueryResult {
            tokens,
            documents,
            time_ms,
        }
    }

    fn get_term_postings(&mut self, term: &str) -> Option<PostingsList> {
        self.vocabulary
            .get_term_index(term)
            .map(|i| self.postings.load_postings_list(i))
    }

    fn get_term_doc_ids(&mut self, term: &str) -> Option<DocumentIdsList> {
        self.vocabulary
            .get_term_index(term)
            .map(|i| self.postings.load_doc_ids_list(i))
    }

    fn compute_score(document_score: &DocumentScore, num_tokens: usize) -> f64 {
        let mut window = u32::MAX;

        let mut arr: Vec<(u32, u32)> = document_score
            .term_positions
            .iter()
            .flat_map(|(id, positions)| positions.iter().map(|p| (*p, *id)))
            .collect();

        arr.sort_unstable();

        let mut j = 0;
        let mut seen: HashMap<u32, u32> = HashMap::new();
        for (pos, id) in arr.iter().copied() {
            seen.entry(id).and_modify(|c| *c += 1).or_insert(1);

            while seen.len() == num_tokens && j < arr.len() {
                let (j_pos, j_id) = arr[j];
                window = min(window, pos - j_pos + 1);

                seen.entry(j_id).and_modify(|c| *c -= 1);
                if *seen.get(&j_id).unwrap() == 0 {
                    seen.remove(&j_id);
                }

                j += 1;
            }
        }

        WINDOW_SCORE_MULTIPLIER * (num_tokens as f64 / window as f64)
            + BM25_SCORE_MULTIPLIER * document_score.tf_idf
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::utils::create_temporary_dir_path;

    #[test]
    fn test_build() {
        let index_path = &create_temporary_dir_path();

        Engine::build_engine("test_data/docs", index_path, 1.0, 0);

        let mut idx = Engine::load_index(index_path);

        for ele in ["hello", "man", "world"] {
            assert!(idx.vocabulary.get_term_index(ele).is_some());
        }

        let mut query: Vec<String> = idx
            .free_query("hello", 10)
            .documents
            .iter()
            .map(|d| d.path.clone())
            .collect();

        query.sort();

        assert_eq!(query, ["test_data/docs/1.txt", "test_data/docs/2.txt"]);

        // println!(
        //     "{:?}",
        //     idx.boolean_query(vec!["hello", "man", "OR"]).documents_ids
        // );
        // println!(
        //     "{:?}",
        //     idx.boolean_query(vec!["hello", "man", "AND"]).documents_ids
        // );
        // println!(
        //     "{:?}",
        //     idx.boolean_query(vec!["man", "NOT"]).documents_ids[0]
        // );
    }
}
