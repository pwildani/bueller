use std::fmt;
use super::bits::BitData;
use super::bits::BEU16Field;
use super::bits::BEU32Field;
use super::domain_name::DomainName;
use std::ops::Range;

const TYPE:BEU16Field = BEU16Field{index:0};
const CLASS:BEU16Field = BEU16Field{index:2};
const TTL:BEU32Field = BEU32Field{index:4};
const LENGTH:BEU16Field = BEU16Field{index:8};
const SIZE:usize = 10;

pub struct Resource<'d> {
    name: DomainName<'d>,
    footer: &'d [u8],
}

impl<'d> Resource<'d> {
    pub fn name<'a>(&'a self) -> Option<&'a DomainName<'d>> { Some(&self.name) }
    pub fn payload_range(&self) -> Option<Range<usize>> { 
        if let Some(len) = self.data_length() {
            return Some(Range {
                start:self.name.end_offset() + SIZE,
                end: self.end_offset(),
            });
        }
        None
    }
    pub fn end_offset(&self) -> usize {
        self.name.end_offset() + SIZE + self.data_length().unwrap_or(0) as usize
    }

    pub fn payload<D:?Sized + BitData>(&self, message: &'d D) -> Option<&'d <D as BitData>::Slice> {
        if let Some(range) = self.payload_range() {
            return message.get_range(range);
        }
        None
    }

    pub fn rtype(&self) -> Option<u16> { TYPE.get(self.footer) }
    pub fn rclass(&self) -> Option<u16> { CLASS.get(self.footer) }
    pub fn ttl(&self) -> Option<u32> { TTL.get(self.footer) }
    pub fn data_length(&self) -> Option<u16> { LENGTH.get(self.footer) }
}

impl<'d> Resource<'d> {
    pub fn from_message<D:'d + ?Sized + BitData<Slice=[u8]>>(message: &'d D, at:usize) -> Option<Resource<'d>> {
        if let Some(name) = DomainName::from_message(message, at) {
            if let Some(footer) = message.get_range(Range{start:name.end_offset(), end:name.end_offset() + SIZE}) {
                return Some(Resource { name: name, footer: footer })
            }
        }
        None
    }
}

impl<'d> fmt::Debug for Resource<'d> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Resource")
            .field("name", &self.name())
            .field("rtype", &self.rtype())
            .field("rclass", &self.rclass())
            .field("ttl", &self.ttl())
            .field("rdata length", &self.data_length())
            .finish()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn payload() {
        let data = &[0, 0, 1, 0, 2, 0, 0, 0, 3, 0, 2, 0xaa, 0xab][..];
        let r = Resource::from_message(data, 0).unwrap();
        assert_eq!(Some(1), r.rtype());
        assert_eq!(Some(2), r.rclass());
        assert_eq!(Some(3), r.ttl());
        assert_eq!(Some(2), r.data_length());

        // Truncated packet:
        assert_eq!(2, r.payload(data).unwrap().len());
        assert_eq!(0xaa, r.payload(data).unwrap()[0]);
        assert_eq!(0xab, r.payload(data).unwrap()[1]);
    }

    #[test]
    fn truncated_resource() {
        let data = &[0, 0, 1, 0, 2, 0, 0, 0, 3, 0, 4][..];
        let r = Resource::from_message(data, 0).unwrap();
        assert_eq!(Some(1), r.rtype());
        assert_eq!(Some(2), r.rclass());
        assert_eq!(Some(3), r.ttl());
        assert_eq!(Some(4), r.data_length());

        // Truncated packet:
        assert!(r.payload(data).is_none());
    }
    
}
