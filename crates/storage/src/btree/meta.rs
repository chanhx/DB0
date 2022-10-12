use std::mem::size_of;

use {
    super::PageType,
    crate::{PageNum, PAGE_SIZE},
    bytemuck::from_bytes_mut,
    common::pub_fields_struct,
};

pub_fields_struct! {
    #[derive(Debug, Copy, Clone)]
    #[repr(C)]
    struct Meta {
        page_type: PageType,
        page_size: u16,
        key_size: u16,
        value_size: u32,
        node_capacity: u32,
        magic: u32,
        version: u32,
        root: PageNum,
        free_list: PageNum,
        page_count: u32,
    }
}
unsafe impl bytemuck::Zeroable for Meta {}
unsafe impl bytemuck::Pod for Meta {}

impl Meta {
    pub fn new(bytes: &mut [u8]) -> &mut Self {
        from_bytes_mut(&mut bytes[..size_of::<Meta>()])
    }

    pub fn init(&mut self, key_size: u16, value_size: u32, node_capacity: u32) {
        self.page_type = PageType::Meta;
        self.page_size = PAGE_SIZE as u16;
        self.key_size = key_size;
        self.value_size = value_size;
        self.node_capacity = node_capacity;
    }
}
