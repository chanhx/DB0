use {
    super::{error, InsertEffect, PageNum, PageType, Result},
    crate::{
        buffer::{BufferManager, BufferRef, FileNode},
        slotted_page::SlottedPage,
    },
    bytemuck::from_bytes_mut,
    core::{mem::size_of, ops::Range},
    def::storage::{Decoder, Encoder},
    snafu::ResultExt,
};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Header {
    page_type: PageType,
    dirty: bool,
    prev_page_num: PageNum,
    next_page_num: PageNum,
}
unsafe impl bytemuck::Zeroable for Header {}
unsafe impl bytemuck::Pod for Header {}

pub struct Leaf<'a, 'b, C> {
    header: &'a mut Header,
    slotted_page: SlottedPage<'a>,

    key_codec: &'b C,
    page_num: PageNum,
    capacity: usize,
}

impl<'a, 'b, C, K> Leaf<'a, 'b, C>
where
    C: Encoder<Item = K> + Decoder<Item = K>,
    K: Ord,
{
    pub fn new(page_ref: &'a mut BufferRef, capacity: usize, key_codec: &'b C) -> Self {
        let page_num = page_ref.page_num();

        let bytes = page_ref.as_slice_mut();
        let (header, bytes) = bytes.split_at_mut(size_of::<Header>());

        Self {
            header: from_bytes_mut(header),
            slotted_page: SlottedPage::new(bytes),
            key_codec,
            page_num,
            capacity,
        }
    }

    pub fn init(&mut self, next_page_num: PageNum, prev_page_num: PageNum) {
        self.header.page_type = PageType::Leaf;
        self.header.next_page_num = next_page_num;
        self.header.prev_page_num = prev_page_num;

        self.slotted_page.init();
    }

    fn raw_key(&self, range: Range<usize>) -> Vec<u8> {
        let bytes = self.slotted_page.get_range(range);
        let (_, len) = self.key_codec.decode(bytes).unwrap();

        bytes[..len].to_vec()
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

    // TODO: check slot state
    fn search_pos(&self, key: &K) -> std::result::Result<usize, usize> {
        let slots = self.slotted_page.slots();

        slots.binary_search_by_key(key, |slot| self.key(slot.range()).unwrap())
    }

    pub(super) fn retrieve(&self, key: &K) -> Result<Option<&[u8]>> {
        let slots = self.slotted_page.slots();

        match slots.binary_search_by_key(key, |slot| self.key(slot.range()).unwrap()) {
            Err(_) => {
                println!("key: {:#?}", self.key_codec.encode(key).unwrap());

                // for slot in self.slotted_page.slots() {
                //     print!("{:#?}", self.raw_key(slot.range()));
                // }
                Ok(None)
            }
            Ok(i) => {
                let slot = slots[i];

                let value = self.get_value(slot.range()).unwrap();
                Ok(Some(value))
            }
        }
    }

    fn get_value(&self, range: Range<usize>) -> Result<&[u8]> {
        let bytes = self.slotted_page.get_range(range);
        self.key_codec
            .decode(bytes)
            .map(|(_, len)| &bytes[len..])
            .map_err(|e| error::Error::Decoding {
                source: Box::new(e),
            })
    }

    pub fn insert(
        &mut self,
        key: &K,
        value: &[u8],

        manager: &BufferManager,
        file_node: &FileNode,
    ) -> Result<Option<InsertEffect>> {
        let mut update_high_key = false;

        let index = match self.search_pos(key) {
            Err(i) if i == self.slotted_page.slot_count() => {
                update_high_key = true;
                i
            }
            Err(i) => i,
            // TODO: handle duplicate key
            Ok(_) => return Ok(None),
        };

        let key = self.key_codec.encode(key).unwrap();
        self.slotted_page.insert(index, &[&key, value]).unwrap();

        if self.slotted_page.slot_count() < self.capacity {
            return Ok(if update_high_key {
                Some(InsertEffect::UpdateHighKey(key))
            } else {
                None
            });
        }

        // TODO: rebalance
        let mut splited_page_ref = manager.new_page(file_node).context(error::BufferSnafu)?;
        let splited_page_num = splited_page_ref.page_num();

        let mut splited_leaf = Leaf::new(&mut splited_page_ref, self.capacity, self.key_codec);
        splited_leaf.init(0, self.page_num);

        let slots_count = self.slotted_page.slot_count() / 2;
        self.slotted_page
            .split_slots(slots_count, &mut splited_leaf.slotted_page);

        self.header.next_page_num = splited_page_num;

        let raw_high_key = splited_leaf.raw_high_key();

        splited_page_ref.set_dirty();

        Ok(Some(InsertEffect::Split {
            raw_new_key: self.raw_high_key(),
            raw_high_key,
            splited_page_num,
        }))
    }
}
