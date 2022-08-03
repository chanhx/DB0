use {
    super::{replacer::Replacer, FrameId},
    lru::LruCache,
};

pub struct LruReplacer(LruCache<FrameId, ()>);

impl LruReplacer {
    pub fn new(cap: usize) -> Self {
        Self(LruCache::new(cap))
    }
}

impl Replacer for LruReplacer {
    fn pin(&mut self, frame_id: FrameId) {
        self.0.pop(&frame_id);
    }

    fn unpin(&mut self, frame_id: FrameId) {
        self.0.put(frame_id, ());
    }

    fn victim(&mut self) -> Option<FrameId> {
        self.0.pop_lru().map(|(k, _)| k)
    }
}
