#![feature(let_chains)]

pub mod buffer;
mod manager;

pub type PageNum = u32;

pub const DEFAULT_PAGE_SIZE: usize = 1 << 12;
