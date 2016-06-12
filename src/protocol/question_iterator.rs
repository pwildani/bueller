use std::iter::Iterator;
use super::Header;
use super::Question;
use super::bits::BitData;

pub struct QuestionIterator<'d, D: 'd> {
    header: &'d D,
    next: usize,
    remaining: u16,
}

impl<'d, D: 'd + ?Sized + BitData> Iterator<Question> for QuestionIterator<'d, D> {
    type Item = Question;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let query = Question::from_message(self.message, self.next);
        self.next = query.end_offset();

        Some(query)
    }
}

pub fn over<'a, 'd, D: 'd + ?Sized + BitData>(header: &'a Header<'d, D>) -> QuestionIterator<'d, D> {
    let qdcount = header.qdcount().unwrap_or(0);
    let next = header.end_offset();

    QuestionIterator {
        header: header,
        next: next,
        remaining: qdcount,
    }
}
