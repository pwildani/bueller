extern crate bueller;
extern crate mio;

use bueller::protocol::{Header, HeaderMut};
use bueller::protocol::MessageCursor;
use bueller::protocol::Resource;
use bueller::protocol::{Question, QuestionMut};
use bueller::rfc4390::{encode_dotted_name, vec_ref};
use mio::udp::UdpSocket;
use std::io::Read;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::iter;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::collections::HashMap;

const UDP: mio::Token = mio::Token(0);
const LOCAL_LOOKUP: mio::Token = mio::Token(1);

struct IncomingUdp {
    server: UdpSocket,
}

impl IncomingUdp {
    fn new(address: SocketAddr) -> IncomingUdp {
        let server = UdpSocket::bound(&address).unwrap();
        IncomingUdp { server: server }
    }

    fn register(&self, event_loop: &mut mio::EventLoop<Self>) {
        event_loop.register(&self.server,
                            UDP,
                            mio::EventSet::readable(),
                            mio::PollOpt::level());
    }

    fn read_message(&mut self) -> io::Result<(SocketAddr, Vec<u8>)> {
        // TODO self.config.max_packet_size
        let mut buf = Vec::with_capacity(1024);
        if let Some(addr) = try!(self.server.recv_from(&mut buf)) {
            return Ok((addr, buf));
        }
        Err(Error::new(ErrorKind::WouldBlock, "Would block"))
    }

    fn dispatch_message(&mut self, from: SocketAddr, message: Vec<u8>) {
        println!("Got a request from {:?}", from);
        println!("{:?}", message);
        let msg = &message[..];
        let header = Header::at(msg);
        println!("Header {:?}", &header);
        let mut next = header.end_offset();
        if let Some(qdcount) = header.qd() {
            for q in 0..qdcount {
                println!("Question {}@{}:", q, next);
                if let Some(query) = Question::from_message(msg, next) {
                    println!(" .. {:?}", &query);
                    if let Some(name) = query.name() {
                        println!(" .. name = {:?}", name.segments(msg));
                    }
                    next = query.end_offset();
                } else {
                    println!(" .. None");
                }
            }
        }
        if let Some(ancount) = header.an() {
            for a in 0..ancount {
                if let Some(answer) = Resource::from_message(msg, next) {
                    next = answer.end_offset();
                    println!("Answer {}: {:?}", a, &answer);
                }
            }
        }
    }
}


impl mio::Handler for IncomingUdp {
    type Timeout = ();
    type Message = Vec<u8>;

    fn ready(&mut self,
             event_loop: &mut mio::EventLoop<Self>,
             token: mio::Token,
             events: mio::EventSet) {
        println!("ready...");
        match token {
            UDP => {
                if events.is_readable() {
                    match self.read_message() {
                        Ok((addr, buf)) => {
                            self.dispatch_message(addr, buf);
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
    let mut udp_event_loop = mio::EventLoop::new().unwrap();
    let mut udpserver = IncomingUdp::new("0.0.0.0:5300".parse().unwrap());
    udpserver.register(&mut udp_event_loop);
    udp_event_loop.run(&mut udpserver);
}
