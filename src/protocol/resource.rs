use super::bits::U16Field;
use super::bits::U32Field;
use super::packet::Block;
use super::packet::Packet;
use super::packet::Piece;
use super::domain_name::DomainName;

struct ResourceData<'d> {
    data: Piece<'d>,
}

pub struct Resource<'d> {
    name: DomainName<'d>,
    footer: ResourceData<'d>,
    data: Piece<'d>,
}

impl<'d> ResourceData<'d> {
    const TYPE:U16Field = U16Field{index:0};
    const CLASS:U16Field = U16Field{index:2};
    const TTL:U32Field = U32Field{index:4};
    const LENGTH:U16Field = U16Field{index:8};
    const SIZE:usize = 4;

    fn rtype(&self) -> Option<u16> { ResourceData::TYPE.get(self.data.data()) }
    fn rclass(&self) -> Option<u16> { ResourceData::CLASS.get(self.data.data()) }
    fn ttl(&self) -> Option<u32> { ResourceData::TTL.get(self.data.data()) }
    fn data_length(&self) -> Option<u16> { ResourceData::LENGTH.get(self.data.data()) }
}

impl<'d> Resource<'d> {
    pub fn name<'a>(&'a self) -> Option<&'a DomainName<'d>> { Some(&self.name) }
}

impl<'d> Block<'d, Resource<'d>> for Resource<'d> {
    fn at<'p>(src: &'p mut Packet<'d>, at:usize) -> Resource<'d> {
        let name = src.next::<DomainName<'d>>();
        let footer = src.next::<ResourceData<'d>>();
        let payload = src.next_slice(footer.data_length().unwrap_or(0) as usize);
        Resource { name: name, footer: footer, data:payload }
    }
}

impl<'d> Block<'d, ResourceData<'d>> for ResourceData<'d> {
    fn at<'p>(src: &'p mut Packet<'d>, at:usize) -> ResourceData<'d> {
        ResourceData { data: src.next_slice(ResourceData::SIZE) }
    }
}


#[cfg(test)]
mod test {
    use protocol::packet::Packet;
    use super::Resource;

    #[test]
    fn resource() {
        let data = &[0];
        let mut p = Packet::new(data);
        let q = p.next::<Resource>();
    }
    
}
