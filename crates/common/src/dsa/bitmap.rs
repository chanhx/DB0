pub struct BitmapMut<'a>(&'a mut [u8]);

impl<'a> BitmapMut<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self(bytes)
    }

    fn byte_for_index_mut_unchecked(&mut self, idx: usize) -> &mut u8 {
        unsafe { self.0.get_unchecked_mut(idx >> 3) }
    }

    pub fn set_unchecked(&mut self, idx: usize) {
        *self.byte_for_index_mut_unchecked(idx) |= 1 << (idx & 7);
    }

    pub fn unset_unchecked(&mut self, idx: usize) {
        *self.byte_for_index_mut_unchecked(idx) &= !(1 << (idx & 7));
    }
}

pub struct Bitmap<'a>(&'a [u8]);

impl<'a> Bitmap<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    fn byte_for_index_unchecked(&self, idx: usize) -> &u8 {
        unsafe { self.0.get_unchecked(idx >> 3) }
    }

    pub fn is_set_unchecked(&self, idx: usize) -> bool {
        (self.byte_for_index_unchecked(idx) >> (idx & 7)) & 1 == 1
    }
}
