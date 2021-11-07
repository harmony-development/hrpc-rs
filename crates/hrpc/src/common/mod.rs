pub(crate) mod buf;
pub(crate) mod socket;

/// Extension type used by [`crate::Request`] and [`crate::Response`].
pub mod extensions;
/// Common future types used by `hrpc-rs`.
pub mod fut;
/// Common code to work with transports.
pub mod transport;
