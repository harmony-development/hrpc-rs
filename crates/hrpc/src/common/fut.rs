use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Creates a future that is immediately ready with a value.
///
/// This `struct` is created by [`ready()`]. See its
/// documentation for more.
#[derive(Debug, Clone)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Ready<T>(Option<T>);

impl<T> Ready<T> {
    /// Extract the inner value of this [`Ready`].
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> Unpin for Ready<T> {}

impl<T> Future for Ready<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        Poll::Ready(self.0.take().expect("`Ready` polled after completion"))
    }
}

/// Creates a future that is immediately ready with a value.
///
/// Futures created through this function are functionally similar to those
/// created through `async {}`. The main difference is that futures created
/// through this function are named and implement `Unpin`.
///
/// # Examples
///
/// ```
/// use std::future;
///
/// # async fn run() {
/// let a = future::ready(1);
/// assert_eq!(a.await, 1);
/// # }
/// ```
pub fn ready<T>(t: T) -> Ready<T> {
    Ready(Some(t))
}
