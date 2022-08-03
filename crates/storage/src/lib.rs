pub mod btree;
pub mod buffer;
pub mod disk;

const PAGE_SIZE: usize = 4096;
type PageId = u64;
