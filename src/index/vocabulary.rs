use std::collections::BTreeMap;

use fxhash::FxHashMap;

use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

use super::VOCABULARY_ALPHA_EXTENSION;

pub fn write_vocabulary(vocab: &BTreeMap<String, usize>, output_path: &str) {
    let terms_path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_writer = BitsWriter::new(&terms_path);

    terms_writer.write_vbyte(vocab.len() as u32);

    vocab.keys().for_each(|s| {
        terms_writer.write_str(s);
    });

    terms_writer.flush();
}

pub fn load_vocabulary(input_path: &str) -> FxHashMap<String, usize> {
    let terms_path: String = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut terms_reader = BitsReader::new(&terms_path);

    let num_terms: u32 = terms_reader.read_vbyte();

    (0..num_terms)
        .map(|i| (terms_reader.read_str(), i as usize))
        .collect()
}
