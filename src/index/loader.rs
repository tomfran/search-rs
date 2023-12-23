use std::collections::BTreeMap;

use super::{
    DOCUMENT_LENGHTS_EXTENSION, OFFSETS_EXTENSION, POSTINGS_EXTENSION, VOCABULARY_ALPHA_EXTENSION,
    VOCABULARY_LENGHTS_EXTENSION,
};
use crate::disk::{bits_reader::BitsReader, terms_reader::TermsReader};

pub fn load_terms_to_offsets_map(input_path: &str) -> BTreeMap<String, u64> {
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

pub fn load_document_lenghts(input_path: &str) -> Vec<u32> {
    let mut reader = BitsReader::new(&(input_path.to_string() + DOCUMENT_LENGHTS_EXTENSION));
    let n = reader.read_vbyte();
    (0..n).map(|_| reader.read_gamma()).collect()
}

pub fn build_postings_reader(input_path: &str) -> BitsReader {
    BitsReader::new(&(input_path.to_string() + POSTINGS_EXTENSION))
}
