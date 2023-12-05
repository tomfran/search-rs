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
            file: BufReader::new(File::open(filename).expect("can not open input file")),
            buffer: 0,
            byte_buffer: [0; 16],
            read: 0,
        };
        r.fill_buffer();
        r
    }

    pub fn read_gamma(&mut self) -> u32 {
        let len = self.read_unary() - 1;
        (self.read_len(len) as u32 | (1 << len)) - 1
    }

    fn read_unary(&mut self) -> u32 {
        let remaining = BUFFER_SIZE - self.read;

        let zeros = self.buffer.trailing_zeros();

        if zeros >= remaining {
            self.fill_buffer();
            return remaining + self.read_unary();
        }

        self.buffer >>= zeros + 1;
        self.read += zeros + 1;

        zeros + 1
    }

    fn read_len(&mut self, len: u32) -> u128 {
        let mask = (1 << len) - 1;

        let remaining = BUFFER_SIZE - self.read;

        let mut res = self.buffer & mask;
        self.buffer >>= len;

        if remaining <= len {
            self.fill_buffer();

            let delta = len - remaining;
            res |= self.read_len(delta) << remaining;

            return res;
        }

        self.read += len;
        res
    }

    fn fill_buffer(&mut self) {
        self.file
            .read_exact(&mut self.byte_buffer)
            .expect("error while filling byte buffer");

        self.buffer = u128::from_be_bytes(self.byte_buffer);
        self.read = 0;
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
        for i in 1..100 {
            w.write_gamma(i);
        }
        w.flush();

        let mut r = Reader::new("data/test/writer.bin");

        for i in 1..100 {
            let a = r.read_gamma();
            assert_eq!(i, a);
        }
    }
}
