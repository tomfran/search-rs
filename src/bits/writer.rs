use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub struct Writer {
    file: BufWriter<File>,
    buffer: u128,
    written: u32,
}

impl Writer {
    pub fn new(filename: &str) -> Writer {
        Writer {
            file: BufWriter::new(File::create(filename).expect("Can not create output file")),
            buffer: 0,
            written: 0,
        }
    }

    pub fn write_gamma(&mut self, n: u32) -> u64 {
        let (gamma, len) = Writer::int_to_gamma(n + 1);
        self.write_internal(gamma, len)
    }

    fn int_to_gamma(n: u32) -> (u128, u32) {
        let msb = 31 - n.leading_zeros();
        let unary: u32 = 1 << msb;
        let gamma: u128 = (((n ^ unary) as u128) << (msb + 1)) | unary as u128;
        (gamma, 2 * msb + 1)
    }

    pub fn write_vbyte(&mut self, n: u32) -> u64 {
        let (vbyte, len) = Writer::int_to_vbyte(n + 1);
        self.write_internal(vbyte, len)
    }

    fn int_to_vbyte(n: u32) -> (u128, u32) {
        let mut vbyte: u128 = 0;

        let mut n = n;
        let mut byte_num = 0;
        let mask = (1 << 7) - 1;

        while n > 0 {
            vbyte |= ((n & mask) as u128) << (8 * byte_num);
            n >>= 7;
            byte_num += 1;
        }
        vbyte |= 1 << (8 * byte_num - 1);

        (vbyte, 8 * byte_num)
    }

    fn write_internal(&mut self, payload: u128, len: u32) -> u64 {
        let free = 128 - self.written;
        self.buffer |= payload << self.written;

        if free > len {
            self.written += len;
        } else {
            self.update_buffer();
            if len > free {
                self.buffer |= payload >> free;
                self.written += len - free;
            }
        }

        len as u64
    }

    fn update_buffer(&mut self) {
        self.file
            .write_all(&self.buffer.to_le_bytes())
            .expect("error while writing buffer to BufWriter");

        self.buffer = 0;
        self.written = 0;
    }

    pub fn flush(&mut self) {
        if self.written != 0 {
            self.update_buffer();
        }

        self.update_buffer();
        self.file
            .flush()
            .expect("error while flushing BufWriter buffer");
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::fs::create_dir_all;

    #[test]
    fn test_gamma_coding() {
        let (g, l) = Writer::int_to_gamma(1);
        assert_eq!(format!("{g:b}"), "1");
        assert_eq!(l, 1);

        let (g, l) = Writer::int_to_gamma(7);
        assert_eq!(format!("{g:b}"), "11100");
        assert_eq!(l, 5);
    }

    #[test]
    fn test_vbyte_coding() {
        let (vb, l) = Writer::int_to_vbyte(1024);
        assert_eq!(format!("{vb:b}"), "1000100000000000");
        assert_eq!(l, 16);

        let (vb, l) = Writer::int_to_vbyte(1);
        assert_eq!(format!("{vb:b}"), "10000001");
        assert_eq!(l, 8);
    }

    #[test]
    fn test_buffer_overflow() {
        create_dir_all("data/test/").expect("error while creating test dir");

        let word = (1 << 10) - 1;
        let len = 10;

        let mut w = Writer::new("data/test/writer.bin");
        w.written = 125;

        w.write_internal(word, len);

        let b = w.buffer;
        println!("{:b}", b);
        assert_eq!(b, (1 << 7) - 1)
    }
}
