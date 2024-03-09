//! This module provides a simple wrapper
//! around the `tokio::task` module.

pub use tokio::task::JoinHandle;

/// Spawn a task globally (could be sent between threads).
pub fn spawn<F>(future: F) -> JoinHandle<()>
where
    F: futures::Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future)
}

/// Spawn a task locally (in the same thread).
pub fn spawn_local<F>(future: F) -> JoinHandle<()>
where
    F: futures::Future<Output = ()> + 'static,
{
    tokio::task::spawn_local(future)
}
