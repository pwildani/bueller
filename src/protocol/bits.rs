use std::ops::{Index, IndexMut, Range};


pub trait BitData {
    type Slice: ?Sized + Index<usize,Output=u8>;

    fn get(&self, index: usize) -> Option<&u8>;
    fn get_range(&self, index: Range<usize>) -> Option<&Self::Slice>;
    fn len(&self) -> usize;
}

pub trait BitDataMut: BitData {
    type SliceMut: ?Sized + IndexMut<usize,Output=u8>;

    fn get_mut(&mut self, index: usize) -> Option<&mut u8>;
    fn get_mut_range(&mut self, index: Range<usize>) -> Option<&mut Self::SliceMut>;
}

impl BitData for [u8] {
    type Slice = [u8];
    #[inline]
    fn get(&self, index: usize) -> Option<&u8> {
        (self as &[u8]).get(index)
    }

    #[inline]
    fn get_range(&self, index: Range<usize>) -> Option<&Self::Slice> {
        if index.end > self.len() {
            return None;
        }
        Some(&self[index])
    }

    #[inline]
    fn len(&self) -> usize {
        (self as &[u8]).len()
    }
}

impl BitDataMut for [u8] {
    type SliceMut = [u8];

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        (self as &mut [u8]).get_mut(index)
    }

    #[inline]
    fn get_mut_range(&mut self, index: Range<usize>) -> Option<&mut Self::SliceMut> {
        if index.end > self.len() {
            return None;
        }
        Some(&mut self[index])
    }
}

type U8Slice = [u8];

impl BitData for Vec<u8> {
    type Slice = [u8];

    #[inline]
    fn get(&self, index: usize) -> Option<&u8> {
        U8Slice::get(self, index)
    }

    #[inline]
    fn get_range(&self, index: Range<usize>) -> Option<&Self::Slice> {
        if index.end > self.len() {
            return None;
        }
        Some(&self[index])
    }

    #[inline]
    fn len(&self) -> usize {
        (self as &[u8]).len()
    }
}

impl BitDataMut for Vec<u8> {
    type SliceMut = [u8];

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        (self as &mut [u8]).get_mut(index)
    }

    #[inline]
    fn get_mut_range(&mut self, index: Range<usize>) -> Option<&mut Self::SliceMut> {
        if index.end > self.len() {
            return None;
        }
        Some(&mut self[index])
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
    pub fn get<T: BitData + ?Sized>(&self, data: &T) -> Option<u8> {
        if let Some(val) = data.get(self.index) {
            return Some((val & self.mask) >> self.mask.trailing_zeros());
        }
        None
    }

    #[inline]
    pub fn nonzero<T: BitData + ?Sized>(&self, data: &T) -> Option<bool> {
        if let Some(val) = data.get(self.index) {
            return Some(0 != (val & self.mask));
        }
        None
    }

    #[inline]
    pub fn set<T: BitDataMut + ?Sized>(&self, data: &mut T, value: u8) {
        if let Some(elem) = data.get_mut(self.index) {
            *elem = (*elem & !self.mask) |
                    ((value as u8 & (self.mask >> self.mask.trailing_zeros())) <<
                     self.mask.trailing_zeros());
        }
    }
}

pub struct BEU16Field {
    pub index: usize,
}

impl BEU16Field {
    #[inline]
    pub fn get<T: BitData + ?Sized>(&self, data: &T) -> Option<u16> {
        if let Some(split) = data.get_range(self.index..self.index + 2) {
            return Some(((split[0] as u16) << 8) + (split[1] as u16));
        }
        None
    }

    #[inline]
    pub fn set<T: BitDataMut + ?Sized>(&self, data: &mut T, value: u16) {
        if let Some(split) = data.get_mut_range(self.index..self.index + 2) {
            split[0] = ((value & 0xff00) >> 8) as u8;
            split[1] = (value & 0x00ff) as u8;
        }
    }
}

pub struct BEU32Field {
    pub index: usize,
}

impl BEU32Field {
    #[inline]
    pub fn get<T: BitData + ?Sized>(&self, data: &T) -> Option<u32> {
        if let Some(split) = data.get_range(self.index..self.index + 4) {
            return Some(((split[0] as u32) << 24) + ((split[1] as u32) << 16) +
                        ((split[2] as u32) << 8) +
                        ((split[3] as u32) << 0));
        }
        None
    }

    #[inline]
    pub fn set<T: BitDataMut + ?Sized>(&self, data: &mut T, value: u32) {
        // TODO unsafe impl, once I can tell what the native endianness is.
        if let Some(split) = data.get_mut_range(self.index..self.index + 4) {
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
    use super::BEU16Field;
    use super::BEU32Field;

    #[test]
    fn u8_extract0() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BitField {
            index: 0,
            mask: 0xff,
        };
        assert_eq!(Some(0xab), field.get(view));
    }

    #[test]
    fn u8_extract1() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BitField {
            index: 1,
            mask: 0xff,
        };
        assert_eq!(Some(0xcd), field.get(view));
    }

    #[test]
    fn u4_extract() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BitField {
            index: 1,
            mask: 0xf0,
        };
        assert_eq!(Some(0xc), field.get(view));
    }

    #[test]
    fn u1_extract() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BitField {
            index: 1,
            mask: 0x80,
        };
        assert_eq!(Some(0x1), field.get(view));
    }

    #[test]
    fn bitfield_invalid_adddress() {
        let data = [0xab];
        let view: &[u8] = &data[..];
        let field = BitField {
            index: 1,
            mask: 0xf0,
        };
        assert_eq!(None, field.get(view));
    }

    #[test]
    fn u16_extract() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BEU16Field { index: 0 };
        assert_eq!(Some(0xabcd), field.get(view));
    }

    #[test]
    fn u16_unaligned() {
        let data = [0xab, 0xcd, 0xef];
        let view: &[u8] = &data[..];
        let field = BEU16Field { index: 1 };
        assert_eq!(Some(0xcdef), field.get(view));
    }

    #[test]
    fn u16_invalid_address() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BEU16Field { index: 1 };
        assert_eq!(None, field.get(view));
    }

    #[test]
    fn u32_extract() {
        let data = [0xab, 0xcd, 0xef, 0x01];
        let view: &[u8] = &data[..];
        let field = BEU32Field { index: 0 };
        assert_eq!(Some(0xabcdef01), field.get(view));
    }

    #[test]
    fn u32_unaligned() {
        let data = [0x00, 0xab, 0xcd, 0xef, 0x01];
        let view: &[u8] = &data[..];
        let field = BEU32Field { index: 1 };
        assert_eq!(Some(0xabcdef01), field.get(view));
    }

    #[test]
    fn u32_invalid_address() {
        let data = [0xab, 0xcd];
        let view: &[u8] = &data[..];
        let field = BEU16Field { index: 1 };
        assert_eq!(None, field.get(view));
    }


    #[test]
    fn u8_write0() {
        let mut data = [0xab, 0xcd];
        let view: &mut [u8] = &mut data[..];
        let field = BitField {
            index: 0,
            mask: 0xff,
        };
        field.set(view, 0x11);
        assert_eq!(0x11, view[0]);
        assert_eq!(0xcd, view[1]);
    }

    #[test]
    fn u8_write() {
        let mut data = [0xab, 0xcd];
        let view: &mut [u8] = &mut data[..];
        let field = BitField {
            index: 1,
            mask: 0xff,
        };
        field.set(view, 0x11);
        assert_eq!(0xab, view[0]);
        assert_eq!(0x11, view[1]);
    }

    #[test]
    fn u4_write() {
        let mut data = [0xab, 0xcd];
        let view: &mut [u8] = &mut data[..];
        let field = BitField {
            index: 1,
            mask: 0xf0,
        };
        field.set(view, 0x11);
        assert_eq!(0xab, view[0]);
        assert_eq!(0x1d, view[1]);
    }

    #[test]
    fn u1_write0() {
        let mut data = [0xab, 0xcd];
        let view: &mut [u8] = &mut data[..];
        let field = BitField {
            index: 1,
            mask: 0x80,
        };
        field.set(view, 0xfe);
        assert_eq!(0xab, view[0]);
        assert_eq!(0x4d, view[1]);
    }

    #[test]
    fn u1_write1() {
        let mut data = [0xab, 0x7d];
        let view: &mut [u8] = &mut data[..];
        let field = BitField {
            index: 1,
            mask: 0x80,
        };
        field.set(view, 0x01);
        assert_eq!(0xab, view[0]);
        assert_eq!(0xfd, view[1]);
    }

    #[test]
    fn bitfield_write_invalid_adddress() {
        let mut data = [0xab];
        let view: &mut [u8] = &mut data[..];
        let field = BitField {
            index: 1,
            mask: 0xf0,
        };
        field.set(view, 0x01);
        assert_eq!(0xab, view[0]);
    }

    #[test]
    fn u16_write() {
        let mut data = [0xab, 0xcd];
        let view: &mut [u8] = &mut data[..];
        let field = BEU16Field { index: 0 };
        field.set(view, 0x1122);
        assert_eq!(0x11, view[0]);
        assert_eq!(0x22, view[1]);
    }

    #[test]
    fn u16_write_unaligned() {
        let mut data = [0xab, 0xcd, 0xef];
        let view: &mut [u8] = &mut data[..];
        let field = BEU16Field { index: 1 };
        field.set(view, 0x1122);
        assert_eq!(0xab, view[0]);
        assert_eq!(0x11, view[1]);
        assert_eq!(0x22, view[2]);
    }

    #[test]
    fn u16_write_invalid_address() {
        let mut data = [0xab, 0xcd];
        let view: &mut [u8] = &mut data[..];
        let field = BEU16Field { index: 1 };
        field.set(view, 0x1122);
        assert_eq!(0xab, view[0]);
        assert_eq!(0xcd, view[1]);
    }

    #[test]
    fn u32_write() {
        let mut data = [0xab, 0xcd, 0xef, 0x01];
        let view: &mut [u8] = &mut data[..];
        let field = BEU32Field { index: 0 };
        field.set(view, 0xabcdef01);
        assert_eq!(0xab, view[0]);
        assert_eq!(0xcd, view[1]);
        assert_eq!(0xef, view[2]);
        assert_eq!(0x01, view[3]);
    }

    #[test]
    fn u32_write_unaligned() {
        let mut data = [0x00, 0xab, 0xcd, 0xef, 0x01];
        let view: &mut [u8] = &mut data[..];
        let field = BEU32Field { index: 1 };
        field.set(view, 0xabcdef01);
        assert_eq!(0x00, view[0]);
        assert_eq!(0xab, view[1]);
        assert_eq!(0xcd, view[2]);
        assert_eq!(0xef, view[3]);
        assert_eq!(0x01, view[4]);
    }

    #[test]
    fn u32_write_invalid_address() {
        let mut data = [0x11, 0x22];
        let view: &mut [u8] = &mut data[..];
        let field = BEU16Field { index: 1 };
        field.set(view, 0xabcd);
        assert_eq!(0x11, view[0]);
        assert_eq!(0x22, view[1]);
    }

}
