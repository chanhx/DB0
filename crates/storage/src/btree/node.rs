use {
    super::{
        branch::Branch,
        error::{self, Result},
        leaf::Leaf,
        PageNum, PageType,
    },
    crate::buffer::BufferRef,
    def::storage::{Decoder, Encoder},
};

pub(super) enum Node<'a, 'b, C> {
    Branch(Branch<'a, 'b, C>),
    Leaf(Leaf<'a, 'b, C>),
}

impl<'a, 'b, C, K> Node<'a, 'b, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub(super) fn new(page_ref: &'a mut BufferRef, capacity: usize, codec: &'b C) -> Result<Self> {
        Ok(match page_ref.as_slice()[0] {
            ty if ty == PageType::Branch as u8 => {
                Node::Branch(Branch::new(page_ref.as_slice_mut(), capacity, codec))
            }
            ty if ty == PageType::Leaf as u8 => Node::Leaf(Leaf::new(page_ref, capacity, codec)),
            ty => return Err(error::InvalidPageTypeSnafu { page_type: ty }.build()),
        })
    }
}

pub enum InsertEffect {
    Split {
        raw_new_key: Vec<u8>,
        raw_high_key: Vec<u8>,
        splited_page_num: PageNum,
    },
    UpdateHighKey(Vec<u8>),
}
