use std::{fs::create_dir_all, path::Path};

use tokenizers::Tokenizer;

pub fn load_tokenizer(filename: &str, force_download: bool) -> Tokenizer {
    let path = Path::new(filename);

    if !path.exists() || force_download {
        path.parent().map(create_dir_all);

        let identifier = path.file_name().unwrap().to_str().unwrap();

        Tokenizer::from_pretrained(identifier, None)
            .expect("error while retrieving tokenizer from the web")
            .save(filename, false)
            .expect("error while saving tokenizer to file");
    }

    Tokenizer::from_file(filename).expect("error while loading tokenizer from file")
}

pub fn tokenize(tokenizer: &Tokenizer, text: &str) -> Vec<String> {
    tokenizer
        .encode(text, false)
        .expect("error while tokenizing text")
        .get_tokens()
        .to_vec()
}
