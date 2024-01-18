use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

use super::DOCUMENTS_EXTENSION;

#[derive(Clone)]
pub struct Document {
    pub path: String,
    pub lenght: u32,
}

pub fn write_documents(documents: &Vec<Document>, output_path: &str) {
    let doc_path = output_path.to_string() + DOCUMENTS_EXTENSION;
    let mut doc_writer = BitsWriter::new(&doc_path);

    doc_writer.write_vbyte(documents.len() as u32);
    documents.iter().for_each(|l| {
        doc_writer.write_str(&l.path);
        doc_writer.write_vbyte(l.lenght);
    });

    doc_writer.flush();
}

pub fn load_documents(input_path: &str) -> Vec<Document> {
    let mut reader = BitsReader::new(&(input_path.to_string() + DOCUMENTS_EXTENSION));

    (0..reader.read_vbyte())
        .map(|_| Document {
            path: reader.read_str(),
            lenght: reader.read_vbyte(),
        })
        .collect()
}
