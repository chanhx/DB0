mod branch;
mod cursor;
pub mod error;
mod leaf;
mod meta;
mod node;

#[cfg(test)]
mod tests;

pub use cursor::Cursor;
use {
    self::{
        branch::Branch,
        leaf::Leaf,
        meta::Meta,
        node::{InsertEffect, Node},
    },
    def::storage::{Decoder, Encoder},
    error::Result,
    snafu::ResultExt,
    std::collections::VecDeque,
    storage::{
        buffer::{BufferManager, BufferRef, FileNode, PageTag},
        PageNum, DEFAULT_PAGE_SIZE,
    },
};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PageType {
    Deleted = 1,
    Meta,
    Branch,
    Leaf,
}

const META_PAGE_NUM: PageNum = 0;

pub struct BTree<'a, C> {
    key_codec: C,
    node_capacity: usize,
    file_node: FileNode,

    manager: &'a BufferManager,
}

impl<'a, C, K> BTree<'a, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub fn new(
        key_codec: C,
        max_value_size: usize,
        // page_size: usize,
        file_node: FileNode,
        manager: &'a BufferManager,
    ) -> Self {
        let max_key_size = key_codec.max_size();
        let max_entry_size = max_key_size + max_value_size;

        let leaf_capacity = Leaf::<C>::capacity(DEFAULT_PAGE_SIZE, max_entry_size);
        let branch_capacity = Branch::<C>::capacity(DEFAULT_PAGE_SIZE, max_key_size);

        let node_capacity = leaf_capacity.min(branch_capacity);

        Self {
            key_codec,
            node_capacity,
            file_node,
            manager,
        }
    }

    pub fn init(file_node: FileNode, manager: &BufferManager) -> Result<()> {
        let mut meta_page_ref = manager.new_page(&file_node).context(error::BufferSnafu)?;
        let meta = Meta::from_bytes_mut(meta_page_ref.as_slice_mut());
        meta.root = 0;

        meta_page_ref.set_dirty();

        Ok(())
    }

    fn create_root_page(&mut self) -> Result<PageNum> {
        let mut root_page_ref = self
            .manager
            .new_page(&self.file_node)
            .context(error::BufferSnafu)?;

        let mut root = Leaf::new(&mut root_page_ref, self.node_capacity, &self.key_codec);
        root.init(0, 0);
        root_page_ref.set_dirty();

        let page_num = root_page_ref.page_num();

        let mut meta_page_ref = self.fetch_page(META_PAGE_NUM)?;
        let meta = Meta::from_bytes_mut(meta_page_ref.as_slice_mut());
        meta.init();
        meta.root = page_num;
        meta.level = 1;
        meta_page_ref.set_dirty();

        Ok(page_num)
    }

    fn root_page_num(&self) -> Result<PageNum> {
        let mut meta_page_ref = self.fetch_page(META_PAGE_NUM)?;
        Ok(Meta::from_bytes(meta_page_ref.as_slice_mut()).root)
    }

    pub fn insert(&mut self, key: &K, value: &[u8]) -> Result<()> {
        let mut page_num = self.root_page_num()?;
        if page_num == 0 {
            page_num = self.create_root_page()?;
        }

        let mut insert_effect = None;
        let (mut stack, _) = self.search(key)?;

        while let Some(node) = stack.pop_back() {
            let StackNode { page_num, slot_num } = node;
            let mut page_ref = self.fetch_page(page_num)?;
            let node = Node::new(&mut page_ref, self.node_capacity, &self.key_codec)?;

            insert_effect = match (node, insert_effect.take()) {
                (Node::Leaf(mut leaf), _) => leaf
                    .insert(key, value, self.manager, &self.file_node)
                    .unwrap(),

                (Node::Branch(mut branch), Some(InsertEffect::UpdateHighKey(high_key))) => {
                    if branch.is_right_most_slot(slot_num) {
                        branch.update_high_key(&high_key);
                    }
                    None
                }

                (
                    Node::Branch(mut branch),
                    Some(InsertEffect::Split {
                        raw_new_key,
                        raw_high_key,
                        splited_page_num,
                    }),
                ) => branch
                    .insert(
                        &raw_new_key,
                        splited_page_num,
                        slot_num,
                        raw_high_key,
                        self.manager,
                        &self.file_node,
                    )
                    .unwrap(),
                (Node::Branch(_), None) => break,
            };

            match insert_effect {
                Some(_) => page_ref.set_dirty(),
                None => return Ok(()),
            };
        }

        // split the root
        if stack.is_empty() && let Some(InsertEffect::Split {
            raw_new_key,
            raw_high_key,
            splited_page_num,
        }) = insert_effect
        {
            let mut new_root_page = self
                .manager
                .new_page(&self.file_node)
                .context(error::BufferSnafu)?;
            let mut new_root = Branch::new(
                new_root_page.as_slice_mut(),
                self.node_capacity,
                &self.key_codec,
            );
            new_root.init(&raw_new_key, &raw_high_key, page_num, splited_page_num, 0);
            new_root_page.set_dirty();

            let mut meta_page = self.fetch_page(META_PAGE_NUM)?;
            let meta = Meta::from_bytes_mut(meta_page.as_slice_mut());
            meta.root = new_root_page.page_num();
            meta.level += 1;
            meta_page.set_dirty();
        }

        Ok(())
    }

    fn search<'b, 'c>(&'b self, key: &'c K) -> Result<(VecDeque<StackNode>, bool)> {
        let (mut page_num, level) = {
            let meta_page = self.fetch_page(META_PAGE_NUM)?;
            let meta = Meta::from_bytes(meta_page.as_slice());

            (meta.root, meta.level as usize)
        };

        let mut stack = VecDeque::with_capacity(level);
        let mut is_matched = false;

        for i in 0..level {
            let mut page_ref = self.fetch_page(page_num)?;

            let node = Node::new(&mut page_ref, self.node_capacity, &self.key_codec)?;
            let node = match node {
                Node::Branch(_) if i == level - 1 => Err(error::InvalidTreeStructSnafu.build())?,
                Node::Leaf(_) if i < level - 1 => Err(error::InvalidTreeStructSnafu.build())?,

                Node::Branch(branch) => {
                    let (slot_num, child) = branch.search(key);
                    let node = StackNode { page_num, slot_num };
                    page_num = child;
                    node
                }
                Node::Leaf(leaf) => {
                    let (slot_num, matched) = match leaf.search(key) {
                        Ok(i) => (i, true),
                        Err(i) => (i, false),
                    };

                    is_matched = matched;

                    StackNode { page_num, slot_num }
                }
            };

            stack.push_back(node);
        }

        Ok((stack, is_matched))
    }

    pub fn cursor<'b, 'c>(&'b self, key: &'c K) -> Result<Option<(Cursor<'a, 'b, C>, bool)>> {
        let (mut stack, is_matched) = self.search(key)?;

        match stack.pop_back() {
            Some(StackNode { page_num, slot_num }) => {
                Ok(Some((Cursor::new(self, page_num, slot_num), is_matched)))
            }
            _ => Err(error::InvalidTreeStructSnafu.build()),
        }
    }

    // pub fn delete(&mut self, key: &K, manager: &BufferManager) -> Result<usize> {
    //     unimplemented!()
    // }

    fn fetch_page(&self, page_num: PageNum) -> Result<BufferRef<'a>> {
        let page_tag = PageTag {
            file_node: self.file_node,
            page_num,
        };

        self.manager
            .fetch_page(page_tag)
            .context(error::BufferSnafu)
    }
}

struct StackNode {
    page_num: PageNum,
    slot_num: usize,
}
