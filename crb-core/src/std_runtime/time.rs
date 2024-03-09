//! Utilities for tracking time.

use futures::Future;
pub use std::time::{Duration, Instant};
pub use tokio::time::error::Elapsed;

/// Waits until duration has elapsed.
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration.into()).await;
}

/// Requires a Future to complete before
/// the specified duration has elapsed.
///
/// If `duration` is not set it waits for
///  the `fut` completion only.
pub async fn timeout<T>(duration: Option<Duration>, fut: T) -> Result<T::Output, Elapsed>
where
    T: Future,
{
    if let Some(duration) = duration {
        tokio::time::timeout(duration, fut).await
    } else {
        Ok(fut.await)
    }
}
