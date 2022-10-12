mod branch;
mod error;
mod leaf;
mod meta;
mod node;

use {
    self::{
        branch::Branch,
        leaf::Leaf,
        meta::Meta,
        node::{InsertEffect, Node},
    },
    crate::{
        buffer::{BufferManager, FileNode, Page, PageTag, Replacer},
        PageNum,
    },
    std::{cell::RefCell, rc::Rc},
};

pub use error::{Error, Result};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PageType {
    Deleted = 0,
    Meta = 1,
    Branch = 2,
    Leaf = 3,
}

const META_PAGE_NUM: PageNum = 0;

pub struct BTree {
    node_capacity: usize,
    key_size: u16,
    value_size: u32,
    file_node: FileNode,

    // TODO: there should be a manager or something records the pages count of every table in memory,
    // neither by the btree struct, nor by the meta page
    page_count: u32,
}

impl BTree {
    pub fn new<R: Replacer>(
        manager: &mut BufferManager<R>,
        node_capacity: u32,
        key_size: u16,
        value_size: u32,
        file_node: FileNode,
    ) -> Result<Self> {
        let mut btree = Self {
            node_capacity: node_capacity as usize,
            key_size,
            value_size,
            file_node,
            page_count: 0,
        };

        let meta_page = btree.create_page(manager)?;
        let mut meta_page = meta_page.borrow_mut();
        let meta = Meta::new(meta_page.data_mut());
        meta.init(key_size, value_size, node_capacity);

        let root_page = btree.create_page(manager)?;
        let mut root_page = root_page.borrow_mut();

        let mut root = Leaf::new(
            root_page.num,
            root_page.data_mut(),
            node_capacity as usize,
            key_size,
            value_size,
        );
        root.init(0, 0);
        meta.root = root_page.num;

        root_page.is_dirty = true;
        meta_page.is_dirty = true;

        Ok(btree)
    }

    fn init<R: Replacer>(file_node: FileNode, manager: &mut BufferManager<R>) -> Result<Self> {
        let mut btree = Self {
            node_capacity: 0,
            key_size: 0,
            value_size: 0,
            file_node,
            page_count: 0,
        };

        let meta_page = btree.fetch_page(manager, META_PAGE_NUM)?;
        let mut meta_page = meta_page.borrow_mut();
        let meta = Meta::new(meta_page.data_mut());

        btree.node_capacity = meta.node_capacity as usize;
        btree.key_size = meta.key_size;
        btree.value_size = meta.value_size;

        // btree.page_count =

        Ok(btree)
    }

    fn root_page_num<R: Replacer>(&self, manager: &mut BufferManager<R>) -> Result<PageNum> {
        let meta_page = self.fetch_page(manager, META_PAGE_NUM)?;
        let mut meta_page = meta_page.borrow_mut();
        Ok(Meta::new(meta_page.data_mut()).root)
    }

    pub fn insert<R: Replacer>(
        &mut self,
        manager: &mut BufferManager<R>,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let mut page_num = self.root_page_num(manager)?;
        let mut insert_effect;
        let mut stack = None;

        loop {
            let page = self.fetch_page(manager, page_num)?;
            let mut page = page.borrow_mut();

            let node = Node::new(
                page_num,
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
                        .insert(key, value, || self.create_page(manager))
                        .unwrap()
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
                page_num = stk.page_num;
                stack = stk.parent;

                let page = self.fetch_page(manager, page_num)?;
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
                        splited_page_num,
                    } => {
                        if stk.slot_num == branch.slotted_page.slot_count() - 1 {
                            branch.update_high_key(&splited_high_key);
                        }

                        // TODO: insert into slot directly by stack information without searching
                        if let Some(effect) = branch
                            .insert(&new_key, splited_page_num, || self.create_page(manager))
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
                        new_key,
                        splited_high_key,
                        splited_page_num,
                    } => {
                        let meta_page = self.fetch_page(manager, META_PAGE_NUM)?;
                        let mut meta_page = meta_page.borrow_mut();
                        let meta = Meta::new(meta_page.data_mut());

                        let new_root_page = self.create_page(manager)?;
                        let mut new_root_page = new_root_page.borrow_mut();

                        let mut new_root = Branch::new(
                            new_root_page.data_mut(),
                            self.node_capacity,
                            self.key_size,
                        );
                        new_root.init(&new_key, &splited_high_key, page_num, splited_page_num, 0);

                        meta.root = new_root_page.num;
                        meta_page.is_dirty = true;
                        new_root_page.is_dirty = true;
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
        let mut page_num = self.root_page_num(manager)?;

        loop {
            let page = self.fetch_page(manager, page_num)?;
            let mut page = page.borrow_mut();

            let node = Node::new(
                page_num,
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
                    (_, page_num) = branch.find_child(key);
                }
                Node::Leaf(leaf) => {
                    return Ok(leaf.search_by_key(key).map(|v| v.to_vec()));
                }
            }
        }
    }

    fn fetch_page<R: Replacer>(
        &self,
        manager: &mut BufferManager<R>,
        page_num: PageNum,
    ) -> Result<Rc<RefCell<Page>>> {
        let page_tag = PageTag {
            file_node: self.file_node,
            page_num,
        };

        manager.fetch_page(page_tag).map_err(|err| Error::Internal {
            details: format!("fetch page {}", page_num),
            source: Some(Box::new(err)),
        })
    }

    fn create_page<R: Replacer>(
        &mut self,
        manager: &mut BufferManager<R>,
    ) -> Result<Rc<RefCell<Page>>> {
        let page_tag = PageTag {
            file_node: self.file_node,
            page_num: self.page_count,
        };

        // TODO: inspect
        manager
            .new_page(page_tag)
            .map(|page| {
                self.page_count += 1;
                page
            })
            // .inspect(|_| self.page_count += 1)
            .map_err(|err| Error::Internal {
                details: "create new page".to_string(),
                source: Some(Box::new(err)),
            })
    }

    // #[cfg(test)]
    // fn print_tree<R: Replacer>(&self, manager: &mut BufferManager<R>) -> Result<()> {
    //     let root_page_num = self.root_page_num(manager)?;
    //     let mut v = std::collections::VecDeque::from([root_page_num]);

    //     loop {
    //         let len = v.len();
    //         if len == 0 {
    //             break;
    //         }

    //         for _ in 0..len {
    //             let page_num = v.pop_front().unwrap();

    //             let page = self.fetch_page(manager, page_num)?;
    //             let mut page = page.borrow_mut();

    //             let node = Node::new(
    //                 page_num,
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
    //                         let (key, _) = leaf.get_key_value(slot.offset());
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
    page_num: PageNum,
    parent: Option<Box<StackNode>>,
}

#[cfg(test)]
mod tests {
    use {super::*, crate::buffer::LruReplacer, rand::prelude::*, std::num::NonZeroUsize};

    #[test]
    fn sequential_insertion() -> Result<()> {
        let size = NonZeroUsize::new(10).unwrap();
        let replacer = LruReplacer::new(size);

        let mut manager = BufferManager::new(size, replacer);
        let file_node = FileNode { table_id: 123 };
        let mut btree = BTree::new(&mut manager, 30, 0, 0, file_node)?;

        let range = 0..120;

        for i in range.clone() {
            btree.insert(&mut manager, &[i], &[i * 2 + 5])?;
        }

        for i in range {
            let btree_value = btree.search_by_key(&mut manager, &[i]).unwrap().unwrap();

            assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
        }

        Ok(())
    }

    #[test]
    fn random_insertion() -> Result<()> {
        let size = NonZeroUsize::new(10).unwrap();
        let replacer = LruReplacer::new(size);

        let mut manager = BufferManager::new(size, replacer);
        let file_node = FileNode { table_id: 123 };
        let mut btree = BTree::new(&mut manager, 30, 0, 0, file_node)?;

        let mut rng = rand::thread_rng();
        let mut nums: Vec<u8> = (0..120).collect();
        nums.shuffle(&mut rng);

        for &i in nums.iter() {
            btree.insert(&mut manager, &[i], &[i * 2 + 5])?;
        }

        for &i in nums.iter() {
            let btree_value = btree.search_by_key(&mut manager, &[i]).unwrap().unwrap();

            assert_eq!(&[i * 2 + 5].as_ref(), &btree_value);
        }

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
