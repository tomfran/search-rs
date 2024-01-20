use super::{InMemoryIndex, OFFSETS_EXTENSION, POSTINGS_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

#[derive(Debug, Default)]
pub struct PostingList {
    pub documents: Vec<PostingEntry>,
    pub collection_frequency: u32,
}

#[derive(Debug, Default)]
pub struct PostingEntry {
    pub document_id: u32,
    pub document_frequency: u32,
    pub positions: Vec<u32>,
}

pub fn write_postings(index: &InMemoryIndex, output_path: &str) {
    let postings_path = output_path.to_string() + POSTINGS_EXTENSION;
    let mut postings_writer = BitsWriter::new(&postings_path);

    let offsets_path = output_path.to_string() + OFFSETS_EXTENSION;
    let mut offsets_writer = BitsWriter::new(&offsets_path);

    let mut offset: u64 = 0;
    let mut prev_offset = 0;

    offsets_writer.write_vbyte(index.term_index_map.len() as u32);

    for (_, idx) in index.term_index_map.iter() {
        offsets_writer.write_gamma(offset as u32 - prev_offset);
        prev_offset = offset as u32;

        let postings = &index.postings[*idx];

        offset += postings_writer.write_vbyte(postings.documents.len() as u32);

        let mut prev_doc_id = 0;
        for entry in postings.documents.iter() {
            offset += postings_writer.write_gamma(entry.document_id - prev_doc_id);
            offset += postings_writer.write_gamma(entry.document_frequency);

            let mut prev_pos = 0;
            offset += postings_writer.write_vbyte(entry.positions.len() as u32);
            for pos in entry.positions.iter() {
                offset += postings_writer.write_gamma(*pos - prev_pos);
                prev_pos = *pos;
            }

            prev_doc_id = entry.document_id;
        }
    }

    postings_writer.flush();
    offsets_writer.flush();
}

pub fn build_postings_reader(input_path: &str) -> BitsReader {
    BitsReader::new(&(input_path.to_string() + POSTINGS_EXTENSION))
}

pub fn load_postings_list(postings_reader: &mut BitsReader, offset: u64) -> PostingList {
    postings_reader.seek(offset);

    let n = postings_reader.read_vbyte();

    let mut document_id = 0;
    let documents: Vec<PostingEntry> = (0..n)
        .map(|_| {
            let doc_id_delta = postings_reader.read_gamma();
            let document_frequency = postings_reader.read_gamma();

            document_id += doc_id_delta;

            PostingEntry {
                document_id,
                document_frequency,
                positions: postings_reader.read_vbyte_gamma_gap_vector(),
            }
        })
        .collect();

    let collection_frequency = documents.len() as u32;

    PostingList {
        documents,
        collection_frequency,
    }
}

pub fn load_offsets(input_path: &str) -> Vec<u64> {
    let path = input_path.to_string() + OFFSETS_EXTENSION;
    let mut reader = BitsReader::new(&path);

    let mut offset = 0;
    (0..reader.read_vbyte())
        .map(|_| {
            offset += reader.read_gamma() as u64;
            offset
        })
        .collect()
}
