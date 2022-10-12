use {
    super::{replacer::Replacer, BufferId},
    lru::LruCache,
    std::num::NonZeroUsize,
};

pub struct LruReplacer(LruCache<BufferId, ()>);

impl LruReplacer {
    pub fn new(cap: NonZeroUsize) -> Self {
        Self(LruCache::new(cap))
    }
}

impl Replacer for LruReplacer {
    fn pin(&mut self, buffer_id: BufferId) {
        self.0.pop(&buffer_id);
    }

    fn unpin(&mut self, buffer_id: BufferId) {
        self.0.put(buffer_id, ());
    }

    fn victim(&mut self) -> Option<BufferId> {
        self.0.pop_lru().map(|(k, _)| k)
    }
}
