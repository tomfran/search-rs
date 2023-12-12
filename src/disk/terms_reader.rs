use std::{
    fs::File,
    io::{BufReader, Read},
};

pub struct TermsReader {
    file: BufReader<File>,
}

impl TermsReader {
    pub fn new(filename: &str) -> TermsReader {
        TermsReader {
            file: BufReader::new(File::open(filename).expect("can not open input file")),
        }
    }

    pub fn read_to_string(&mut self) -> String {
        let mut buffer = String::new();
        self.file
            .read_to_string(&mut buffer)
            .expect("error while reading to string");
        buffer
    }
}
