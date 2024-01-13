use std::collections::BTreeMap;

use fxhash::FxHashMap;

use crate::disk::{
    bits_reader::BitsReader, bits_writer::BitsWriter, terms_reader::TermsReader,
    terms_writer::TermsWriter,
};

use super::{OFFSETS_EXTENSION, VOCABULARY_ALPHA_EXTENSION, VOCABULARY_LENGHTS_EXTENSION};

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

pub fn load_vocabulary(input_path: &str) -> FxHashMap<String, u64> {
    let terms_path: String = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let terms_buffer = TermsReader::new(&terms_path).read_to_string();

    let lenghts_path = input_path.to_string() + VOCABULARY_LENGHTS_EXTENSION;
    let mut lenghts_reader = BitsReader::new(&lenghts_path);

    let offsets_path = input_path.to_string() + OFFSETS_EXTENSION;
    let mut offsets_reader = BitsReader::new(&offsets_path);

    let num_terms: u32 = offsets_reader.read_vbyte();

    let mut start_term_offset: usize = 0;
    let mut postings_offset = 0;

    let mut res = FxHashMap::default();

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
