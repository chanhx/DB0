use std::mem::size_of;

use {
    super::PageType,
    crate::{PageId, PAGE_SIZE},
    bytemuck::from_bytes_mut,
};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Meta {
    page_type: PageType,
    page_size: u16,
    key_size: u16,
    value_size: u32,
    magic: u32,
    version: u32,
    pub root: PageId,
    free_list: PageId,
}
unsafe impl bytemuck::Zeroable for Meta {}
unsafe impl bytemuck::Pod for Meta {}

impl Meta {
    pub fn new(bytes: &mut [u8]) -> &mut Self {
        from_bytes_mut(&mut bytes[..size_of::<Meta>()])
    }

    pub fn init(&mut self, key_size: u16, value_size: u32) {
        self.page_type = PageType::Meta;
        self.page_size = PAGE_SIZE as u16;
        self.key_size = key_size;
        self.value_size = value_size;
    }
}
