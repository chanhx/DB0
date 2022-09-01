mod error;
mod page;

use {
    self::page::{Branch, InsertEffect, Leaf, Meta, Node},
    crate::{
        buffer::{BufferManager, Replacer},
        PageId,
    },
};

pub use error::{Error, Result};

pub struct BTree {
    meta_page_id: PageId,
    root_page_id: PageId,
    node_capacity: usize,
    key_size: u16,
    value_size: u32,
}

impl BTree {
    pub fn new<R: Replacer>(
        manager: &mut BufferManager<R>,
        node_capacity: usize,
        key_size: u16,
        value_size: u32,
    ) -> Result<Self> {
        let meta_page = manager.new_page().map_err(|err| Error::Internal {
            details: "create new page".to_string(),
            source: Some(Box::new(err)),
        })?;
        let mut meta_page = meta_page.borrow_mut();
        let meta = Meta::new(meta_page.data_mut());
        meta.init(key_size, value_size);

        let root_page = manager.new_page().map_err(|err| Error::Internal {
            details: "create new page".to_string(),
            source: Some(Box::new(err)),
        })?;
        let mut root_page = root_page.borrow_mut();

        let mut root = Leaf::new(
            root_page.id,
            root_page.data_mut(),
            node_capacity,
            key_size,
            value_size,
        );
        root.init(0, 0);
        meta.root = root_page.id;

        root_page.is_dirty = true;
        meta_page.is_dirty = true;

        Ok(Self {
            meta_page_id: meta_page.id,
            root_page_id: root_page.id,
            node_capacity,
            key_size,
            value_size,
        })
    }

    pub fn insert<R: Replacer>(
        &mut self,
        manager: &mut BufferManager<R>,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let mut page_id = self.root_page_id;
        let mut insert_effect;
        let mut stack = None;

        loop {
            let page = manager.fetch_page(page_id).map_err(|err| Error::Internal {
                details: format!("fetch page {}", self.root_page_id),
                source: Some(Box::new(err)),
            })?;
            let mut page = page.borrow_mut();

            let node = Node::new(
                page_id,
                page.data_mut(),
                self.node_capacity,
                self.key_size as usize,
                self.value_size as usize,
            )
            .map_err(|err| Error::Internal {
                details: "".to_string(),
                source: Some(Box::new(err)),
            })?;

            match node {
                Node::Branch(branch) => {
                    let (slot_num, child_page_id) = branch.find_child(key);

                    stack = Some(Box::new(StackNode {
                        page_id,
                        slot_num,
                        parent: stack,
                    }));

                    page_id = child_page_id;
                }
                Node::Leaf(mut leaf) => {
                    if let Some(effect) = leaf.insert(key, value, manager).unwrap() {
                        insert_effect = effect;
                        break;
                    }

                    return Ok(());
                }
            }
        }

        loop {
            if let Some(stk) = stack {
                page_id = stk.page_id;
                stack = stk.parent;

                let page = manager.fetch_page(page_id).map_err(|err| Error::Internal {
                    details: format!("fetch page {}", page_id),
                    source: Some(Box::new(err)),
                })?;
                let mut page = page.borrow_mut();

                let mut branch = Branch::new(page.data_mut(), self.node_capacity, self.key_size);

                match insert_effect {
                    InsertEffect::UpdateHighKey(high_key) => {
                        if stk.slot_num == branch.slotted_page.slot_count() - 1 {
                            branch.update_high_key(&high_key);
                        }
                        break;
                    }
                    InsertEffect::Split {
                        new_key,
                        splited_high_key,
                        splited_page_id,
                    } => {
                        if stk.slot_num == branch.slotted_page.slot_count() - 1 {
                            branch.update_high_key(&splited_high_key);
                        }

                        // TODO: insert into slot directly by stack information without searching
                        if let Some(effect) =
                            branch.insert(&new_key, splited_page_id, manager).unwrap()
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
                        new_key,
                        splited_high_key,
                        splited_page_id,
                    } => {
                        let meta_page = manager.fetch_page(self.meta_page_id).map_err(|err| {
                            Error::Internal {
                                details: format!("fetch page {}", self.meta_page_id),
                                source: Some(Box::new(err)),
                            }
                        })?;
                        let mut meta_page = meta_page.borrow_mut();
                        let meta = Meta::new(meta_page.data_mut());

                        let root_page = manager.new_page().map_err(|err| Error::Internal {
                            details: "create new page".to_string(),
                            source: Some(Box::new(err)),
                        })?;
                        let mut root_page = root_page.borrow_mut();

                        let mut root =
                            Branch::new(root_page.data_mut(), self.node_capacity, self.key_size);
                        root.init(&new_key, &splited_high_key, page_id, splited_page_id, 0);

                        meta.root = root_page.id;
                        meta_page.is_dirty = true;
                        root_page.is_dirty = true;

                        self.root_page_id = root_page.id;
                    }
                    _ => {}
                }
                break;
            }
        }

        Ok(())
    }

    pub fn search_by_key<R: Replacer>(
        &self,
        manager: &mut BufferManager<R>,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let mut page_id = self.root_page_id;

        loop {
            let page = manager.fetch_page(page_id).map_err(|err| Error::Internal {
                details: format!("fetch page {}", self.root_page_id),
                source: Some(Box::new(err)),
            })?;
            let mut page = page.borrow_mut();

            let node = Node::new(
                page_id,
                page.data_mut(),
                self.node_capacity,
                self.key_size as usize,
                self.value_size as usize,
            )
            .map_err(|err| Error::Internal {
                details: "".to_string(),
                source: Some(Box::new(err)),
            })?;

            match node {
                Node::Branch(branch) => {
                    (_, page_id) = branch.find_child(key);
                }
                Node::Leaf(leaf) => {
                    return Ok(leaf.search_by_key(key).map(|v| v.to_vec()));
                }
            }
        }
    }

    // fn print_tree<R: Replacer>(&self, manager: &mut BufferManager<R>) -> Result<()> {
    //     let mut v = std::collections::VecDeque::from([self.root_page_id]);

    //     loop {
    //         let len = v.len();
    //         if len == 0 {
    //             break;
    //         }

    //         for _ in 0..len {
    //             let page_id = v.pop_front().unwrap();

    //             let page = manager.fetch_page(page_id).map_err(|err| Error::Internal {
    //                 details: format!("fetch page {}", self.root_page_id),
    //                 source: Some(Box::new(err)),
    //             })?;
    //             let mut page = page.borrow_mut();

    //             let node = Node::new(
    //                 page_id,
    //                 page.data_mut(),
    //                 self.node_capacity,
    //                 self.key_size as usize,
    //                 self.value_size as usize,
    //             )
    //             .map_err(|err| Error::Internal {
    //                 details: "".to_string(),
    //                 source: Some(Box::new(err)),
    //             })?;

    //             match node {
    //                 Node::Branch(branch) => {
    //                     let slots = branch.slotted_page.slots();

    //                     slots.iter().enumerate().for_each(|(i, slot)| {
    //                         let (key, pid) = branch.get_key_value(slot.offset());
    //                         if i < slots.len() - 1 {
    //                             print!("({}), ", key[0]);
    //                         }
    //                         v.push_back(pid);
    //                     });
    //                 }
    //                 Node::Leaf(leaf) => {
    //                     let slots = leaf.slotted_page.slots();

    //                     slots.iter().for_each(|slot| {
    //                         let (key, value) = leaf.get_key_value(slot.offset());
    //                         print!("({}), ", key[0]);
    //                     });
    //                 }
    //             }

    //             print!("  ***   ");
    //         }

    //         println!();
    //     }

    //     Ok(())
    // }
}

struct StackNode {
    slot_num: usize,
    page_id: PageId,
    parent: Option<Box<StackNode>>,
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{buffer::LruReplacer, PAGE_SIZE},
        rand::prelude::*,
        std::io::Read,
        tempfile::NamedTempFile,
    };

    #[test]
    fn meta_info() -> Result<()> {
        let (mut file, path) = NamedTempFile::new().unwrap().into_parts();

        let size = 10;
        let replacer = LruReplacer::new(size);

        let mut buffer_manager = BufferManager::new(&path, size, replacer);
        let btree = BTree::new(&mut buffer_manager, 10, 0, 0)?;

        let meta_page_id = btree.meta_page_id;
        let root_page_id = btree.root_page_id;

        buffer_manager.flush_page(meta_page_id).unwrap();
        buffer_manager.flush_page(root_page_id).unwrap();

        let mut buf = [0; PAGE_SIZE];
        file.read_exact(&mut buf).unwrap();

        assert_eq!(buf[0], 0x01);
        assert_eq!(buf[2], (PAGE_SIZE & 0xFF) as u8);
        assert_eq!(buf[3], ((PAGE_SIZE >> 8) & 0xFF) as u8);

        assert_eq!(buf[24], (root_page_id & 0xFF) as u8);

        Ok(())
    }

    #[test]
    fn sequential_insertion() -> Result<()> {
        let (_, path) = NamedTempFile::new().unwrap().into_parts();

        let size = 10;
        let replacer = LruReplacer::new(size);

        let mut manager = BufferManager::new(&path, size, replacer);
        let mut btree = BTree::new(&mut manager, 30, 0, 0)?;

        for i in 0..120 {
            btree.insert(&mut manager, &[i], &[i * 2 + 5])?;
        }

        for i in 0..120 {
            let btree_value = btree.search_by_key(&mut manager, &[i]).unwrap().unwrap();

            assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
        }

        Ok(())
    }

    #[test]
    fn random_insertion() -> Result<()> {
        let (_, path) = NamedTempFile::new().unwrap().into_parts();

        let size = 10;
        let replacer = LruReplacer::new(size);

        let mut manager = BufferManager::new(&path, size, replacer);
        let mut btree = BTree::new(&mut manager, 30, 0, 0)?;

        let mut rng = rand::thread_rng();
        let mut nums: Vec<u8> = (0..120).collect();
        nums.shuffle(&mut rng);

        for &i in nums.iter() {
            btree.insert(&mut manager, &[i], &[i * 2 + 5])?;
        }

        // btree.print_tree(&mut manager)?;

        for &i in nums.iter() {
            let btree_value = btree.search_by_key(&mut manager, &[i]).unwrap().unwrap();

            assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
        }

        Ok(())
    }
}
