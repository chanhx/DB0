use {super::BufferId, lru::LruCache, std::num::NonZeroUsize};

pub struct Replacer(LruCache<BufferId, ()>);

impl Replacer {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap();
        Self(LruCache::new(capacity))
    }
}

impl Replacer {
    pub fn pin(&mut self, buffer_id: BufferId) {
        self.0.pop(&buffer_id);
    }

    pub fn unpin(&mut self, buffer_id: BufferId) {
        self.0.put(buffer_id, ());
    }

    pub fn victim(&mut self) -> Option<BufferId> {
        self.0.pop_lru().map(|(k, _)| k)
    }
}
