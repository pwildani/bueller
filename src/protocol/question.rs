use std::fmt;
use super::bits::BEU16Field;
use super::bits::BitData;
use super::domain_name::DomainName;
use super::message::MessageCursor;
use std::ops::Range;

pub const TYPE: BEU16Field = BEU16Field { index: 0 };
pub const CLASS: BEU16Field = BEU16Field { index: 2 };
pub const SIZE: usize = 4;

pub const QTYPE_A: u16 = 1;
pub const QTYPE_NS: u16 = 2;

pub const QTYPE_MD: u16 = 3;
pub const QTYPE_MF: u16 = 4;
pub const QTYPE_CNAME: u16 = 5;
pub const QTYPE_SOA: u16 = 6;
pub const QTYPE_MB: u16 = 7;
pub const QTYPE_MG: u16 = 8;
pub const QTYPE_MR: u16 = 9;
pub const QTYPE_NULL: u16 = 10;
pub const QTYPE_WKS: u16 = 11;
pub const QTYPE_PTR: u16 = 12;
pub const QTYPE_HINFO: u16 = 13;
pub const QTYPE_MINFO: u16 = 14;
pub const QTYPE_MX: u16 = 15;
pub const QTYPE_TXT: u16 = 16;

#[derive(Clone)]
pub struct Question<'d> {
    name: DomainName,
    footer: &'d [u8],
}

impl<'d> Question<'d> {
    pub fn name<'a>(&'a self) -> Option<&'a DomainName> {
        Some(&self.name)
    }
    pub fn qtype(&self) -> Option<u16> {
        TYPE.get(self.footer)
    }
    pub fn qclass(&self) -> Option<u16> {
        CLASS.get(self.footer)
    }
    pub fn from_message<D: ?Sized + BitData>(message: &'d D, at: usize) -> Option<Question<'d>>
        where D: BitData<Slice = [u8]>
    {
        if let Some(name) = DomainName::from_message(message, at) {
            if let Some(footer) = message.get_range(Range {
                start: name.end_offset(),
                end: name.end_offset() + SIZE,
            }) {
                return Some(Question {
                    name: name,
                    footer: footer,
                });
            }
        }
        None
    }

    pub fn end_offset(&self) -> usize {
        self.name.end_offset() + SIZE
    }

    pub fn estimate_response_size(&self) -> usize {
        self.name.max_encoding_size() + SIZE +
        match self.qtype() {
            Some(QTYPE_A) => 4,
            Some(QTYPE_NS) => 64,
            Some(QTYPE_MD) => 64,
            Some(QTYPE_MF) => 64,
            Some(QTYPE_CNAME) => 64,
            Some(QTYPE_SOA) => 64,
            Some(QTYPE_MB) => 64,
            Some(QTYPE_MR) => 64,
            Some(QTYPE_NULL) => 128,
            Some(QTYPE_WKS) => 128,
            Some(QTYPE_PTR) => 64,
            Some(QTYPE_HINFO) => 128,
            Some(QTYPE_MINFO) => 128,
            Some(QTYPE_MX) => 128,
            Some(QTYPE_TXT) => 512,
            _ => 128,
        }
    }
}

impl<'d> fmt::Debug for Question<'d> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Question")
           .field("name", &self.name())
           .field("qtype", &self.qtype())
           .field("qclass", &self.qclass())
           .finish()
    }
}

#[derive(Debug)]
pub struct QuestionMut<'d> {
    start: usize,
    name: DomainName,
    data: &'d mut [u8],
}

impl<'d> QuestionMut<'d> {
    pub fn at<'a, 'b, 'c>(idx: &'a mut MessageCursor,
                          data: &'d mut [u8],
                          name: &'b [&'c [u8]],
                          qtype: u16,
                          qclass: u16)
        -> Option<QuestionMut<'d>> {
        if let Some(name) = DomainName::write_at(idx, data, name) {
            if let Some(footer_idx) = idx.alloc(SIZE) {
                {
                    let footer = &mut data[footer_idx];
                    TYPE.set(footer, qtype);
                    CLASS.set(footer, qclass);
                }
                return Some(QuestionMut {
                    start: 0,
                    name: name,
                    data: data,
                });
            }
        }
        None
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn question() {
        let data = &[0, 0x1, 0x2, 0x3, 0x4][..];
        let q = Question::from_message(data, 0).unwrap();
        assert_eq!(Some(0x0102u16), q.qtype());
        assert_eq!(Some(0x0304u16), q.qclass());
    }

    #[test]
    fn question_missing_footer() {
        let data = &[0][..];
        let q = Question::from_message(data, 0);
        assert!(q.is_none());
    }
}
