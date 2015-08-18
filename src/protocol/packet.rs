use std::cmp;

#[derive(Debug,Copy,Clone)]
pub struct PacketData<'d> {
    data: &'d [u8],
}


#[derive(Debug)]
pub struct Packet<'d> {
    root: PacketData<'d>,

    // For parsing: where does the next block start?
    next: usize,

    // TODO: Track the domain names in the packet here, for common compression.
}

#[derive(Debug,Copy,Clone)]
pub struct Piece<'d> {
    pub root: PacketData<'d>,
    start: usize,
    end: usize,
}

impl<'d> Packet<'d> {
    pub fn new(data: &'d [u8]) -> Packet<'d> {
        Packet{root: PacketData{data: data}, next: 0}
    }

    pub fn next<'a, B: Block<'d, B>>(&'a mut self) -> B {
        let at = self.next;
        B::at(self, at)
    }

    pub fn next_slice(&mut self, length: usize) -> Piece<'d> {
        let at = self.next;
        self.next += length;
        self.data_slice(at, length)
    }
    pub fn consume_data_range(&mut self, start: usize, end: usize) {
        if end > self.next {
            self.next = end
        }
    }

    pub fn data_slice(&self, at: usize, length: usize) -> Piece<'d> {
        let start = cmp::min(at, self.root.data.len());
        let end = cmp::min(at+length, self.root.data.len());
        Piece{root: self.root, start: start, end: end}
    }

    pub fn data(&self) -> &'d[u8] { self.root.data }
}

impl<'d> Piece<'d> {
    pub fn data(&self) -> &'d[u8] { &self.root.data[self.start..self.end] }
    pub fn whole_packet(&self) -> &'d[u8] { self.root.data }
}

pub trait Block<'d, T: Block<'d, T>> {
    fn at<'p>(src: &'p mut Packet<'d>, at:usize) -> T;
}
