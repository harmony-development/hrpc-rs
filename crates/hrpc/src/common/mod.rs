pub(crate) mod buf;

/// Extension type used by [`crate::Request`] and [`crate::Response`].
pub mod extensions;
/// Common future types used by `hrpc-rs`.
pub mod future;
/// Common layers that can be used in both clients and servers.
#[cfg(feature = "_common")]
pub mod layer;
/// Common code to work with sockets.
#[cfg(feature = "_common")]
pub mod socket;
/// Common code to work with transports.
#[cfg(feature = "_common")]
pub mod transport;
