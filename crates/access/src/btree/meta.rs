use {
    super::PageType,
    bytemuck::{from_bytes, from_bytes_mut},
    common::pub_fields_struct,
    core::mem::size_of,
    storage::PageNum,
};

pub_fields_struct! {
    #[derive(Debug, Copy, Clone)]
    #[repr(C)]
    struct Meta {
        page_type: PageType,
        /// the count of tree levels
        level: u8,
        // page_size: u8,
        node_capacity: u32,
        magic: u32,
        version: u32,
        root: PageNum,
        free_list: PageNum,
    }
}
unsafe impl bytemuck::Zeroable for Meta {}
unsafe impl bytemuck::Pod for Meta {}

impl Meta {
    pub fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        from_bytes_mut(&mut bytes[..size_of::<Meta>()])
    }

    pub fn from_bytes(bytes: &[u8]) -> &Self {
        from_bytes(&bytes[..size_of::<Meta>()])
    }

    pub fn init(&mut self, node_capacity: u32) {
        self.level = 0;
        self.page_type = PageType::Meta;
        self.node_capacity = node_capacity;
    }
}

// impl From<&mut [u8]> for &mut Self {
//     fn from(bytes: &mut [u8]) -> Self {
//         from_bytes_mut(&mut bytes[..size_of::<Meta>()])
//     }
// }
