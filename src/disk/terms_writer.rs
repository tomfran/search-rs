use std::{
    fs::File,
    io::{BufWriter, Write},
};

use super::file_utils;

pub struct TermsWriter {
    file: BufWriter<File>,
}

impl TermsWriter {
    pub fn new(path: &str) -> TermsWriter {
        TermsWriter {
            file: BufWriter::new(file_utils::create_and_open_file(path)),
        }
    }

    pub fn write_term(&mut self, term: &str) {
        self.file
            .write_all(term.as_bytes())
            .expect("error while writing term to file");
    }

    pub fn flush(&mut self) {
        self.file
            .flush()
            .expect("error while flushing BufWriter buffer");
    }
}
