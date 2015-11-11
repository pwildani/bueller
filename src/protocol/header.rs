use super::bits::BitField;
use super::bits::BitData;
use super::bits::BitDataMut;
use super::bits::BEU16Field;
use std::ops::Deref;
use std::fmt;
use super::message::MessageCursor;

const ID: BEU16Field = BEU16Field { index: 0 };

const QR: BitField = BitField {
    index: 2,
    mask: 0b1000_0000u8,
};
const OP: BitField = BitField {
    index: 2,
    mask: 0b0111_1000u8,
};
const AA: BitField = BitField {
    index: 2,
    mask: 0b0000_0100u8,
};
const TC: BitField = BitField {
    index: 2,
    mask: 0b0000_0010u8,
};
const RD: BitField = BitField {
    index: 2,
    mask: 0b0000_0001u8,
};

const RA: BitField = BitField {
    index: 3,
    mask: 0b1000_0000u8,
};
const Z: BitField = BitField {
    index: 3,
    mask: 0b0111_0000u8,
};
const RC: BitField = BitField {
    index: 3,
    mask: 0b0000_1111u8,
};

const QD: BEU16Field = BEU16Field { index: 4 };
const AN: BEU16Field = BEU16Field { index: 6 };
const NS: BEU16Field = BEU16Field { index: 8 };
const AR: BEU16Field = BEU16Field { index: 10 };

const SIZE: usize = 12;

pub const RC_OK: u8 = 0;
pub const RC_FORMAT_ERROR: u8 = 1;
pub const RC_SERVER_ERROR: u8 = 2;
pub const RC_NAME_ERROR: u8 = 3;
pub const RC_NOT_IMPLEMENTED: u8 = 4;
pub const RC_REFUSED: u8 = 5;

pub const OP_QUERY: u8 = 0;
pub const OP_IQUERY: u8 = 1;
pub const OP_STATUS: u8 = 2;

#[derive(Copy,Clone)]
pub struct Header<'d, D: 'd + ?Sized> {
    start: usize,
    data: &'d D,
}

pub struct HeaderMut<'d, D: 'd + ?Sized> {
    start: usize,
    data: &'d mut D,
}


impl<'d, D: 'd + ?Sized> Header<'d, D> where D: BitData {

    pub fn at(data: &'d D) -> Header<'d, D> {
        Header {
            start: 0,
            data: data,
        }
    }

    pub fn id(&self) -> Option<u16> {
        ID.get(self.data)
    }
    pub fn qr(&self) -> Option<bool> {
        QR.nonzero(self.data)
    }
    pub fn op(&self) -> Option<u8> {
        OP.get(self.data)
    }
    pub fn aa(&self) -> Option<bool> {
        AA.nonzero(self.data)
    }
    pub fn tc(&self) -> Option<bool> {
        TC.nonzero(self.data)
    }
    pub fn rd(&self) -> Option<bool> {
        RD.nonzero(self.data)
    }
    pub fn ra(&self) -> Option<bool> {
        RA.nonzero(self.data)
    }
    pub fn z(&self) -> Option<u8> {
        Z.get(self.data)
    }
    pub fn rc(&self) -> Option<u8> {
        RC.get(self.data)
    }
    pub fn qd(&self) -> Option<u16> {
        QD.get(self.data)
    }
    pub fn an(&self) -> Option<u16> {
        AN.get(self.data)
    }
    pub fn ns(&self) -> Option<u16> {
        NS.get(self.data)
    }
    pub fn ar(&self) -> Option<u16> {
        AR.get(self.data)
    }

    pub fn is_query(&self) -> bool {
        self.qr() == Some(false)
    }
    pub fn is_response(&self) -> bool {
        self.qr() == Some(true)
    }
    pub fn is_truncated(&self) -> bool {
        self.tc() == Some(true)
    }

    // TODO questions iterator
    // TODO answers iterator
    // TODO nameservers iterator
    // TODO additional records iterator

    pub fn end_offset(&self) -> usize {
        self.start + SIZE
    }
}

impl<'d, D: 'd + ?Sized> HeaderMut<'d, D> where D: BitDataMut {
    pub fn at_raw(data: &'d mut D) -> HeaderMut<'d, D> {
        HeaderMut {
            start: 0,
            data: data,
        }
    }
    pub fn at(idx: &mut MessageCursor, data: &'d mut D) -> Option<HeaderMut<'d, D>> {
        if let Some(at) = idx.alloc(SIZE) {
            return Some(HeaderMut {
                start: at.start,
                data: data,
            });
        }
        None
    }

    pub fn set_id(&mut self, val: u16) -> &mut Self {
        ID.set(self.data, val);
        self
    }
    pub fn set_qr(&mut self, val: bool) -> &mut Self {
        QR.set(self.data, val as u8);
        self
    }
    pub fn set_op(&mut self, val: u8) -> &mut Self {
        OP.set(self.data, val);
        self
    }
    pub fn set_aa(&mut self, val: bool) -> &mut Self {
        AA.set(self.data, val as u8);
        self
    }
    pub fn set_tc(&mut self, val: bool) -> &mut Self {
        TC.set(self.data, val as u8);
        self
    }
    pub fn set_rd(&mut self, val: bool) -> &mut Self {
        RD.set(self.data, val as u8);
        self
    }
    pub fn set_ra(&mut self, val: bool) -> &mut Self {
        RA.set(self.data, val as u8);
        self
    }
    pub fn set_rc(&mut self, val: u8) -> &mut Self {
        RC.set(self.data, val);
        self
    }
    pub fn set_qd(&mut self, val: u16) -> &mut Self {
        QD.set(self.data, val);
        self
    }
    pub fn set_an(&mut self, val: u16) -> &mut Self {
        AN.set(self.data, val);
        self
    }
    pub fn set_ns(&mut self, val: u16) -> &mut Self {
        NS.set(self.data, val);
        self
    }
    pub fn set_ar(&mut self, val: u16) -> &mut Self {
        AR.set(self.data, val);
        self
    }

    pub fn end_offset(&self) -> usize {
        self.start + SIZE
    }

    pub fn allow_recursion(&mut self) -> &mut Self {
        self.set_rd(true)
    }

    pub fn make_query(&mut self, id: u16) -> &mut Self {
        self.set_id(id)
            .set_qr(false)
            .set_op(OP_QUERY)
            .set_aa(false)
            .set_tc(false)
            .allow_recursion()
            .set_ra(false)
            .set_rc(0)
    }
}

impl<'d, D: 'd + ?Sized> fmt::Debug for Header<'d, D> where D: BitData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Header")
           .field("id", &self.id())
           .field("qr", &self.qr())
           .field("op", &self.op())
           .field("aa", &self.aa())
           .field("tc", &self.tc())
           .field("rd", &self.rd())
           .field("rc", &self.rc())
           .field("qd", &self.qd())
           .field("an", &self.an())
           .field("ns", &self.ns())
           .field("ar", &self.ar())
           .field("is_query", &self.is_query())
           .field("is_response", &self.is_response())
           .field("is_truncated", &self.is_truncated())
           .finish()
    }
}


impl<'d, D: 'd> HeaderMut<'d, D> where D: BitDataMut {
    fn as_header(&'d self) -> Header<'d, D> {
        let readonly: &D = &self.data;
        Header {
            start: self.start,
            data: readonly,
        }
    }
}



impl<'d, D: 'd> fmt::Debug for HeaderMut<'d, D> where D: BitDataMut {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let h = self.as_header();
        fmt.debug_struct("HeaderMut")
           .field("id", &h.id())
           .field("qr", &h.qr())
           .field("op", &h.op())
           .field("aa", &h.aa())
           .field("tc", &h.tc())
           .field("rd", &h.rd())
           .field("rc", &h.rc())
           .field("qd", &h.qd())
           .field("an", &h.an())
           .field("ns", &h.ns())
           .field("ar", &h.ar())
           .field("is_query", &h.is_query())
           .field("is_response", &h.is_response())
           .field("is_truncated", &h.is_truncated())
           .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id() {
        let data: &[u8] = &[0xab, 0xcd];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0xabcdu16), h.id());

        let data: &[u8] = &[0xab];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.id());
    }

    #[test]
    fn query_or_response() {
        let data: &[u8] = &[0, 0, 0x00];
        let h = Header::at(&data[..]);
        assert_eq!(Some(false), h.qr());
        assert_eq!(true, h.is_query());
        assert_eq!(false, h.is_response());

        let data: &[u8] = &[0, 0, 0x80];
        let h = Header::at(&data[..]);
        assert_eq!(Some(true), h.qr());
        assert_eq!(false, h.is_query());
        assert_eq!(true, h.is_response());

        let data: &[u8] = &[0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.qr());
        assert_eq!(false, h.is_query());
        assert_eq!(false, h.is_response());
    }

    #[test]
    fn operation() {
        let data: &[u8] = &[0, 0, 0x78];
        let h = Header::at(&data[..]);
        assert_eq!(Some(15), h.op());

        let data: &[u8] = &[0, 0, 0x18];
        let h = Header::at(&data[..]);
        assert_eq!(Some(3), h.op());

        let data: &[u8] = &[0, 0, 0x10];
        let h = Header::at(&data[..]);
        assert_eq!(Some(OP_STATUS), h.op());

        let data: &[u8] = &[0, 0, 0x08];
        let h = Header::at(&data[..]);
        assert_eq!(Some(OP_IQUERY), h.op());

        let data: &[u8] = &[0, 0, 0x87];
        let h = Header::at(&data[..]);
        assert_eq!(Some(OP_QUERY), h.op());
    }

    #[test]
    fn authoritative() {
        let data: &[u8] = &[0, 0, 0x04];
        let h = Header::at(&data[..]);
        assert_eq!(Some(true), h.aa());

        let data: &[u8] = &[0, 0, 0xfb];
        let h = Header::at(&data[..]);
        assert_eq!(Some(false), h.aa());

        let data: &[u8] = &[0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.aa());
    }

    #[test]
    fn truncated() {
        let data: &[u8] = &[0, 0, 0x02];
        let h = Header::at(&data[..]);
        assert_eq!(Some(true), h.tc());

        let data: &[u8] = &[0, 0, 0xfc];
        let h = Header::at(&data[..]);
        assert_eq!(Some(false), h.tc());

        let data: &[u8] = &[0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.tc());
    }

    #[test]
    fn please_recurse() {
        let data: &[u8] = &[0, 0, 0x01];
        let h = Header::at(&data[..]);
        assert_eq!(Some(true), h.rd());

        let data: &[u8] = &[0, 0, 0xfe];
        let h = Header::at(&data[..]);
        assert_eq!(Some(false), h.rd());

        let data: &[u8] = &[0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.rd());
    }

    #[test]
    fn recursion_available() {
        let data: &[u8] = &[0, 0, 0, 0x80];
        let h = Header::at(&data[..]);
        assert_eq!(Some(true), h.ra());

        let data: &[u8] = &[0, 0, 0, 0x7f];
        let h = Header::at(&data[..]);
        assert_eq!(Some(false), h.ra());

        let data: &[u8] = &[0, 0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.ra());
    }

    #[test]
    fn response_code() {
        let data: &[u8] = &[0, 0, 0, 0x00];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0), h.rc());

        let data: &[u8] = &[0, 0, 0, 0x0f];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0xf), h.rc());

        let data: &[u8] = &[0, 0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.rc());
    }

    #[test]
    fn query_count() {
        let data: &[u8] = &[0, 0, 0, 0, 0xab, 0xcd];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0xabcd), h.qd());

        let data: &[u8] = &[0, 0, 0, 0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.qd());
    }

    #[test]
    fn answer_count() {
        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0xab, 0xcd];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0xabcd), h.an());

        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.an());
    }

    #[test]
    fn name_response_count() {
        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0xab, 0xcd];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0xabcd), h.ns());

        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.ns());
    }

    #[test]
    fn additional_record_count() {
        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xab, 0xcd];
        let h = Header::at(&data[..]);
        assert_eq!(Some(0xabcd), h.ar());

        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let h = Header::at(&data[..]);
        assert_eq!(None, h.ar());
    }

    #[test]
    fn set_id() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_id(0xabcd);
        let h = Header::at(data);
        assert_eq!(Some(0xabcd), h.id());
    }

    #[test]
    fn set_qr() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_qr(true);
        let h = Header::at(data);
        assert_eq!(Some(true), h.qr());
    }

    #[test]
    fn set_op() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_op(0xff);
        let h = Header::at(data);
        assert_eq!(Some(0xf), h.op());
    }

    #[test]
    fn set_aa() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_aa(true);
        let h = Header::at(data);
        assert_eq!(Some(true), h.aa());
    }

    #[test]
    fn set_tc() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_tc(true);
        let h = Header::at(data);
        assert_eq!(Some(true), h.tc());
    }

    #[test]
    fn set_rd() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_rd(true);
        let h = Header::at(data);
        assert_eq!(Some(true), h.rd());
    }

    #[test]
    fn set_ra() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_ra(true);
        let h = Header::at(data);
        assert_eq!(Some(true), h.ra());
    }

    #[test]
    fn set_rc() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_rc(0xff);
        let h = Header::at(data);
        assert_eq!(Some(0xf), h.rc());
    }

    #[test]
    fn set_qd() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_qd(0xabcd);
        let h = Header::at(data);
        assert_eq!(Some(0xabcd), h.qd());
    }

    #[test]
    fn set_an() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_an(0xabcd);
        let h = Header::at(data);
        assert_eq!(Some(0xabcd), h.an());
    }

    #[test]
    fn set_ns() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_ns(0xabcd);
        let h = Header::at(data);
        assert_eq!(Some(0xabcd), h.ns());
    }

    #[test]
    fn set_ar() {
        let data: &mut Vec<u8> = &mut vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        HeaderMut::at_raw(data).set_ar(0xabcd);
        let h = Header::at(data);
        assert_eq!(Some(0xabcd), h.ar());
    }
}
