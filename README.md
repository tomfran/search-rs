# Search <img alt="Rust's Crab" width="25px" src="https://rustacean.net/assets/rustacean-flat-noshadow.png"/>

Search engine written in Rust, based on an inverted index on disk.

## Implementation status 

**IO**
- [x] Classes for writing and reading bit-streams;
- [x] Proper strings writer and reader.

**Text preprocessing** 
- [x] Tokenization;
- [x] Stemming;
- [ ] Parametrization at build time.

**Index construction**
- [x] In-memory datasets index construction;
- [x] Proper vocabulary and paths on disk;
- [x] Spelling correction index;.

**Queries**
- [x] BM25 scoring;
- [x] Window computation;

**Evaluation**
- [ ] Query speed;
- [ ] Query quality; 
- [ ] Disk overhead.

**Client**
- [x] CLI;
- [x] Web interface.

## References
[Introduction to Information Retrieval](https://nlp.stanford.edu/IR-book/information-retrieval-book.html) - Christopher D. Manning, Prabhakar Raghavan and Hinrich Schütze

---

*Feel free to get in touch to discuss the project!*