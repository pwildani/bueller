extern crate mio;
extern crate time;

use std::u64;
use std::cmp::min;
use std::collections::BTreeMap;

pub type Name = Vec<Vec<u8>>;

pub type time_t = u64;
pub const TIME_T_MAX: time_t = u64::MAX;

#[derive(Clone, Debug)]
pub struct CacheResource {
    data: Option<Vec<u8>>,
    absolute_ttl: time_t,
    rcode: u16,
}

#[derive(Clone, Debug)]
pub struct CacheRecord {
    name: Name,
    /// Records for this name ordered by resource code.
    resources: BTreeMap<u16, CacheResource>,
}

impl CacheRecord {
    fn new(name: Name) -> CacheRecord {
        CacheRecord {
            name: name,
            resources: BTreeMap::new(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn add(&mut self, r: CacheResource) {
        self.resources.insert(r.rcode, r);
    }

    /// Merges other into self, overriding values.
    /// Returns true if the ttl has changed.
    pub fn merge_from(&mut self, other: CacheRecord) -> bool {
        for rec in other.resources.values() {
            self.add(rec.clone());
        }
        true
    }

    pub fn next_absolute_ttl(&self) -> time_t {
        self.resources.values().map(|r| r.absolute_ttl).min().unwrap_or(0)
    }

    /// Removes all resources that expire after 'now'.
    pub fn expire_after(&mut self, now: time_t) {
        let mut to_remove = Vec::with_capacity(self.resources.len());
        for r in self.resources.values() {
            if r.absolute_ttl < now {
                to_remove.push(r.rcode);
            }
        }
        for rcode in to_remove {
            self.resources.remove(&rcode);
        }
    }

    pub fn empty(&self) -> bool {
        return self.resources.len() == 0;
    }
}


#[cfg(test)]
mod tests {
    use std::iter::Iterator;
    use super::*;

    fn name() -> Name {
        vec![vec![1, 2, 3]]
    }

    fn rcodes<'a, T: Iterator<Item = &'a CacheResource>>(resources: T) -> Vec<u16> {
        resources.map(|r| r.rcode).collect::<Vec<u16>>()
    }

    fn ttls<'a, T: Iterator<Item = &'a CacheResource>>(resources: T) -> Vec<time_t> {
        resources.map(|r| r.absolute_ttl).collect::<Vec<time_t>>()
    }


    #[test]
    fn test_empty() {
        let mut rec = CacheRecord::new(name());
        assert!(rec.empty());
    }

    #[test]
    fn test_add_and_empty() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        assert!(!rec.empty());
    }

    #[test]
    fn test_add_duplicate() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        rec.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 1,
        });
        assert_eq!(vec![1], ttls(rec.resources.values()));
    }

    #[test]
    fn test_add_distinct() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 0,
        });
        rec.add(CacheResource {
            rcode: 2,
            data: None,
            absolute_ttl: 1,
        });
        assert_eq!(vec![1, 2], rcodes(rec.resources.values()));
    }


    #[test]
    fn test_add_disordered() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 2,
            data: None,
            absolute_ttl: 1,
        });
        rec.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 0,
        });
        assert_eq!(2, rec.resources.len());
        assert_eq!(vec![1, 2], rcodes(rec.resources.values()));
    }

    #[test]
    fn test_expire_single_resource() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        assert!(!rec.empty());
        rec.expire_after(1);
        assert!(rec.empty());
    }

    #[test]
    fn test_expire_single_resource_of_many() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 2,
        });
        rec.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 0,
        });
        rec.add(CacheResource {
            rcode: 2,
            data: None,
            absolute_ttl: 2,
        });
        assert_eq!(3, rec.resources.len());
        rec.expire_after(1);
        assert_eq!(vec![0, 2], rcodes(rec.resources.values()));
    }

    #[test]
    fn test_expire_multiple_resources() {
        let mut rec = CacheRecord::new(name());
        rec.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        rec.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 2,
        });
        rec.add(CacheResource {
            rcode: 2,
            data: None,
            absolute_ttl: 0,
        });
        assert_eq!(3, rec.resources.len());
        rec.expire_after(1);
        assert_eq!(vec![1], rcodes(rec.resources.values()));
    }

    #[test]
    fn test_merge_distinct() {
        let mut target = CacheRecord::new(name());
        target.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        let mut new = CacheRecord::new(name());
        new.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 0,
        });
        target.merge_from(new);
        assert_eq!(vec![0, 1], rcodes(target.resources.values()));
    }

    #[test]
    fn test_merge_overlap() {
        let mut target = CacheRecord::new(name());
        target.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        let mut new = CacheRecord::new(name());
        new.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 1,
        });

        target.merge_from(new);

        assert_eq!(vec![0], rcodes(target.resources.values()));
        assert_eq!(vec![1], ttls(target.resources.values()));
    }

    #[test]
    fn test_merge_overlap_middle() {
        let mut target = CacheRecord::new(name());
        target.add(CacheResource {
            rcode: 0,
            data: None,
            absolute_ttl: 0,
        });
        target.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 0,
        });
        target.add(CacheResource {
            rcode: 4,
            data: None,
            absolute_ttl: 0,
        });
        let mut new = CacheRecord::new(name());
        new.add(CacheResource {
            rcode: 1,
            data: None,
            absolute_ttl: 1,
        });
        new.add(CacheResource {
            rcode: 3,
            data: None,
            absolute_ttl: 3,
        });

        target.merge_from(new);

        assert_eq!(vec![0, 1, 3, 4], rcodes(target.resources.values()));
        assert_eq!(vec![0, 1, 3, 0], ttls(target.resources.values()));
    }
}
