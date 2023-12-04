use std::{
    fs::File,
    io::{BufReader, Read},
};

#[allow(dead_code)]
const BUFFER_SIZE: u32 = 128;

#[allow(dead_code)]
pub struct Reader {
    file: BufReader<File>,
    buffer: u128,
    byte_buffer: [u8; 16],
    read: u32,
}

#[allow(dead_code)]
impl Reader {
    pub fn new(filename: &str) -> Reader {
        let mut r = Reader {
            file: BufReader::new(File::open(filename).expect("Can not create output file")),
            buffer: 0,
            byte_buffer: [0; 16],
            read: 0,
        };
        r.update_buffer();
        r
    }

    pub fn read_gamma(&mut self) -> u32 {
        let len = self.read_unary() - 1;
        (self.read_len(len) | (1 << len)) - 1
    }

    fn read_unary(&mut self) -> u32 {
        let zeros = self.buffer.trailing_zeros();

        self.buffer >>= zeros + 1;
        self.read += zeros + 1;

        zeros + 1
    }

    fn read_len(&mut self, len: u32) -> u32 {
        let mask = (1 << len) - 1;

        let res = self.buffer & mask;
        self.buffer >>= len;
        self.read += len;

        res as u32
    }

    fn update_buffer(&mut self) {
        self.file
            .read_exact(&mut self.byte_buffer)
            .expect("erorr while filling byte buffer");

        self.buffer = u128::from_be_bytes(self.byte_buffer);
    }
}

#[cfg(test)]
mod test {

    use std::fs::create_dir_all;

    use super::*;
    use crate::io::writer::Writer;

    #[test]
    fn test_read_gamma() {
        create_dir_all("data/test/").expect("error while creating test dir");

        let mut w = Writer::new("data/test/writer.bin");
        for i in 1..5 {
            w.write_gamma(i);
        }
        w.flush();

        let mut r = Reader::new("data/test/writer.bin");

        for i in 1..5 {
            assert_eq!(i, r.read_gamma());
        }
    }
}
