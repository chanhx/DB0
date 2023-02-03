use {
    bytemuck::{cast_slice, cast_slice_mut, from_bytes_mut},
    core::{mem::size_of, ops::Range},
    snafu::prelude::*,
    std::backtrace::Backtrace,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("index {} out of range", index))]
    IndexOutOfRange { backtrace: Backtrace, index: usize },

    #[snafu(display("space is not enough for insertion"))]
    SpaceNotEnough,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// | flag:2 | offset:15 | length: 15 |
#[derive(Debug, Copy, Clone)]
pub struct Slot(u32);
unsafe impl bytemuck::Zeroable for Slot {}
unsafe impl bytemuck::Pod for Slot {}

impl Slot {
    fn new(offset: u16, len: u16, state: SlotState) -> Self {
        Self(((state as u32) << 30) | (offset as u32) << 15 | len as u32)
    }

    pub fn offset(&self) -> usize {
        ((self.0 >> 15) & 0x7FFF) as usize
    }

    pub fn len(&self) -> usize {
        (self.0 & 0x7FFF) as usize
    }

    pub fn range(&self) -> Range<usize> {
        let offset = self.offset();
        offset..offset + self.len()
    }

    fn state(&self) -> SlotState {
        match (self.0 >> 30) & 0x03 {
            0 => SlotState::Unused,
            1 => SlotState::Normal,
            2 => SlotState::Redirect,
            _ => SlotState::Dead,
        }
    }

    fn update_offset(&mut self, offset: u16) {
        self.0 &= 0xC0_00_7F_FF;
        self.0 |= ((offset as u32) << 15) | 0xC0_00_7F_FF;
    }

    fn set_state(&mut self, state: SlotState) {
        self.0 &= 0x3F_FF_FF_FF;
        self.0 |= ((state as u32) << 30) | 0x3F_FF_FF_FF;
    }
}

#[repr(u8)]
enum SlotState {
    Unused = 0,
    Normal = 1,
    Redirect = 2,
    Dead = 3,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Header {
    slot_count: u16,
    total_free_space: u16,
    fragment_list: u16,
    free_area_end: u16,
}
unsafe impl bytemuck::Zeroable for Header {}
unsafe impl bytemuck::Pod for Header {}

#[derive(Debug)]
pub struct SlottedPage<'a> {
    header: &'a mut Header,
    body: &'a mut [u8],
}

impl<'a> SlottedPage<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        let (header, body) = bytes.split_at_mut(size_of::<Header>());
        Self {
            header: from_bytes_mut(header),
            body,
        }
    }

    pub fn init(&mut self) {
        *self.header = Header {
            slot_count: 0,
            total_free_space: self.body.len() as u16,
            fragment_list: 0,
            free_area_end: self.body.len() as u16 - 1,
        };
    }

    pub fn slot_count(&self) -> usize {
        self.header.slot_count as usize
    }

    fn total_free_space(&self) -> u16 {
        self.header.total_free_space
    }

    fn slots_size(&self) -> usize {
        self.slot_count() * size_of::<Slot>()
    }

    pub fn slots(&self) -> &[Slot] {
        cast_slice(&self.body[..self.slots_size()])
    }

    fn slots_mut(&mut self) -> &mut [Slot] {
        let slots_size = self.slots_size();
        cast_slice_mut(&mut self.body[..slots_size])
    }

    pub fn get_range(&self, range: Range<usize>) -> &[u8] {
        &self.body[range]
    }

    fn insert_slot(&mut self, index: usize, slot: Slot) {
        let slot_count = self.slot_count();
        let slots_mut = cast_slice_mut(&mut self.body[..(slot_count + 1) * size_of::<Slot>()]);

        slots_mut.copy_within(index..slot_count, index + 1);
        slots_mut[index] = slot;
    }

    pub fn insert(&mut self, index: usize, data: &[&[u8]]) -> Result<()> {
        let len = data.iter().map(|d| d.len()).sum::<usize>() as u16;
        let space_cost = size_of::<Slot>() as u16 + len;

        if self.total_free_space() < space_cost {
            return Err(Error::SpaceNotEnough);
        }

        if self.slots_size() as u16 + space_cost > self.header.free_area_end {
            // TODO: find available fragment or defragment
            return Err(Error::SpaceNotEnough);
        }

        let offset = self.header.free_area_end - len;
        let slot = Slot::new(offset, len, SlotState::Normal);

        data.iter().fold(offset as usize, |start, d| {
            let next = start + d.len();
            self.body[start..next].copy_from_slice(d);
            next
        });
        self.insert_slot(index, slot);

        self.header.slot_count += 1;
        self.header.total_free_space -= space_cost as u16;
        self.header.free_area_end = offset;

        Ok(())
    }

    pub fn update_slot(&mut self, index: usize, data: &[&[u8]]) -> Result<()> {
        let len = data.iter().map(|d| d.len()).sum::<usize>() as u16;

        let slot = self.slots().get(index).unwrap();
        let origin_len = slot.len() as u16;

        let offset = if len <= origin_len {
            slot.offset() as u16
        } else {
            if self.total_free_space() < len {
                return Err(Error::SpaceNotEnough);
            }

            if self.slots_size() as u16 + len > self.header.free_area_end {
                // TODO: find available fragment or defragment
                return Err(Error::SpaceNotEnough);
            }

            self.header.total_free_space -= len - origin_len;
            self.header.free_area_end -= len;
            self.header.free_area_end
        };

        *self.slots_mut().get_mut(index).unwrap() =
            Slot::new(offset, len as u16, SlotState::Normal);

        data.iter().fold(offset as usize, |start, d| {
            let next = start + d.len();
            self.body[start..next].copy_from_slice(d);
            next
        });

        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<&[u8]> {
        let slots = self.slots();

        let slot = slots
            .get(index)
            .ok_or(IndexOutOfRangeSnafu { index }.build())?;

        Ok(&self.body[slot.range()])
    }

    pub fn delete(&mut self, index: usize) -> Result<()> {
        let slot_count = self.slot_count();
        let slots = self.slots_mut();

        let slot = slots
            .get(index)
            .ok_or(IndexOutOfRangeSnafu { index }.build())?;
        let offset = slot.offset();
        let len = slot.len() as u16;

        slots.copy_within(index + 1.., index);
        slots.iter_mut().take(slot_count - 1).for_each(|slot| {
            let o = slot.offset();
            if o < offset {
                slot.update_offset(o as u16 + len);
            }
        });

        self.body.copy_within(
            self.header.free_area_end as usize..offset,
            (self.header.free_area_end + len) as usize,
        );
        self.header.slot_count -= 1;
        self.header.total_free_space += len + size_of::<Slot>() as u16;
        self.header.free_area_end += len;

        Ok(())
    }

    pub fn split_slots<'b>(&mut self, count: usize, other: &mut SlottedPage<'b>) {
        let mut space_free = 0;

        self.slots()
            .iter()
            .skip(self.slot_count() - count)
            .enumerate()
            .for_each(|(i, slot)| {
                // slot.set_state(SlotState::Unused);
                space_free += slot.len() + size_of::<Slot>();

                let data = &self.body[slot.range()];
                other.insert(i, &[data]).unwrap();
            });

        self.header.slot_count -= count as u16;
        self.header.total_free_space += space_free as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let input_size = 50;
        let mut bytes = vec![0; input_size];

        let mut page = SlottedPage::new(bytes.as_mut_slice());
        page.init();

        assert_eq!(
            page.header.total_free_space as usize,
            input_size - size_of::<Header>(),
        );

        let d1 = &[1, 2, 3, 4, 5];
        page.insert(0, &[d1])?;

        let d2 = &[6, 7, 8, 9, 10];
        page.insert(1, &[d2])?;

        let d3 = &[12, 56, 89];
        page.insert(0, &[d3])?;

        assert_eq!(page.header.slot_count, 3);

        assert_eq!(&page.get(1)?, d1);
        assert_eq!(&page.get(2)?, d2);
        assert_eq!(&page.get(0)?, d3);

        Ok(())
    }
}
