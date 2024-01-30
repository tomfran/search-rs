# Search <img alt="Rust's Crab" width="25px" src="https://rustacean.net/assets/rustacean-flat-noshadow.png"/>

Search engine written in Rust, based on an inverted index on disk.

## Commands

**Index a new document collection**

```
make cli folder=path/to/folder action=build min_f=1 max_p=0.99
```

The `min_f` param filters terms appearing less that it, while `max_p` filters terms appearing more than 
in `max_p` percentage of the documents.

The folder param is a path to a folder with the following structure: 
```
├── docs
│   ├── 1.txt
│   ├── 2.txt
│   └── 3.txt
└── index
    ├── idx.alphas
    ├── idx.docs
    ├── idx.offsets
    └── idx.postings
```

The index folder will be created after the build command.

**Load a document collection**

You can load a pre-build index by running:

```
make web folder=path/to/folder
```

You can then visit `http://0.0.0.0:3000` to find a web interface to enter free text and boolean queries.

![web.png](misc%2Fweb.png)

**Query Syntax**

You can perform Google-like free test queries, results will 
be ranked via [BM25](https://en.wikipedia.org/wiki/Okapi_BM25) scoring.

You can also specify boolean queries with `"b: "` prefix such as: 
```
b: hello AND there OR NOT man
```

## References
[Introduction to Information Retrieval](https://nlp.stanford.edu/IR-book/information-retrieval-book.html) - Christopher D. Manning, Prabhakar Raghavan and Hinrich Schütze

---

*Feel free to get in touch to discuss the project!*