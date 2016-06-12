use std::ops::Range;
use super::bits::BitData;

#[derive(Debug, Copy, Clone)]
pub struct MessageCursor {
    next_byte: usize,
    limit: usize, // TODO index of name suffixes.
}

impl MessageCursor {
    pub fn new(limit: usize) -> MessageCursor {
        MessageCursor {
            next_byte: 0,
            limit: limit,
        }
    }

    pub fn tell(&self) -> usize {
        self.next_byte
    }

    pub fn alloc(&mut self, size: usize) -> Option<Range<usize>> {
        if size < self.limit && self.limit - size > self.next_byte {
            let start = self.next_byte;
            self.next_byte += size;
            return Some(Range {
                start: start,
                end: self.next_byte,
            });
        }
        None
    }

    pub fn register_name_suffix<'a, 'b, 'c>(&'a mut self, at: usize, suffix: &'b [&'c [u8]]) {
        // Not implemented
    }

    pub fn lookup_name_suffix<'a, 'b, 'c, 'd, D: 'd + ?Sized + BitData>(&'a mut self,
                                                                        data: &'d D,
                                                                        suffix: &'b [&'c [u8]])
        -> Option<u16> {
        // Not implemented
        None
    }

}
