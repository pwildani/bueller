use super::bits::BitField;
use super::bits::U16Field;
use super::packet::Block;
use protocol::packet::Packet;
use protocol::packet::Piece;
use std::fmt;

#[derive(Copy,Clone)]
pub struct Header<'d> {
    data: Piece<'d>
}

impl<'d> Header<'d> {
    const ID:U16Field = U16Field{index:0};

    const QR:BitField = BitField{index:2, mask: 0b1000_0000u8};
    const OP:BitField = BitField{index:2, mask: 0b0111_1000u8};
    const AA:BitField = BitField{index:2, mask: 0b0000_0100u8};
    const TC:BitField = BitField{index:2, mask: 0b0000_0010u8};
    const RD:BitField = BitField{index:2, mask: 0b0000_0001u8};

    const RA:BitField = BitField{index:3, mask: 0b1000_0000u8};
    const RC:BitField = BitField{index:3, mask: 0b0000_1111u8};

    const QD:U16Field = U16Field{index:4};
    const AN:U16Field = U16Field{index:6};
    const NS:U16Field = U16Field{index:8};
    const AR:U16Field = U16Field{index:10};

    const SIZE:usize = 12;

    pub const RC_OK:u8 = 0;
    pub const RC_FORMAT_ERROR:u8 = 1;
    pub const RC_SERVER_ERROR:u8 = 2;
    pub const RC_NAME_ERROR:u8 = 3;
    pub const RC_NOT_IMPLEMENTED:u8 = 4;
    pub const RC_REFUSED:u8 = 5;

    pub const OP_QUERY:u8 = 0;
    pub const OP_IQUERY:u8 = 1;
    pub const OP_STATUS:u8 = 2;

    pub fn id(&self) -> Option<u16>  {Header::ID.get(self.data.data())}
    pub fn qr(&self) -> Option<bool> {Header::QR.nonzero(self.data.data())}
    pub fn op(&self) -> Option<u8>   {Header::OP.get(self.data.data())}
    pub fn aa(&self) -> Option<bool> {Header::AA.nonzero(self.data.data())}
    pub fn tc(&self) -> Option<bool> {Header::TC.nonzero(self.data.data())}
    pub fn rd(&self) -> Option<bool> {Header::RD.nonzero(self.data.data())}
    pub fn ra(&self) -> Option<bool> {Header::RA.nonzero(self.data.data())}
    pub fn rc(&self) -> Option<u8>   {Header::RC.get(self.data.data())}
    pub fn qd(&self) -> Option<u16>  {Header::QD.get(self.data.data())}
    pub fn an(&self) -> Option<u16>  {Header::AN.get(self.data.data())}
    pub fn ns(&self) -> Option<u16>  {Header::NS.get(self.data.data())}
    pub fn ar(&self) -> Option<u16>  {Header::AR.get(self.data.data())}

    pub fn is_query(&self) -> bool { self.qr() == Some(false) }
    pub fn is_response(&self) -> bool { self.qr() == Some(true) }
    pub fn is_truncated(&self) -> bool { self.tc() == Some(true) }
}

impl<'d> fmt::Debug for Header<'d> {
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

impl<'d> Block<'d, Header<'d>> for Header<'d> {
    fn at(src: &mut Packet<'d>, at:usize) -> Header<'d> {
        Header{data: src.next_slice(Header::SIZE)}
    }
}

#[cfg(test)]
mod tests {
    use super::Header;
    use protocol::packet::Packet;

    #[test]
    fn id() {
        let data = [0xab, 0xcd];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0xabcdu16), h.id());

        let data = [0xab];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.id());
    }

    #[test]
    fn query_or_response() {
        let data = [0,0,0x00];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(false), h.qr());
        assert_eq!(true, h.is_query());
        assert_eq!(false, h.is_response());

        let data = [0,0,0x80];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(true), h.qr());
        assert_eq!(false, h.is_query());
        assert_eq!(true, h.is_response());

        let data = [0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.qr());
        assert_eq!(false, h.is_query());
        assert_eq!(false, h.is_response());
    }

    #[test]
    fn operation() {
        let data = [0,0,0x78];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(15), h.op());

        let data = [0,0,0x18];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(3), h.op());
        
        let data = [0,0,0x10];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(Header::OP_STATUS), h.op());

        let data = [0,0,0x08];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(Header::OP_IQUERY), h.op());

        let data = [0,0,0x87];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(Header::OP_QUERY), h.op());
    }

    #[test]
    fn authoritative() {
        let data = [0,0,0x04];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(true), h.aa());

        let data = [0,0,0xfb];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(false), h.aa());

        let data = [0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.aa());
    }

    #[test]
    fn truncated() {
        let data = [0,0,0x02];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(true), h.tc());

        let data = [0,0,0xfc];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(false), h.tc());

        let data = [0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.tc());
    }

    #[test]
    fn please_recurse() {
        let data = [0,0,0x01];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(true), h.rd());

        let data = [0,0,0xfe];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(false), h.rd());

        let data = [0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.rd());
    }

    #[test]
    fn recursion_available() {
        let data = [0,0,0,0x80];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(true), h.ra());

        let data = [0,0,0,0x7f];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(false), h.ra());

        let data = [0,0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.ra());
    }

    #[test]
    fn response_code() {
        let data = [0,0,0,0x00];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0), h.rc());

        let data = [0,0,0,0x0f];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0xf), h.rc());

        let data = [0,0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.rc());
    }

    #[test]
    fn query_count() {
        let data = [0,0,0,0,0xab,0xcd];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0xabcd), h.qd());

        let data = [0,0,0,0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.qd());
    }

    #[test]
    fn answer_count() {
        let data = [0,0,0,0,0,0,0xab,0xcd];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0xabcd), h.an());

        let data = [0,0,0,0,0,0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.an());
    }

    #[test]
    fn name_response_count() {
        let data = [0,0,0,0,0,0,0,0,0xab,0xcd];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0xabcd), h.ns());

        let data = [0,0,0,0,0,0,0,0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.ns());
    }

    #[test]
    fn additional_record_count() {
        let data = [0,0,0,0,0,0,0,0,0,0,0xab,0xcd];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(Some(0xabcd), h.ar());

        let data = [0,0,0,0,0,0,0,0,0,0,0];
        let mut p = Packet::new(&data);
        let h = p.next::<Header>();
        assert_eq!(None, h.ar());
    }

}
