use crate::{
    Wal,
    constants::TOMBSTONE,
    helpers::errors::{Errs, GetErrs},
};
use anyhow::Result;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use std::sync::Arc;

pub struct MemTable {
    wal: Option<Wal>,
    map: Arc<SkipMap<Bytes, Bytes>>,
    id: usize,
    approx_size: usize,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            wal: None,
            map: Arc::new(SkipMap::new()),
            id: 0,
            approx_size: 0,
        }
    }

    pub fn create(&self, k: Bytes, v: Bytes) {
        if !self.contains_key(&k) {
            self.insert_map(k, v);
        }
    }

    pub fn put(&mut self, k: Bytes, v: Bytes) {
        if self.contains_key(&k) {
            let byte_len = v.len() + k.len();
            // updates if the k-v pair exists
            self.insert_map(k, v);
            // will add errors, so this should happen after insert
            self.incr_approx_size(byte_len);
        }
    }

    pub fn get(&self, k: &Bytes) -> Result<Bytes, Errs> {
        // a deleted key will store a tombstone, so we have to ignore it
        // TODO: Optimise this later
        let Some(value) = self.get_key(k) else {
            return Err(Errs::Get(GetErrs::Unavailable));
        };

        if value == TOMBSTONE {
            return Err(Errs::Get(GetErrs::Tombstone));
        }

        Ok(value)
    }

    pub fn delete(&self, k: Bytes) {
        self.del_key(k);
    }

    fn insert_map(&self, k: Bytes, v: Bytes) {
        self.map.insert(k, v);
    }

    fn contains_key(&self, k: &Bytes) -> bool {
        self.map.contains_key(k)
    }

    fn get_key(&self, k: &Bytes) -> Option<Bytes> {
        self.map.get(k).map(|v| v.value().clone())
    }

    fn del_key(&self, k: Bytes) {
        self.map.insert(k, TOMBSTONE);
    }

    fn incr_approx_size(&mut self, val: usize) {
        self.approx_size.checked_add(val).unwrap();
    }
}
