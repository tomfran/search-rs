use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

use super::DOCUMENT_LENGHTS_EXTENSION;

pub fn write_doc_lentghts(document_lenghts: &Vec<u32>, output_path: &str) {
    let doc_path = output_path.to_string() + DOCUMENT_LENGHTS_EXTENSION;
    let mut doc_writer = BitsWriter::new(&doc_path);

    doc_writer.write_vbyte(document_lenghts.len() as u32);
    document_lenghts.iter().for_each(|l| {
        doc_writer.write_gamma(*l);
    });

    doc_writer.flush();
}

pub fn load_document_lenghts(input_path: &str) -> Vec<u32> {
    let mut reader = BitsReader::new(&(input_path.to_string() + DOCUMENT_LENGHTS_EXTENSION));
    reader.read_vbyte_gamma_vector()
}
