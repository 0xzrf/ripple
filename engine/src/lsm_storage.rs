use crate::{MemTable, table::SSTable};
use bytes::Bytes;
use std::{collections::HashMap, sync::Arc};
// exp: Try using smallvec in LsmStorage?

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
    pub fn create() -> Self {
        Self {
            memtable: Arc::new(MemTable::new()),
            imm_memtables: vec![],
            l0_sstables: vec![],
            levels: vec![],
            sstables: HashMap::new(),
        }
    }

    pub fn get_key(&self, k: &Bytes) {
        self.memtable.get(k);
    }

    pub fn create_key(&self, k: Bytes, v: Bytes) {
        self.memtable.create(k, v);
    }

    pub fn put_key(&self, k: Bytes, v: Bytes) {
        self.memtable.put(k, v);
    }

    pub fn delete_key(&self, k: Bytes) {
        self.memtable.delete(k);
    }
}

pub(crate) struct LsmStorageInner {}

pub(crate) struct MiniLsm {}
