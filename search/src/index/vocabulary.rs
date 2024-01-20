use super::{utils, InMemoryIndex, VOCABULARY_ALPHA_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};
use fxhash::FxHashMap;

pub fn write_vocabulary(index: &InMemoryIndex, output_path: &str) {
    let path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut writer = BitsWriter::new(&path);

    let vocab = &index.term_index_map;

    writer.write_vbyte(vocab.len() as u32);

    let mut prev = "";

    vocab.keys().for_each(|s| {
        let p_len = utils::get_matching_prefix_len(prev, s);
        writer.write_gamma(p_len as u32);
        let remaining: String = s.chars().skip(p_len).collect();
        prev = s;

        writer.write_str(&remaining);
    });

    writer.flush();
}

pub fn load_vocabulary(input_path: &str) -> FxHashMap<String, usize> {
    let path = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
    let mut reader = BitsReader::new(&path);

    let num_terms: u32 = reader.read_vbyte();

    let mut prev = "".to_string();
    (0..num_terms)
        .map(|i| {
            let p_len = reader.read_gamma();
            let prefix: String = prev.chars().take(p_len as usize).collect();
            let s = prefix + &reader.read_str();
            prev = s.clone();
            (s, i as usize)
        })
        .collect()
}
