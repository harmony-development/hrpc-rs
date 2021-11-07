pub(crate) mod buf;

/// Extension type used by [`crate::Request`] and [`crate::Response`].
pub mod extensions;
/// Common future types used by `hrpc-rs`.
pub mod fut;
/// Common code to work with transports.
pub mod transport;
/// Common code to work with sockets.
pub mod socket;