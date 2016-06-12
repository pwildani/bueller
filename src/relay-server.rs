extern crate bueller;
extern crate mio;

use bueller::protocol::Packet;
use bueller::protocol::Header;
use bueller::protocol::Resource;
use bueller::protocol::Question;
use std::io;
use std::io::Read;
use std::net::SocketAddr;

const RESPONSE: mio::Token = mio::Token(0);
const QUERY: mio::Token = mio::Token(1);



struct Recurse {
    server: mio::udp::UdpListener,
    SocketAddr: upstream,
}

impl Recurse {
    fn new(server: UdpListener) -> Recurse {
        let upstream = "8.8.8.8:53".parse().unwrap();
        Recurse {
            server: server,
            upstream: upstream,
        }
    }
}


impl mio::Handler for Recurse {
    type Timeout = ();
    type Message = &[u8];
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
        server.send_to(message, self.upstream);
    }

    fn read(&mut self,
            event_loop: &mut mio::EventLoop<Recurse>,
            token: mio::Token,
            events: mio::EventSet) {
        match token {
            RESPONSE => {
                assert!(events.is_readable());
                let mut buffer = Vec::new();
                match self.server.recv_from(buf) {
                    Ok(addr) => {
                        println!("Got a response from {}", addr);
                        println!("{}", buffer);
                        event_loop.shutdown();
                    }
                    Err(e) => {
                        println!("recv_from() failed: {}", e);
                        event_loop.shutdown();
                    }
                }
            }
            t => {
                panic!("Invalid token: {}", t);
            }
        }
    }
}

fn main() {

    let address = "127.0.0.1:5300".parse().unwrap();
    let listener = mio::UdpListener::bind(&address).unwrap();
    let mut event_loop = mio::EventLoop::new().unwrap();

    let mut buffer = Vec::new();
    let num = io::stdin().read_to_end(&mut buffer).ok().unwrap();
    println!("Read {} bytes", num);

    event_loop.channel().send(&buffer[..]);

    event_loop.run(&mut Recurse { server: listener })
}
