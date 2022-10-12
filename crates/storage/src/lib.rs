pub mod btree;
pub mod buffer;
pub mod file;
mod slotted_page;

pub(crate) type PageNum = u32;
pub(crate) const PAGE_SIZE: usize = 1 << 12;
