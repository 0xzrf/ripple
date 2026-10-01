use anyhow::Result;
use bytes::Bytes;
use parking_lot::Mutex;
use std::sync::Arc;
use std::{fs::File, io::BufWriter};

pub struct Wal {
    file: Arc<Mutex<BufWriter<File>>>,
}
