pub(crate) mod buf;

/// Extension type used by [`crate::Request`] and [`crate::Response`].
pub mod extensions;
/// Common future types used by `hrpc-rs`.
pub mod fut;
/// Common code to work with sockets.
#[cfg(any(feature = "client", feature = "server"))]
pub mod socket;
/// Common code to work with transports.
#[cfg(any(feature = "client", feature = "server"))]
pub mod transport;
