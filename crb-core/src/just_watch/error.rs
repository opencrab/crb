//! Watch error types.

use std::fmt;

/// Error produced when sending a value fails.
#[derive(Debug)]
pub struct SendError<T> {
    pub(crate) inner: T,
}

// ===== impl SendError =====

impl<T> SendError<T> {
    /// Returns the data being sent (by [`send`](fn@crate::Sender::send))
    /// so it can be recovered.
    pub fn value(self) -> T {
        self.inner
    }
}

impl<T: fmt::Debug> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

/// Error produced when receiving a value fails.
#[derive(Debug)]
pub struct RecvError {}

impl fmt::Display for RecvError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl std::error::Error for RecvError {}
