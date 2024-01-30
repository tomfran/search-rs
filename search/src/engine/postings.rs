use std::cmp::min;

use super::{InMemory, OFFSETS_EXTENSION, POSTINGS_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};
use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(Default)]
pub struct Posting {
    pub document_id: u32,
    pub document_frequency: u32,
    pub positions: Vec<u32>,
}

pub type PostingsList = Vec<Posting>;
pub type DocumentIdsList = Vec<u32>;

pub struct Postings {
    reader: BitsReader,
    offsets: Vec<u64>,
}

impl Postings {
    pub fn load_postings_reader(input_path: &str) -> Postings {
        let path = input_path.to_string() + OFFSETS_EXTENSION;
        let mut offsets_reader = BitsReader::new(&path);

        let mut offset = 0;
        let offsets = (0..offsets_reader.read_vbyte())
            .map(|_| {
                offset += offsets_reader.read_gamma() as u64;
                offset
            })
            .collect();

        let path = input_path.to_string() + POSTINGS_EXTENSION;
        let reader = BitsReader::new(&path);

        Postings { reader, offsets }
    }

    pub fn write_postings(index: &InMemory, output_path: &str) {
        let postings_path = output_path.to_string() + POSTINGS_EXTENSION;
        let mut postings_writer = BitsWriter::new(&postings_path);

        let offsets_path = output_path.to_string() + OFFSETS_EXTENSION;
        let mut offsets_writer = BitsWriter::new(&offsets_path);

        let mut offset: u64 = 0;
        let mut prev_offset = 0;

        offsets_writer.write_vbyte(index.term_index_map.len() as u32);

        for idx in index.term_index_map.values() {
            offsets_writer.write_gamma(offset as u32 - prev_offset);
            prev_offset = offset as u32;

            let postings = &index.postings[*idx];

            offset += postings_writer.write_vbyte(postings.len() as u32);

            let mut prev_doc_id = 0;
            for entry in postings {
                offset += postings_writer.write_gamma(entry.document_id - prev_doc_id);
                offset += postings_writer.write_gamma(entry.document_frequency);

                let mut prev_pos = 0;
                offset += postings_writer.write_vbyte(entry.positions.len() as u32);
                for pos in &entry.positions {
                    offset += postings_writer.write_gamma(*pos - prev_pos);
                    prev_pos = *pos;
                }

                prev_doc_id = entry.document_id;
            }
        }

        postings_writer.flush();
        offsets_writer.flush();
    }

    pub fn load_postings_list(&mut self, index: usize) -> PostingsList {
        self.reader.seek(self.offsets[index]);

        let n = self.reader.read_vbyte();
        let mut document_id = 0;
        let documents: Vec<Posting> = (0..n)
            .map(|_| {
                let doc_id_delta = self.reader.read_gamma();
                let document_frequency = self.reader.read_gamma();

                document_id += doc_id_delta;

                Posting {
                    document_id,
                    document_frequency,
                    positions: self.reader.read_vbyte_gamma_gap_vector(),
                }
            })
            .collect();

        documents
    }

    pub fn load_doc_ids_list(&mut self, index: usize) -> DocumentIdsList {
        self.load_postings_list(index)
            .iter()
            .map(|e| e.document_id)
            .collect()
    }

    pub fn and_operator(p1: DocumentIdsList, p2: DocumentIdsList) -> DocumentIdsList {
        if p1.is_empty() || p2.is_empty() {
            return DocumentIdsList::default();
        }

        let mut result = Vec::with_capacity(min(p1.len(), p2.len()));

        let mut iter1 = p1.iter();
        let mut iter2 = p2.iter();

        let (mut e1, mut e2) = (iter1.next(), iter2.next());

        while let (Some(v1), Some(v2)) = (e1, e2) {
            match v1.cmp(v2) {
                Equal => {
                    result.push(*v1);
                    e1 = iter1.next();
                    e2 = iter2.next();
                }
                Less => e1 = iter1.next(),
                Greater => e2 = iter2.next(),
            }
        }

        result
    }

    pub fn or_operator(mut p1: DocumentIdsList, mut p2: DocumentIdsList) -> DocumentIdsList {
        let mut result = Vec::with_capacity(p1.len() + p2.len());

        let mut iter1 = p1.drain(..);
        let mut iter2 = p2.drain(..);

        let (mut e1, mut e2) = (iter1.next(), iter2.next());

        while let (Some(v1), Some(v2)) = (e1, e2) {
            match v1.cmp(&v2) {
                Equal => {
                    result.push(v1);
                    e1 = iter1.next();
                    e2 = iter2.next();
                }
                Less => {
                    result.push(v1);
                    e1 = iter1.next();
                }
                Greater => {
                    result.push(v2);
                    e2 = iter2.next();
                }
            }
        }

        if let Some(v) = e1 {
            result.push(v);
        }

        if let Some(v) = e2 {
            result.push(v);
        }

        result.extend(iter1);
        result.extend(iter2);

        result
    }

    pub fn not_operator(mut p: DocumentIdsList, n: u32) -> DocumentIdsList {
        if p.is_empty() {
            return (1..=n).collect();
        }

        let mut result = Vec::with_capacity((n - p.len() as u32) as usize);

        let mut iter = p.drain(..);
        let mut next_val = iter.next().unwrap();

        for val in 0..n {
            if val == next_val {
                next_val = match iter.next() {
                    Some(v) => v,
                    None => n + 1,
                };
            } else {
                result.push(val);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::postings::Postings;

    #[test]
    fn test_or_operator() {
        let p1 = vec![1, 3, 5, 7, 9];
        let p2 = vec![2, 4, 6, 8, 10];

        let result = Postings::or_operator(p1.clone(), p2.clone());
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let result_empty = Postings::or_operator(vec![], vec![]);
        assert_eq!(result_empty, vec![]);
    }

    #[test]
    fn test_and_operator() {
        let p1 = vec![1, 3, 5, 7, 10];
        let p2 = vec![2, 3, 6, 7, 10];

        let result = Postings::and_operator(p1.clone(), p2.clone());
        assert_eq!(result, vec![3, 7, 10]);

        let result_empty = Postings::and_operator(vec![1, 2, 3], vec![]);
        assert_eq!(result_empty, vec![]);

        let result_both_empty = Postings::and_operator(vec![], vec![]);
        assert_eq!(result_both_empty, vec![]);
    }

    #[test]
    fn test_not_operator() {
        let p = vec![2, 4, 6, 8];
        let n = 10;

        let result = Postings::not_operator(p.clone(), n);
        assert_eq!(result, vec![0, 1, 3, 5, 7, 9]);

        let result_empty = Postings::not_operator(vec![], n);
        assert_eq!(result_empty, (1..=n).collect::<Vec<u32>>());

        let result_full = Postings::not_operator(vec![0, 1, 2], 3);
        assert_eq!(result_full, []);
    }
}
