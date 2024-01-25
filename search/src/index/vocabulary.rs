use super::{utils, InMemory, VOCABULARY_ALPHA_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};
use fxhash::FxHashMap;

#[allow(dead_code)]
pub struct Vocabulary {
    term_to_index: FxHashMap<String, usize>,
    frequencies: Vec<u32>,
    index_to_term: Vec<String>,
    trigram_index: FxHashMap<String, Vec<usize>>,
}

impl Vocabulary {
    pub fn write_vocabulary(index: &InMemory, output_path: &str) {
        let path = output_path.to_string() + VOCABULARY_ALPHA_EXTENSION;
        let mut writer = BitsWriter::new(&path);

        let vocab = &index.term_index_map;

        writer.write_vbyte(vocab.len() as u32);

        // write all terms with prefix compression
        let mut prev = "";

        for s in vocab.keys() {
            let p_len = utils::get_matching_prefix_len(prev, s);
            writer.write_gamma(p_len as u32);
            let remaining: String = s.chars().skip(p_len).collect();
            prev = s;

            writer.write_str(&remaining);
        }

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
        let mut prev = String::new();

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
            if term_chars.len() < 3 {
                continue;
            }

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
        self.term_to_index.get(term).copied()
    }

    #[allow(dead_code)]

    pub fn get_term_index_spellcheck(&self, term: &str) -> Option<usize> {
        self.get_term_index(term)
            .or_else(|| self.get_closest_index(term))
    }
    #[allow(dead_code)]

    fn get_closest_index(&self, term: &str) -> Option<usize> {
        let candidates = (0..term.len() - 2)
            .map(|i| term[i..i + 3].to_string())
            .filter_map(|t| self.trigram_index.get(&t))
            .flat_map(|v| v.iter());

        candidates
            .min_by_key(|i| Self::distance(term, &self.index_to_term[**i]))
            .copied()
    }

    #[allow(unused_variables)]
    fn distance(s1: &str, s2: &str) -> u32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{index::postings::PostingList, test_utils::utils::create_temporary_file_path};

    use super::*;

    #[test]
    fn test_write_and_load() {
        let dir = create_temporary_file_path("vocab_unit");

        let mut map = BTreeMap::new();
        map.insert("hello".to_string(), 0);
        map.insert("world".to_string(), 0);

        let postings = vec![
            PostingList {
                collection_frequency: 1,
                documents: Vec::new(),
            },
            PostingList {
                collection_frequency: 2,
                documents: Vec::new(),
            },
        ];

        let index = InMemory {
            term_index_map: map,
            postings,
            documents: Vec::new(),
        };

        Vocabulary::write_vocabulary(&index, &dir);
        let loaded_vocabulary = Vocabulary::load_vocabulary(&dir);

        assert_eq!(loaded_vocabulary.index_to_term, ["hello", "world"]);
        assert_eq!(loaded_vocabulary.frequencies, [1, 2]);

        assert_eq!(*loaded_vocabulary.trigram_index.get("hel").unwrap(), [0]);
        assert_eq!(*loaded_vocabulary.trigram_index.get("ell").unwrap(), [0]);
        assert_eq!(*loaded_vocabulary.trigram_index.get("rld").unwrap(), [1]);
    }
}
