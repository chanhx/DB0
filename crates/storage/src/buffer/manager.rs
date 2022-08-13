use {
    super::{
        Error, FrameId, Replacer, Result,
        {page::Page, pool::BufferPool},
    },
    crate::{file::FileManager, PageId, PAGE_SIZE},
    core::cell::RefCell,
    std::{
        collections::{HashMap, LinkedList},
        path::{Path, PathBuf},
        rc::Rc,
    },
};

pub struct BufferManager<R: Replacer> {
    file_manager: FileManager,
    pool: BufferPool,
    page_table: HashMap<PageId, FrameId>,
    free_frames: LinkedList<FrameId>,
    replacer: R,
    next_page_id: PageId,

    file_path: PathBuf,
}

impl<R: Replacer> BufferManager<R> {
    pub fn new(path: &Path, size: usize, replacer: R) -> Self {
        let mut free_frames = LinkedList::new();
        (0..size).for_each(|i| free_frames.push_back(i as FrameId));

        Self {
            file_manager: FileManager::new(path.parent().unwrap()).unwrap(),
            pool: BufferPool::new(size),
            page_table: HashMap::new(),
            free_frames,
            replacer,
            next_page_id: 0,

            file_path: PathBuf::from(path.file_name().unwrap()),
        }
    }

    fn reuse_page(&mut self) -> Result<(FrameId, Rc<RefCell<Page>>)> {
        let frame_id = if let Some(frame_id) = self.free_frames.pop_back() {
            frame_id
        } else if let Some(frame_id) = self.replacer.victim() {
            let page = self.pool.get_buffer(frame_id);
            let page = page.borrow();

            if page.is_dirty {
                self.flush_page(page.id)?;
            }

            self.page_table.remove(&page.id);

            frame_id
        } else {
            return Err(Error::BufferPoolIsFull);
        };

        let page = self.pool.get_buffer(frame_id);
        self.replacer.pin(frame_id);

        Ok((frame_id, page))
    }

    pub(crate) fn new_page(&mut self) -> Result<Rc<RefCell<Page>>> {
        let (frame_id, page) = self.reuse_page()?;

        let page_id = self.next_page_id;
        self.next_page_id += 1;

        page.borrow_mut().id = page_id;

        self.page_table.insert(page_id, frame_id);

        return Ok(page);
    }

    pub(crate) fn fetch_page(&mut self, page_id: PageId) -> Result<Rc<RefCell<Page>>> {
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let page = self.pool.get_buffer(frame_id);
            self.replacer.pin(frame_id);

            return Ok(page);
        }

        let (frame_id, page) = self.reuse_page()?;

        self.file_manager
            .read(
                &PathBuf::from(self.file_path.file_name().unwrap()),
                page_id * PAGE_SIZE as u64,
                page.borrow_mut().data_mut(),
            )
            .map_err(|err| Error::Internal {
                details: "read page".to_string(),
                source: Some(Box::new(err)),
            })?;

        self.page_table.insert(page_id, frame_id);

        Ok(page)
    }

    fn flush_page(&mut self, page_id: PageId) -> Result<()> {
        let &frame_id = self.page_table.get(&page_id).ok_or(Error::Internal {
            details: "page is not in buffer".to_string(),
            source: None,
        })?;

        let page = self.pool.get_buffer(frame_id);
        let page = page.borrow();

        if !page.is_dirty {
            return Ok(());
        }

        self.file_manager
            .write(
                &PathBuf::from(self.file_path.file_name().unwrap()),
                page_id * PAGE_SIZE as u64,
                page.data(),
            )
            .map_err(|err| Error::Internal {
                details: "write page".to_string(),
                source: Some(Box::new(err)),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use {super::*, crate::buffer::LruReplacer, tempfile::NamedTempFile};

    #[test]
    fn it_works() {
        let (mut file, path) = NamedTempFile::new().unwrap().into_parts();

        let replacer = LruReplacer::new(10);
        let mut manager = BufferManager::new(path.as_ref(), 10, replacer);

        let page_id = {
            let page = manager.new_page().unwrap();

            let mut page = page.borrow_mut();
            page.id = 0;

            let data = page.data_mut();
            data.fill(2);

            page.is_dirty = true;

            page.id
        };

        manager.flush_page(page_id).unwrap();

        let mut buf = vec![0u8; PAGE_SIZE];
        file.read_exact(&mut buf).unwrap();

        assert_eq!(&buf, &vec![2; PAGE_SIZE]);
    }
}
