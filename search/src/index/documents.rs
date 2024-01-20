use super::{utils, DOCUMENTS_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

#[derive(Clone)]
pub struct Document {
    pub path: String,
    pub lenght: u32,
}

pub fn write_documents(documents: &Vec<Document>, output_path: &str) {
    let path = output_path.to_string() + DOCUMENTS_EXTENSION;
    let mut writer = BitsWriter::new(&path);

    let mut prev = "";

    writer.write_vbyte(documents.len() as u32);
    documents.iter().for_each(|l| {
        let p_len = utils::get_matching_prefix_len(prev, &l.path);
        writer.write_gamma(p_len as u32);
        let remaining: String = l.path.chars().skip(p_len).collect();
        prev = &l.path;

        writer.write_str(&remaining);
        writer.write_vbyte(l.lenght);
    });

    writer.flush();
}

pub fn load_documents(input_path: &str) -> Vec<Document> {
    let mut reader = BitsReader::new(&(input_path.to_string() + DOCUMENTS_EXTENSION));

    let mut prev: String = "".to_string();
    (0..reader.read_vbyte())
        .map(|_| {
            let p_len = reader.read_gamma();
            let prefix: String = prev.chars().take(p_len as usize).collect();
            let s = prefix + &reader.read_str();
            prev = s.clone();

            Document {
                path: s,
                lenght: reader.read_vbyte(),
            }
        })
        .collect()
}
