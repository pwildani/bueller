extern crate bueller;
extern crate mio;

use bueller::protocol::Header;
use bueller::protocol::Question;
use bueller::protocol::Resource;
use mio::udp::UdpSocket;
use std::io::Read;
use std::io;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::sync::Arc;

const RESPONSE: mio::Token = mio::Token(0);
const QUERY: mio::Token = mio::Token(1);



struct Recurse {
    server: UdpSocket,
    upstream: SocketAddr,
}

impl Recurse {
    fn new(server: UdpSocket) -> Recurse {
        let upstream = "8.8.8.8:53".parse().unwrap();
        Recurse {
            server: server,
            upstream: upstream,
        }
    }
}


impl mio::Handler for Recurse {
    type Timeout = ();
    type Message = Vec<u8>;

    fn ready(&mut self,
             event_loop: &mut mio::EventLoop<Recurse>,
             token: mio::Token,
             events: mio::EventSet) {
        println!("ready...");
        match token {
            RESPONSE => {
                if (events.is_readable()) {
                    let mut recv_buf = Vec::with_capacity(1024);
                    match self.server.recv_from(&mut recv_buf) {
                        Ok(addr) => {
                            let msg = &recv_buf[..];
                            println!("Got a response from {:?}", addr);
                            println!("{:?}", recv_buf);
                            let header = Header::at(msg);
                            println!("Header {:?}", &header);
                            let mut next = header.end_offset();
                            for qdcount in header.qd() {
                                for q in 0..qdcount {
                                    if let Some(query) = Question::from_message(msg, next) {
                                        next = query.end_offset();
                                        println!("Question {}: {:?}", q, &query);
                                    }
                                }
                            }
                            for ancount in header.an() {
                                for a in 0..ancount {
                                    if let Some(answer) = Resource::from_message(msg, next) {
                                        next = answer.end_offset();
                                        println!("Answer {}: {:?}", a, &answer);
                                    }
                                }
                            }
                            event_loop.shutdown();
                        }
                        Err(e) => {
                            println!("recv_from() failed: {}", e);
                            event_loop.shutdown();
                        }
                    }
                }
            }
            t => {
                panic!("Invalid token: {:?}", t);
            }
        }
    }
}

fn main() {
    println!("Init...");

    let mut event_loop = mio::EventLoop::new().unwrap();

    println!("Binding socket...");
    let address = "0.0.0.0:5300".parse().unwrap();
    let listener = UdpSocket::bound(&address).unwrap();
    event_loop.register(&listener,
                        RESPONSE,
                        mio::EventSet::readable(),
                        mio::PollOpt::level());

    println!("Sending query ...");
    let mut buffer = Vec::new();
    let num = io::stdin().read_to_end(&mut buffer).ok().unwrap();
    println!("Read {} bytes", num);
    let upstream = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53));
    listener.send_to(&mut io::Cursor::new(buffer), &upstream);
    let mut server = Recurse::new(listener);
    println!("Running...");
    event_loop.run(&mut server);
}
