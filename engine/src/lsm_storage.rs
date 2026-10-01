use crate::{MemTable, table::SSTable};
use std::{collections::HashMap, sync::Arc};
// exp: Try using smallvec in LsmStorage?

pub struct LsmStorage {
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
