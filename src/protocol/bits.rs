pub struct BitField {
    pub index: usize,
    pub mask: u8,
}

impl BitField {
    #[inline]
    pub fn get(&self, data: &[u8]) -> Option<u8> {
        if let Some(val) = data.get(self.index) {
            return Some((val & self.mask) >> self.mask.trailing_zeros());
        }
        None
    }

    #[inline]
    pub fn nonzero(&self, data: &[u8]) -> Option<bool> {
        if let Some(val) = data.get(self.index) {
            return Some(0 != (val & self.mask));
        }
        None
    }
}

pub struct U16Field {
    pub index: usize
}

impl U16Field {
    #[inline]
    pub fn get(&self, data: &[u8]) -> Option<u16> {
        if let Some(hi) = data.get(self.index) {
            if let Some(lo) = data.get(self.index + 1) {
              return Some(((*hi as u16) << 8) + *lo as u16);
            }
        }
        None
    }
}

pub struct U32Field {
    pub index: usize
}

impl U32Field {
    #[inline]
    pub fn get(&self, data: &[u8]) -> Option<u32> {
        let hi = U16Field{index:self.index}.get(data);
        let lo = U16Field{index:self.index+2}.get(data);
        if let Some(a) = hi {
            if let Some(b) = lo {
              return Some(((a as u32) << 16) + b as u32);
            }
        }
        None
    }
}
