mod branch;
mod error;
mod leaf;
mod meta;

use crate::PageId;

pub(super) use self::{branch::Branch, leaf::Leaf, meta::Meta};

pub use error::{Error, Result};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub(super) enum PageType {
    Meta = 1,
    Branch = 2,
    Leaf = 3,
}

pub(super) enum Node<'a> {
    Branch(Branch<'a>),
    Leaf(Leaf<'a>),
}

impl Node<'_> {
    pub(super) fn new(
        page_id: PageId,
        page: &mut [u8],
        capacity: usize,
        key_size: usize,
        value_size: usize,
    ) -> Result<Node> {
        Ok(match page[0] {
            2 => Node::Branch(Branch::new(page, capacity, key_size as u16)),
            3 => Node::Leaf(Leaf::new(
                page_id,
                page,
                capacity,
                key_size as u16,
                value_size as u32,
            )),
            ty => return Err(Error::InvalidPageType(ty)),
        })
    }

    pub(super) fn high_key(&self) -> Option<&[u8]> {
        Some(match self {
            Self::Branch(branch) => branch.high_key(),
            Self::Leaf(leaf) => leaf.high_key()?,
        })
    }
}

pub enum InsertEffect {
    Split {
        new_key: Vec<u8>,
        splited_high_key: Vec<u8>,
        splited_page_id: PageId,
    },
    UpdateHighKey(Vec<u8>),
}
