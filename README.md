# Search <img alt="Rust's Crab" width="25px" src="https://rustacean.net/assets/rustacean-flat-noshadow.png"/>

Search engine written in Rust, based on an inverted index on disk.

## Implementation status 

**IO**
- [x] Classes for writing and reading bit-streams;
- [ ] Proper strings writer and reader.

**Text preprocessing** 
- [x] Tokenization;
- [x] Stemming;
- [ ] Parametrization at build time.

**Index construction**
- [x] In-memory datasets index construction;
- [ ] Proper vocabulary and paths on disk;
- [ ] Spelling correction index;
- [ ] Disk-based partial index construction and merging;

**Queries**
- [x] Tf-idf ranked retrieval;
- [x] Window computation.
- [ ] Boolean queries;
- [ ] Parallel scoring.

**Evaluation**
- [ ] Query speed;
- [ ] Query quality; 
- [ ] Disk overhead.

## Crates in use
- [stemmer-rs](https://github.com/lise-henry/stemmer-rs)
- [tokenizers](https://github.com/huggingface/tokenizers)
- [indicatif](https://github.com/console-rs/indicatif)
- [fxhash](https://github.com/cbreeden/fxhash)

## References
[Introduction to Information Retrieval](https://nlp.stanford.edu/IR-book/information-retrieval-book.html) - Christopher D. Manning, Prabhakar Raghavan and Hinrich Schütze

---

*Feel free to get in touch to discuss the project!*