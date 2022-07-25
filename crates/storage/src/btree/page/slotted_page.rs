use {
    super::{Error, Result},
    bytemuck::{cast_slice, cast_slice_mut, from_bytes_mut},
    std::mem::size_of,
};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Header {
    slot_count: u16,
    total_free_space: u16,
    first_free: u16,
    cell_area_start: u16,
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
            first_free: self.body.len() as u16 - 4,
            cell_area_start: self.body.len() as u16,
        };

        self.body[self.header.first_free as usize..].copy_from_slice(&[
            (self.header.total_free_space & 0xFF) as u8,
            (self.header.total_free_space >> 8) as u8,
            0,
            0,
        ]);
    }

    fn get_u16(&self, offset: usize) -> u16 {
        (self.body[offset + 1] as u16) << 8 | self.body[offset] as u16
    }

    fn set_u16(&mut self, offset: usize, value: u16) {
        self.body[offset] = (value & 0xFF) as u8;
        self.body[offset + 1] = (value >> 8) as u8;
    }

    pub fn slot_count(&self) -> usize {
        self.header.slot_count as usize
    }

    fn total_free_space(&self) -> usize {
        self.header.total_free_space as usize
    }

    fn pointers_size(&self) -> usize {
        self.slot_count() * size_of::<u16>()
    }

    pub fn pointers(&self) -> &[u16] {
        cast_slice(&self.body[..self.pointers_size()])
    }

    fn find_free_space(&mut self, size: usize) -> Option<u16> {
        let offset = &mut self.header.first_free as *mut u16;
        let mut pointer = FreeBlockPointer::U16(offset);

        let mut min_remain = None;

        while pointer.get_address() != 0 {
            let free_block_size = self.get_u16(pointer.get_address()) as usize;

            if free_block_size >= size {
                match min_remain {
                    Some((_, blk_size)) if free_block_size >= blk_size => {}
                    _ => min_remain = Some((pointer, free_block_size)),
                }

                if free_block_size == size {
                    break;
                }
            }

            let address = &mut self.body[pointer.get_address() + 2] as *mut u8;
            pointer = FreeBlockPointer::U8(address);
        }

        min_remain.map(|(pointer, blk_size)| {
            let offset = pointer.get_address();

            if blk_size < size + 2 {
                // leave the fragment alone
                let next = self.get_u16(offset + 2);
                pointer.update_address(next);
            } else {
                self.set_u16(offset, (blk_size - size) as u16);
                self.body.copy_within(offset..offset + 4, offset - size);
                pointer.update_address((offset - size) as u16);
            }

            (offset + 4 - size) as u16
        })
    }

    fn insert_pointer(&mut self, index: usize, pointer: u16) {
        let slot_count = self.slot_count();
        let pointers_mut = cast_slice_mut(&mut self.body[..(slot_count + 1) * size_of::<u16>()]);

        pointers_mut.copy_within(index..slot_count, index + 1);
        pointers_mut[index] = pointer;
    }

    pub fn insert(&mut self, index: usize, data: &[u8]) -> Result<()> {
        let size = data.len();

        if self.total_free_space() < size_of::<u16>() + size {
            return Err(Error::SpaceNotEnough);
        }

        if self.pointers_size() + 2 > self.header.cell_area_start as usize {
            // TODO: defragment
            return Err(Error::SpaceNotEnough);
        }

        let pointer = self.find_free_space(size).ok_or(Error::SpaceNotEnough)?;
        self.insert_pointer(index, pointer);

        let first_free_size = self.get_u16(self.header.first_free as usize);
        self.set_u16(self.header.first_free as usize, first_free_size - 2);

        self.body[pointer as usize..pointer as usize + size].copy_from_slice(data);

        self.header.slot_count += 1;
        self.header.total_free_space -= size as u16 + 2;
        self.header.cell_area_start = self.header.cell_area_start.min(pointer);

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum FreeBlockPointer {
    U16(*mut u16),
    U8(*mut u8),
}

impl FreeBlockPointer {
    fn get_address(&self) -> usize {
        unsafe {
            match self {
                &Self::U16(p) => *p as usize,
                &Self::U8(p) => (*p as usize) << 8 | *p.offset(1) as usize,
            }
        }
    }

    fn update_address(&self, address: u16) {
        unsafe {
            match self {
                &Self::U16(p) => *p = address,
                &Self::U8(p) => {
                    *p = (address & 0xFF) as u8;
                    *(p.offset(1)) = (address >> 8) as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let mut bytes = vec![0; 100];

        let mut page = SlottedPage::new(bytes.as_mut_slice());
        page.init();

        assert_eq!(
            page.header.total_free_space as usize,
            100 - size_of::<Header>(),
        );

        let d1 = &[1, 2, 3, 4, 5];
        page.insert(0, &[1, 2, 3, 4, 5])?;

        let d2 = &[6, 7, 8, 9, 10];
        page.insert(1, d2)?;

        let d3 = &[12, 56, 89];
        page.insert(0, d3)?;

        assert_eq!(page.header.slot_count, 3);

        assert_eq!(&page.body[page.body.len() - 5..], d1);
        assert_eq!(&page.body[page.body.len() - 10..page.body.len() - 5], d2);
        assert_eq!(&page.body[page.body.len() - 13..page.body.len() - 10], d3);

        // println!("{:#?}", page);

        assert_eq!(
            page.get_u16(page.header.first_free as usize),
            page.header.total_free_space,
        );

        assert_eq!(
            page.header.total_free_space as usize,
            bytes.len() - size_of::<Header>() - 2 * 3 - d1.len() - d2.len() - d3.len()
        );

        Ok(())
    }
}
