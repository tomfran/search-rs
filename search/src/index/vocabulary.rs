use super::{utils, InMemoryIndex, VOCABULARY_ALPHA_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};
use fxhash::FxHashMap;

pub struct Vocabulary {
    term_to_index: FxHashMap<String, usize>,
    frequencies: Vec<u32>,
    index_to_term: Vec<String>,
    trigram_index: FxHashMap<String, Vec<usize>>,
}

impl Vocabulary {
    pub fn write_vocabulary(index: &InMemoryIndex, output_path: &str) {
        let path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
        let mut writer = BitsWriter::new(&path);

        let vocab = &index.term_index_map;

        writer.write_vbyte(vocab.len() as u32);

        // write all terms with prefix compression
        let mut prev = "";

        vocab.keys().for_each(|s| {
            let p_len = utils::get_matching_prefix_len(prev, s);
            writer.write_gamma(p_len as u32);
            let remaining: String = s.chars().skip(p_len).collect();
            prev = s;

            writer.write_str(&remaining);
        });

        // write all collection frequencies
        index.postings.iter().for_each(|p| {
            writer.write_vbyte(p.collection_frequency);
        });

        writer.flush();
    }

    pub fn load_vocabulary(input_path: &str) -> Vocabulary {
        let path = input_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
        let mut reader = BitsReader::new(&path);

        let num_terms: u32 = reader.read_vbyte();

        // read prefix compressed terms
        let mut prev = "".to_string();

        let mut index_to_term = Vec::new();

        let term_to_index = (0..num_terms)
            .map(|i| {
                let p_len = reader.read_gamma();
                let prefix: String = prev.chars().take(p_len as usize).collect();
                let s = prefix + &reader.read_str();
                prev = s.clone();

                index_to_term.push(s.clone());

                (s, i as usize)
            })
            .collect();

        // read frequencies
        let frequencies = (0..num_terms).map(|_| reader.read_vbyte()).collect();

        // build trigram index
        let mut trigram_index = FxHashMap::default();
        for (index, term) in index_to_term.iter().enumerate() {
            let term_chars: Vec<char> = term.chars().collect();

            for i in 0..term_chars.len() - 2 {
                let trigram = &term_chars[i..i + 3];
                let trigram_key = trigram.iter().collect();

                trigram_index
                    .entry(trigram_key)
                    .or_insert_with(Vec::new)
                    .push(index);
            }
        }

        Vocabulary {
            term_to_index,
            frequencies,
            index_to_term,
            trigram_index,
        }
    }

    pub fn get_term_index(&self, term: &str) -> Option<usize> {
        self.term_to_index.get(term).map(|i| *i)
    }
}
