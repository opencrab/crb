//! A multi-producer, single-consumer queue for
//! transferring values between asynchronous tasks.
//!
//! The module provides `mpsc` implementation for
//!  the WASM environment where `tokio` crate
//!  is not available.

use futures::{channel::mpsc, StreamExt};
use thiserror::Error;

/// An error returned by the `Sender`.
#[derive(Debug, Error)]
#[error("channel closed")]
pub struct SendError<T>(pub T);

/// Send values to the associated `UnboundedReceiver`.
#[derive(Debug)]
pub struct UnboundedSender<T> {
    inner: mpsc::UnboundedSender<T>,
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        UnboundedSender {
            inner: self.inner.clone(),
        }
    }
}

impl<T> UnboundedSender<T> {
    /// Attempts to send a message with this `UnboundedSender`
    /// without blocking.
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.inner
            .unbounded_send(message)
            .map_err(|err| SendError(err.into_inner()))
    }
}

/// Receive values from the associated `UnboundedSender`.
#[derive(Debug)]
pub struct UnboundedReceiver<T> {
    inner: mpsc::UnboundedReceiver<T>,
}

impl<T> UnboundedReceiver<T> {
    /// Receives the next value for this receiver.
    pub async fn recv(&mut self) -> Option<T> {
        self.inner.next().await
    }

    /// Closes the receiving half of a channel, without dropping it.
    pub fn close(&mut self) {
        self.inner.close();
    }
}

/// Creates an unbounded mpsc channel for communicating between asynchronous tasks.
pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = mpsc::unbounded();
    let sender = UnboundedSender { inner: tx };
    let receiver = UnboundedReceiver { inner: rx };
    (sender, receiver)
}
