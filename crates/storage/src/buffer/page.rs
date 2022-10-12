use crate::{PageNum, PAGE_SIZE};

#[derive(Debug, Clone)]
#[repr(align(64))]
struct PageData([u8; PAGE_SIZE]);

#[derive(Debug, Clone)]
pub struct Page {
    pub num: PageNum,
    pub is_dirty: bool,
    data: PageData,
}

impl Default for Page {
    fn default() -> Self {
        Self::new(0, [0; PAGE_SIZE])
    }
}

impl Page {
    pub fn new(num: PageNum, data: [u8; PAGE_SIZE]) -> Self {
        Self {
            num,
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
