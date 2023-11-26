use std::fs::File;

const BUFFER_SIZE: usize = 1024;

#[allow(dead_code)]
pub struct Writer {
    file: File,
    buffer: u128,
    free: u16,
    vec_buffer: [u128; BUFFER_SIZE],
}

#[allow(dead_code)]
impl Writer {
    pub fn new(filename: &str) -> Writer {
        let file = File::create(filename).unwrap();
        let buffer = 0;
        let free = 128;
        let vec_buffer: [u128; BUFFER_SIZE] = [0; BUFFER_SIZE];
        Writer {
            file,
            buffer,
            free,
            vec_buffer,
        }
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
