#![feature(let_chains)]

pub mod btree;
pub mod buffer;
mod codec;
mod manager;
mod slotted_page;

pub const DEFAULT_PAGE_SIZE: usize = 1 << 12;
