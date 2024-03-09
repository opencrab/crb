//! Initially a fork of the [`async_watch`](https://github.com/cynecx/async-watch) crate.
//!
//! A single-producer, multi-consumer channel that only retains the *last* sent
//! value.
//!
//! Extracted from [Tokio's](https://github.com/tokio-rs/tokio/) `tokio::sync::watch`
//! implementation, which was written by [Carl Lerche](https://github.com/carllerche).
//!
//! This channel is useful for watching for changes to a value from multiple
//! points in the code base, for example, changes to configuration values.
//!
//! # Usage
//!
//! [`channel`] returns a [`Sender`] / [`Receiver`] pair. These are
//! the producer and sender halves of the channel. The channel is
//! created with an initial value. The **latest** value stored in the channel is accessed with
//! [`Receiver::borrow()`]. Awaiting [`Receiver::changed()`] waits for a new
//! value to sent by the [`Sender`] half. Awaiting [`Receiver::recv()`] combines
//! [`Receiver::changed()`] and [`Receiver::borrow()`] where the borrowed value
//! is cloned and returned.
//!
//!
//! # Examples
//!
//! ```
//! # let executor = async_executor::LocalExecutor::new();
//! # executor.run(async {
//! let (tx, mut rx) = just_watch::channel("hello");
//! let mut rx2 = rx.clone();
//!
//! // First variant
//! executor.spawn(async move {
//!     while let Ok(value) = rx.recv().await {
//!         println!("received = {:?}", value);
//!     }
//! });
//!
//! // Second variant
//! executor.spawn(async move {
//!     while rx2.changed().await.is_ok() {
//!         println!("received = {:?}", *rx2.borrow());
//!     }
//! });
//!
//! tx.send("world").unwrap();
//! # });
//! ```
//!
//! # Closing
//!
//! [`Sender::closed`] allows the producer to detect when all [`Receiver`]
//! handles have been dropped. This indicates that there is no further interest
//! in the values being produced and work can be stopped.
//!
//! # Thread safety
//!
//! Both [`Sender`] and [`Receiver`] are thread safe. They can be moved to other
//! threads and can be used in a concurrent environment. Clones of [`Receiver`]
//! handles may be moved to separate threads and also used concurrently.
//!
//! [`Sender`]: crate::Sender
//! [`Receiver`]: crate::Receiver
//! [`Receiver::recv`]: crate::Receiver::recv
//! [`channel`]: crate::channel
//! [`Sender::closed`]: crate::Sender::closed

pub mod channel;
pub mod error;
