use super::{utils, DOCUMENTS_EXTENSION};
use crate::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

#[derive(Clone)]
pub struct Document {
    pub path: String,
    pub length: u32,
}

pub struct Documents {
    docs: Vec<Document>,
    avg_len: f64,
}

impl Documents {
    pub fn load_documents(input_path: &str) -> Documents {
        let mut reader = BitsReader::new(&(input_path.to_string() + DOCUMENTS_EXTENSION));

        let mut prev = String::new();

        let mut length_sum = 0;

        let docs: Vec<Document> = (0..reader.read_vbyte())
            .map(|_| {
                let p_len = reader.read_gamma();
                let prefix: String = prev.chars().take(p_len as usize).collect();
                let path = prefix + &reader.read_str();
                prev = path.clone();

                let length = reader.read_vbyte();
                length_sum += length;

                Document { path, length }
            })
            .collect();

        let avg_len = length_sum as f64 / docs.len() as f64;

        Documents { docs, avg_len }
    }

    pub fn write_documents(documents: &Vec<Document>, output_path: &str) {
        let path = output_path.to_string() + DOCUMENTS_EXTENSION;
        let mut writer = BitsWriter::new(&path);

        let mut prev = "";

        writer.write_vbyte(documents.len() as u32);
        for l in documents {
            let p_len = utils::get_matching_prefix_len(prev, &l.path);
            writer.write_gamma(p_len as u32);
            let remaining: String = l.path.chars().skip(p_len).collect();
            prev = &l.path;

            writer.write_str(&remaining);
            writer.write_vbyte(l.length);
        }

        writer.flush();
    }

    pub fn get_num_documents(&self) -> u32 {
        self.docs.len() as u32
    }

    pub fn get_doc_len(&self, doc_id: u32) -> u32 {
        self.docs[doc_id as usize].length
    }

    pub fn get_avg_doc_len(&self) -> f64 {
        self.avg_len
    }

    pub fn get_doc_path(&self, doc_id: u32) -> String {
        self.docs[doc_id as usize].path.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::utils::create_temporary_file_path;

    use super::*;

    #[test]
    fn test_write_and_load() {
        let dir = create_temporary_file_path("docs_unit");

        let documents = vec![
            Document {
                path: "document1.txt".to_string(),
                length: 100,
            },
            Document {
                path: "document2.txt".to_string(),
                length: 150,
            },
        ];

        Documents::write_documents(&documents, &dir);
        let loaded_documents = Documents::load_documents(&dir);

        assert_eq!(loaded_documents.get_num_documents(), documents.len() as u32);

        for (i, d) in documents.iter().enumerate() {
            assert_eq!(loaded_documents.get_doc_path(i as u32), d.path);
            assert_eq!(loaded_documents.get_doc_len(i as u32), d.length);
        }
    }

    #[test]
    fn test_methods() {
        let documents = vec![
            Document {
                path: "document1.txt".to_string(),
                length: 100,
            },
            Document {
                path: "document2.txt".to_string(),
                length: 150,
            },
        ];

        let doc_collection = Documents {
            docs: documents.clone(),
            avg_len: 125.0,
        };

        assert_eq!(doc_collection.get_num_documents(), documents.len() as u32);

        for (i, d) in documents.iter().enumerate() {
            assert_eq!(doc_collection.get_doc_path(i as u32), d.path);
            assert_eq!(doc_collection.get_doc_len(i as u32), d.length);
        }
    }
}
