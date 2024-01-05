use std::{
    collections::{BTreeMap, HashMap},
    fs,
    sync::Mutex,
};

use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
use rust_stemmers::Stemmer;
use tokenizers::Tokenizer;

use super::{
    documents::write_doc_lentghts,
    postings::{write_postings, PostingEntry, PostingList},
    text,
    vocabulary::write_vocabulary,
    InMemoryIndex,
};

const PROGRESS_STYLE: &str =
    " Documents per second: {per_sec:<3}\n\n [{elapsed_precise}] [{bar:50}] {pos}/{len} ({eta})";
const PROGRESS_CHARS: &str = "=> ";

pub fn build_index(input_dir: &str, output_path: &str, tokenizer: &Tokenizer, stemmer: &Stemmer) {
    let index: InMemoryIndex = build_in_memory(input_dir, tokenizer, stemmer);
    write_postings(&index, output_path);
    write_vocabulary(&index.term_index_map, output_path);
    write_doc_lentghts(&index.document_lengths, output_path);
}

fn build_in_memory(input_dir: &str, tokenizer: &Tokenizer, stemmer: &Stemmer) -> InMemoryIndex {
    let documents: Vec<fs::DirEntry> = fs::read_dir(input_dir)
        .expect("error while retrieving input directory content")
        .map(|p| p.unwrap())
        .collect();

    let doc_id_mutex = Mutex::new(0);
    let term_index_map = Mutex::new(BTreeMap::new());

    let postings: Mutex<Vec<PostingList>> = Mutex::new(Vec::new());
    let term_doc_map: Mutex<Vec<HashMap<u32, usize>>> = Mutex::new(Vec::new());

    let document_lengths = Mutex::new(Vec::new());

    documents
        .into_par_iter()
        .progress_with_style(
            ProgressStyle::with_template(PROGRESS_STYLE)
                .unwrap()
                .progress_chars(PROGRESS_CHARS),
        )
        .for_each(|d| {
            let file_content = fs::read_to_string(d.path()).expect("error while reading file");
            let tokens = text::tokenize_and_stem(tokenizer, stemmer, &file_content);

            let mut doc_id = doc_id_mutex.lock().unwrap();

            document_lengths.lock().unwrap().push(tokens.len() as u32);

            let mut l_term_index_map = term_index_map.lock().unwrap();
            let mut l_postings = postings.lock().unwrap();
            let mut l_term_doc_map = term_doc_map.lock().unwrap();

            for (word_pos, t) in tokens.iter().enumerate() {
                if !l_term_index_map.contains_key(t) {
                    let idx = l_term_index_map.len();
                    l_term_index_map.insert(t.clone(), idx);
                    l_postings.push(PostingList::default());
                    l_term_doc_map.push(HashMap::new());
                }
                let term_index = *l_term_index_map.get(t).unwrap();

                let postings_list = &mut l_postings[term_index];
                postings_list.collection_frequency += 1;

                if !l_term_doc_map[term_index].contains_key(&doc_id) {
                    let idx = postings_list.documents.len();
                    l_term_doc_map[term_index].insert(*doc_id, idx);
                    postings_list.documents.push(PostingEntry::default());
                }
                let posting_entry_index = *l_term_doc_map[term_index].get(&doc_id).unwrap();

                let posting_entry = &mut postings_list.documents[posting_entry_index];

                posting_entry.document_frequency += 1;
                posting_entry.document_id = *doc_id;
                posting_entry.positions.push(word_pos as u32);
            }
            *doc_id += 1;
        });

    InMemoryIndex {
        term_index_map: term_index_map.into_inner().unwrap(),
        postings: postings.into_inner().unwrap(),
        document_lengths: document_lengths.into_inner().unwrap(),
    }
}
