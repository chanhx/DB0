#![feature(let_chains)]

pub mod btree;
pub mod codec;
mod slotted_page;

pub use {btree::BTree, codec::Codec};
