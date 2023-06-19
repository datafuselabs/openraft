use std::fmt::Debug;
use std::fmt::Display;
use std::future::Future;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::time::Duration;
use std::time::Instant;

/// A trait defining interfaces with an asynchronous runtime.
///
/// The intention of this trait is to allow an application using this crate to bind an asynchronous
/// runtime that suits it the best.
///
/// ## Note
///
/// The default asynchronous runtime is `tokio`.
pub trait AsyncRuntime: Debug + Default + Send + Sync + 'static {
    /// The error type of [`Self::JoinHandle`].
    type JoinError: Debug + Display + Send;

    /// The return type of [`Self::spawn`].
    type JoinHandle<T: Send + 'static>: Future<Output = Result<T, Self::JoinError>> + Send + Sync + Unpin;

    /// The type that enables the user to sleep in an asynchronous runtime.
    type Sleep: Future<Output = ()> + Send + Sync;

    /// A measurement of a monotonically non-decreasing clock.
    type Instant: Add<Duration, Output = Self::Instant>
        + AddAssign<Duration>
        + Clone
        + Copy
        + Debug
        + Eq
        + From<Instant> // Conversion between `Self::Instant` and `Instant` must be a linear
        // transformation, e.g., preserving ordering.
        + Into<Instant>
        + Ord
        + PartialEq
        + PartialOrd
        + RefUnwindSafe
        + Send
        + Sub<Duration, Output = Self::Instant>
        + Sub<Self::Instant, Output = Duration>
        + SubAssign<Duration>
        + Sync
        + Unpin
        + UnwindSafe;

    /// The timeout error type.
    type TimeoutError: Debug + Display + Send;

    /// The timeout type used by [`Self::timeout`] and [`Self::timeout_at`] that enables the user
    /// to await the outcome of a [`Future`].
    type Timeout<R, T: Future<Output = R> + Send>: Future<Output = Result<R, Self::TimeoutError>> + Send;

    /// Spawn a new task.
    fn spawn<T>(future: T) -> Self::JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;

    /// Wait until `duration` has elapsed.
    fn sleep(duration: Duration) -> Self::Sleep;

    /// Wait until `deadline` is reached.
    fn sleep_until(deadline: Self::Instant) -> Self::Sleep;

    /// Return the current instant.
    fn now() -> Self::Instant;

    /// Require a [`Future`] to complete before the specified duration has elapsed.
    fn timeout<R, F: Future<Output = R> + Send>(duration: Duration, future: F) -> Self::Timeout<R, F>;

    /// Require a [`Future`] to complete before the specified instant in time.
    fn timeout_at<R, F: Future<Output = R> + Send>(deadline: Self::Instant, future: F) -> Self::Timeout<R, F>;

    /// Check if the [`Self::JoinError`] is `panic`.
    fn is_panic(join_error: &Self::JoinError) -> bool;

    /// Abort the task associated with the supplied join handle.
    fn abort<T: Send + 'static>(join_handle: &Self::JoinHandle<T>);
}

/// `Tokio` is the default asynchronous executor.
#[derive(Debug, Default)]
pub struct Tokio;

impl AsyncRuntime for Tokio {
    type JoinError = tokio::task::JoinError;
    type JoinHandle<T: Send + 'static> = tokio::task::JoinHandle<T>;
    type Sleep = tokio::time::Sleep;
    type Instant = tokio::time::Instant;
    type TimeoutError = tokio::time::error::Elapsed;
    type Timeout<R, T: Future<Output = R> + Send> = tokio::time::Timeout<T>;

    #[inline]
    fn spawn<T>(future: T) -> Self::JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::task::spawn(future)
    }

    #[inline]
    fn sleep(duration: Duration) -> Self::Sleep {
        tokio::time::sleep(duration)
    }

    #[inline]
    fn sleep_until(deadline: Self::Instant) -> Self::Sleep {
        tokio::time::sleep_until(deadline)
    }

    #[inline]
    fn now() -> Self::Instant {
        tokio::time::Instant::now()
    }

    #[inline]
    fn timeout<R, F: Future<Output = R> + Send>(duration: Duration, future: F) -> Self::Timeout<R, F> {
        tokio::time::timeout(duration, future)
    }

    #[inline]
    fn timeout_at<R, F: Future<Output = R> + Send>(deadline: Self::Instant, future: F) -> Self::Timeout<R, F> {
        tokio::time::timeout_at(deadline, future)
    }

    #[inline]
    fn is_panic(join_error: &Self::JoinError) -> bool {
        join_error.is_panic()
    }

    #[inline]
    fn abort<T: Send + 'static>(join_handle: &Self::JoinHandle<T>) {
        join_handle.abort();
    }
}
