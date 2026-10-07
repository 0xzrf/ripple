use crate::helpers::constants::TOMBSTONE;
use crate::iterators::StorageIterator;
use crate::key::{Key, KeySlice};
use crate::{Wal, constants::MEMTABLE_MAX_LIMIT};
use anyhow::{Context, Result};
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use ouroboros::self_referencing;
use std::ops::Bound;
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

    pub fn req_exceeds_memt_size(&self, len: usize) -> bool {
        self.approx_size.load(Ordering::Relaxed) + len < MEMTABLE_MAX_LIMIT
    }

    pub fn scan(&self, lower: Bound<Bytes>, upper: Bound<Bytes>) -> Result<MemTableIterator> {
        let skip_map = self.map.clone();

        let mut mem_t_iter = MemTableIterator::new(
            skip_map,
            |map| map.range((lower, upper)),
            (Bytes::new(), Bytes::new()),
        );

        while mem_t_iter.is_valid() {
            let key = mem_t_iter.key();
            let value = mem_t_iter.value();

            mem_t_iter.next()?;
        }

        Ok(mem_t_iter)
    }

    #[inline]
    fn insert_map(&self, k: Bytes, v: Bytes) {
        self.map.insert(k, v);
    }

    #[inline]
    fn contains_key(&self, k: &Bytes) -> bool {
        self.map.contains_key(k)
    }

    #[inline]
    fn get_key(&self, k: &Bytes) -> Option<Bytes> {
        self.map.get(k).map(|v| v.value().clone())
    }

    #[inline]
    fn incr_approx_size(&self, val: usize) {
        self.approx_size.fetch_add(val, Ordering::Relaxed);
    }
}

type SkipMapRangeIter<'a> =
    crossbeam_skiplist::map::Range<'a, Bytes, (Bound<Bytes>, Bound<Bytes>), Bytes, Bytes>;

#[self_referencing]
pub struct MemTableIterator {
    map: Arc<SkipMap<Bytes, Bytes>>,

    #[borrows(map)]
    #[not_covariant]
    pub iter: SkipMapRangeIter<'this>,

    item: (Bytes, Bytes),
}

impl MemTableIterator {}

impl StorageIterator for MemTableIterator {
    type KeyType<'a>
        = KeySlice<'a>
    where
        Self: 'a;
    fn is_valid(&self) -> bool {
        self.with_item(|item| item.0 == TOMBSTONE)
    }

    fn key(&self) -> Self::KeyType<'_> {
        self.with_item(|item| Key::from_slice(&item.0))
    }

    fn value(&self) -> &[u8] {
        self.with_item(|item| &item.1)
    }

    fn next(&mut self) -> anyhow::Result<()> {
        self.with_mut(|fields| {
            let iter = fields.iter;
            let item = fields.item;

            if let Some(new_item) = iter.next() {
                *item = (
                    new_item.key().clone(),
                    new_item.value().clone(),
                );
            } else {
                *item = (Bytes::new(), Bytes::new());
            };
        });
        Ok(())
    }
}
