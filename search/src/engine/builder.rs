use crate::disk::file_utils::walk_dir;

use super::{
    documents::{Document, Documents},
    postings::{Posting, Postings, PostingsList},
    preprocessor::Preprocessor,
    vocabulary::Vocabulary,
    InMemory,
};
use fxhash::FxHashMap;
use indicatif::{ParallelProgressIterator, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::{
    collections::{hash_map::Entry, BTreeMap},
    fs::{self},
};
use walkdir::DirEntry;

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
    println!("- Writing postings");
    Postings::write_postings(&index, output_path);
    println!("- Writing vocabulary");
    Vocabulary::write_vocabulary(&index, output_path);
    println!("- Writing documents");
    Documents::write_documents(&index.documents, output_path);
}

fn build_in_memory(
    input_dir: &str,
    preprocessor: &Preprocessor,
    max_freq_percentage_threshold: f64,
    min_freq_threshold: u32,
) -> InMemory {
    let iterator_style = ProgressStyle::with_template(PROGRESS_STYLE)
        .unwrap()
        .progress_chars(PROGRESS_CHARS);

    let files = walk_dir(input_dir);

    println!("- Pre-processing phase");
    let processed_documents: Vec<(String, Vec<String>)> = files
        .into_par_iter()
        .progress_with_style(iterator_style.clone())
        .filter_map(|d| process_document(d, preprocessor))
        .collect();

    println!("- Indexing phase");

    // document counter
    let mut doc_id = 0;
    // postings list
    let mut postings = Vec::new();
    // word to postings index
    let mut term_index_map = FxHashMap::default();
    // per-word doc id to posting list index
    let mut term_doc_map: Vec<FxHashMap<u32, usize>> = Vec::new();
    // documents data
    let mut documents = Vec::new();

    let processed_docs_iterator = processed_documents
        .into_iter()
        .progress_with_style(iterator_style);

    for (path, tokens) in processed_docs_iterator {
        // update documents array
        documents.push(Document {
            path,
            length: tokens.len() as u32,
        });

        for (word_pos, t) in tokens.iter().enumerate() {
            // obtain postings for this word and increment collection frequency
            if !term_index_map.contains_key(t) {
                let idx = term_index_map.len();
                term_index_map.insert(t.clone(), idx);
                postings.push(PostingsList::new());
                term_doc_map.push(FxHashMap::default());
            }
            let term_index = *term_index_map.get(t).unwrap();

            // obtain document entry for this word and update it
            let postings_list = &mut postings[term_index];
            if let Entry::Vacant(e) = term_doc_map[term_index].entry(doc_id) {
                let idx = postings_list.len();
                e.insert(idx);
                postings_list.push(Posting::default());
            }
            let posting_entry_index = *term_doc_map[term_index].get(&doc_id).unwrap();

            let posting_entry = &mut postings_list[posting_entry_index];

            posting_entry.document_frequency += 1;
            posting_entry.document_id = doc_id;
            posting_entry.positions.push(word_pos as u32);
        }
        doc_id += 1;
    }

    let frequency_threshold = (doc_id as f64 * max_freq_percentage_threshold) as u32;

    let term_index_map: BTreeMap<String, usize> = term_index_map
        .into_iter()
        .filter(|(_, v)| {
            let f = postings[*v].len() as u32;
            f <= frequency_threshold && f > min_freq_threshold
        })
        .collect();

    InMemory {
        term_index_map,
        postings,
        documents,
    }
}

fn process_document(
    dir_entry: DirEntry,
    preprocessor: &Preprocessor,
) -> Option<(String, Vec<String>)> {
    let file_path = dir_entry.path();
    match fs::read_to_string(file_path) {
        Ok(file_content) => Some((
            dir_entry.path().to_str().unwrap().to_string(),
            preprocessor.tokenize_and_stem(&file_content),
        )),
        Err(err) => {
            // Print an error message including the file path
            eprintln!("Error reading file {:?}: {}", file_path, err);
            None
        }
    }
}
