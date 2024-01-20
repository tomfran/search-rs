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

    pub fn read_vbyte_gamma_gap_vector(&mut self) -> Vec<u32> {
        let mut prefix = 0;
        (0..self.read_vbyte())
            .map(|_| {
                let gap = self.read_gamma();
                prefix += gap;
                prefix
            })
            .collect()
    }

    pub fn read_str(&mut self) -> String {
        String::from_utf8(
            (0..self.read_gamma())
                .map(|_| self.read_internal(8) as u8)
                .collect(),
        )
        .unwrap()
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
mod tests {
    use super::*;
    use crate::{disk::bits_writer::BitsWriter, test_utils::utils::create_temporary_file_path};

    #[test]
    fn test_read() {
        let test_output_path = create_temporary_file_path("test_read");

        let mut w = BitsWriter::new(&test_output_path);

        (1..100).for_each(|i| {
            w.write_vbyte(i);
        });

        (1..100).for_each(|i| {
            w.write_gamma(i);
        });

        w.write_vbyte(3);
        (1..4).for_each(|i| {
            w.write_gamma(i);
        });

        w.write_str("hello");
        w.write_str("");

        w.flush();

        let mut r = BitsReader::new(&test_output_path);

        (1..100).for_each(|i| assert_eq!(i, r.read_vbyte()));
        (1..100).for_each(|i| assert_eq!(i, r.read_gamma()));

        assert_eq!(r.read_vbyte_gamma_gap_vector(), [1, 3, 6]);

        assert_eq!(r.read_str(), "hello");
        assert_eq!(r.read_str(), "");
    }

    #[test]
    fn test_seek() {
        let test_output_path = create_temporary_file_path("test_seek");

        let mut w = BitsWriter::new(&test_output_path);

        let offset = (0..1000).map(|i| w.write_gamma(i)).sum();
        w.write_gamma(10);

        w.flush();

        let mut r = BitsReader::new(&test_output_path);

        r.seek(offset);
        assert_eq!(r.read_gamma(), 10);
    }
}
