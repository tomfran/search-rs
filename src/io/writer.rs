use std::{
    fs::File,
    io::{BufWriter, Write},
};

const BUFFER_SIZE: u32 = 128;

#[allow(dead_code)]
pub struct Writer {
    file: BufWriter<File>,
    buffer: u128,
    written: u32,
}

#[allow(dead_code)]
impl Writer {
    pub fn new(filename: &str) -> Writer {
        Writer {
            file: BufWriter::new(File::create(filename).expect("Can not create output file")),
            buffer: 0,
            written: 0,
        }
    }

    pub fn write_int(&mut self, n: u32) {
        let free = BUFFER_SIZE - self.written;

        let (gamma, len) = Writer::int_to_gamma(n + 1);
        self.buffer |= (gamma as u128) << self.written;

        if free > len {
            self.written += len;
            return;
        }

        self.update_buffer();
        if len > free {
            self.buffer |= (gamma as u128) >> (len - free);
            self.written += len - free;
        }
    }

    pub fn update_buffer(&mut self) {
        self.file
            .write_all(&self.buffer.to_be_bytes())
            .expect("error while writing buffer to BufWriter");

        self.buffer = 0;
        self.written = 0;
    }

    pub fn flush(&mut self) {
        self.file
            .flush()
            .expect("error while flushing BufWriter buffer");
    }

    pub fn int_to_gamma(n: u32) -> (u64, u32) {
        let msb = 31 - n.leading_zeros();
        let unary: u32 = 1 << msb;
        let gamma: u64 = (((n ^ unary) as u64) << (msb + 1)) | unary as u64;
        (gamma, 2 * msb + 1)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_gamma_coding() {
        let (g, l) = Writer::int_to_gamma(1);
        assert_eq!(format!("{g:b}"), "1");
        assert_eq!(l, 1);

        let (g, l) = Writer::int_to_gamma(7);
        assert_eq!(format!("{g:b}"), "11100");
        assert_eq!(l, 5);
    }
}
