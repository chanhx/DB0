use {
    super::{node::Node, BTree},
    def::storage::{Decoder, Encoder},
    snafu::prelude::*,
    storage::PageNum,
};

#[derive(Debug, Snafu)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

pub struct Cursor<'a, 'b, C> {
    btree: &'b BTree<'a, C>,
    page_num: PageNum,
    slot_num: usize,
}

impl<'a, 'b, C, K> Cursor<'a, 'b, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub(super) fn new(btree: &'b BTree<'a, C>, page_num: PageNum, slot_num: usize) -> Self {
        Self {
            btree,
            page_num,
            slot_num,
        }
    }
}

impl<C, K> Iterator for Cursor<'_, '_, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    type Item = (K, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.page_num == 0 {
            return None;
        }

        let mut page_ref = self.btree.fetch_page(self.page_num).unwrap();

        let node = Node::new(
            &mut page_ref,
            self.btree.node_capacity,
            &self.btree.key_codec,
        )
        .unwrap();

        match node {
            Node::Leaf(leaf) => {
                let entry = leaf.get_entry(self.slot_num);
                self.slot_num += 1;

                if self.slot_num >= leaf.entries_count() {
                    self.page_num = leaf.next_page_num();
                    self.slot_num = 0;
                }

                entry
            }
            Node::Branch(_) => unreachable!(),
        }
    }
}
