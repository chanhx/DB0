use {
    super::{page::Page, FrameId},
    std::{cell::RefCell, rc::Rc},
};

#[derive(Debug)]
pub(super) struct BufferPool {
    pages: Vec<Rc<RefCell<Page>>>,
}

impl BufferPool {
    pub(super) fn new(size: usize) -> Self {
        let pages = (0..size)
            .into_iter()
            .map(|_| Rc::new(RefCell::new(Page::default())))
            .collect();

        BufferPool { pages }
    }

    pub(super) fn size(&self) -> usize {
        self.pages.len()
    }

    pub(super) fn get_buffer(&self, frame_id: FrameId) -> Rc<RefCell<Page>> {
        self.pages[frame_id].clone()
    }
}
