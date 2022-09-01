use crate::{PageId, PAGE_SIZE};

#[derive(Debug, Clone)]
#[repr(align(64))]
struct PageData([u8; PAGE_SIZE]);

#[derive(Debug, Clone)]
pub(crate) struct Page {
    pub id: PageId,
    pub is_dirty: bool,
    data: PageData,
}

impl Default for Page {
    fn default() -> Self {
        Self::new(0, [0; PAGE_SIZE])
    }
}

impl Page {
    pub fn new(id: PageId, data: [u8; PAGE_SIZE]) -> Self {
        Self {
            id,
            is_dirty: false,
            data: PageData(data),
        }
    }

    pub fn data(&self) -> &[u8] {
        self.data.0.as_ref()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.data.0.as_mut()
    }
}
