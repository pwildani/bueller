use std::fmt;
use std::marker::PhantomData;
use std::slice;
use std::str;
use std::vec::Vec;
use super::bits::BitData;
use super::bits::BitField;
use std::ops::Range;

// TODO generalize segment storage over BitData::Slice.

pub struct DomainName<'d> {
    /// The next byte offset in the message after this name.
    end: usize,
    segments: Vec<&'d [u8]>,
}

const TAG:u8 = 0b1100_0000u8;
const SEGMENT:u8 = 0b0000_0000u8;
const POINTER:u8 = 0b1100_0000u8;


impl<'d> DomainName<'d> {
    fn from_message<D: ?Sized + BitData>(message: &'d D, at:usize) -> DomainName<'d>
        where D: BitData<Slice=[u8]> {
        // Consume the inline portion of the name from the message.
        let mut end = at;
        loop {
          match DomainName::parse_segment_at(message, end)  {
              (Some(_), Some(next)) => { end = next; },
              (Some(_), None) => { end += 1; break; }, // Root: 1 octet
              (None, Some(_))=> { end += 2; break; }, // Pointer: 2 octets
              _ => break,
          }
        }
        let mut name = DomainName{
            end: end,
            segments: Vec::with_capacity(6),
        };
        name.parse_segments(message, at);
        return name;
    }

    fn parse_segments<D: ?Sized + BitData>(&mut self, message: &'d D, start: usize) 
    where D: BitData<Slice=[u8]> {
        let mut level = 64; // Allow at most 64 pointers. (RFC: unbounded)
        let mut parts = 64; // Allow at most 64 name parts.
        let mut pos = start;
        while level > 0 && parts > 0 {
          match DomainName::parse_segment_at(message, pos)  {
              (Some(piece), Some(next)) => {
                  // Normal name part.
                  self.segments.push(piece);
                  parts -= 1;
                  pos = next;
              },
              (Some(piece), None) => {
                  // Root found. No more name parts.
                  self.segments.push(piece);
                  break
              },
              (None, Some(next)) => {
                  // Pointer. Resume at some random other point in the message.
                  level -= 1;
                  pos = next;
              }
              (None, None) => break, // Invalid segment.
          }
        }
    }

    fn parse_segment_at<D: ?Sized + BitData>(message: &'d D, pos: usize) -> (Option<&'d [u8]>, Option<usize>) 
    where D: BitData<Slice=[u8]> {
        // The first two bits on the segment header octet are a type tag.
        // TODO check that this produces reasonable assembly.
        match (BitField{index:pos, mask:0xff}.get(message)) {
            // End marker: 0 octet. Valid name.
            Some(0) => return (message.get_range(Range{start:pos, end:pos}), None),
            Some(segment) if SEGMENT == TAG & segment => {
                // The next 6 bits are the size of this segment.
                let len = (segment & !TAG) as usize;
                let start = pos + 1;
                let end = start + len;
                return (message.get_range(Range{start:start, end:end}), Some(end));
            },
            Some(pointer) if POINTER == TAG & pointer => {
                // Jump to the pointed-to byte in the message.
                // The next 14 bits are the offset from the beginning of the message.
                let high = pointer & !TAG;
                let ptr = match (BitField{index:pos+1,mask:0xff}.get(message)) {
                    Some(low) => Some(((high as usize) << 8) + (low as usize)),
                    None => None,
                };
                return (None, ptr);
            },

            // Unknown tag or pos is outside the message. Invalid message.
            _ => return (None, None)
        }
    }

    pub fn valid(&self) -> bool {
        // Ends in a root token.
        return 0 == self.segments[self.segments.len() - 1].len();
    }

    pub fn iter<'a>(&'a self) -> slice::Iter<&'a [u8]> {
        self.segments.iter()
    }

    pub fn end_offset(&self) -> usize { self.end }
}

impl<'d> fmt::Debug for DomainName<'d> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DomainName")
            .field("part count", &self.segments.len())
            .field("parts", &self.segments)
            .finish()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::vec::Vec;
    use std::iter::FromIterator;

    #[test]
    fn root() {
        let data = &[0][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(1, v.len());
        assert_eq!(0, v[0].len());
    }

    #[test]
    fn doubleroot() {
        let data = &[0, 0][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(1, v.len());
        assert_eq!(0, v[0].len());

        let name2 = DomainName::from_message(data, name.end_offset());
        let v2 = Vec::from_iter(name2.iter());
        assert_eq!(1, v2.len());
        assert_eq!(0, v2[0].len());
    }

    #[test]
    fn after_root() {
        let data = &[0, 1, 'x' as u8, 0][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(1, v.len());
        assert_eq!(0, v[0].len());

        let name2 = DomainName::from_message(data, name.end_offset());
        let v2 = Vec::from_iter(name2.iter());
        assert_eq!(2, v2.len());
        assert_eq!(&['x' as u8], v2[0]);
        assert_eq!(0, v2[1].len());
    }


    #[test]
    fn only_tld() {
        let data = &[3, 'c' as u8, 'o' as u8, 'm' as u8, 0][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(2, v.len());
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v[0]);
        assert_eq!(0, v[1].len());
    }

    #[test]
    fn two_parts() {
        let data = &[1,'x' as u8, 3, 'c' as u8, 'o' as u8, 'm' as u8, 0][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(3, v.len());
        assert_eq!(&['x' as u8], v[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v[1]);
        assert_eq!(0, v[2].len());
    }

    #[test]
    fn initial_pointer() {
        let data = &[0xc0, 0x04, 1, 'x' as u8, 3, 'c' as u8, 'o' as u8, 'm' as u8, 0][..];

        let name1 = DomainName::from_message(data, 0);
        let v1 = Vec::from_iter(name1.iter());
        assert_eq!(2, v1.len());
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v1[0]);
        assert_eq!(0, v1[1].len());

        let name2 = DomainName::from_message(data, name1.end_offset());
        let v2 = Vec::from_iter(name2.iter());
        assert_eq!(3, v2.len());
        assert_eq!(&['x' as u8], v2[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v2[1]);
        assert_eq!(0, v2[2].len());
    }

    #[test]
    fn trailing_pointer() {
        let data = &[
            1, 'y' as u8,
            0xc0, 0x06,
            1, 'x' as u8,
            3, 'c' as u8, 'o' as u8, 'm' as u8,
            0][..];

        let name1 = DomainName::from_message(data, 0);
        let v1 = Vec::from_iter(name1.iter());
        assert_eq!(3, v1.len());
        assert_eq!(&['y' as u8], v1[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v1[1]);
        assert_eq!(0, v1[2].len());

        let name2 = DomainName::from_message(data, name1.end_offset());
        let v2 = Vec::from_iter(name2.iter());
        assert_eq!(3, v2.len());
        assert_eq!(&['x' as u8], v2[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v2[1]);
        assert_eq!(0, v2[2].len());
    }


    #[test]
    fn pointer_recursion_limit() {
        let data = &[ 0xc0, 0, 1, 'x' as u8, 0 ][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(0, v.len());
    }

    #[test]
    fn name_count_limit() {
        let data = &[ 1, 'x' as u8, 1, 'y' as u8, 0xc0, 0][..];
        let name = DomainName::from_message(data, 0);
        let v = Vec::from_iter(name.iter());
        assert_eq!(64, v.len());
    }
}
