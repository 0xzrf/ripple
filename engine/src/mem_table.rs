use crate::Wal;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use std::sync::Arc;

pub struct MemTable {
    wal: Option<Wal>,
    map: Arc<SkipMap<Bytes, Bytes>>,
    id: usize,
}

impl MemTable {
    pub fn create(&self, k: Bytes, v: Bytes) {}

    pub fn put(&self, k: Bytes, v: Bytes) {}

    pub fn get(&self, k: Bytes) {}

    fn insert_to_map(&self, k: Bytes, v: Bytes) {
        self.map.insert(k, v);
    }
}
