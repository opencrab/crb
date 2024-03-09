//! The crate includes a universal channel and
//! a function for initiating asynchronous activities.

#![warn(missing_docs)]

pub use futures;
pub use uuid;

#[cfg(not(target_arch = "wasm32"))]
mod std_runtime;
#[cfg(not(target_arch = "wasm32"))]
pub use std_runtime::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm_runtime;
#[cfg(target_arch = "wasm32")]
pub use wasm_runtime::*;
