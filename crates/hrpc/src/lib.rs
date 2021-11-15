//! Common code used in hRPC code generation.
#![deny(missing_docs)]
#![allow(clippy::blocks_in_if_conditions)]

/// Some re-exported crates that might be useful while writing software with `hrpc`.
pub mod exports {
    pub use bytes;
    pub use futures_util;
    pub use prost;
    pub use tracing;

    #[cfg(feature = "_common_http")]
    pub use http;
    #[cfg(feature = "server")]
    pub use tower;
}

/// Common client types and functions.
#[cfg(feature = "client")]
pub mod client;
/// Common server types and functions.
#[cfg(feature = "server")]
pub mod server;

/// Body utitilies and types.
pub mod body;
/// Common utilities.
pub mod common;
/// Decoding utilities.
pub mod decode;
/// Encoding utilities.
pub mod encode;
/// The hRPC generated protocol.
pub mod proto;
/// The `Request` type used by hRPC.
pub mod request;
/// The `Response` type used by hRPC.
pub mod response;

#[doc(inline)]
pub use request::Request;
#[doc(inline)]
pub use response::Response;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// The hRPC protobuf mimetype.
pub const HRPC_CONTENT_MIMETYPE: &[u8] = b"application/hrpc";
/// The hRPC spec version this version of `hrpc-rs` implements.
pub const HRPC_SPEC_VERSION: &str = "1";

/// Include generated proto server and client items.
///
/// You must specify the hRPC package name.
///
/// ```rust,ignore
/// mod pb {
///     hrpc::include_proto!("helloworld");
/// }
/// ```
///
/// # Note:
/// **This only works if the hrpc-build output directory has been unmodified**.
/// The default output directory is set to the [`OUT_DIR`] environment variable.
/// If the output directory has been modified, the following pattern may be used
/// instead of this macro.
///
/// ```rust,ignore
/// mod pb {
///     include!("/relative/protobuf/directory/helloworld.rs");
/// }
/// ```
/// You can also use a custom environment variable using the following pattern.
/// ```rust,ignore
/// mod pb {
///     include!(concat!(env!("PROTOBUFS"), "/helloworld.rs"));
/// }
/// ```
///
/// [`OUT_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
#[macro_export]
macro_rules! include_proto {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}
