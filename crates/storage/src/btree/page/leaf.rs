use {
    super::{Error, InsertEffect, PageType, Result},
    crate::{
        buffer::{BufferManager, Replacer},
        slotted_page::SlottedPage,
        PageId,
    },
    bytemuck::from_bytes_mut,
    std::mem::size_of,
};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Header {
    page_type: PageType,
    dirty: bool,
    prev_page_id: PageId,
    next_page_id: PageId,
}
unsafe impl bytemuck::Zeroable for Header {}
unsafe impl bytemuck::Pod for Header {}

pub struct Leaf<'a> {
    header: &'a mut Header,
    pub slotted_page: SlottedPage<'a>,
    page_id: PageId,
    capacity: usize,
    key_size: u16,
    value_size: u32,
}

impl<'a> Leaf<'a> {
    pub fn new(
        page_id: PageId,
        bytes: &'a mut [u8],
        capacity: usize,
        key_size: u16,
        value_size: u32,
    ) -> Self {
        let (header, bytes) = bytes.split_at_mut(size_of::<Header>());

        Self {
            header: from_bytes_mut(header),
            slotted_page: SlottedPage::new(bytes),
            page_id,
            capacity,
            key_size,
            value_size,
        }
    }

    pub fn init(&mut self, next_page_id: PageId, prev_page_id: PageId) {
        self.header.page_type = PageType::Leaf;
        self.header.next_page_id = next_page_id;
        self.header.prev_page_id = prev_page_id;

        self.slotted_page.init();
    }

    pub fn get_key_value(&self, offset: usize) -> (&[u8], &[u8]) {
        let (start, key_size, value_size) = match (self.key_size as usize, self.value_size as usize)
        {
            (0, 0) => {
                let bytes = self.slotted_page.get_range(offset..offset + 2);
                let key_size = u16::from_le_bytes(bytes.try_into().unwrap()) as usize;

                let bytes = self.slotted_page.get_range(offset + 2..offset + 6);
                let value_size = u32::from_le_bytes(bytes.try_into().unwrap()) as usize;

                (offset + 6, key_size, value_size)
            }
            (key_size, 0) => {
                let bytes = self.slotted_page.get_range(offset + 2..offset + 6);
                let value_size = u32::from_le_bytes(bytes.try_into().unwrap()) as usize;

                (offset + 4, key_size, value_size)
            }
            (0, value_size) => {
                let bytes = self.slotted_page.get_range(offset..offset + 2);
                let key_size = u16::from_le_bytes(bytes.try_into().unwrap()) as usize;

                (offset + 2, key_size, value_size)
            }
            (key_size, value_size) => (offset, key_size, value_size),
        };

        (
            self.slotted_page.get_range(start..start + key_size),
            self.slotted_page
                .get_range(start + key_size..start + key_size + value_size),
        )
    }

    pub(super) fn high_key(&self) -> Option<&[u8]> {
        self.slotted_page
            .slots()
            .last()
            .map(|slot| self.get_key_value(slot.offset()).0)
    }

    pub fn insert<R: Replacer>(
        &mut self,
        key: &[u8],
        value: &[u8],
        manager: &mut BufferManager<R>,
    ) -> Result<Option<InsertEffect>> {
        let slots = self.slotted_page.slots();

        let index =
            match slots.binary_search_by(|&slot| self.get_key_value(slot.offset()).0.cmp(key)) {
                Ok(_) => return Err(Error::KeyAlreadyExists),
                Err(i) => i,
            };

        let update_high_key = index == slots.len();

        let data = match (self.key_size, self.value_size) {
            (0, 0) => [
                (key.len() as u16).to_le_bytes().as_slice(),
                (value.len() as u32).to_le_bytes().as_slice(),
                key,
                value,
            ]
            .concat(),
            (_, 0) => [(value.len() as u32).to_le_bytes().as_slice(), key, value].concat(),
            (0, _) => [(key.len() as u16).to_le_bytes().as_slice(), key, value].concat(),
            (_, _) => [key, value].concat(),
        };

        self.slotted_page.insert(index, &data).unwrap();

        if self.slotted_page.slot_count() < self.capacity {
            return Ok(if update_high_key {
                Some(InsertEffect::UpdateHighKey(key.to_vec()))
            } else {
                None
            });
        }

        // TODO: rebalance
        let splited_page = manager.new_page().map_err(|err| Error::Internal {
            details: "create new page".to_string(),
            source: Some(Box::new(err)),
        })?;
        let mut splited_page = splited_page.borrow_mut();

        let mut splited_leaf = Leaf::new(
            splited_page.id,
            splited_page.data_mut(),
            self.capacity,
            self.key_size,
            self.value_size,
        );
        splited_leaf.init(0, self.page_id);

        let slots_count = self.slotted_page.slot_count() / 2;
        self.slotted_page
            .split_slots(slots_count, &mut splited_leaf.slotted_page);

        let new_key = self.high_key().unwrap().to_vec();
        let splited_high_key = splited_leaf.high_key().unwrap().to_vec();

        self.header.next_page_id = splited_page.id;

        splited_page.is_dirty = true;

        Ok(Some(InsertEffect::Split {
            new_key,
            splited_high_key,
            splited_page_id: splited_page.id,
        }))
    }

    pub fn delete(&mut self, key: &[u8]) -> Option<()> {
        let slots = self.slotted_page.slots();

        let index =
            match slots.binary_search_by(|&slot| self.get_key_value(slot.offset()).0.cmp(key)) {
                Ok(i) => i,
                Err(_) => return None,
            };

        self.slotted_page.delete(index).ok()
    }

    pub fn search_by_key(&self, key: &[u8]) -> Option<&[u8]> {
        let slots = self.slotted_page.slots();

        match slots.binary_search_by(|&slot| self.get_key_value(slot.offset()).0.cmp(key)) {
            Ok(i) => Some(self.get(i)),
            Err(_) => None,
        }
    }

    pub fn get(&self, index: usize) -> &[u8] {
        let slots = self.slotted_page.slots();
        let slot = slots[index];

        self.get_key_value(slot.offset()).1
    }
}
