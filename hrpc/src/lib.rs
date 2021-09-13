//! Common code used in hRPC code generation.
use std::{
    error::Error as StdError,
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
    pin::Pin,
};

use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
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

pub(crate) const HRPC_HEADER: &[u8] = b"application/hrpc";

pub(crate) fn hrpc_header_value() -> HeaderValue {
    unsafe { http::HeaderValue::from_maybe_shared_unchecked(Bytes::from_static(HRPC_HEADER)) }
}

pub type BodyError = Box<dyn StdError + Send + Sync>;
pub type BodyResult<T> = Result<T, BodyError>;
pub type BodyStream = Pin<Box<dyn futures_util::Stream<Item = BodyResult<Bytes>> + Send + Sync>>;

pub enum BodyKind<T> {
    DecodedMessage(T),
    Stream(BodyStream),
}

impl<T: prost::Message + Default> BodyKind<T> {
    pub async fn into_message(self) -> BodyResult<Result<T, prost::DecodeError>> {
        match self {
            BodyKind::DecodedMessage(msg) => Ok(Ok(msg)),
            BodyKind::Stream(mut body) => {
                use bytes::{Buf, BufMut};

                // If there's only 1 chunk, we can just return Buf::to_bytes()
                let mut first = if let Some(buf) = body.next().await {
                    buf?
                } else {
                    return Ok(Err(prost::DecodeError::new("no message in stream")));
                };

                let second = if let Some(buf) = body.next().await {
                    buf?
                } else {
                    return Ok(T::decode(first.copy_to_bytes(first.remaining()).as_ref()));
                };

                // With more than 1 buf, we gotta flatten into a Vec first.
                let cap = first.remaining() + second.remaining() + body.size_hint().0 as usize;
                let mut vec = Vec::with_capacity(cap);
                vec.put(first);
                vec.put(second);

                while let Some(buf) = body.next().await {
                    vec.put(buf?);
                }

                Ok(T::decode(vec.as_slice()))
            }
        }
    }
}

impl<T: prost::Message + Default> BodyKind<Option<T>> {
    pub async fn into_optional_message(self) -> BodyResult<Result<Option<T>, prost::DecodeError>> {
        match self {
            BodyKind::DecodedMessage(msg) => Ok(Ok(msg)),
            BodyKind::Stream(mut body) => {
                use bytes::{Buf, BufMut};

                // If there's only 1 chunk, we can just return Buf::to_bytes()
                let mut first = if let Some(buf) = body.next().await {
                    buf?
                } else {
                    return Ok(Ok(None));
                };

                let second = if let Some(buf) = body.next().await {
                    buf?
                } else {
                    return Ok(T::decode(first.copy_to_bytes(first.remaining()).as_ref()).map(Some));
                };

                // With more than 1 buf, we gotta flatten into a Vec first.
                let cap = first.remaining() + second.remaining() + body.size_hint().0 as usize;
                let mut vec = Vec::with_capacity(cap);
                vec.put(first);
                vec.put(second);

                while let Some(buf) = body.next().await {
                    vec.put(buf?);
                }

                Ok(T::decode(vec.as_slice()).map(Some))
            }
        }
    }
}

impl<T: Debug> Debug for BodyKind<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BodyKind::DecodedMessage(msg) => msg.fmt(f),
            BodyKind::Stream(_) => write!(f, "stream"),
        }
    }
}

#[derive(Debug)]
/// A hRPC request.
pub struct Request<T> {
    body: BodyKind<T>,
    header_map: HeaderMap,
    socket_addr: Option<SocketAddr>,
}

impl<T> Request<T> {
    /// Create a new request with the specified message.
    ///
    /// This adds the default "content-type" header used for hRPC unary requests.
    pub fn new(message: T) -> Self {
        Self {
            body: BodyKind::DecodedMessage(message),
            header_map: {
                #[allow(clippy::mutable_key_type)]
                let mut map: HeaderMap = HeaderMap::with_capacity(1);
                map.insert(http::header::CONTENT_TYPE, hrpc_header_value());
                map
            },
            socket_addr: None,
        }
    }

    /// Create an empty request.
    ///
    /// This is useful for hRPC socket requests, since they don't send any messages.
    pub fn empty() -> Request<()> {
        Request {
            body: BodyKind::DecodedMessage(()),
            header_map: HeaderMap::new(),
            socket_addr: None,
        }
    }

    /// Change / add a header.
    pub fn header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.header_map.insert(key, value);
        self
    }

    /// Change the contained body.
    pub fn body<S>(self, body: BodyKind<S>) -> Request<S> {
        let Request {
            body: _,
            header_map,
            socket_addr,
        } = self;

        Request {
            body,
            header_map,
            socket_addr,
        }
    }

    /// Get a reference to the body.
    pub async fn get_body(&self) -> &BodyKind<T> {
        &self.body
    }

    /// Get a reference to the inner header map.
    pub const fn get_header_map(&self) -> &HeaderMap {
        &self.header_map
    }

    /// Get a reference to the inner socket address this request came from.
    ///
    /// It will be none if the underlying transport doesn't use socket addresses.
    pub const fn get_socket_addr(&self) -> Option<&SocketAddr> {
        self.socket_addr.as_ref()
    }

    /// Get a header.
    pub fn get_header(&self, key: &HeaderName) -> Option<&HeaderValue> {
        self.header_map.get(key)
    }

    /// Destructure this request into parts.
    pub fn into_parts(self) -> (BodyKind<T>, HeaderMap, Option<SocketAddr>) {
        (self.body, self.header_map, self.socket_addr)
    }

    /// Create a request from parts.
    pub fn from_parts(parts: (BodyKind<T>, HeaderMap, Option<SocketAddr>)) -> Self {
        Self {
            body: parts.0,
            header_map: parts.1,
            socket_addr: parts.2,
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

/// Encodes a protobuf message into the given `BytesMut` buffer.
pub fn encode_protobuf_message_to(buf: &mut BytesMut, msg: impl prost::Message) {
    buf.reserve(msg.encoded_len().saturating_sub(buf.len()));
    buf.clear();
    // ignore the error since this can never fail
    let _ = msg.encode(buf);
}

/// Encodes a protobuf message into a new `BytesMut` buffer.
pub fn encode_protobuf_message(msg: impl prost::Message) -> BytesMut {
    let mut buf = BytesMut::new();
    encode_protobuf_message_to(&mut buf, msg);
    buf
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
