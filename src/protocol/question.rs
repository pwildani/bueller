use std::fmt;
use super::bits::U16Field;
use super::packet::Block;
use super::packet::Packet;
use super::packet::Piece;
use super::domain_name::DomainName;

const TYPE:U16Field = U16Field{index:0};
const CLASS:U16Field = U16Field{index:2};
const SIZE:usize = 4;


struct QuestionFooter<'d> {
    data: Piece<'d> 
}

pub struct Question<'d> {
    name: DomainName<'d>,
    footer: QuestionFooter<'d>,
}

impl<'d> QuestionFooter<'d> {
    fn qtype(&self) -> Option<u16> { TYPE.get(self.data.data()) }
    fn qclass(&self) -> Option<u16> { CLASS.get(self.data.data()) }
}

impl<'d> Question<'d> {
    pub fn name<'a>(&'a self) -> Option<&'a DomainName<'d>> { Some(&self.name) }
    pub fn qtype(&self) -> Option<u16> { self.footer.qtype() }
    pub fn qclass(&self) -> Option<u16> { self.footer.qclass() }
}

impl<'d> Block<'d, Question<'d>> for Question<'d> {
    fn at(src: &mut Packet<'d>, at:usize) -> Question<'d> {
        let name = src.next::<DomainName<'d>>();
        let footer = src.next::<QuestionFooter<'d>>();
        Question { name: name, footer: footer }
    }
}

impl<'d> Block<'d, QuestionFooter<'d>> for QuestionFooter<'d> {
    fn at<'p>(src: &'p mut Packet<'d>, at:usize) -> QuestionFooter<'d> {
        QuestionFooter { data: src.next_slice(SIZE) }
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


#[cfg(test)]
mod test {
    use protocol::packet::Packet;
    use super::Question;

    #[test]
    fn question() {
        let data = &[0, 0x1, 0x2, 0x3, 0x4];
        let mut p = Packet::new(data);
        let q = p.next::<Question>();
        assert_eq!(Some(0x0102u16), q.qtype());
        assert_eq!(Some(0x0304u16), q.qclass());
    }
    
    #[test]
    fn question_missing_footer() {
        let data = &[0];
        let mut p = Packet::new(data);
        let q = p.next::<Question>();
        assert_eq!(None, q.qtype());
        assert_eq!(None, q.qclass());
    }
}
