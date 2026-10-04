use crate::{MemTable, table::SSTable};
use anyhow::{Error, Result};
use bytes::Bytes;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
// EXP: Try using smallvec in LsmStorage?

#[derive(Clone)]
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
            state: Arc::new(RwLock::new(Arc::new(
                LsmStorageState::create(0),
            ))),
        }
    }

    pub fn get_key(&self, k: &Bytes) -> Result<Bytes> {
        // look at the mutable mem table before searching older ones
        // TODO: maybe increase the readability here?
        if let Ok(result) = self.with_memt_read_lock(|mem_table| mem_table.get(k)) {
            return Ok(result);
        }

        // check all the imm memtables, first found returns
        for mem_table in &self.state.read().imm_memtables {
            if let Ok(result) = mem_table.get(k) {
                return Ok(result);
            }
        }

        // else, we couldn't find the key
        Err(Error::msg("Couldn't find the key"))
    }

    pub fn create_key(&self, k: Bytes, v: Bytes) {
        self.with_memt_read_lock(|mem_table| mem_table.create(k, v));
    }

    pub fn put_key(&self, k: Bytes, v: Bytes) -> Result<()> {
        self.with_memt_read_lock(|memt| {
            if memt.req_exceeds_memt_size(k.len() + v.len()) {
                self.freeze_memtable()?; // exchanges the current mem_table with a new one
            }

            memt.put(k, v);
            Ok(())
        })
    }

    pub fn delete_key(&self, k: Bytes) {
        self.with_memt_read_lock(|memt| memt.delete(k));
    }

    fn freeze_memtable(&self) -> Result<()> {
        let id = self.next_sst_id();
        let memtable = Arc::new(MemTable::create_with_wal(
            id,
            self.path_of_wal(id),
        )?); // <- Could take several milliseconds.
        {
            let mut guard = self.state.write();
            let mut snapshot = guard.as_ref().clone();
            let old_memtable = std::mem::replace(&mut snapshot.memtable, memtable);
            snapshot.imm_memtables.insert(0, old_memtable);
            *guard = Arc::new(snapshot);
        }
        Ok(())
    }

    fn next_sst_id(&self) -> usize {
        0
    }

    fn path_of_wal(&self, id: usize) -> String {
        unimplemented!()
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
