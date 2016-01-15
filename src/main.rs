extern crate bueller;
extern crate mio;
extern crate time;


use std::collections::BinaryHeap;
use time::now;
use time::Duration;
use bueller::protocol;
use bueller::server::CacheRecord;
use bueller::server::cache_record::CacheResource;
use bueller::server::time::{time_t, TIME_T_MAX};
use bueller::protocol::{Header, HeaderMut};
use bueller::protocol::MessageCursor;
use bueller::protocol::Resource;
use bueller::protocol::question;
use bueller::protocol::{Question, QuestionMut};
use bueller::rfc4390::{encode_dotted_name, vec_ref};
use mio::udp::UdpSocket;
use mio::util::Slab;
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

type Name = Vec<Vec<u8>>;

struct LocalCache {
    cache: HashMap<Name, CacheRecord>,
    ttl: BinaryHeap<(time_t, Name)>,
}

impl LocalCache {
    fn new() -> LocalCache {
        LocalCache {
            cache: HashMap::new(),
            ttl: BinaryHeap::new(),
        }
    }

    fn update(&mut self, mut rec: CacheRecord) {
        let key = rec.name().clone();
        if self.cache.contains_key(&key) {
            let merged = self.cache.get_mut(&key).unwrap();
            if merged.merge_from(rec) {
                self.ttl.push((TIME_T_MAX - merged.next_absolute_ttl(), key));
            }
        } else {
            self.ttl.push((TIME_T_MAX - rec.next_absolute_ttl(), key.clone()));
            self.cache.insert(key, rec);
        }
    }

    fn get(&self, key: &Name) -> Option<&CacheRecord> {
        self.cache.get(key)
    }

    fn next_ttl(&self) -> time_t {
        if let Some(&(ttl, _)) = self.ttl.peek() {
            ttl
        } else {
            TIME_T_MAX
        }
    }

    fn expire_after(&mut self, now: time_t) {
        while self.next_ttl() < now {
            let (_, name) = self.ttl.pop().unwrap();
            let mut empty = false;
            if let Some(rec) = self.cache.get_mut(&name) {
                rec.expire_after(now);
                empty = rec.empty();
            }
            if empty {
                self.cache.remove(&name);
            }
        }
    }
}

struct Response {
    response_cursor: MessageCursor,
    response: Vec<u8>,
}

impl Response {
    pub fn for_message<'a, 'b>(message: &'b Vec<u8>) -> Response {
        let response_len = Self::estimate_response_size(&message);
        let mut response = Vec::with_capacity(response_len);
        let mut response_cursor = MessageCursor::new(response_len);
        {
            let qheader = Header::at(&message[..]);
            if message.len() >= protocol::header::SIZE {
                // Normal request. Clone it to the response.
                HeaderMut::at(&mut response_cursor, &mut response)
                    .unwrap()
                    .set_id(qheader.id().unwrap())
                    .set_qr(true)
                    .set_op(qheader.op().unwrap())
                    .set_aa(false)
                    .set_tc(false)
                    .set_rd(qheader.rd().unwrap())
                    .set_ra(false)
                    .set_rc(0);
            } else {
                // Partial request. The RFC says to clone it anyway.
                let mut rheader = HeaderMut::at(&mut response_cursor, &mut response).unwrap();
                if let Some(id) = qheader.id() {
                    rheader.set_id(id);
                }
                rheader.set_qr(true);
                if let Some(op) = qheader.op() {
                    rheader.set_op(op);
                }
                rheader.set_aa(false);
                if let Some(rd) = qheader.rd() {
                    rheader.set_rd(rd);
                }
                rheader.set_ra(false);
                rheader.set_rc(protocol::header::RC_FORMAT_ERROR);
            }
        }

        Response {
            response_cursor: response_cursor,
            response: response,
        }
    }

    fn estimate_response_size(message: &Vec<u8>) -> usize {
        let mut response_len = protocol::header::SIZE;
        let msg = &message[..];
        let header = Header::at(msg);
        let mut next = header.end_offset();
        if let Some(qdcount) = header.qd() {
            for _ in 0..qdcount {
                if let Some(query) = Question::from_message(msg, next) {
                    response_len += query.estimate_response_size();
                    next = query.end_offset();
                }
            }
        }
        return response_len;
    }

}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum SessionId {
    UdpId(SocketAddr, u16),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum SessionState {
    Invalid,

    /// Waiting for local cache to respond.
    AwaitLocalLookup{
        next_question: u16
    },

    /// Waiting for multiple remote server to respond to a query in this message.
    AwaitRecursiveLookup,
    //{
    //     waiting_for: Vec<SessionId>,
    //},

    /// Waiting for remote server to respond.
    RunningRecursiveLookup {
        cause: SessionId,
    },
}

struct Session {
    id: SessionId,
    state: SessionState,

    message: Option<Vec<u8>>,
    response: Option<Response>,
    in_service_of: Option<SessionId>,
}

impl Session {
    fn new(id: SessionId) -> Session {
        Session {
            id: id,
            state: SessionState::Invalid,
            message: None,
            response: None,
            in_service_of: None,
        }
    }

    fn new_for_message(from: SessionId, message: Vec<u8>) -> Session {
        let response = Response::for_message(&message);
        Session {
            id: from,
            state: SessionState::AwaitLocalLookup { next_question: 0 },
            message: Some(message),
            response: Some(response),
            in_service_of: None,
        }
    }
}



struct UdpServer {
    server: UdpSocket,
    sessions: HashMap<SessionId, Session>,
    send_queue: Vec<Response>,

    /// Cache for the IN class values.
    in_cache: LocalCache,
}

impl UdpServer {
    fn new(address: SocketAddr) -> UdpServer {
        let server = UdpSocket::bound(&address).unwrap();
        UdpServer {
            server: server,
            sessions: HashMap::with_capacity(1023),
            send_queue: Vec::new(),
            in_cache: LocalCache::new(),
        }
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
        if Header::at(&message[..]).id().is_none() {
            // Bad packet format. Can't even reply with an error coherently.
            return;
        }
        let id = SessionId::UdpId(from, Header::at(&message[..]).id().unwrap());
        if Header::at(&message[..]).is_query() {
            self.new_session_or_error(id, message);
        } else {
            self.update_session_or_drop(id, message);
        }
    }

    fn new_session_or_error(&mut self, from: SessionId, message: Vec<u8>) {
        if self.sessions.contains_key(&from) {
            // TODO: accept and ignore if it's a duplicate.
            self.send_client_error_reply(from, message);
            return;
        }
        let session = Session::new_for_message(from.clone(), message);
        self.sessions.insert(from, session);
    }

    fn update_session_or_drop(&mut self, from: SessionId, message: Vec<u8>) {
    }

    fn send_client_error_reply(&mut self, from: SessionId, message: Vec<u8>) {
    }
}



impl mio::Handler for UdpServer {
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
    let mut udpserver = UdpServer::new("0.0.0.0:5300".parse().unwrap());
    let mut github = CacheRecord::new(encode_dotted_name("github.com").unwrap());
    github.add(CacheResource{
        rcode: question::QTYPE_A,
        data: Some(vec![192, 30, 252, 128]),
        absolute_ttl: 0,
    });
    //udpserver.in_cache.add(github);
    udpserver.register(&mut udp_event_loop);
    udp_event_loop.run(&mut udpserver);
}
