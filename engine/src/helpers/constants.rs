use bytes::Bytes;

pub const TOMBSTONE: Bytes = Bytes::from_static(&[]);
pub const MEMTABLE_MAX_LIMIT: usize = 2 << 28;
