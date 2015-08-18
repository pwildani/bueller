extern crate bueller;

use bueller::protocol::Packet;
use bueller::protocol::Header;
use bueller::protocol::Resource;
use bueller::protocol::Question;
use std::io;
use std::io::Read;

fn main() {
    let mut buffer = Vec::new();
    let num = io::stdin().read_to_end(&mut buffer).ok().unwrap();
    println!("Read {} bytes", num);

    let mut packet = Packet::new(&buffer[..]);
    let header = packet.next::<Header>();
    println!("Header: {:?}", header);
}
