//! Task module for spawning async tasks
//! in WASM environment.

use futures::future::abortable;
use futures::stream::AbortHandle;
use futures::FutureExt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Re-export of the `spawn_local` function.
pub use wasm_bindgen_futures::spawn_local;

/// A function to spawn asynchronous task.
///
/// It's necessary, because actors rely on `async_trait,
/// and has to support spawning tasks with `Send` requirement.
pub fn spawn<F>(future: F) -> JoinHandle<()>
where
    F: futures::Future<Output = ()> + Send + 'static,
{
    let (fut, handle) = abortable(future);
    let alive = Arc::new(());
    let alive_hook = Arc::downgrade(&alive);
    let fut = fut.map(move |_| {
        drop(alive_hook);
    });
    spawn_local(fut);
    JoinHandle {
        _res: PhantomData,
        handle,
        alive,
    }
}

/// An alternative to `tokio::task::JoinHandle`.
///
/// Is used to equip `spawn_local` with a way to abort the task,
/// that is compatible with the `JoinHandle` from `tokio::task`.
#[derive(Debug)]
pub struct JoinHandle<T> {
    _res: PhantomData<T>,
    handle: AbortHandle,
    /// A workaround to check if the future has finished.
    alive: Arc<()>,
}

impl<T> JoinHandle<T> {
    /// Abort the task associated with the handle.
    pub fn abort(&self) {
        self.handle.abort();
    }

    /// Checks if the task associated with this `JoinHandle` has finished.
    pub fn is_finished(&self) -> bool {
        Arc::weak_count(&self.alive) == 0
    }
}
