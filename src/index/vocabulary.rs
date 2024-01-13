use std::collections::BTreeMap;

use fxhash::FxHashMap;

use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

use super::{OFFSETS_EXTENSION, VOCABULARY_ALPHA_EXTENSION};

pub fn write_vocabulary(vocab: &BTreeMap<String, usize>, output_path: &str) {
    let terms_path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_writer = BitsWriter::new(&terms_path);

    vocab.keys().for_each(|s| {
        terms_writer.write_str(s);
    });

    terms_writer.flush();
}

pub fn load_vocabulary(input_path: &str) -> FxHashMap<String, u64> {
    let terms_path: String = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_reader = BitsReader::new(&terms_path);

    let offsets_path = input_path.to_string() + OFFSETS_EXTENSION;
    let mut offsets_reader = BitsReader::new(&offsets_path);

    let num_terms: u32 = offsets_reader.read_vbyte();
    let mut postings_offset = 0;

    let mut res = FxHashMap::default();

    for _ in 0..num_terms {
        let postings_offset_delta = offsets_reader.read_gamma() as u64;
        postings_offset += postings_offset_delta;

        res.insert(terms_reader.read_str(), postings_offset);
    }

    res
}
