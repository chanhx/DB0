use super::{PageId, PAGE_SIZE};

#[derive(Debug, Clone)]
#[repr(align(64))]
pub(crate) struct PageData([u8; PAGE_SIZE]);

#[derive(Debug, Clone)]
pub(crate) struct Page {
    pub id: PageId,
    pub is_dirty: bool,
    data: PageData,
}

impl Default for Page {
    fn default() -> Self {
        Self::new(0, PageData([0; PAGE_SIZE]))
    }
}

impl Page {
    pub fn new(id: PageId, data: PageData) -> Self {
        Self {
            id,
            data,
            is_dirty: false,
        }
    }

    pub fn data(&self) -> &[u8] {
        self.data.0.as_ref()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.0.as_mut()
    }
}
