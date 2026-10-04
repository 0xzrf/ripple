pub trait StorageIterator {
    type KeyType<'a>: PartialEq + Eq + PartialOrd + Ord
    where
        Self: 'a;

    fn value(&self) -> &[u8];
    fn key(&self) -> Self::KeyType<'_>;

    /// Check if the current iterator is valid.
    fn is_valid(&self) -> bool;

    /// Move to the next position.
    fn next(&mut self) -> anyhow::Result<()>;
}
