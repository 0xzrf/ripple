use crate::{
    Wal,
    constants::{MEMTABLE_MAX_LIMIT, TOMBSTONE},
};
use anyhow::{Context, Result};
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub struct MemTable {
    wal: Option<Wal>,
    map: Arc<SkipMap<Bytes, Bytes>>,
    id: usize,
    approx_size: AtomicUsize,
}

impl MemTable {
    pub fn new(id: usize) -> Self {
        Self {
            wal: None,
            map: Arc::new(SkipMap::new()),
            id,
            approx_size: AtomicUsize::new(0),
        }
    }

    pub fn create_with_wal(id: usize, path_to_wal: String) -> Result<Self> {
        unimplemented!()
    }

    pub fn create(&self, k: Bytes, v: Bytes) {
        if !self.contains_key(&k) {
            self.insert_map(k, v);
        }
    }

    pub fn put(&self, k: Bytes, v: Bytes) {
        if self.contains_key(&k) {
            let byte_len = k.len() + v.len();
            // updates if the k-v pair exists
            self.insert_map(k, v);
            self.incr_approx_size(byte_len);
        }
    }

    pub fn get(&self, k: &Bytes) -> Result<Bytes> {
        // a deleted key will store a tombstone, so we have to ignore it
        // TODO: Optimise this later
        let value = self.get_key(k).context("No key found")?;

        Ok(value)
    }

    pub fn delete(&self, k: Bytes) {
        self.del_key(k);
    }

    pub fn req_exceeds_memt_size(&self, len: usize) -> bool {
        self.approx_size.load(Ordering::Relaxed) + len < MEMTABLE_MAX_LIMIT
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

    fn incr_approx_size(&self, val: usize) {
        self.approx_size.fetch_add(val, Ordering::Relaxed);
    }
}
