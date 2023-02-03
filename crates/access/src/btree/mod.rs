mod branch;
mod error;
mod leaf;
mod meta;
mod node;

pub use error::{Error, Result};
use {
    self::{
        branch::Branch,
        leaf::Leaf,
        meta::Meta,
        node::{InsertEffect, Node},
    },
    def::storage::{Decoder, Encoder},
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

pub struct BTree<C> {
    key_codec: C,
    node_capacity: usize,
    file_node: FileNode,
}

impl<C, K> BTree<C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub fn new(
        key_codec: C,
        manager: &mut BufferManager,
        node_capacity: u32,
        file_node: FileNode,
    ) -> Result<Self> {
        let mut btree = Self {
            key_codec,
            node_capacity: node_capacity as usize,
            file_node,
        };

        let mut meta_page_ref = btree.create_page(manager)?;
        let meta = Meta::new(meta_page_ref.as_slice_mut());
        meta.init(node_capacity);

        // TODO: create root page when it is needed
        let mut root_page_ref = btree.create_page(manager)?;
        let mut root = Leaf::new(&mut root_page_ref, node_capacity as usize, &btree.key_codec);
        root.init(0, 0);
        meta.root = root_page_ref.page_num();

        root_page_ref.set_dirty();
        meta_page_ref.set_dirty();

        Ok(btree)
    }

    fn root_page_num(&self, manager: &BufferManager) -> Result<PageNum> {
        let mut meta_page_ref = self.fetch_page(manager, META_PAGE_NUM)?;
        Ok(Meta::new(meta_page_ref.as_slice_mut()).root)
    }

    pub fn insert(&mut self, key: &K, value: &[u8], manager: &BufferManager) -> Result<()> {
        let mut page_num = self.root_page_num(manager)?;
        let mut insert_effect;
        let mut stack = None;

        loop {
            let mut page_ref = self.fetch_page(manager, page_num)?;

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
                    if let Some(effect) = leaf.insert(key, value, manager, &self.file_node).unwrap()
                    {
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

                let mut page_ref = self.fetch_page(manager, page_num)?;

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
                                manager,
                                &self.file_node,
                            )
                            .unwrap()
                        {
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
                        let mut meta_page = self.fetch_page(manager, META_PAGE_NUM)?;
                        let meta = Meta::new(meta_page.as_slice_mut());

                        let mut new_root_page = self.create_page(manager)?;
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

    pub fn retrieve(&self, key: &K, manager: &mut BufferManager) -> Result<Option<Vec<u8>>> {
        let mut page_num = self.root_page_num(manager)?;

        loop {
            let mut page_ref = self.fetch_page(manager, page_num)?;

            let node = Node::new(&mut page_ref, self.node_capacity, &self.key_codec)?;

            match node {
                Node::Branch(branch) => match branch.retrieve(key) {
                    Some(num) => page_num = num,
                    None => return Ok(None),
                },
                Node::Leaf(leaf) => {
                    return leaf.retrieve(key).map(|r| r.map(|v| v.to_vec()));
                }
            }
        }
    }

    // pub fn delete(&mut self, key: &K, manager: &mut BufferManager) -> Result<usize> {
    //     unimplemented!()
    // }

    fn fetch_page<'a>(
        &'_ self,
        manager: &'a BufferManager,
        page_num: PageNum,
    ) -> Result<BufferRef<'a>> {
        let page_tag = PageTag {
            file_node: self.file_node,
            page_num,
        };

        manager.fetch_page(page_tag).context(error::BufferSnafu)
    }

    fn create_page<'a>(&'_ mut self, manager: &'a BufferManager) -> Result<BufferRef<'a>> {
        manager
            .new_page(&self.file_node)
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
        def::{attribute::Attribute, Value},
        rand::prelude::*,
        storage::DEFAULT_PAGE_SIZE,
        tempfile::tempdir,
    };

    #[test]
    fn sequential_insertion() -> Result<()> {
        let dir = tempdir().unwrap();

        let attr = Attribute::new("abc".to_string(), 0, def::DataType::TinyUint, false);
        let codec = Codec::new(vec![attr]);

        let mut manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
        let file_node = FileNode::new(1, 2, 3);
        let mut btree = BTree::new(codec, &mut manager, 30, file_node)?;

        let range = 0..120;

        for i in range.clone() {
            btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5], &mut manager)?;
        }

        for i in range {
            let btree_value = btree
                .retrieve(&vec![Value::TinyUint(i)], &mut manager)
                .unwrap()
                .unwrap();

            assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
        }

        dir.close().unwrap();

        Ok(())
    }

    #[test]
    fn random_insertion() -> Result<()> {
        let dir = tempdir().unwrap();

        let attr = Attribute::new("abc".to_string(), 0, def::DataType::TinyUint, false);
        let codec = Codec::new(vec![attr]);

        let mut manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
        let file_node = FileNode::new(1, 2, 3);
        let mut btree = BTree::new(codec, &mut manager, 30, file_node)?;

        let mut rng = rand::thread_rng();
        let mut nums: Vec<u8> = (0..120).collect();
        nums.shuffle(&mut rng);

        for &i in nums.iter() {
            btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5], &mut manager)?;
        }

        for &i in nums.iter() {
            let btree_value = btree
                .retrieve(&vec![Value::TinyUint(i)], &mut manager)
                .unwrap()
                .unwrap();

            assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
        }

        dir.close().unwrap();

        Ok(())
    }

    // #[test]
    // fn flush() -> Result<()> {
    //     let size = NonZeroUsize::new(10).unwrap();
    //     let replacer = LruReplacer::new(size);

    //     let mut manager = BufferManager::new(size, replacer);
    //     let file_node = FileNode { table_id: 123 };
    //     let mut btree = BTree::new(&mut manager, 30, 0, 0, file_node)?;

    //     let range = 0..120;

    //     for i in range.clone() {
    //         btree.insert(&mut manager, &[i], &[i * 2 + 5])?;
    //     }

    //     // let page_count = btree.page_count;
    //     // for i in 0..page_count {
    //     //     let page_tag = PageTag {
    //     //         file_node,
    //     //         page_num: i,
    //     //     };
    //     //     manager.flush_page(&page_tag).map_err(|e| Error::Internal {
    //     //         details: "flush page error".to_string(),
    //     //         source: Some(Box::new(e)),
    //     //     })?;
    //     // }

    //     let btree2 = BTree::init(file_node, &mut manager)?;

    //     for i in range {
    //         let btree_value = btree2.search_by_key(&mut manager, &[i]).unwrap().unwrap();

    //         assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
    //     }

    //     Ok(())
    // }
}
