//! Common code used in hRPC code generation.
#[doc(inline)]
pub use async_trait::async_trait;
#[doc(hidden)]
pub use async_tungstenite::{self, tungstenite};
#[doc(hidden)]
pub use bytes;
#[doc(hidden)]
pub use futures_util;
#[doc(hidden)]
pub use log;
#[doc(hidden)]
pub use reqwest;
#[doc(hidden)]
pub use url;
#[doc(hidden)]
pub use warp;

/// Common client types and functions.
pub mod client;
/// Common server types and functions.
pub mod server;

#[doc(hidden)]
pub fn encode_protobuf_message(buf: &mut bytes::BytesMut, msg: impl prost::Message) {
    buf.reserve(msg.encoded_len().saturating_sub(buf.len()));
    buf.clear();
    msg.encode(buf)
        .expect("failed to encode protobuf message, something must be terribly wrong");
}

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
