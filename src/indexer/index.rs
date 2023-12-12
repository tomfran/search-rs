use super::disk_utils;
use crate::disk::bits_reader::BitsReader;
use std::collections::BTreeMap;

pub struct Index {
    postings: BitsReader,
    terms_to_offsets: BTreeMap<String, u64>,
}

impl Index {
    pub fn build_index(input_dir: &str, output_path: &str) {
        let (words, postings) = disk_utils::build_in_memory_postings(input_dir);
        disk_utils::write_postings(&words, &postings, output_path);
        disk_utils::write_vocabulary(&words, output_path);
    }

    pub fn load_index(input_path: &str) -> Index {
        Index {
            postings: disk_utils::build_postings_reader(input_path),
            terms_to_offsets: disk_utils::read_terms_to_offsets_map(input_path),
        }
    }

    pub fn get_postings(&mut self, term: &str) -> Option<Vec<u32>> {
        let offset = self.terms_to_offsets.get(term)?;
        Some(self.get_postings_internal(*offset))
    }

    fn get_postings_internal(&mut self, offset: u64) -> Vec<u32> {
        self.postings.seek(offset);
        let mut prev = 0;

        (0..self.postings.read_vbyte())
            .map(|_| {
                prev += self.postings.read_gamma();
                prev
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    fn test_build() {
        Index::build_index("data/dummy/docs", "data/dummy/index/dum");

        let mut idx = Index::load_index("data/dummy/index/dum");

        println!("{:?}", idx.terms_to_offsets);
        println!("{:?}", idx.get_postings("my"));
    }
}
