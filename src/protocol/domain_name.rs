use std::io::Write;
use std::ops::Range;
use std::vec::Vec;
use super::bits::BEU16Field;
use super::bits::BitData;
use super::bits::BitDataMut;
use super::bits::BitField;
use super::bits::HasLength;
use super::message::MessageCursor;


#[derive(Debug, Copy, Clone)]
pub struct DomainName {
    start: usize,
    end: usize,
    total_bytes: usize,
}

const TAG_MASK: u8 = 0b1100_0000u8;
const SEGMENT_TAG: u8 = 0b0000_0000u8;
const POINTER_TAG: u8 = 0b1100_0000u8;


impl DomainName {

    /// Returns a DomainName from message if the bytes are valid.
    pub fn from_message<'d, D: 'd + ?Sized + BitData>(message: &'d D,
                                                      at: usize)
        -> Option<DomainName> {
        // Consume the inline portion of the name from the message.
        let mut end = at;
        loop {
            match DomainName::parse_segment_at(message, end) {
                (Some(_), Some(next)) => {
                    end = next;
                }
                (Some(_), None) => {
                    end += 1;
                    break;
                } // Root: 1 octet
                (None, Some(_)) => {
                    end += 2;
                    break;
                } // Pointer: 2 octets
                _ => break,
            }
        }
        let mut name = DomainName {
            start: at,
            end: end,
            total_bytes: 0,
        };

        // Check if the value here is parsable.
        if let Some(segments) = name.segments(message) {
            name.total_bytes = segments.iter().fold(0, |a, &s| a + s.len() + 1);
            return Some(name);
        }
        None
    }

    pub fn max_encoding_size(&self) -> usize {
        self.total_bytes
    }

    pub fn segments<'d, D: 'd + ?Sized + BitData>(&self,
                                                  message: &'d D)
        -> Option<Vec<&'d <D as BitData>::Slice>> {
        // Allow at most 63 pointers. RFC: unbounded, but more pointers than segments
        // is is an
        // inefficient encoding.
        let mut level = 64;

        // Allow at most 63 name parts.
        let mut parts = 64;
        let mut pos = self.start;
        let mut segments = Vec::with_capacity(7);
        while level > 0 && parts > 0 {
            match DomainName::parse_segment_at(message, pos) {
                (Some(piece), Some(next)) => {
                    // Normal name part.
                    segments.push(piece);
                    parts -= 1;
                    pos = next;
                }
                (Some(piece), None) => {
                    // Root found. No more name parts.
                    segments.push(piece);
                    return Some(segments);
                }
                (None, Some(next)) => {
                    // Pointer. Resume at some random other point in the message.
                    level -= 1;
                    pos = next;
                }
                (None, None) => return None, // Invalid segment.
            }
        }
        // overflow in pointer or part count.
        return None;
    }

    fn parse_segment_at<'d, D: 'd + ?Sized + BitData>
                                                      (message: &'d D,
                                                       pos: usize)
        -> (Option<&'d <D as BitData>::Slice>, Option<usize>) {
        // The first two bits on the segment header octet are a type tag.
        // TODO check that this produces reasonable assembly.
        match (BitField {
                   index: pos,
                   mask: 0xff,
               }
               .get(message)) {
            // End marker: 0 octet. Valid name.
            Some(0) => return (message.get_range(Range {
                start: pos,
                end: pos,
            }),
                               None),
            Some(segment) if SEGMENT_TAG == TAG_MASK & segment => {
                // The next 6 bits are the size of this segment.
                let len = (segment & !TAG_MASK) as usize;
                let start = pos + 1;
                let end = start + len;
                return (message.get_range(Range {
                    start: start,
                    end: end,
                }),
                        Some(end));
            }
            Some(pointer) if POINTER_TAG == TAG_MASK & pointer => {
                // Jump to the pointed-to byte in the message.
                // The next 14 bits are the offset from the beginning of the message.
                let high = pointer & !TAG_MASK;
                let ptr = match (BitField {
                                     index: pos + 1,
                                     mask: 0xff,
                                 }
                                 .get(message)) {
                    Some(low) => Some(((high as usize) << 8) + (low as usize)),
                    None => None,
                };
                return (None, ptr);
            }

            // Unknown tag or pos is outside the message. Invalid message.
            _ => return (None, None),
        }
    }

    pub fn end_offset(&self) -> usize {
        self.end
    }

    // TODO figure out how to make SliceMut cover IndexMut<usize> and
    // IndexMut<Range> simultaneously so this doesn't require SliceMut
    // to be [u8].
    pub fn write_at<'a, 'b, 'c, 'd, D: 'd + ?Sized>(idx: &'a mut MessageCursor,
                                                    data: &'d mut D,
                                                    name: &'c [&'b [u8]])
        -> Option<DomainName>
        where D: BitDataMut<SliceMut = [u8]>,
              D: BitData<Slice = [u8]>
    {
        // If name ends in a root token, ignore it.
        let name_len = name.len() -
                       match name.last() {
            Some(tail) if tail.len() == 0 => 1,
            _ => 0,
        };

        let start = idx.tell();
        for i in 0..name_len {
            let suffix = &name[i..name_len];
            match idx.lookup_name_suffix(data, suffix) {
                Some(mut offset) => {
                    // Suffix is already in the message. Write out a pointer to it.
                    if let Some(ptr_idx) = idx.alloc(2) {
                        offset |= (POINTER_TAG as u16) << 8;
                        BEU16Field { index: ptr_idx.start }.set(data, offset);
                        break;
                    } else {
                        // No more space in the buffer.
                        return None;
                    }
                }
                None => {
                    // Write out the next segment.
                    let segment_data = name[i];
                    if segment_data.len() > 63 {
                        // Invalid name
                        return None;
                    }
                    if let Some(segment_idx) = idx.alloc(1 + segment_data.len()) {
                        idx.register_name_suffix(segment_idx.clone().start, suffix);
                        if let Some(ref mut segment) = data.get_mut_range(segment_idx) {
                            segment[0] = segment_data.len() as u8;
                            // clone_from_slice?
                            (&mut segment[1..segment_data.len() + 1]).write(segment_data).unwrap();
                        }
                    } else {
                        // No more space in the buffer.
                        return None;
                    }
                }
            }
        }
        // Append a root segment
        if let Some(segment_idx) = idx.alloc(1) {
            if let Some(ref mut segment) = data.get_mut_range(segment_idx) {
                segment[0] = 0;
            }
        } else {
            return None;
        }
        return DomainName::from_message(data, start);
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::vec::Vec;
    use std::iter::repeat;
    use super::super::message::MessageCursor;

    #[test]
    fn root() {
        let data = &[0][..];
        let name = DomainName::from_message(data, 0).unwrap();
        let v = name.segments(data).unwrap();
        assert_eq!(1, v.len());
        assert_eq!(0, v[0].len());
    }

    #[test]
    fn doubleroot() {
        let data = &[0, 0][..];
        let name = DomainName::from_message(data, 0).unwrap();
        let v = name.segments(data).unwrap();
        assert_eq!(1, v.len());
        assert_eq!(0, v[0].len());

        let name2 = DomainName::from_message(data, name.end_offset()).unwrap();
        let v2 = name2.segments(data).unwrap();
        assert_eq!(1, v2.len());
        assert_eq!(0, v2[0].len());
    }

    #[test]
    fn after_root() {
        let data = &[0, 1, 'x' as u8, 0][..];
        let name = DomainName::from_message(data, 0).unwrap();
        let v = name.segments(data).unwrap();
        assert_eq!(1, v.len());
        assert_eq!(0, v[0].len());

        let name2 = DomainName::from_message(data, name.end_offset()).unwrap();
        let v2 = name2.segments(data).unwrap();
        assert_eq!(2, v2.len());
        assert_eq!(&['x' as u8], v2[0]);
        assert_eq!(0, v2[1].len());
    }


    #[test]
    fn only_tld() {
        let data = &[3, 'c' as u8, 'o' as u8, 'm' as u8, 0][..];
        let name = DomainName::from_message(data, 0).unwrap();
        let v = name.segments(data).unwrap();
        assert_eq!(2, v.len());
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v[0]);
        assert_eq!(0, v[1].len());
    }

    #[test]
    fn two_parts() {
        let data = &[1, 'x' as u8, 3, 'c' as u8, 'o' as u8, 'm' as u8, 0][..];
        let name = DomainName::from_message(data, 0).unwrap();
        let v = name.segments(data).unwrap();
        assert_eq!(3, v.len());
        assert_eq!(&['x' as u8], v[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v[1]);
        assert_eq!(0, v[2].len());
    }

    #[test]
    fn initial_pointer() {
        let data = &[0xc0, 0x04, 1, 'x' as u8, 3, 'c' as u8, 'o' as u8, 'm' as u8, 0][..];

        let name1 = DomainName::from_message(data, 0).unwrap();
        let v1 = name1.segments(data).unwrap();
        assert_eq!(2, v1.len());
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v1[0]);
        assert_eq!(0, v1[1].len());

        let name2 = DomainName::from_message(data, name1.end_offset()).unwrap();
        let v2 = name2.segments(data).unwrap();
        assert_eq!(3, v2.len());
        assert_eq!(&['x' as u8], v2[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v2[1]);
        assert_eq!(0, v2[2].len());
    }

    #[test]
    fn trailing_pointer() {
        let data = &[1, 'y' as u8, 0xc0, 0x06, 1, 'x' as u8, 3, 'c' as u8, 'o' as u8, 'm' as u8,
                     0][..];

        let name1 = DomainName::from_message(data, 0).unwrap();
        let v1 = name1.segments(data).unwrap();
        assert_eq!(3, v1.len());
        assert_eq!(&['y' as u8], v1[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v1[1]);
        assert_eq!(0, v1[2].len());

        let name2 = DomainName::from_message(data, name1.end_offset()).unwrap();
        let v2 = name2.segments(data).unwrap();
        assert_eq!(3, v2.len());
        assert_eq!(&['x' as u8], v2[0]);
        assert_eq!(&['c' as u8, 'o' as u8, 'm' as u8], v2[1]);
        assert_eq!(0, v2[2].len());
    }

    #[test]
    fn invalid_pointer() {
        let data = &[0xc0, 5][..];
        assert!(DomainName::from_message(data, 0).is_none());
    }

    #[test]
    fn pointer_recursion_limit() {
        let data = &[0xc0, 0, 1, 'x' as u8, 0][..];
        assert!(DomainName::from_message(data, 0).is_none());
    }

    #[test]
    fn name_count_limit() {
        let data = &[1, 'x' as u8, 1, 'y' as u8, 0xc0, 0][..];
        assert!(DomainName::from_message(data, 0).is_none());
    }


    #[test]
    fn write_at_no_root() {
        let buffer = &mut repeat(0u8).take(8).collect::<Vec<u8>>();
        let idx = &mut MessageCursor::new(buffer.len());
        idx.alloc(1); // Skip the first byte to see if write_at goes outside its bounds.
        DomainName::write_at(idx, buffer, &[&[1u8, 2u8][..], &[3u8][..]][..]);
        assert_eq!(&vec![0u8, 2, 1, 2, 1, 3, 0, 0], buffer);
    }

    #[test]
    fn write_at_with_root() {
        let buffer = &mut repeat(0u8).take(8).collect::<Vec<u8>>();
        let idx = &mut MessageCursor::new(buffer.len());
        idx.alloc(1); // Skip the first byte to see if write_at goes outside its bounds.
        DomainName::write_at(idx, buffer, &[&[1u8, 2u8][..], &[3u8][..], &[][..]][..]);
        assert_eq!(&vec![0u8, 2, 1, 2, 1, 3, 0, 0], buffer);
    }

// TODO: test write_at with compression, once MessageCursor supports it.
}
