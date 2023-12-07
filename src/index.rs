use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs, vec,
};

use crate::{
    bits::{self, reader::Reader},
    text::tokens,
};

const OUTPUT_DIR: &str = "data/index";
const POSTINGS_EXTENSION: &str = ".postings";
const OFFSETS_EXTENSION: &str = ".offsets";

pub struct Index {
    postings: Reader,
    offsets: Vec<u64>,
    vocabulary: BTreeMap<String, u64>,
}

impl Index {
    fn build_in_memory_postings(
        input_dir: &str,
    ) -> (BTreeMap<String, usize>, Vec<BTreeMap<u32, u32>>) {
        let documents =
            fs::read_dir(input_dir).expect("error while retrieving input directory content");

        println!("{:?}", documents);
        let tokens_regex = tokens::build_tokenization_regex();

        let tokenized_docs_iter = documents
            .into_iter()
            .map(|p| p.unwrap())
            .map(|p| fs::read_to_string(p.path()).expect("error while reading file"))
            .map(|s| tokens::tokenize(&s, &tokens_regex));

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

    fn write_to_file(vocab: &BTreeMap<String, usize>, postings: &Vec<BTreeMap<u32, u32>>) {
        let postings_path = OUTPUT_DIR.to_string() + "/index" + POSTINGS_EXTENSION;
        let offsets_path = OUTPUT_DIR.to_string() + "/index" + OFFSETS_EXTENSION;

        let mut postings_writer = bits::writer::Writer::new(&postings_path);
        let mut offsets_writer = bits::writer::Writer::new(&offsets_path);

        let mut offset: u64 = 0;
        let mut prev_offset = 0;
        for (_, idx) in vocab.iter() {
            offsets_writer.write_gamma(offset as u32 - prev_offset);

            let postings = &postings[*idx];
            offset += postings_writer.write_vbyte(postings.len() as u32);

            let mut prev = 0;
            for (doc_id, frequency) in postings.iter() {
                offset += postings_writer.write_gamma(doc_id - prev);
                offset += postings_writer.write_gamma(*frequency);
                prev = *doc_id;
            }

            prev_offset = offset as u32;
        }

        postings_writer.flush();
        offsets_writer.flush();
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_build() {
//         let (a, b) = Index::build_in_memory_postings("data/wiki-data/docs");

//         Index::write_to_file(&a, &b);
//     }
// }
