//! Utilities for tracking time.

pub use std::time::Duration;

use futures::channel::oneshot;
use futures::future::{select, Either, Future};
use futures::pin_mut;
use gloo_timers::future::TimeoutFuture;
use ordered_float::OrderedFloat;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use thiserror::Error;

/// Waits until duration has elapsed.
pub async fn sleep(duration: Duration) {
    // The workaround, as the `TimeoutFuture` doesn't implement the `Send` trait.
    let (tx, rx) = oneshot::channel();
    crate::task::spawn_local(async move {
        TimeoutFuture::new(duration.as_millis() as u32).await;
        tx.send(()).ok();
    });
    rx.await.ok();
}

/// Requires a Future to complete before the specified duration has elapsed.
///
/// If `duration` is not set it waits for the `future` completion only.
pub async fn timeout<T>(duration: Option<Duration>, fut: T) -> Result<T::Output, Elapsed>
where
    T: Future,
{
    if let Some(duration) = duration {
        let timeout = sleep(duration);
        pin_mut!(timeout);
        pin_mut!(fut);
        let res = select(timeout, fut).await;
        match res {
            Either::Left(((), _unfinished_fut)) => Err(Elapsed),
            Either::Right((output, _timeout_fut)) => Ok(output),
        }
    } else {
        Ok(fut.await)
    }
}

/// Errors returned by `timeout`.
pub struct Elapsed;

// TIME TYPES

/// A measurement of a monotonically nondecreasing clock. Opaque and useful only with `Duration`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    /// Unit is milliseconds.
    inner: OrderedFloat<f64>,
}

impl Instant {
    const fn new(value: f64) -> Instant {
        Self {
            inner: OrderedFloat(value),
        }
    }

    /// Returns an instant corresponding to “now”.
    pub fn now() -> Instant {
        let value = web_sys::window()
            .expect("not in a browser")
            .performance()
            .expect("performance object not available")
            .now();
        Instant::new(value)
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        *self - earlier
    }

    /// Returns the amount of time elapsed since this instant was created.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        let inner = self.inner + OrderedFloat(other.as_millis() as f64);
        Instant { inner }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        let inner = self.inner - OrderedFloat(other.as_millis() as f64);
        Instant { inner }
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        let ms = self.inner - other.inner;
        assert!(ms >= OrderedFloat(0.0));
        Duration::from_millis(ms.into_inner() as u64)
    }
}

/// A replacement for [`std::time::SystemTimeError`] in WASM.
#[derive(Debug, Error)]
#[error("system time error")]
pub struct SystemTimeError;

/// The constant is defined to be “1970-01-01 00:00:00 UTC”.
pub const UNIX_EPOCH: SystemTime = SystemTime::new(0.0);

/// A measurement of the system clock.
///
/// An alternative for [`std::time::SystemTime`], but in WASM environment.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemTime {
    /// Unit is milliseconds.
    inner: OrderedFloat<f64>,
}

impl SystemTime {
    const fn new(value: f64) -> Self {
        Self {
            inner: OrderedFloat(value),
        }
    }

    /// Returns the system time corresponding to “now”.
    pub fn now() -> SystemTime {
        let val = js_sys::Date::now();
        SystemTime::new(val)
    }

    /// Returns the amount of time elapsed from an earlier point in time.
    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        let dur_ms = self.inner - earlier.inner;
        if dur_ms < OrderedFloat(0.0) {
            Err(SystemTimeError)
        } else {
            Ok(Duration::from_millis(dur_ms.into_inner() as u64))
        }
    }

    /// Returns the difference between the clock time when this system time was created, and the current clock time.
    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        self.duration_since(SystemTime::now())
    }

    /// Replacement for [`std::time::SystemTime::checked_add`] in the WASM runtime.
    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        Some(*self + duration)
    }

    /// Replacement for [`std::time::SystemTime::checked_sub`] in the WASM runtime.
    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        Some(*self - duration)
    }
}

impl Add<Duration> for SystemTime {
    type Output = SystemTime;

    fn add(self, other: Duration) -> SystemTime {
        let inner = self.inner + OrderedFloat(other.as_millis() as f64);
        SystemTime { inner }
    }
}

impl Sub<Duration> for SystemTime {
    type Output = SystemTime;

    fn sub(self, other: Duration) -> SystemTime {
        let inner = self.inner - OrderedFloat(other.as_millis() as f64);
        SystemTime { inner }
    }
}

impl AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
