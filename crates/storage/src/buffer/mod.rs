mod error;
mod lru_replacer;
mod manager;
mod page;
mod pool;
mod replacer;

pub use self::{
    error::{Error, Result},
    lru_replacer::LruReplacer,
    manager::BufferManager,
    replacer::Replacer,
};

type FrameId = usize;
