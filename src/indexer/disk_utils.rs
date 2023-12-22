use tokenizers::Tokenizer;

use crate::disk::{
    bits_reader::BitsReader, bits_writer::BitsWriter, terms_reader::TermsReader,
    terms_writer::TermsWriter,
};
use std::{collections::BTreeMap, fs};

use super::text_utils;

const POSTINGS_EXTENSION: &str = ".postings";
const OFFSETS_EXTENSION: &str = ".offsets";

const VOCABULARY_ALPHA_EXTENSION: &str = ".alphas";
const VOCABULARY_LENGHTS_EXTENSION: &str = ".lengths";

pub fn build_in_memory_postings(
    input_dir: &str,
    tokenizer: &Tokenizer,
) -> (BTreeMap<String, usize>, Vec<BTreeMap<u32, u32>>) {
    let documents =
        fs::read_dir(input_dir).expect("error while retrieving input directory content");

    let tokenized_docs_iter = documents
        .into_iter()
        .map(|p| p.unwrap())
        .map(|p| fs::read_to_string(p.path()).expect("error while reading file"))
        .map(|s| text_utils::tokenize(tokenizer, &s));

    let mut words: BTreeMap<String, usize> = BTreeMap::new();
    let mut in_memory_postings: Vec<BTreeMap<u32, u32>> = Vec::new();

    for (doc_id, tokens) in tokenized_docs_iter.enumerate() {
        for t in tokens.iter() {
            let value: Option<&usize> = words.get(t);

            let postings_counter = match value {
                Some(idx) => &mut in_memory_postings[*idx],
                None => {
                    let idx = words.len();
                    words.insert(t.clone(), idx);
                    in_memory_postings.push(BTreeMap::new());
                    &mut in_memory_postings[idx]
                }
            };
            let key = doc_id as u32;
            postings_counter
                .entry(key)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    (words, in_memory_postings)
}

pub fn write_postings(
    vocab: &BTreeMap<String, usize>,
    postings: &[BTreeMap<u32, u32>],
    output_path: &str,
) {
    let postings_path = output_path.to_string() + POSTINGS_EXTENSION;
    let mut postings_writer = BitsWriter::new(&postings_path);

    let offsets_path = output_path.to_string() + OFFSETS_EXTENSION;
    let mut offsets_writer = BitsWriter::new(&offsets_path);

    let mut offset: u64 = 0;
    let mut prev_offset = 0;

    offsets_writer.write_vbyte(vocab.len() as u32);

    for (_, idx) in vocab.iter() {
        offsets_writer.write_gamma(offset as u32 - prev_offset);
        prev_offset = offset as u32;

        let postings: &BTreeMap<u32, u32> = &postings[*idx];
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

pub fn write_vocabulary(vocab: &BTreeMap<String, usize>, output_path: &str) {
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

pub fn read_terms_to_offsets_map(input_path: &str) -> BTreeMap<String, u64> {
    let terms_path: String = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let terms_buffer = TermsReader::new(&terms_path).read_to_string();

    let lenghts_path = input_path.to_string() + VOCABULARY_LENGHTS_EXTENSION;
    let mut lenghts_reader = BitsReader::new(&lenghts_path);

    let offsets_path = input_path.to_string() + OFFSETS_EXTENSION;
    let mut offsets_reader = BitsReader::new(&offsets_path);

    let num_terms: u32 = offsets_reader.read_vbyte();

    let mut start_term_offset: usize = 0;
    let mut postings_offset = 0;

    let mut res: BTreeMap<String, u64> = BTreeMap::new();

    for _ in 0..num_terms {
        let term_length = lenghts_reader.read_gamma() as usize;

        let postings_offset_delta = offsets_reader.read_gamma() as u64;
        postings_offset += postings_offset_delta;

        res.insert(
            terms_buffer[start_term_offset..start_term_offset + term_length].to_string(),
            postings_offset,
        );

        start_term_offset += term_length;
    }

    res
}

pub fn build_postings_reader(input_path: &str) -> BitsReader {
    BitsReader::new(&(input_path.to_string() + POSTINGS_EXTENSION))
}
