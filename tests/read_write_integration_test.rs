use rand::Rng;
use search::disk::{bits_reader::BitsReader, bits_writer::BitsWriter};

#[test]
fn test_read_write() {
    let path = "data/test/writer_io_integration.bin";

    let n = 100_000;
    let mut rng = rand::thread_rng();

    let values: Vec<u32> = (0..n).map(|_| rng.gen_range(0..u32::MAX - 1)).collect();
    let mut coding: Vec<u32> = (0..n).map(|_| rng.gen()).collect();

    let mut writer = BitsWriter::new(path);

    writer.write_vbyte(n);

    values.iter().zip(coding.iter_mut()).for_each(|(v, c)| {
        if *c % 2 == 0 {
            writer.write_vbyte(*v);
        } else {
            writer.write_gamma(*v);
        }
    });
    writer.flush();

    let mut reader = BitsReader::new(path);
    assert_eq!(n, reader.read_vbyte());

    values.iter().zip(coding.iter_mut()).for_each(|(v, c)| {
        let r = if *c % 2 == 0 {
            reader.read_vbyte()
        } else {
            reader.read_gamma()
        };
        assert_eq!(r, *v)
    });
}
