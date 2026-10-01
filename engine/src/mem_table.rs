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
    pub fn new() -> Self {
        Self {
            wal: None,
            map: Arc::new(SkipMap::new()),
            id: 0,
        }
    }

    pub fn create(&self, k: Bytes, v: Bytes) {
        if !self.contains_key(&k) {
            self.insert_map(k, v);
        }
    }

    pub fn put(&self, k: Bytes, v: Bytes) {
        if self.contains_key(&k) {
            self.insert_map(k, v); // updates if the k-v pair exists
        }
    }

    pub fn get(&self, k: &Bytes) -> Option<Bytes> {
        self.get_key(k)
    }

    pub fn delete(&self, k: &Bytes) {
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

    fn del_key(&self, k: &Bytes) {
        self.map.remove(k);
    }
}
