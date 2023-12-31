# Search <img alt="Rust's Crab" width="25px" src="https://rustacean.net/assets/rustacean-flat-noshadow.png"/>

Search engine written in Rust, based on an inverted index on disk.

### Implementation status 
- [x] IO classes for writing and reading bit-streams;
- [x] Text preprocessing: 
  - [x] Tokenization;
  - [x] Stemming.
- [ ] Index construction:
  - [x] In-memory datasets index construction;
  - [ ] Disk-based partial index construction and merging;
  - [ ] Additional indexes to support things such as spelling correction.
- [ ] Index queries:
  - [ ] Boolean queries;
  - [x] Tf-idf ranked retrieval;
  - [x] Window computation.

### Crates in use
- [stemmer-rs](https://github.com/lise-henry/stemmer-rs)
- [tokenizers](https://github.com/huggingface/tokenizers)
- [indicatif](https://github.com/console-rs/indicatif)


### References
[Introduction to Information Retrieval - Christopher D. Manning, Prabhakar Raghavan and Hinrich Schütze](https://nlp.stanford.edu/IR-book/information-retrieval-book.html)