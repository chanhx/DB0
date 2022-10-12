use {
    super::{page::Page, BufferId},
    std::{cell::RefCell, num::NonZeroUsize, rc::Rc},
};

#[derive(Debug)]
pub(super) struct BufferPool {
    pages: Vec<Rc<RefCell<Page>>>,
}

impl BufferPool {
    pub(super) fn new(size: NonZeroUsize) -> Self {
        let pages = (0..usize::from(size))
            .into_iter()
            .map(|_| Rc::new(RefCell::new(Page::default())))
            .collect();

        BufferPool { pages }
    }

    pub(super) fn size(&self) -> usize {
        self.pages.len()
    }

    pub(super) fn get_buffer(&self, buffer_id: BufferId) -> Rc<RefCell<Page>> {
        self.pages[buffer_id].clone()
    }
}
