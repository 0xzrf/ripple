use crate::{MemTable, table::SSTable};
use anyhow::Result;
use bytes::Bytes;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
// EXP: Try using smallvec in LsmStorage?

pub struct LsmStorageState {
    // current mutable memtable
    pub memtable: Arc<MemTable>,
    // immutable memtables, latest to ealiest
    pub imm_memtables: Vec<Arc<MemTable>>,
    // L0 SST, latest to earliest
    pub l0_sstables: Vec<usize>,
    // SsTables sorted by key range; L1 - L_max for leveled compaction, or tiers for tiered compaction
    pub levels: Vec<Vec<usize>>,
    pub sstables: HashMap<usize, Vec<SSTable>>,
}

impl LsmStorageState {
    pub fn create(id: usize) -> Self {
        Self {
            memtable: Arc::new(MemTable::new(id)),
            imm_memtables: vec![],
            l0_sstables: vec![],
            levels: vec![],
            sstables: HashMap::new(),
        }
    }
}

pub(crate) struct LsmStorageInner {
    pub(crate) state: Arc<RwLock<Arc<LsmStorageState>>>,
}

impl LsmStorageInner {
    pub fn init() -> Self {
        Self {
            state: Arc::new(RwLock::new(Arc::new(LsmStorageState::create(0)))),
        }
    }

    pub fn get_key(&self, k: &Bytes) -> Result<Bytes> {
        self.with_memt_read_lock(|mem_table| mem_table.get(k))
    }

    pub fn create_key(&self, k: Bytes, v: Bytes) {
        self.with_memt_read_lock(|mem_table| mem_table.create(k, v));
    }

    pub fn put_key(&self, k: Bytes, v: Bytes) {
        self.with_memt_read_lock(|memt| memt.put(k, v));
    }

    pub fn delete_key(&self, k: Bytes) {
        self.with_memt_read_lock(|memt| memt.delete(k));
    }

    #[inline]
    fn with_memt_read_lock<T, F: FnOnce(Arc<MemTable>) -> T>(&self, func: F) -> T {
        let mem_table = self.state.read().memtable.clone();
        func(mem_table)
    }

    #[inline]
    fn with_memt_write_lock<T, F: FnMut(Arc<MemTable>) -> T>(&self, mut func: F) -> T {
        let mem_table = self.state.write().memtable.clone();
        func(mem_table)
    }
}

pub(crate) struct MiniLsm {}
