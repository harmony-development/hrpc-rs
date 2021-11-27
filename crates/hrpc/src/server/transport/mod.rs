use futures_util::future::BoxFuture;

use super::MakeRoutes;

/// Trait for enabling generic transport implementations over a [`MakeRoutes`].
pub trait Transport: Sized {
    /// The type of the error returned by a transport if it fails.
    type Error;

    /// Start serving a [`MakeRoutes`].
    fn serve<S>(self, mk_routes: S) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        S: MakeRoutes;
}

/// Server implementation for a HTTP transport.
#[cfg(feature = "http_server")]
pub mod http;

/// The mock transport. Useful for testing.
#[cfg(feature = "mock_server")]
pub mod mock;
