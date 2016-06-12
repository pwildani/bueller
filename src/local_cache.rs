
type IPv4 = [u8;4];
type IPv6 = [u8;16];


enum Record {
    A(IPv4),
    AAAA(IPv6),
    NS,
    MD,
    MF,
    CNAME,
    SOA,
    MB,
    MG,
    MR,
    NULL,
    WKS,
    PTR,
    HINFO,
    MINFO,
    MX,
    TXT,
}

enum Maybe<T> {
    /// No matching value.
    NoneSuch,
    
    /// Cannot answer.
    Unknown,

    /// Correct response.
    Have(T),
}

const AXFR:u16 = 252u16;
const MAILB:u16 = 253u16;
const MAILA:u16 = 254u16;
const ANY:u16 = 255u16;

impl Record {
    fn rtype(&self) -> u16 {
        match self {
            A => 1,
            NS => 2,
            MD => 3,
            MF => 4,
            CNAME => 5,
            SOA => 6,
            MB => 7,
            MG => 8,
            MR => 9,
            NULL => 10,
            WKS => 10,
            PTR => 12,
            HINFO => 13,
            MINFO => 14,
            MX => 15,
            TXT => 16,
            _ => panic!("Undefined rtype: {:?}"),
        }
    }

    fn matches_qtype(&self, qtype: u16) -> bool {
        match qtype {
            MAILB => match self { MG|MF|MR => true, _ => false },
            MAILA => match self { MX => true, _ => false },
            ANY => true,
            x if x == self.rtype() => true,
            AXFR | _ => false,
        }
    }
}

type TTL = u32;

struct Resource {
    name: String,
    data: Vec<(TTL, Record)>
}

struct IncomingUdp {
    server: UdpSocket,
}
struct LocalCache {
    cache_in: HashMap<Vec<u8>, Maybe<Resource>>,
    // Not implemented: cache_cs, cache_ch, cache_hs
}

enum LocalCacheCommand {
    LookupRequest {
        message: Vec<u8>,
        name: DomainName,
        rtype: u16,
        class: u16,
    },
}

enum LocalCacheResponse {
    LookupResponse {
        message: Vec<u8>,
        record: Maybe<Record>,
    }
}

struct NullEvented;
impl mio::Evented for NullEvented {
    fn register(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        Ok()
    }

    fn reregister(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        Ok()
    }

    fn deregister(&self, selector: &mut Selector) -> io::Result<()> {
        Ok()
    }
}

const NO_IO_EVENTS: NullEvented = NullEvented{};

impl LocalCache {
    fn register(&self, event_loop: &mut mio::EventLoop<Lookup>) {
        event_loop.register(NO_IO_EVENTS, LOCAL_LOOKUP,
                            mio::EventSet::readable(),
                            mio::PollOpt::level());
    }
}



    // let local_event_loop = mio::EventLoop::new().unwrap();
    // let mut cache = LocalCache{};
    // cache.register(local_event_loop);
    // let cache_comm = local_event_loop.sender()

