mod branch;
mod cursor;
pub mod error;
mod leaf;
mod meta;
mod node;

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
    storage::{
        buffer::{BufferManager, BufferRef, FileNode, PageTag},
        PageNum,
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
        node_capacity: u32,
        file_node: FileNode,
        manager: &'a BufferManager,
    ) -> Self {
        Self {
            key_codec,
            node_capacity: node_capacity as usize,
            file_node,
            manager,
        }
    }

    pub fn init(file_node: FileNode, manager: &BufferManager) -> Result<()> {
        let mut meta_page_ref = manager.new_page(&file_node).context(error::BufferSnafu)?;
        let meta = Meta::new(meta_page_ref.as_slice_mut());
        meta.root = 0;

        meta_page_ref.set_dirty();

        Ok(())
    }

    fn create_root_page(&mut self) -> Result<PageNum> {
        let mut root_page_ref = self
            .manager
            .new_page(&self.file_node)
            .context(error::BufferSnafu)?;

        let mut root = Leaf::new(
            &mut root_page_ref,
            self.node_capacity as usize,
            &self.key_codec,
        );
        root.init(0, 0);
        root_page_ref.set_dirty();

        let page_num = root_page_ref.page_num();

        let mut meta_page_ref = self.fetch_page(META_PAGE_NUM)?;
        let meta = Meta::new(meta_page_ref.as_slice_mut());
        meta.init(self.node_capacity as u32);
        meta.root = page_num;

        Ok(page_num)
    }

    fn root_page_num(&self) -> Result<PageNum> {
        let mut meta_page_ref = self.fetch_page(META_PAGE_NUM)?;
        Ok(Meta::new(meta_page_ref.as_slice_mut()).root)
    }

    pub fn insert(&mut self, key: &K, value: &[u8]) -> Result<()> {
        let mut page_num = self.root_page_num()?;
        if page_num == 0 {
            page_num = self.create_root_page()?;
        }

        let mut insert_effect;
        let mut stack = None;

        loop {
            let mut page_ref = self.fetch_page(page_num)?;

            let node = Node::new(&mut page_ref, self.node_capacity, &self.key_codec)?;

            match node {
                Node::Branch(branch) => {
                    let (slot_num, child_page_num) = branch.find_child(key);

                    stack = Some(Box::new(StackNode {
                        page_num,
                        slot_num,
                        parent: stack,
                    }));

                    page_num = child_page_num;
                }
                Node::Leaf(mut leaf) => {
                    if let Some(effect) = leaf
                        .insert(key, value, self.manager, &self.file_node)
                        .unwrap()
                    {
                        page_ref.set_dirty();
                        insert_effect = effect;
                        break;
                    }

                    return Ok(());
                }
            }
        }

        loop {
            if let Some(stk) = stack {
                let StackNode {
                    slot_num,
                    page_num,
                    parent,
                } = *stk;
                stack = parent;

                let mut page_ref = self.fetch_page(page_num)?;

                let mut branch =
                    Branch::new(page_ref.as_slice_mut(), self.node_capacity, &self.key_codec);

                match insert_effect {
                    InsertEffect::UpdateHighKey(high_key) => {
                        if branch.is_right_most_slot(slot_num) {
                            branch.update_high_key(&high_key);
                        }
                        break;
                    }
                    InsertEffect::Split {
                        raw_new_key,
                        raw_high_key,
                        splited_page_num,
                    } => {
                        if let Some(effect) = branch
                            .insert(
                                &raw_new_key,
                                splited_page_num,
                                slot_num,
                                raw_high_key,
                                self.manager,
                                &self.file_node,
                            )
                            .unwrap()
                        {
                            page_ref.set_dirty();
                            insert_effect = effect
                        } else {
                            break;
                        }
                    }
                }
            } else {
                match insert_effect {
                    InsertEffect::Split {
                        raw_new_key,
                        raw_high_key,
                        splited_page_num,
                    } => {
                        let mut meta_page = self.fetch_page(META_PAGE_NUM)?;
                        let meta = Meta::new(meta_page.as_slice_mut());

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

                        meta.root = new_root_page.page_num();
                        meta_page.set_dirty();
                        new_root_page.set_dirty();
                    }
                    _ => {}
                }
                break;
            }
        }

        Ok(())
    }

    pub fn search<'b, 'c>(&'b self, key: &'c K) -> Result<Option<(Cursor<'a, 'b, C>, bool)>> {
        let mut page_num = self.root_page_num()?;

        loop {
            let mut page_ref = self.fetch_page(page_num)?;

            let node = Node::new(&mut page_ref, self.node_capacity, &self.key_codec)?;

            match node {
                Node::Branch(branch) => match branch.retrieve(key) {
                    Some(num) => page_num = num,
                    None => return Ok(None),
                },
                Node::Leaf(leaf) => {
                    let (index, is_matched) = match leaf.search(key) {
                        Ok(i) => (i, true),
                        Err(i) => (i, false),
                    };

                    return Ok(Some((Cursor::new(self, page_num, index), is_matched)));
                }
            }
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
    slot_num: usize,
    page_num: PageNum,
    parent: Option<Box<StackNode>>,
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::codec::Codec,
        def::{meta::Column, SqlType, Value},
        rand::prelude::*,
        storage::DEFAULT_PAGE_SIZE,
        tempfile::tempdir,
    };

    #[test]
    fn sequential_insertion() -> Result<()> {
        let dir = tempdir().unwrap();

        let attr = Column::new(1, 1, "abc".to_string(), SqlType::TinyUint, 4, false);
        let codec = Codec::new(vec![attr]);

        let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
        let file_node = FileNode::new(1, 2, 3);

        BTree::<Codec>::init(file_node, &manager)?;
        let mut btree = BTree::new(codec, 30, file_node, &manager);

        let range = 0..120;

        for i in range.clone() {
            btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5])?;
        }

        for i in range {
            let (mut cursor, is_matched) =
                btree.search(&vec![Value::TinyUint(i)]).unwrap().unwrap();

            let (_, value) = cursor.next().unwrap();

            assert!(is_matched);
            assert_eq!(&[i * 2 + 5].as_ref(), &value);
        }

        dir.close().unwrap();

        Ok(())
    }

    #[test]
    fn random_insertion() -> Result<()> {
        let dir = tempdir().unwrap();

        let attr = Column::new(1, 1, "abc".to_string(), SqlType::TinyUint, 4, false);
        let codec = Codec::new(vec![attr]);

        let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
        let file_node = FileNode::new(1, 2, 3);

        BTree::<Codec>::init(file_node, &manager)?;
        let mut btree = BTree::new(codec, 30, file_node, &manager);

        let mut rng = rand::thread_rng();
        let mut nums: Vec<u8> = (0..120).collect();
        nums.shuffle(&mut rng);

        for &i in nums.iter() {
            btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5])?;
        }

        for &i in nums.iter() {
            let (mut cursor, is_matched) =
                btree.search(&vec![Value::TinyUint(i)]).unwrap().unwrap();

            let (_, value) = cursor.next().unwrap();

            assert!(is_matched);
            assert_eq!(&[i * 2 + 5].as_ref(), &value);
        }

        dir.close().unwrap();

        Ok(())
    }

    #[test]
    fn flush() -> Result<()> {
        let dir = tempdir().unwrap();

        let attr = Column::new(1, 1, "abc".to_string(), SqlType::TinyUint, 4, false);
        let key_codec = Codec::new(vec![attr]);

        let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
        let file_node = FileNode::new(1, 2, 3);

        BTree::<Codec>::init(file_node, &manager)?;
        let mut btree = BTree::new(key_codec.clone(), 30, file_node, &manager);

        let range = 0..120;

        for i in range.clone() {
            btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5])?;
        }

        manager.flush_pages().unwrap();

        let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
        let btree2 = BTree::new(key_codec, 30, file_node, &manager);

        for i in range {
            let (mut cursor, is_matched) =
                btree2.search(&vec![Value::TinyUint(i)]).unwrap().unwrap();

            let (_, value) = cursor.next().unwrap();

            assert!(is_matched);
            assert_eq!(&[i * 2 + 5].as_ref(), &value);
        }

        dir.close().unwrap();

        Ok(())
    }
}
