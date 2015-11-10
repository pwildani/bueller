use std::fmt;
use super::bits::BEU16Field;
use super::bits::BitData;
use super::domain_name::DomainName;
use super::message::MessageCursor;
use std::ops::Range;

const TYPE: BEU16Field = BEU16Field { index: 0 };
const CLASS: BEU16Field = BEU16Field { index: 2 };
const SIZE: usize = 4;



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
                          name: &'b Vec<&'c [u8]>,
                          qtype: u16,
                          qclass: u16)
                          -> Option<QuestionMut<'d>> {
        if let Some(name) = DomainName::write_at(idx, data, name) {
            let footer_start = name.end_offset();
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
