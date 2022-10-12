use {
    super::{
        branch::Branch,
        error::{Error, Result},
        leaf::Leaf,
    },
    crate::PageNum,
};

pub(super) enum Node<'a> {
    Branch(Branch<'a>),
    Leaf(Leaf<'a>),
}

impl Node<'_> {
    pub(super) fn new(
        page_num: PageNum,
        page: &mut [u8],
        capacity: usize,
        key_size: usize,
        value_size: usize,
    ) -> Result<Node> {
        Ok(match page[0] {
            2 => Node::Branch(Branch::new(page, capacity, key_size as u16)),
            3 => Node::Leaf(Leaf::new(
                page_num,
                page,
                capacity,
                key_size as u16,
                value_size as u32,
            )),
            ty => return Err(Error::InvalidPageType(ty)),
        })
    }
}

pub enum InsertEffect {
    Split {
        new_key: Vec<u8>,
        splited_high_key: Vec<u8>,
        splited_page_num: PageNum,
    },
    UpdateHighKey(Vec<u8>),
}
