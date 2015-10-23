use std::ops::{Index, IndexMut, Range};


pub trait Data {
    type Slice: ?Sized + Index<usize,Output=u8>;

    fn get(&self, index: usize) -> Option<&u8>;
    fn get_range(&self, index: Range<usize>) -> Option<&Self::Slice>;
}

pub trait MutData: Data {
    type SliceMut: ?Sized + IndexMut<usize,Output=u8>;

    fn get_mut(&mut self, index: usize) -> Option<&mut u8>;
    fn get_mut_range(&mut self, index: Range<usize>) -> Option<&mut Self::SliceMut>;
}

impl Data for [u8] {
    type Slice = [u8];
    #[inline] fn get(&self, index: usize) -> Option<&u8> { (self as &[u8]).get(index) }

    #[inline]
    fn get_range(&self, index: Range<usize>) -> Option<&Self::Slice> {
        if index.end > self.len() { return None; }
        Some(&self[index])
    }
}

impl MutData for [u8] {
    type SliceMut = [u8];

    #[inline] fn get_mut(&mut self, index: usize) -> Option<&mut u8> { (self as &mut[u8]).get_mut(index) }

    #[inline]
    fn get_mut_range(&mut self, index: Range<usize>) -> Option<&mut Self::SliceMut> {
        if index.end > self.len() { return None; }
        Some(&mut self[index])
    }
}

type U8Slice = [u8];

impl Data for Vec<u8> {
    type Slice = [u8];

    #[inline] fn get(&self, index: usize) -> Option<&u8> { U8Slice::get(self, index) }

    #[inline]
    fn get_range(&self, index: Range<usize>) -> Option<&Self::Slice> {
        if index.end > self.len() { return None; }
        Some(&self[index])
    }
}
    
/// Bit field manipulation. Does not yet work across byte boundaries.
pub struct BitField {
    /// Byte offset from start of data.
    pub index: usize,
    /// Bits to extract from the target byte.
    pub mask: u8,
}

impl BitField {
    #[inline]
    pub fn get<T: Data + ?Sized>(&self, data: &T) -> Option<u8> {
        if let Some(val) = data.get(self.index) {
            return Some((val & self.mask) >> self.mask.trailing_zeros());
        }
        None
    }

    #[inline]
    pub fn nonzero<T: Data + ?Sized>(&self, data: &T) -> Option<bool> {
        if let Some(val) = data.get(self.index) {
            return Some(0 != (val & self.mask));
        }
        None
    }

    #[inline]
    pub fn set<T: MutData + ?Sized>(&self, data: &mut T, value: u8) {
        if let Some(elem) = data.get_mut(self.index) {
            *elem = (*elem & !self.mask) | ((value as u8 & self.mask) << self.mask.trailing_zeros());
        }
    }
}

pub struct BEU16Field {
    pub index: usize
}

impl BEU16Field {
    #[inline]
    pub fn get<T: Data + ?Sized>(&self, data: &T) -> Option<u16> {
        if let Some(split) = data.get_range(self.index..self.index+2) {
            return Some((split[0] as u16) << 8 + (split[1] as u16));
        }
        None
    }

    #[inline]
    pub fn set<T: MutData + ?Sized>(&self, data: &mut T, value: u16) {
        if let Some(split) = data.get_mut_range(self.index..self.index+2) {
            split[0] = ((value & 0xff00) >> 8) as u8;
            split[1] = (value & 0x00ff) as u8;
        }
    }
}

pub struct BEU32Field {
    pub index: usize
}

impl BEU32Field {
    #[inline]
    pub fn get<T: Data + ?Sized>(&self, data: &T) -> Option<u32> {
        if let Some(split) = data.get_range(self.index..self.index+4) {
            return Some(((split[0] as u32) << 24) +
                        ((split[1] as u32) << 16) +
                        ((split[2] as u32) << 8) +
                        ((split[3] as u32) << 0))
        }
        None
    }

    #[inline]
    pub fn set<T: MutData + ?Sized>(&self, data: &mut T, value: u32) {
        // TODO unsafe impl, once I can tell what the native endianness is.
        if let Some(split) = data.get_mut_range(self.index..self.index+4) {
            split[0] = ((value & 0xff000000) >> 24) as u8;
            split[1] = ((value & 0x00ff0000) >> 16) as u8;
            split[2] = ((value & 0x0000ff00) >> 8) as u8;
            split[3] = ((value & 0x000000ff) >> 0) as u8;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::BitField;

    #[test]
    fn bits0() {
        let data = [0xab, 0xcd];
        let view:&[u8] = &data[..];
        let field = BitField{index:0, mask:0xf0};
        assert_eq!(Some(0xa), field.get(view));
    }
    #[test]
    fn bits1() {
        let data = [0xab, 0xcd];
        let view:&[u8] = &data[..];
        let field = BitField{index:1, mask:0xf0};
        assert_eq!(Some(0xc), field.get(view));
    }
    #[test]
    fn bits1() {
        let data = [0xab, 0xcd];
        let view:&[u8] = &data[..];
        let field = BitField{index:1, mask:0xf0};
        assert_eq!(Some(0xc), field.get(view));
    }
}
