use super::super::level_db::DB;
use super::rusty_leveldb_vec_private::RustyLevelDbVecPrivate;
use super::{traits::*, Index};
use crate::sync::{AtomicRwReadGuard, AtomicRwWriteGuard};
use leveldb::batch::WriteBatch;
use serde::{de::DeserializeOwned, Serialize};
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// A concurrency safe database-backed Vec with in memory read/write caching for all operations.
#[derive(Debug, Clone)]
pub struct RustyLevelDbVec<T: Serialize + DeserializeOwned> {
    inner: Rc<RefCell<RustyLevelDbVecPrivate<T>>>,
}

impl<T: Serialize + DeserializeOwned + Clone> RustyLevelDbVec<T> {
    #[allow(dead_code)] // used by tests in mod.rs
    pub(crate) fn with_inner<R, F>(&self, cb: F) -> R
    where
        F: FnOnce(&RustyLevelDbVecPrivate<T>) -> R,
    {
        cb(&self.inner.borrow())
    }
}

impl<T: Serialize + DeserializeOwned + Clone> StorageVec<T> for RustyLevelDbVec<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    #[inline]
    fn len(&self) -> Index {
        self.inner.borrow().len()
    }

    #[inline]
    fn get(&self, index: Index) -> T {
        self.inner.borrow().get(index)
    }

    fn many_iter(
        &self,
        indices: impl IntoIterator<Item = Index> + 'static,
    ) -> Box<dyn Iterator<Item = (Index, T)> + '_> {
        // note: this lock is moved into the iterator closure and is not
        //       released until caller drops the returned iterator
        let inner = self.inner.borrow();

        Box::new(indices.into_iter().map(move |i| {
            assert!(
                i < inner.len(),
                "Out-of-bounds. Got index {} but length was {}. persisted vector name: {}",
                i,
                inner.len(),
                inner.name
            );

            if inner.cache.contains_key(&i) {
                (i, inner.cache[&i].clone())
            } else {
                let key = inner.get_index_key(i);
                (i, inner.get_u8(&key))
            }
        }))
    }

    fn many_iter_values(
        &self,
        indices: impl IntoIterator<Item = Index> + 'static,
    ) -> Box<dyn Iterator<Item = T> + '_> {
        // note: this lock is moved into the iterator closure and is not
        //       released until caller drops the returned iterator
        let inner = self.inner.borrow();

        Box::new(indices.into_iter().map(move |i| {
            assert!(
                i < inner.len(),
                "Out-of-bounds. Got index {} but length was {}. persisted vector name: {}",
                i,
                inner.len(),
                inner.name
            );

            if inner.cache.contains_key(&i) {
                inner.cache[&i].clone()
            } else {
                let key = inner.get_index_key(i);
                inner.get_u8(&key)
            }
        }))
    }

    #[inline]
    fn get_many(&self, indices: &[Index]) -> Vec<T> {
        self.inner.borrow().get_many(indices)
    }

    /// Return all stored elements in a vector, whose index matches the StorageVec's.
    /// It's the caller's responsibility that there is enough memory to store all elements.
    #[inline]
    fn get_all(&self) -> Vec<T> {
        self.inner.borrow().get_all()
    }

    #[inline]
    fn set(&self, index: Index, value: T) {
        self.inner.borrow_mut().set(index, value)
    }

    /// set multiple elements.
    ///
    /// panics if key_vals contains an index not in the collection
    ///
    /// It is the caller's responsibility to ensure that index values are
    /// unique.  If not, the last value with the same index will win.
    /// For unordered collections such as HashMap, the behavior is undefined.
    #[inline]
    fn set_many(&self, key_vals: impl IntoIterator<Item = (Index, T)>) {
        self.inner.borrow_mut().set_many(key_vals)
    }

    #[inline]
    fn pop(&self) -> Option<T> {
        self.inner.borrow_mut().pop()
    }

    #[inline]
    fn push(&self, value: T) {
        self.inner.borrow_mut().push(value)
    }

    #[inline]
    fn clear(&self) {
        self.inner.borrow_mut().clear();
    }
}

impl<T: Serialize + DeserializeOwned> StorageVecRwLock<T> for RustyLevelDbVec<T> {
    type LockedData = RustyLevelDbVecPrivate<T>;

    #[inline]
    fn try_write_lock(&self) -> Option<AtomicRwWriteGuard<'_, Self::LockedData>> {
        None
    }

    #[inline]
    fn try_read_lock(&self) -> Option<AtomicRwReadGuard<'_, Self::LockedData>> {
        None
    }
}

impl<T: Serialize + DeserializeOwned + Clone> RustyLevelDbVec<T> {
    // Return the key used to store the length of the persisted vector
    #[inline]
    pub fn get_length_key(key_prefix: u8) -> [u8; 2] {
        RustyLevelDbVecPrivate::<T>::get_length_key(key_prefix)
    }

    /// Return the length at the last write to disk
    #[inline]
    pub fn persisted_length(&self) -> Index {
        self.inner.borrow().persisted_length()
    }

    /// Return the level-DB key used to store the element at an index
    #[inline]
    pub fn get_index_key(&self, index: Index) -> [u8; 9] {
        self.inner.borrow().get_index_key(index)
    }

    #[inline]
    pub fn new(db: Arc<DB>, key_prefix: u8, name: &str) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RustyLevelDbVecPrivate::<T>::new(
                db, key_prefix, name,
            ))),
        }
    }

    /// Collect all added elements that have not yet bit persisted
    #[inline]
    pub fn pull_queue(&self, write_batch: &WriteBatch) {
        self.inner.borrow_mut().pull_queue(write_batch)
    }
}
