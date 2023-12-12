use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub struct TermsWriter {
    file: BufWriter<File>,
}

impl TermsWriter {
    pub fn new(filename: &str) -> TermsWriter {
        TermsWriter {
            file: BufWriter::new(File::create(filename).expect("Can not create output file")),
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
