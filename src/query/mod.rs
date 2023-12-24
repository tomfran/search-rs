use crate::index::Index;

struct QueryProcessor {
    index: Index,
}

impl QueryProcessor {
    pub fn build_query_processor(
        index_input_path: &str,
        index_tokenizer_path: &str,
    ) -> QueryProcessor {
        QueryProcessor {
            index: Index::load_index(index_input_path, index_tokenizer_path),
        }
    }

    pub fn query(query: &str) -> Vec<u32> {
        todo!()
    }
}
