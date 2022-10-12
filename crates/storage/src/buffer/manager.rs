use {
    super::{
        BufferId, Error, PageTag, Replacer, Result,
        {page::Page, pool::BufferPool},
    },
    crate::{file::FileManager, PageNum, PAGE_SIZE},
    core::cell::RefCell,
    std::{
        collections::{HashMap, LinkedList},
        num::NonZeroUsize,
        rc::Rc,
    },
};

struct BufferDescriptor {
    page_tag: PageTag,
    buffer_id: BufferId,
}

pub struct BufferManager<R: Replacer> {
    file_manager: FileManager,
    pool: BufferPool,
    page_table: HashMap<PageTag, BufferId>,
    descriptors: Vec<BufferDescriptor>,
    free_frames: LinkedList<BufferId>,
    replacer: R,
}

impl<R: Replacer> BufferManager<R> {
    pub fn new(size: NonZeroUsize, replacer: R) -> Self {
        let mut free_frames = LinkedList::new();
        (0..usize::from(size)).for_each(|i| free_frames.push_back(i as BufferId));

        let mut descriptors = Vec::with_capacity(size.get());
        unsafe {
            descriptors.set_len(size.get());
        }

        Self {
            file_manager: FileManager::new(),
            pool: BufferPool::new(size),
            page_table: HashMap::new(),
            descriptors,
            free_frames,
            replacer,
        }
    }

    fn reuse_page(&mut self) -> Result<(BufferId, Rc<RefCell<Page>>)> {
        let buffer_id = if let Some(buffer_id) = self.free_frames.pop_back() {
            buffer_id
        } else if let Some(buffer_id) = self.replacer.victim() {
            let page = self.pool.get_buffer(buffer_id);
            let page = page.borrow();

            let page_tag = self.descriptors[buffer_id].page_tag.clone();

            if page.is_dirty {
                self.flush_page(&page_tag)?;
            }

            self.page_table.remove(&page_tag);

            buffer_id
        } else {
            return Err(Error::BufferPoolIsFull);
        };

        let page = self.pool.get_buffer(buffer_id);
        self.replacer.pin(buffer_id);

        Ok((buffer_id, page))
    }

    pub(crate) fn new_page(&mut self, page_tag: PageTag) -> Result<Rc<RefCell<Page>>> {
        let (buffer_id, page) = self.reuse_page()?;
        page.borrow_mut().num = page_tag.page_num;

        self.page_table.insert(page_tag, buffer_id);

        return Ok(page);
    }

    pub(crate) fn fetch_page(&mut self, page_tag: PageTag) -> Result<Rc<RefCell<Page>>> {
        if let Some(&buffer_id) = self.page_table.get(&page_tag) {
            let page = self.pool.get_buffer(buffer_id);
            self.replacer.pin(buffer_id);

            return Ok(page);
        }

        let (buffer_id, page) = self.reuse_page()?;

        self.file_manager
            .read(
                &page_tag.file_node.file_path(),
                page_tag.page_num as u64 * PAGE_SIZE as u64,
                page.borrow_mut().data_mut(),
            )
            .map_err(|err| Error::Internal {
                details: "read page".to_string(),
                source: Some(Box::new(err)),
            })?;

        self.page_table.insert(page_tag, buffer_id);

        Ok(page)
    }

    pub(crate) fn flush_page(&mut self, page_tag: &PageTag) -> Result<()> {
        let &buffer_id = self.page_table.get(&page_tag).ok_or(Error::Internal {
            details: "page is not in buffer".to_string(),
            source: None,
        })?;

        let page = self.pool.get_buffer(buffer_id);
        let page = page.borrow();

        if !page.is_dirty {
            return Ok(());
        }

        self.file_manager
            .write(
                &page_tag.file_node.file_path(),
                page_tag.page_num as u64 * PAGE_SIZE as u64,
                page.data(),
            )
            .map_err(|err| Error::Internal {
                details: "write page".to_string(),
                source: Some(Box::new(err)),
            })?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::io::Read;

//     use {super::*, crate::buffer::LruReplacer, tempfile::NamedTempFile};

// #[test]
// fn it_works() {
//     let (mut file, path) = NamedTempFile::new().unwrap().into_parts();

//     let size = NonZeroUsize::new(10).unwrap();
//     let replacer = LruReplacer::new(size);
//     let mut manager = BufferManager::new(size, replacer);

//     let page_id = {
//         let page = manager.new_page().unwrap();

//         let mut page = page.borrow_mut();
//         page.id = 0;

//         let data = page.data_mut();
//         data.fill(2);

//         page.is_dirty = true;

//         page.id
//     };

//     manager.flush_page(page_id).unwrap();

//     let mut buf = vec![0u8; PAGE_SIZE];
//     file.read_exact(&mut buf).unwrap();

//     assert_eq!(&buf, &vec![2; PAGE_SIZE]);
// }
// }
