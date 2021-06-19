//! Common code used in hRPC code generation.
use http::{header::HeaderName, HeaderValue};

#[doc(inline)]
pub use async_trait::async_trait;
#[doc(hidden)]
pub use bytes;
#[doc(hidden)]
pub use futures_util;
#[doc(hidden)]
pub use tracing;
#[doc(hidden)]
pub use url;

#[doc(hidden)]
#[cfg(feature = "client")]
pub use reqwest;
#[doc(hidden)]
#[cfg(feature = "client")]
pub use rustls_native_certs;
#[doc(hidden)]
#[cfg(feature = "client")]
pub use tokio_tungstenite::{self, tungstenite};

#[doc(hidden)]
#[cfg(feature = "server")]
pub use http;
#[doc(hidden)]
#[cfg(feature = "server")]
pub use warp;

/// Common client types and functions.
#[cfg(feature = "client")]
pub mod client;
/// Common server types and functions.
#[cfg(feature = "server")]
pub mod server;

use http::HeaderMap;

#[derive(Debug, Clone)]
/// A hRPC request.
pub struct Request<T> {
    message: T,
    header_map: HeaderMap,
}

impl<T> Request<T> {
    /// Create a new request with the specified message.
    ///
    /// This adds the default "content-type" header used for hRPC unary requests.
    pub fn new(message: T) -> Self {
        Self {
            message,
            header_map: {
                #[allow(clippy::mutable_key_type)]
                let mut map: HeaderMap = HeaderMap::with_capacity(1);
                map.insert("content-type", "application/hrpc".parse().unwrap());
                map
            },
        }
    }

    /// Create an empty request.
    ///
    /// This is useful for hRPC socket requests, since they don't send any messages.
    pub fn empty() -> Request<()> {
        Request {
            message: (),
            header_map: HeaderMap::new(),
        }
    }

    /// Change / add a header.
    pub fn header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.header_map.insert(key, value);
        self
    }

    /// Change the contained message.
    pub fn message<S>(self, message: S) -> Request<S> {
        let Request {
            message: _,
            header_map,
        } = self;

        Request {
            message,
            header_map,
        }
    }

    /// Get a reference to the inner message.
    pub const fn get_message(&self) -> &T {
        &self.message
    }

    /// Get a reference to the inner header map.
    pub const fn get_header_map(&self) -> &HeaderMap {
        &self.header_map
    }

    /// Get a header.
    pub fn get_header(&self, key: &HeaderName) -> Option<&HeaderValue> {
        self.header_map.get(key)
    }

    /// Destructure this request into parts.
    pub fn into_parts(self) -> (T, HeaderMap) {
        (self.message, self.header_map)
    }

    /// Create a request from parts.
    pub fn from_parts(parts: (T, HeaderMap)) -> Self {
        Self {
            message: parts.0,
            header_map: parts.1,
        }
    }
}

/// Trait used for blanket impls on generated protobuf types.
pub trait IntoRequest<T> {
    /// Convert this to a request.
    fn into_request(self) -> Request<T>;
}

impl<T> IntoRequest<T> for T {
    fn into_request(self) -> Request<Self> {
        Request::new(self)
    }
}

impl<T> IntoRequest<T> for Request<T> {
    fn into_request(self) -> Request<T> {
        self
    }
}

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
