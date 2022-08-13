pub mod btree;
pub mod buffer;
pub mod file;

pub(crate) type PageId = u64;
pub(crate) const PAGE_SIZE: usize = 1 << 12;
