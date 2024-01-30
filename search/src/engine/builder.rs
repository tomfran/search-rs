use super::{
    documents::{Document, Documents},
    postings::{Posting, Postings, PostingsList},
    preprocessor::Preprocessor,
    vocabulary::Vocabulary,
    InMemory,
};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    sync::Mutex,
};

const PROGRESS_STYLE: &str =
    "Documents per second: {per_sec:<3}\n\n[{elapsed_precise}] [{bar:50}] {pos}/{len} [{eta_precise}]";
const PROGRESS_CHARS: &str = "=> ";

pub fn build_engine(
    input_path: &str,
    output_path: &str,
    preprocessor: &Preprocessor,
    max_freq_percentage_threshold: f64,
    min_freq_threshold: u32,
) {
    let index: InMemory = build_in_memory(
        input_path,
        preprocessor,
        max_freq_percentage_threshold,
        min_freq_threshold,
    );
    Postings::write_postings(&index, output_path);
    Vocabulary::write_vocabulary(&index, output_path);
    Documents::write_documents(&index.documents, output_path);
}

fn build_in_memory(
    input_dir: &str,
    preprocessor: &Preprocessor,
    max_freq_percentage_threshold: f64,
    min_freq_threshold: u32,
) -> InMemory {
    let files: Vec<fs::DirEntry> = fs::read_dir(input_dir)
        .expect("error while retrieving input directory content")
        .map(std::result::Result::unwrap)
        .collect();

    // document counter
    let doc_id_mutex = Mutex::new(0);
    // postings list
    let postings: Mutex<Vec<PostingsList>> = Mutex::new(Vec::new());
    // word to postings index
    let term_index_map = Mutex::new(HashMap::new());
    // per-word doc id to posting list index
    let term_doc_map: Mutex<Vec<HashMap<u32, usize>>> = Mutex::new(Vec::new());
    // documents data
    let documents = Mutex::new(Vec::new());

    files
        .into_par_iter()
        .progress_with_style(
            ProgressStyle::with_template(PROGRESS_STYLE)
                .unwrap()
                .progress_chars(PROGRESS_CHARS),
        )
        .for_each(|d| {
            let file_content = fs::read_to_string(d.path()).expect("error while reading file");
            let tokens = preprocessor.tokenize_and_stem(&file_content);

            let mut doc_id = doc_id_mutex.lock().unwrap();

            // update documents array
            documents.lock().unwrap().push(Document {
                path: d.path().to_str().unwrap().to_string(),
                length: tokens.len() as u32,
            });

            let mut l_term_index_map = term_index_map.lock().unwrap();
            let mut l_postings = postings.lock().unwrap();
            let mut l_term_doc_map = term_doc_map.lock().unwrap();

            for (word_pos, t) in tokens.iter().enumerate() {
                // obtain postings for this word and increment collection frequency
                if !l_term_index_map.contains_key(t) {
                    let idx = l_term_index_map.len();
                    l_term_index_map.insert(t.clone(), idx);
                    l_postings.push(PostingsList::new());
                    l_term_doc_map.push(HashMap::new());
                }
                let term_index = *l_term_index_map.get(t).unwrap();

                // obtain document entry for this word and update it
                let postings_list = &mut l_postings[term_index];
                if !l_term_doc_map[term_index].contains_key(&doc_id) {
                    let idx = postings_list.len();
                    l_term_doc_map[term_index].insert(*doc_id, idx);
                    postings_list.push(Posting::default());
                }
                let posting_entry_index = *l_term_doc_map[term_index].get(&doc_id).unwrap();

                let posting_entry = &mut postings_list[posting_entry_index];

                posting_entry.document_frequency += 1;
                posting_entry.document_id = *doc_id;
                posting_entry.positions.push(word_pos as u32);
            }
            *doc_id += 1;
        });

    let final_postings = postings.into_inner().unwrap();

    let frequency_threshold =
        (doc_id_mutex.into_inner().unwrap() as f64 * max_freq_percentage_threshold) as u32;

    let sorted_term_index_map: BTreeMap<String, usize> = term_index_map
        .into_inner()
        .unwrap()
        .into_iter()
        .filter(|(_, v)| {
            let f = final_postings[*v].len() as u32;
            f <= frequency_threshold && f > min_freq_threshold
        })
        .collect();

    InMemory {
        term_index_map: sorted_term_index_map,
        postings: final_postings,
        documents: documents.into_inner().unwrap(),
    }
}
