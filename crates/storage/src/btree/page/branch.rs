use {
    super::{Error, InsertEffect, Node, PageType, Result},
    crate::{
        buffer::{BufferManager, Replacer},
        slotted_page::SlottedPage,
        PageId,
    },
    bytemuck::from_bytes_mut,
    std::mem::size_of,
};

const PAGE_ID_SIZE: usize = size_of::<PageId>();

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Header {
    page_type: PageType,
    dirty: bool,
    next_sibling: PageId,
}
unsafe impl bytemuck::Zeroable for Header {}
unsafe impl bytemuck::Pod for Header {}

pub struct Branch<'a> {
    header: &'a mut Header,
    pub slotted_page: SlottedPage<'a>,
    capacity: usize,
    key_size: u16,
}

impl<'a> Branch<'a> {
    pub fn new(bytes: &'a mut [u8], capacity: usize, key_size: u16) -> Self {
        let (header, bytes) = bytes.split_at_mut(size_of::<Header>());

        Self {
            header: from_bytes_mut(header),
            slotted_page: SlottedPage::new(bytes),
            capacity,
            key_size,
        }
    }

    pub fn init(
        &mut self,
        key: &[u8],
        high_key: &[u8],
        left: PageId,
        right: PageId,
        sibling: PageId,
    ) {
        self.header.page_type = PageType::Branch;
        self.slotted_page.init();

        let (left, right) = match self.key_size {
            0 => (
                [
                    (key.len() as u16).to_le_bytes().as_slice(),
                    key,
                    &left.to_le_bytes(),
                ]
                .concat(),
                [
                    (high_key.len() as u16).to_le_bytes().as_slice(),
                    high_key,
                    &right.to_le_bytes(),
                ]
                .concat(),
            ),
            _ => (
                [key, &left.to_le_bytes()].concat(),
                [high_key, &right.to_le_bytes()].concat(),
            ),
        };
        self.slotted_page.insert(0, &left).unwrap();
        self.slotted_page.insert(1, &right).unwrap();

        self.header.next_sibling = sibling;
    }

    pub fn get_key_value(&self, offset: usize) -> (&[u8], PageId) {
        let (start, key_size) = match self.key_size as usize {
            0 => {
                let bytes = self.slotted_page.get_range(offset..offset + 2);
                let key_size = u16::from_le_bytes(bytes.try_into().unwrap()) as usize;

                (offset + 2, key_size)
            }
            key_size => (offset, key_size),
        };

        (
            self.slotted_page.get_range(start..start + key_size),
            PageId::from_le_bytes(
                self.slotted_page
                    .get_range(start + key_size..start + key_size + PAGE_ID_SIZE)
                    .try_into()
                    .unwrap(),
            ),
        )
    }

    pub(super) fn high_key(&self) -> &[u8] {
        let slot = self.slotted_page.slots().last().unwrap();
        self.get_key_value(slot.offset()).0
    }

    pub fn insert<R: Replacer>(
        &mut self,
        key: &[u8],
        page_id: PageId,
        manager: &mut BufferManager<R>,
    ) -> Result<Option<InsertEffect>> {
        let slots = self.slotted_page.slots();

        let index =
            match slots.binary_search_by(|&slot| self.get_key_value(slot.offset()).0.cmp(key)) {
                Ok(i) if i == slots.len() - 1 => {
                    // return the rightmost child, if the key equal to high key
                    i
                }
                Ok(_) => {
                    return Err(Error::KeyAlreadyExists);
                }
                Err(i) if i == slots.len() => i - 1,
                Err(i) => i,
            };

        let update_high_key = index == slots.len() - 1;

        let slot = slots[index];
        let (_, next_page_id) = self.get_key_value(slot.offset());
        let next_page_id = &next_page_id.to_le_bytes();

        let data = match self.key_size {
            0 => [
                (key.len() as u16).to_le_bytes().as_slice(),
                key,
                next_page_id,
            ]
            .concat(),
            _ => [key, next_page_id].concat(),
        };

        self.slotted_page
            .insert(index, &data)
            .map_err(|err| Error::Internal {
                details: "".to_string(),
                source: Some(Box::new(err)),
            })?;

        let page_id = &page_id.to_le_bytes();
        let next_page_id_offset = slot.offset() + slot.len() - PAGE_ID_SIZE;
        let next_page_id_range = next_page_id_offset..next_page_id_offset + PAGE_ID_SIZE;
        self.slotted_page.body[next_page_id_range].copy_from_slice(page_id);

        if self.slotted_page.slot_count() <= self.capacity {
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

        let mut splited_branch = Branch::new(splited_page.data_mut(), self.capacity, self.key_size);
        splited_branch.header.page_type = PageType::Branch;
        splited_branch.slotted_page.init();
        splited_branch.header.next_sibling = self.header.next_sibling;

        let slots_count = (self.slotted_page.slot_count() - 1) / 2;
        self.slotted_page
            .split_slots(slots_count, &mut splited_branch.slotted_page);

        let splited_high_key = splited_branch.high_key().to_vec();
        splited_page.is_dirty = true;

        let new_key = self.high_key().to_vec();
        let slot = self.slotted_page.slots().last().unwrap();
        let last_child_id = self.get_key_value(slot.offset()).1;

        let page = manager
            .fetch_page(last_child_id)
            .map_err(|err| Error::Internal {
                details: format!("fetch page {}", last_child_id),
                source: Some(Box::new(err)),
            })?;
        let mut page = page.borrow_mut();

        let last_child = Node::new(
            last_child_id,
            page.data_mut(),
            self.capacity,
            self.key_size as usize,
            PAGE_ID_SIZE,
        )?;

        self.update_high_key(last_child.high_key().unwrap());
        self.header.next_sibling = splited_page.id;

        Ok(Some(InsertEffect::Split {
            new_key,
            splited_high_key,
            splited_page_id: splited_page.id,
        }))
    }

    pub fn update_high_key(&mut self, high_key: &[u8]) {
        let slot = self.slotted_page.slots().last().unwrap();
        let offset = slot.offset();

        let (key, page_id) = self.get_key_value(offset);
        let page_id = &page_id.to_le_bytes();

        match self.key_size {
            0 => {
                let data = [
                    (high_key.len() as u16).to_le_bytes().as_slice(),
                    high_key,
                    page_id,
                ]
                .concat();

                if high_key.len() <= key.len() {
                    self.slotted_page.body[offset..offset + data.len()].copy_from_slice(&data);
                } else {
                    let index = self.slotted_page.slot_count() - 1;
                    self.slotted_page.update_slot(index, &data).unwrap();
                }
            }
            _ => {
                let range = slot.range();
                self.slotted_page.body[range].copy_from_slice(&[high_key, page_id].concat())
            }
        }
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

    pub fn find_child(&self, key: &[u8]) -> (usize, PageId) {
        let slots = self.slotted_page.slots();

        let index =
            match slots.binary_search_by(|&slot| self.get_key_value(slot.offset()).0.cmp(key)) {
                Ok(i) => i,
                Err(i) if i == slots.len() => i - 1,
                Err(i) => i,
            };

        (index, self.get_key_value(slots[index].offset()).1)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::buffer::{BufferManager, LruReplacer},
        tempfile::NamedTempFile,
    };

    #[test]
    fn insert_without_split() -> Result<()> {
        let (_, path) = NamedTempFile::new().unwrap().into_parts();

        let size = 10;
        let replacer = LruReplacer::new(size);

        let mut manager = BufferManager::new(&path, size, replacer);
        let page = manager.new_page().map_err(|err| Error::Internal {
            details: "create new page".to_string(),
            source: Some(Box::new(err)),
        })?;
        let mut page = page.borrow_mut();

        let mut branch = Branch::new(page.data_mut(), 3, 0);
        branch.init(&[1], &[3], 1, 2, 0);

        let res = branch.insert(&[2], 4, &mut manager);

        assert!(res.is_ok());
        let res = res.unwrap();

        assert!(res.is_none());

        Ok(())
    }
}
