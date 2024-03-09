//! A watch channel for sending values from one task to another.

use super::error;
use event_listener::Event;
use std::ops;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::{Arc, RwLock, RwLockReadGuard};

/// The initial version starts at zero.
const VERSION_0: usize = 0b00;

/// The version counter shifted by one position to the left to leave space for the closed bit.
const VERSION_1: usize = 0b10;

/// The least significant bit signifies a closed channel.
const CLOSED: usize = 0b01;

/// Receives values from the associated [`Sender`](struct@Sender).
///
/// Instances are created by the [`channel`](fn@channel) function.
#[derive(Debug)]
pub struct Receiver<T> {
    /// Pointer to the shared state
    shared: Arc<Shared<T>>,

    /// Last observed version.
    version: usize,
}

/// Sends values to the associated [`Receiver`](struct@Receiver).
///
/// Instances are created by the [`channel`](fn@channel) function.
#[derive(Debug)]
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

/// Returns a reference to the inner value.
///
/// Outstanding borrows hold a read lock on the inner value. This means that
/// long lived borrows could cause the produce half to block. It is recommended
/// to keep the borrow as short lived as possible.
#[derive(Debug)]
pub struct Ref<'a, T> {
    inner: RwLockReadGuard<'a, T>,
}

#[derive(Debug)]
struct Shared<T> {
    /// The most recent value
    value: RwLock<T>,

    /// The current version
    ///
    /// The lowest bit represents a "closed" state. The rest of the bits
    /// represent the current version.
    version: AtomicUsize,

    /// Tracks the number of `Receiver` instances.
    ref_count_rx: AtomicUsize,

    /// Event when the value has changed or the `Sender` has been dropped.
    event_value_changed: Event,

    /// Event when all `Receiver`s have been dropped.
    event_all_recv_dropped: Event,
}

/// Creates a new watch channel, returning the "send" and "receive" handles.
///
/// All values sent by [`Sender`] will become visible to the [`Receiver`] handles.
/// Only the last value sent is made available to the [`Receiver`] half. All
/// intermediate values are dropped.
///
/// # Examples
///
/// ```
/// # let executor = async_executor::LocalExecutor::new();
/// # executor.run(async {
/// let (tx, mut rx) = just_watch::channel("hello");
///
/// executor.spawn(async move {
///     while let Ok(value) = rx.recv().await {
///         println!("received = {:?}", value);
///     }
/// });
///
/// tx.send("world").unwrap();
/// # });
/// ```
///
/// [`Sender`]: struct@Sender
/// [`Receiver`]: struct@Receiver
pub fn channel<T>(init: T) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        value: RwLock::new(init),
        version: AtomicUsize::new(VERSION_0),
        ref_count_rx: AtomicUsize::new(1),
        event_value_changed: Event::new(),
        event_all_recv_dropped: Event::new(),
    });

    let tx = Sender {
        shared: shared.clone(),
    };

    let rx = Receiver {
        shared,
        version: VERSION_0,
    };

    (tx, rx)
}

impl<T> Receiver<T> {
    /// Returns a reference to the most recently sent value.
    ///
    /// Outstanding borrows hold a read lock. This means that long lived borrows
    /// could cause the send half to block. It is recommended to keep the borrow
    /// as short lived as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// let (_, rx) = just_watch::channel("hello");
    /// assert_eq!(*rx.borrow(), "hello");
    /// ```
    pub fn borrow(&self) -> Ref<'_, T> {
        let inner = self.shared.value.read().unwrap();
        Ref { inner }
    }

    /// Wait for a change notification.
    ///
    /// Returns when a new value has been sent by the [`Sender`] since the last
    /// time `changed()` was called. When the `Sender` half is dropped, `Err` is
    /// returned.
    ///
    /// [`Sender`]: struct@Sender
    ///
    /// # Examples
    ///
    /// ```
    /// # let executor = async_executor::LocalExecutor::new();
    /// # executor.run(async {
    /// let (tx, mut rx) = just_watch::channel("hello");
    ///
    /// let task = executor.spawn(async move {
    ///     tx.send("goodbye").unwrap();
    /// });
    ///
    /// assert!(rx.changed().await.is_ok());
    /// assert_eq!(*rx.borrow(), "goodbye");
    ///
    /// // The `tx` handle has been dropped
    /// assert!(rx.changed().await.is_err());
    ///
    /// task.await;
    /// });
    /// ```
    pub async fn changed(&mut self) -> Result<(), error::RecvError> {
        // Fast path: Check the state first.
        if let Some(ret) = self.maybe_changed() {
            return ret;
        }

        // In order to avoid a race condition, we first request a notification,
        // **then** check the current value's version. If a new version exists,
        // the notification request is dropped.
        let listener = self.shared.event_value_changed.listen();

        if let Some(ret) = self.maybe_changed() {
            return ret;
        }

        listener.await;

        self.maybe_changed()
            .expect("[bug] failed to observe change after notificaton.")
    }

    fn maybe_changed(&mut self) -> Option<Result<(), error::RecvError>> {
        // Load the version from the state
        let state = self.shared.version.load(SeqCst);
        let new_version = state & !CLOSED;

        if self.version != new_version {
            // Observe the new version and return
            self.version = new_version;
            return Some(Ok(()));
        }

        if CLOSED == state & CLOSED {
            // All receivers have dropped.
            return Some(Err(error::RecvError {}));
        }

        // No changes.
        None
    }
}

impl<T: Clone> Receiver<T> {
    /// A convenience helper which combines calling [`Receiver::changed()`] and
    /// [`Receiver::borrow()`] where the borrowed value is cloned and returned.
    ///
    /// Note: If this is the first time the function is called on a `Receiver`
    /// instance, then the function **will wait** until a new value is sent into the channel.
    ///
    /// `None` is returned if the `Sender` half is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # let executor = async_executor::LocalExecutor::new();
    /// # executor.run(async {
    /// let (tx, mut rx) = just_watch::channel("hello");
    ///
    /// let task = executor.spawn(async move {
    ///     tx.send("goodbye").unwrap();
    /// });
    ///
    /// assert_eq!(*rx.borrow(), "hello");
    ///
    /// // Waits for the new task to spawn and send the value.
    /// let v = rx.recv().await.unwrap();
    /// assert_eq!(v, "goodbye");
    ///
    /// let v = rx.recv().await;
    /// assert!(v.is_err());
    ///
    /// task.await;
    /// # });
    /// ```
    pub async fn recv(&mut self) -> Result<T, error::RecvError> {
        self.changed().await?;
        Ok(self.borrow().clone())
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.shared.ref_count_rx.fetch_add(1, Relaxed);
        Receiver {
            shared: self.shared.clone(),
            version: self.version,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        if self.shared.ref_count_rx.fetch_sub(1, Relaxed) == 1 {
            // Notify the single sender.
            self.shared.event_all_recv_dropped.notify(usize::MAX);
        }
    }
}

impl<T> Sender<T> {
    /// Creates a new `Receiver` connected to a `Sender`.
    pub fn subscribe(&self) -> Receiver<T> {
        self.shared.ref_count_rx.fetch_add(1, Relaxed);
        let version = self.shared.version.load(SeqCst);
        Receiver {
            shared: self.shared.clone(),
            version,
        }
    }

    /// Sends a new value via the channel, notifying all receivers.
    pub fn send(&self, value: T) -> Result<(), error::SendError<T>> {
        if self.shared.ref_count_rx.load(Relaxed) == 0 {
            // All watchers (`Receiver`s) have been dropped.
            return Err(error::SendError { inner: value });
        }

        // Replace the value.
        *self.shared.value.write().unwrap() = value;

        // Update the version. 2 (`VERSION_1`) is used so that the CLOSED bit is not set.
        self.shared.version.fetch_add(VERSION_1, SeqCst);

        // Notify all watchers.
        self.shared.event_value_changed.notify(usize::MAX);

        Ok(())
    }

    /// Applies a function to the existing value.
    ///
    /// A function gets a mutable reference that can be used to change a stored value.
    pub fn send_modify<F>(&self, func: F)
    where
        F: FnOnce(&mut T),
    {
        // Replace the value.
        func(&mut *self.shared.value.write().unwrap());

        // Update the version. 2 (`VERSION_1`) is used so that the CLOSED bit is not set.
        self.shared.version.fetch_add(VERSION_1, SeqCst);

        // Notify all watchers.
        self.shared.event_value_changed.notify(usize::MAX);
    }

    /// Completes when all receivers have dropped.
    ///
    /// This allows the producer to get notified when interest in the produced
    /// values is canceled and immediately stop doing work.
    pub async fn closed(&self) {
        // Fast path.
        if self.shared.ref_count_rx.load(Relaxed) == 0 {
            return;
        }

        // Listen for events now and check the reference count afterwards to avoid race condition.
        let listener = self.shared.event_all_recv_dropped.listen();

        if self.shared.ref_count_rx.load(Relaxed) == 0 {
            return;
        }

        listener.await;
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.version.fetch_or(CLOSED, SeqCst);
        self.shared.event_value_changed.notify(usize::MAX);
    }
}

// ===== impl Ref =====

impl<T> ops::Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}
