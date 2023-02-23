use {
    super::{node::Node, BTree},
    def::storage::{Decoder, Encoder},
    snafu::prelude::*,
    storage::{buffer::BufferManager, PageNum},
};

#[derive(Debug, Snafu)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

pub struct Cursor<'a, C> {
    btree: &'a BTree<C>,
    page_num: PageNum,
    slot_num: usize,
}

impl<'a, C, K> Cursor<'a, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub(super) fn new(btree: &'a BTree<C>, page_num: PageNum, slot_num: usize) -> Self {
        Self {
            btree,
            page_num,
            slot_num,
        }
    }

    pub fn move_forward(&mut self, manager: &BufferManager) -> Result<()> {
        let mut page_ref = self.btree.fetch_page(manager, self.page_num).unwrap();

        let node = Node::new(
            &mut page_ref,
            self.btree.node_capacity,
            &self.btree.key_codec,
        )
        .unwrap();

        match node {
            Node::Leaf(leaf) => {
                if self.slot_num >= leaf.entries_count() {
                    self.page_num = leaf.next_page_num();
                    self.slot_num = 0;
                } else {
                    self.slot_num += 1;
                }
            }
            Node::Branch(_) => unreachable!(),
        }

        Ok(())
    }

    // the cursor visits a leaf page every time it needs an entry
    // it's better to build a scanner, and pass it to the btree
    // leave this problem alone for now
    pub fn get_entry(&self, manager: &BufferManager) -> Option<(K, Vec<u8>)> {
        let mut page_ref = self.btree.fetch_page(manager, self.page_num).unwrap();

        let node = Node::new(
            &mut page_ref,
            self.btree.node_capacity,
            &self.btree.key_codec,
        )
        .unwrap();

        match node {
            Node::Leaf(leaf) => leaf.get_entry(self.slot_num),
            Node::Branch(_) => unreachable!(),
        }
    }
}