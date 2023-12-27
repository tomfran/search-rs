use std::{fs::create_dir_all, path::Path};

use rust_stemmers::{Algorithm, Stemmer};
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

pub fn load_stemmer() -> Stemmer {
    Stemmer::create(Algorithm::English)
}

pub fn tokenize_and_stem(tokenizer: &Tokenizer, stemmer: &Stemmer, text: &str) -> Vec<String> {
    let tokenized_text = tokenizer
        .encode(text, false)
        .expect("error while tokenizing text");

    tokenized_text
        .get_tokens()
        .iter()
        .map(|t| t.to_lowercase())
        .map(|t| stemmer.stem(&t).to_string())
        .collect()
}
