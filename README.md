![web-l.png](misc/web-l.png#gh-light-mode-only)
![web-d.png](misc/web-d.png#gh-dark-mode-only)

# Search-rs
An on-disk Search Engine with boolean and free text queries and spelling correction.

[![CI (main)](https://github.com/tomfran/search-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/tomfran/search-rs/actions/workflows/rust.yml)

**Table of contents**
- [Architecture](#architecture)
  - [Inverted index](#inverted-index)
  - [Vocabulary and Documents](#vocabulary-and-documents)
  - [Query processing](#query-processing)
- [Commands](#commands)
- [References](#references)


## Architecture

Here is an high level overview of the project architecture, you can 
find a more detailed presentation in the following [Medium article](https://medium.com/itnext/building-a-search-engine-in-rust-c945b6e638f8).

### Inverted index

The backbone of the engine is an inverted index. The main 
idea is to have, for each word appearing in the documents, a list
of document IDs. 
This allows us to quickly find documents containing a given word.

More specifically, for each term we save a postings list as follows: 
$$\text{n}\\;|\\;(\text{id}_i, f_i, [p_0, \dots, p_m]), \dots$$

Where $n$ is the number of documents the term appears in, id is the 
doc id, $f$ is the frequency, and $p_j$ are the positions where 
the term appears in the document $i$.

We also store offsets for each term, allowing us to jump to the beginning of the postings list for a given term. They are stored in a separate file.
$$\text{n}\\;|\\;o_0, \dots, o_n$$

Delta encoding is used to represent document IDs, as they are strictly increasing, the same goes for the term positions and offsets. All those integers are written with [Gamma coding](https://en.wikipedia.org/wiki/Elias_gamma_coding). 
Generic integers, such as list lengths are written in [VByte encoding](https://nlp.stanford.edu/IR-book/html/htmledition/variable-byte-codes-1.html#:~:text=Variable%20byte%20(VB)%20encoding%20uses,gap%20and%20to%200%20otherwise.).

### Vocabulary and Documents

The vocabulary is written on disk using prefix compression. 
The idea is to sort terms and then write them as "matching prefix length", and suffix.

Here is an example with three words: 
$$\text{watermelon}\\;\text{waterfall}\\;\text{waterfront}$$
$$0\\;\text{watermelon}\\;5\\;\text{fall}\\;6\\;\text{ront}$$

Spelling correction is used before answering queries. Given a 
word $w$, we use a trigram index to find terms in the vocabulary 
which shares at least a trigram with it. 
We then select the one with the lowest [Levenshtein Distance](https://en.wikipedia.org/wiki/Levenshtein_distance) and max frequency. 

$$
\text{lev}(a, b) = \begin{cases}
    |a| & \text{if}\\;|b| = 0, \\
    |b| & \text{if}\\;|a| = 0, \\
    1 + \text{min} \begin{cases}
        \text{lev}(\text{tail}(a), b) \\
        \text{lev}(a, \text{tail}(b)) \\
        \text{lev}(\text{tail}(a), \text{tail}(b)) \\
    \end{cases} & \text{otherwise} \\
\end{cases}
$$

Finally, document paths and lenghts are stored with a similar format.
$$\text{n}\\;|\\;p_0, l_0, \dots, p_n, l_n$$

### Query processing

You can query the index with boolean or free test queries. In the first case you can use the usual boolean operators to compose a query, such as: 
$$\text{gun}\\;\text{AND}\\;\text{control}$$

In the second case, you just enter a phrase and receive a ranked collection of documents matching the query, ordered by [BM25 score](https://en.wikipedia.org/wiki/Okapi_BM25). 

$$\text{BM25}(D, Q) = \sum_{i = 1}^{n} \\; \text{IDF}(q_i) \cdot \frac{f(q_i, D) \cdot (k_1 + 1)}{f(q_i, D) + k_1 \cdot \Big (1 - b + b \cdot \frac{|D|}{\text{avgdl}} \Big )}$$

$$\text{IDF}(q_i) = \ln \Bigg ( \frac{N - n(q_i) + 0.5}{n(q_i) + 0.5} + 1 \Bigg )$$

A window score is also computed, as the cardinality of 
the user query, divided by the minimum size windows where all query terms appears in a document, or plus infinity if they don't appear all togheter.

$$\text{window}(D, Q) = \frac{|Q|}{\text{min. window}(Q, D)}$$

Finally they are combined with the following formula: 

$$\text{score}(D, Q) = \alpha \cdot \text{window}(D, Q) + \beta \cdot \text{BM25}(D, Q)$$


## Commands

**Index a new document collection**

```
make cli folder=path/to/folder action=build min_f=1 max_p=0.99
```

The `min_f` param filters terms appearing less that it, while `max_p` filters terms appearing more than 
in `max_p` percentage of the documents.

The folder param is a path to a folder containing the documents to index. 
The index files will be placed inside a subfolder, `.index`.

Here is an example of such structure:
```
example
├── .index
│   ├── idx.alphas
│   ├── idx.docs
│   ├── idx.offsets
│   └── idx.postings
├── 1.txt
├── 2.txt
├── 3.txt
└── subfolder
    ├── 1.txt
    ├── 2.txt
    └── 3.txt
```

The builder will walk recursively down the input folder, skipping hidden ones.
The indexer will skip and show an error for non UTF-8 files.

**Load a document collection**

You can load a pre-build index by running:

```
make web folder=path/to/folder
```

This will load the index inside `path/to/folder/.index`

You can then visit `http://0.0.0.0:3000` to find a web interface to enter free text and boolean queries.

**Query Syntax**

You can perform Google-like free test queries.

You can also specify boolean queries with `"b: "` prefix such as: 
```
b: hello AND there OR NOT man
```

## References
[Introduction to Information Retrieval](https://nlp.stanford.edu/IR-book/information-retrieval-book.html) - Christopher D. Manning, Prabhakar Raghavan and Hinrich Schütze

*Feel free to get in touch to discuss the project!*