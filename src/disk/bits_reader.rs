use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

const BUFFER_SIZE: u32 = 128;

pub struct BitsReader {
    file: BufReader<File>,
    buffer: u128,
    byte_buffer: [u8; 16],
    read: u32,
}

impl BitsReader {
    pub fn new(filename: &str) -> BitsReader {
        let mut r = BitsReader {
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
        (self.read_internal(len) as u32 | (1 << len)) - 1
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

    pub fn read_vbyte(&mut self) -> u32 {
        let mut res = 0;

        let mask = (1 << 7) - 1;
        let mut byte_num = 0;

        let mut exit = false;
        while !exit {
            let byte = self.read_internal(8);
            res |= (byte & mask) << (7 * byte_num);

            byte_num += 1;
            exit = byte & (1 << 7) != 0;
        }
        res as u32 - 1
    }

    fn read_internal(&mut self, len: u32) -> u128 {
        let mask = (1 << len) - 1;

        let remaining = BUFFER_SIZE - self.read;

        let mut res = self.buffer & mask;
        self.buffer >>= len;

        if remaining <= len {
            self.fill_buffer();

            let delta = len - remaining;
            res |= self.read_internal(delta) << remaining;

            return res;
        }

        self.read += len;
        res
    }

    fn fill_buffer(&mut self) {
        self.file
            .read_exact(&mut self.byte_buffer)
            .expect("error while filling byte buffer");

        self.buffer = u128::from_le_bytes(self.byte_buffer);
        self.read = 0;
    }

    pub fn seek(&mut self, bit_offset: u64) {
        let byte_seek = bit_offset / 8;
        let remainder_seek = bit_offset % 8;

        self.file
            .seek(SeekFrom::Start(byte_seek))
            .expect("error while seeking reader");

        self.fill_buffer();
        if remainder_seek > 0 {
            self.read_internal(remainder_seek as u32);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::disk::bits_writer::BitsWriter;
    use std::fs::create_dir_all;

    #[test]
    fn test_read() {
        create_dir_all("data/test/").expect("error while creating test dir");

        let mut w = BitsWriter::new("data/test/writer_unit.bin");

        (1..100).for_each(|i| {
            w.write_vbyte(i);
        });

        (1..100).for_each(|i| {
            w.write_gamma(i);
        });

        w.flush();

        let mut r = BitsReader::new("data/test/writer_unit.bin");

        (1..100).for_each(|i| assert_eq!(i, r.read_vbyte()));
        (1..100).for_each(|i| assert_eq!(i, r.read_gamma()));
    }

    #[test]
    fn test_seek() {
        create_dir_all("data/test/").expect("error while creating test dir");

        let mut w = BitsWriter::new("data/test/writer_seek.bin");

        let offset = (0..1000).map(|i| w.write_gamma(i)).sum();
        w.write_gamma(10);

        w.flush();

        let mut r = BitsReader::new("data/test/writer_seek.bin");

        r.seek(offset);
        assert_eq!(r.read_gamma(), 10);
    }
}
