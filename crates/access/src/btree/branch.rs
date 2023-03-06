use {
    super::{error, node::InsertEffect, PageType, Result},
    crate::slotted_page::{Slot, SlottedPage},
    bytemuck::from_bytes_mut,
    core::{mem::size_of, ops::Range},
    def::storage::{Decoder, Encoder},
    snafu::ResultExt,
    storage::{
        buffer::{BufferManager, FileNode},
        PageNum,
    },
};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Header {
    page_type: PageType,
    dirty: bool,
    right_sibling: PageNum,
}
unsafe impl bytemuck::Zeroable for Header {}
unsafe impl bytemuck::Pod for Header {}

pub struct Branch<'a, 'b, C> {
    header: &'a mut Header,
    slotted_page: SlottedPage<'a>,

    capacity: usize,
    key_codec: &'b C,
}

impl<'a, 'b, C, K> Branch<'a, 'b, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub fn new(bytes: &'a mut [u8], capacity: usize, key_codec: &'b C) -> Self {
        let (header, bytes) = bytes.split_at_mut(size_of::<Header>());

        let header = from_bytes_mut::<Header>(header);
        header.page_type = PageType::Branch;

        Self {
            header,
            slotted_page: SlottedPage::new(bytes),
            capacity,
            key_codec,
        }
    }

    pub fn init(
        &mut self,
        raw_key: &[u8],
        raw_high_key: &[u8],
        left: PageNum,
        right: PageNum,
        sibling: PageNum,
    ) {
        self.header.page_type = PageType::Branch;
        self.slotted_page.init();

        let (left, right) = (
            [raw_key, &left.to_le_bytes()],
            [raw_high_key, &right.to_le_bytes()],
        );
        self.slotted_page.insert(0, &left).unwrap();
        self.slotted_page.insert(1, &right).unwrap();

        self.header.right_sibling = sibling;
    }

    pub(super) fn capacity(page_size: usize, key_size: usize) -> usize {
        const RESERVED: usize = 64;
        (page_size - size_of::<Header>() - RESERVED)
            / (size_of::<Slot>() + key_size + size_of::<PageNum>())
    }

    fn raw_key(&self, range: Range<usize>) -> Vec<u8> {
        self.slotted_page
            .get_range(range.start..range.end - size_of::<PageNum>())
            .to_vec()
    }

    fn raw_high_key(&self) -> Vec<u8> {
        let slot = self.slotted_page.slots().last().unwrap();
        self.raw_key(slot.range())
    }

    fn key(&self, range: Range<usize>) -> Result<K> {
        let bytes = &self.slotted_page.get_range(range);
        self.key_codec
            .decode(bytes)
            .map(|r| r.0)
            .map_err(|e| error::Error::Decoding {
                source: Box::new(e),
            })
    }

    fn get_page_num(&self, range: Range<usize>) -> PageNum {
        PageNum::from_le_bytes(
            self.slotted_page
                .get_range(range.end - size_of::<PageNum>()..range.end)
                .try_into()
                .unwrap(),
        )
    }

    pub fn insert(
        &mut self,
        raw_key: &[u8],
        page_num: PageNum,
        index: usize,
        raw_high_key: Vec<u8>,

        manager: &BufferManager,
        file_node: &FileNode,
    ) -> Result<Option<InsertEffect>> {
        let update_high_key = index == self.slotted_page.slot_count() - 1;
        let slot = self.slotted_page.slots()[index];

        let original_page_num = self.get_page_num(slot.range());
        let original_page_num = original_page_num.to_le_bytes();

        let original_raw_key = self.raw_key(slot.range());
        let page_num = page_num.to_le_bytes();
        self.slotted_page
            .update_slot(index, &[&original_raw_key, &page_num])
            .unwrap();

        self.slotted_page
            .insert(index, &[raw_key, &original_page_num])
            .context(error::SlottedPageSnafu)?;

        if update_high_key {
            self.update_high_key(&raw_high_key);
        }

        if self.slotted_page.slot_count() <= self.capacity {
            return Ok(if update_high_key {
                Some(InsertEffect::UpdateHighKey(raw_high_key))
            } else {
                None
            });
        }

        // TODO: rebalance
        let mut splited_page_ref = manager.new_page(file_node).context(error::BufferSnafu)?;
        let splited_page_num = splited_page_ref.page_num();
        let mut splited_branch = Branch::new(
            splited_page_ref.as_slice_mut(),
            self.capacity,
            self.key_codec,
        );
        splited_branch.header.right_sibling = self.header.right_sibling;
        splited_branch.slotted_page.init();

        let slots_count = (self.slotted_page.slot_count() - 1) / 2;
        self.slotted_page
            .split_slots(slots_count, &mut splited_branch.slotted_page);

        let raw_high_key = splited_branch.raw_high_key();

        splited_page_ref.set_dirty();

        self.header.right_sibling = splited_page_num;

        Ok(Some(InsertEffect::Split {
            raw_new_key: self.raw_high_key(),
            raw_high_key,
            splited_page_num,
        }))
    }

    pub fn retrieve(&self, key: &K) -> Option<PageNum> {
        let slots = self.slotted_page.slots();
        Some(
            match slots.binary_search_by_key(key, |slot| self.key(slot.range()).unwrap()) {
                Err(i) if i == self.slotted_page.slot_count() => {
                    let right_siblilng = self.header.right_sibling;
                    if right_siblilng == 0 {
                        return None;
                    } else {
                        right_siblilng
                    }
                }
                Ok(i) | Err(i) => {
                    let slot = slots[i];
                    self.get_page_num(slot.range())
                }
            },
        )
    }

    pub fn update_high_key(&mut self, high_key: &[u8]) {
        let last_slot = self.slotted_page.slots().last().unwrap();
        let range = last_slot.range();

        let page_num = self
            .slotted_page
            .get_range(range.end - size_of::<PageNum>()..range.end)
            .to_vec();
        let data = [high_key, &page_num];

        let index = self.slotted_page.slot_count() - 1;
        self.slotted_page.update_slot(index, &data).unwrap();
    }

    // // TODO: rebalance
    // pub fn delete(&mut self, index: usize) -> Option<()> {
    //     self.slotted_page.delete(index).ok()
    // }

    pub(super) fn search(&self, key: &K) -> (usize, PageNum) {
        let slots = &self.slotted_page.slots();

        let index = match slots[..self.slotted_page.slot_count() - 1]
            .binary_search_by_key(key, |&slot| self.key(slot.range()).unwrap())
        {
            Ok(i) | Err(i) => i,
        };

        let slot = slots[index];
        let page_num = self.get_page_num(slot.range());

        (index, page_num)
    }

    pub fn is_right_most_slot(&self, slot_num: usize) -> bool {
        slot_num == self.slotted_page.slot_count() - 1
    }
}
