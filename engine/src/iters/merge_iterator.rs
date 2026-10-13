use std::cmp;
use std::collections::{BinaryHeap, binary_heap::PeekMut};

use anyhow::{Error, Result};

use crate::key::{Key, KeySlice};

use crate::StorageIterator;
use crate::check_result;

struct HeapWrapper<I: StorageIterator>(pub usize, pub Box<I>);

impl<I: StorageIterator> PartialEq for HeapWrapper<I> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl<I: StorageIterator> Eq for HeapWrapper<I> {}

impl<I: StorageIterator> PartialOrd for HeapWrapper<I> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: StorageIterator> Ord for HeapWrapper<I> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.1
            .key()
            .cmp(&other.1.key())
            .then(self.0.cmp(&other.0))
            .reverse()
    }
}

/// Merge multiple iterators of the same type. If the same key occurs multiple times in some
/// iterators, prefer the one with smaller index.
pub struct MergeIterator<I: StorageIterator> {
    iters: BinaryHeap<HeapWrapper<I>>,
    current: Option<HeapWrapper<I>>,
}

impl<I: StorageIterator> MergeIterator<I> {
    pub fn create(iters: Vec<Box<I>>) -> Self {
        let mut iters = BinaryHeap::from(
            iters
                .into_iter()
                .enumerate()
                .map(|(i, iter)| HeapWrapper(i, iter))
                .collect::<Vec<HeapWrapper<I>>>(),
        );

        let current = iters.pop();

        Self { current, iters }
    }
}

impl<I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>> StorageIterator
    for MergeIterator<I>
{
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> KeySlice<'_> {
        if let Some(current) = &self.current {
            current.1.key()
        } else {
            // TODO: We can make this error better
            panic!();
        }
    }

    fn value(&self) -> &[u8] {
        if let Some(current) = &self.current {
            current.1.value()
        } else {
            // TODO: We can make this error better
            panic!();
        }
    }

    fn is_valid(&self) -> bool {
        self.current.is_some()
    }

    fn next(&mut self) -> Result<()> {
        let current = self.current.as_mut().unwrap();

        while let Some(mut iter) = self.iters.peek_mut() {
            if iter.1.key() == current.1.key() {
                if let e @ Err(_) = iter.1.next() {
                    PeekMut::pop(iter);
                    return e;
                }

                if !iter.1.is_valid() {
                    PeekMut::pop(iter);
                }
            } else {
                break;
            }
        }

        current.1.next()?;

        if !current.1.is_valid() {
            if let Some(iter) = self.iters.pop() {
                *current = iter;
            }

            return Ok(());
        }

        if let Some(mut inner_iter) = self.iters.peek_mut()
            && *current < *inner_iter
        {
            std::mem::swap(&mut *inner_iter, current);
        }

        Ok(())
    }
}
