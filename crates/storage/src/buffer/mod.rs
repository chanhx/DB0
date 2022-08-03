mod error;
mod lru_replacer;
mod page;
mod pool;
mod replacer;

use {
    self::{
        page::Page,
        pool::{BufferPool, FrameId},
    },
    super::{disk::DiskManager, PageId, PAGE_SIZE},
    core::cell::RefCell,
    std::{
        collections::{HashMap, LinkedList},
        path::Path,
        rc::Rc,
    },
};

pub use self::{
    error::{Error, Result},
    lru_replacer::LruReplacer,
    replacer::Replacer,
};

pub struct BufferPoolManager<R: Replacer> {
    disk_manager: DiskManager,
    next_page_id: PageId,
    pool: BufferPool,
    page_table: HashMap<PageId, FrameId>,
    free_frames: LinkedList<FrameId>,
    replacer: R,
}

impl<R: Replacer> BufferPoolManager<R> {
    pub fn new(path: &Path, size: usize, replacer: R) -> Self {
        let mut free_frames = LinkedList::new();
        (0..size).for_each(|i| free_frames.push_back(i as FrameId));

        Self {
            disk_manager: DiskManager::new(path).unwrap(),
            next_page_id: 0,
            pool: BufferPool::new(size),
            page_table: HashMap::new(),
            free_frames,
            replacer,
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

        self.disk_manager
            .read_page(&page_id, page.borrow_mut().data_mut())
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
        let page = page.borrow_mut();

        if !page.is_dirty {
            return Ok(());
        }

        self.disk_manager
            .write_page(&page_id, page.data())
            .map_err(|err| Error::Internal {
                details: "write page".to_string(),
                source: Some(Box::new(err)),
            })?;

        Ok(())
    }
}
