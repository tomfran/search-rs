use std::num::NonZeroUsize;

use lru::LruCache;

use super::postings::{PostingsList, POSTINGS_CACHE_CAPACITY};

pub struct PostingsCache {
    cache: LruCache<usize, PostingsList>,
}

impl PostingsCache {
    pub fn new() -> PostingsCache {
        PostingsCache {
            cache: LruCache::new(NonZeroUsize::new(POSTINGS_CACHE_CAPACITY).unwrap()),
        }
    }

    pub fn get(&mut self, key: usize) -> Option<&PostingsList> {
        self.cache.get(&key)
    }

    pub fn put(&mut self, key: usize, value: PostingsList) -> Option<PostingsList> {
        self.cache.put(key, value)
    }
}
