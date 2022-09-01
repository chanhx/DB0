use {
    bytemuck::{cast_slice, cast_slice_mut, from_bytes_mut},
    std::{mem::size_of, ops::Range},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IndexOutOfRange { index: usize },
    SpaceNotEnough,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IndexOutOfRange { index } => format!("index {} out of range", index),
                Self::SpaceNotEnough => format!("space is not enough for insertion"),
            }
        )
    }
}

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
    cell_area_start: u16,
}
unsafe impl bytemuck::Zeroable for Header {}
unsafe impl bytemuck::Pod for Header {}

#[derive(Debug)]
pub struct SlottedPage<'a> {
    header: &'a mut Header,
    pub body: &'a mut [u8],
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
            cell_area_start: self.body.len() as u16 - 1,
        };
    }

    pub fn slot_count(&self) -> usize {
        self.header.slot_count as usize
    }

    fn total_free_space(&self) -> usize {
        self.header.total_free_space as usize
    }

    fn slots_size(&self) -> usize {
        self.slot_count() * size_of::<Slot>()
    }

    pub fn slots(&self) -> &[Slot] {
        cast_slice(&self.body[..self.slots_size()])
    }

    fn slots_mut(&mut self) -> &mut [Slot] {
        let slot_count = self.slot_count();
        cast_slice_mut(&mut self.body[..slot_count])
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

    pub fn insert(&mut self, index: usize, data: &[u8]) -> Result<()> {
        let len = data.len() as u16;
        let space_cost = size_of::<Slot>() + len as usize;

        if self.total_free_space() < space_cost {
            return Err(Error::SpaceNotEnough);
        }

        if self.slots_size() + space_cost > self.header.cell_area_start as usize {
            // TODO: find available fragment or defragment
            return Err(Error::SpaceNotEnough);
        }

        let offset = self.header.cell_area_start - len;
        let slot = Slot::new(offset, len, SlotState::Normal);

        self.body[slot.range()].copy_from_slice(data);
        self.insert_slot(index, slot);

        self.header.slot_count += 1;
        self.header.total_free_space -= space_cost as u16;
        self.header.cell_area_start = offset;

        Ok(())
    }

    pub fn update_slot(&mut self, index: usize, data: &[u8]) -> Result<()> {
        let len = data.len();

        if self.total_free_space() < len {
            return Err(Error::SpaceNotEnough);
        }

        if self.slots_size() + len > self.header.cell_area_start as usize {
            // TODO: find available fragment or defragment
            return Err(Error::SpaceNotEnough);
        }

        let len = len as u16;
        let offset = self.header.cell_area_start - len;

        let slot = self.slots_mut().get_mut(index).unwrap();
        *slot = Slot::new(offset, len, SlotState::Normal);
        let range = slot.range();

        self.body[range].copy_from_slice(data);
        self.header.total_free_space -= len;
        self.header.cell_area_start = offset;

        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<&[u8]> {
        let slots = self.slots();

        let slot = slots.get(index).ok_or(Error::IndexOutOfRange { index })?;

        Ok(&self.body[slot.range()])
    }

    pub fn delete(&mut self, index: usize) -> Result<()> {
        let slots = self.slots_mut();

        let slot = slots.get(index).ok_or(Error::IndexOutOfRange { index })?;
        let offset = slot.offset() as u16;
        let len = slot.len() as u16;

        slots.copy_within(index + 1.., index);

        self.header.total_free_space -= len + size_of::<Slot>() as u16;
        if offset == self.header.cell_area_start {
            self.header.cell_area_start += len;
        }

        // TODO: add fragment to fragment list

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
                other.insert(i, data).unwrap();
            });

        self.header.slot_count -= count as u16;
        self.header.total_free_space -= space_free as u16;
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
        page.insert(0, d1)?;

        let d2 = &[6, 7, 8, 9, 10];
        page.insert(1, d2)?;

        let d3 = &[12, 56, 89];
        page.insert(0, d3)?;

        assert_eq!(page.header.slot_count, 3);

        assert_eq!(&page.get(1)?, d1);
        assert_eq!(&page.get(2)?, d2);
        assert_eq!(&page.get(0)?, d3);

        Ok(())
    }
}
