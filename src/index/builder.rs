use rust_stemmers::Stemmer;
use std::{collections::BTreeMap, fs};
use tokenizers::Tokenizer;

use crate::disk::{bits_writer::BitsWriter, terms_writer::TermsWriter};

use super::{
    text_utils,
    {
        DOCUMENT_LENGHTS_EXTENSION, OFFSETS_EXTENSION, POSTINGS_EXTENSION,
        VOCABULARY_ALPHA_EXTENSION, VOCABULARY_LENGHTS_EXTENSION,
    },
};

struct InMemoryIndex {
    term_index_map: BTreeMap<String, usize>,
    postings: Vec<BTreeMap<u32, u32>>,
    document_lenghts: Vec<u32>,
}

pub fn build_index(input_dir: &str, output_path: &str, tokenizer: &Tokenizer, stemmer: &Stemmer) {
    let index = build_in_memory(input_dir, tokenizer, stemmer);
    write_postings(&index, output_path);
    write_vocabulary(&index.term_index_map, output_path);
    write_doc_lentghts(&index.document_lenghts, output_path);
}

fn build_in_memory(input_dir: &str, tokenizer: &Tokenizer, stemmer: &Stemmer) -> InMemoryIndex {
    let documents =
        fs::read_dir(input_dir).expect("error while retrieving input directory content");

    let tokenized_docs_iter = documents
        .into_iter()
        .map(|p| p.unwrap())
        .map(|p| fs::read_to_string(p.path()).expect("error while reading file"))
        .map(|s| text_utils::tokenize_and_stem(tokenizer, stemmer, &s));

    let mut term_index_map: BTreeMap<String, usize> = BTreeMap::new();
    let mut postings: Vec<BTreeMap<u32, u32>> = Vec::new();
    let mut document_lenghts: Vec<u32> = Vec::new();

    for (doc_id, tokens) in tokenized_docs_iter.enumerate() {
        document_lenghts.push(tokens.len() as u32);

        if doc_id % 1000 == 0 && doc_id > 0 {
            println!("Document num: {}", doc_id);
        }

        for t in tokens.iter() {
            let value: Option<&usize> = term_index_map.get(t);

            let postings_counter = match value {
                Some(idx) => &mut postings[*idx],
                None => {
                    let idx = term_index_map.len();
                    term_index_map.insert(t.clone(), idx);
                    postings.push(BTreeMap::new());
                    &mut postings[idx]
                }
            };
            let key = doc_id as u32;
            postings_counter
                .entry(key)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    InMemoryIndex {
        term_index_map,
        postings,
        document_lenghts,
    }
}

fn write_postings(index: &InMemoryIndex, output_path: &str) {
    let postings_path = output_path.to_string() + POSTINGS_EXTENSION;
    let mut postings_writer = BitsWriter::new(&postings_path);

    let offsets_path = output_path.to_string() + OFFSETS_EXTENSION;
    let mut offsets_writer = BitsWriter::new(&offsets_path);

    let mut offset: u64 = 0;
    let mut prev_offset = 0;

    offsets_writer.write_vbyte(index.term_index_map.len() as u32);

    for (_, idx) in index.term_index_map.iter() {
        offsets_writer.write_gamma(offset as u32 - prev_offset);
        prev_offset = offset as u32;

        let postings: &BTreeMap<u32, u32> = &index.postings[*idx];
        offset += postings_writer.write_vbyte(postings.len() as u32);

        let mut prev = 0;
        for (doc_id, frequency) in postings.iter() {
            offset += postings_writer.write_gamma(doc_id - prev);
            offset += postings_writer.write_gamma(*frequency);
            prev = *doc_id;
        }
    }

    postings_writer.flush();
    offsets_writer.flush();
}

fn write_vocabulary(vocab: &BTreeMap<String, usize>, output_path: &str) {
    let terms_path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_writer = TermsWriter::new(&terms_path);

    let lenghts_path = output_path.to_string() + VOCABULARY_LENGHTS_EXTENSION;
    let mut lenghts_writer = BitsWriter::new(&lenghts_path);

    for term in vocab.keys() {
        lenghts_writer.write_gamma(term.len() as u32);
        terms_writer.write_term(term);
    }

    lenghts_writer.flush();
    terms_writer.flush();
}

fn write_doc_lentghts(document_lenghts: &Vec<u32>, output_path: &str) {
    let doc_path = output_path.to_string() + DOCUMENT_LENGHTS_EXTENSION;
    let mut doc_writer = BitsWriter::new(&doc_path);

    doc_writer.write_vbyte(document_lenghts.len() as u32);
    document_lenghts.iter().for_each(|l| {
        doc_writer.write_gamma(*l);
    });

    doc_writer.flush();
}
