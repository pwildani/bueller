extern crate bueller;
extern crate mio;

use bueller::protocol::{Header, HeaderMut};
use bueller::protocol::MessageCursor;
use bueller::protocol::{Question, QuestionMut};
use bueller::protocol::Resource;
use bueller::protocol::encode_dotted_name;
use mio::udp::UdpSocket;
use std::io::Read;
use std::io;
use std::iter;
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

    let mut buffer = iter::repeat(0u8).take(512).collect::<Vec<u8>>();
    let mut idx = MessageCursor::new(buffer.len());
    HeaderMut::at(&mut idx, &mut buffer)
        .unwrap()
        .make_query(1)
        .set_qd(1);
    let qname = encode_dotted_name("github.com").unwrap();
    let mut qref = Vec::with_capacity(qname.len());
    for i in 0..qname.len() {
        qref.push(&qname[i][..]);
    }
    // qref == vec![&[0x67u8, 0x69, 0x74, 0x68, 0x75, 0x62][..],
    //             &[0x63, 0x6f, 0x6d][..]];
    QuestionMut::at(&mut idx,
                    &mut buffer,
                    // github, com, ""
                    &qref[..],
                    1, // 0xff, // QTYPE_ALL
                    1 /* QCLASS_IN */)
        .unwrap();
    buffer.truncate(idx.tell());
    println!("Request: {:?}", buffer);
    println!("header: {:?}", Header::at(&buffer));
    println!("question: {:?}", Question::from_message(&buffer, 12));


    let upstream = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53));
    listener.send_to(&mut io::Cursor::new(buffer), &upstream);
    let mut server = Recurse::new(listener);
    println!("Running...");
    event_loop.run(&mut server);
}
