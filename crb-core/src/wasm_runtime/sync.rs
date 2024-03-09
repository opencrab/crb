//! Synchronization primitives for using in asynchronous contexts.
//!
//! The structs in this module exists to provide `Mutex` and `RwLock`
//! in the WASM environment.

use derive_more::{Deref, DerefMut, From};
pub use futures::lock::{Mutex, MutexGuard};

/// An asynchronous reader-writer lock.
///
/// A workaround for the lack of `RwLock` in the WASM environment
/// by using `Mutex` internally.
#[derive(Debug)]
pub struct RwLock<T> {
    inner: Mutex<T>,
}

impl<T> RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    /// Locks this `RwLock` with exclusive write access, causing the current task
    /// to yield until the lock has been acquired.
    pub async fn write(&self) -> WriteGuard<'_, T> {
        self.inner.lock().await.into()
    }

    /// Locks this `RwLock` with shared read access, causing the current task
    /// to yield until the lock has been acquired.
    pub async fn read(&self) -> ReadGuard<'_, T> {
        self.inner.lock().await.into()
    }
}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] method on
/// [`RwLock`].
///
/// [`read`]: method@crate::sync::RwLock::read
/// [`RwLock`]: struct@crate::sync::RwLock
#[derive(Debug, Deref, From)]
pub struct ReadGuard<'a, T> {
    guard: MutexGuard<'a, T>,
}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] method
/// on [`RwLock`].
///
/// [`write`]: method@crate::sync::RwLock::write
/// [`RwLock`]: struct@crate::sync::RwLock
#[derive(Debug, Deref, DerefMut, From)]
pub struct WriteGuard<'a, T> {
    guard: MutexGuard<'a, T>,
}
