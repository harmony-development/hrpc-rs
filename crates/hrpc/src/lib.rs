//! Common code used in hRPC code generation.
#![deny(missing_docs)]

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
};

use body::{box_body, BoxBody, HyperBody};
use bytes::BytesMut;
use http::{HeaderMap, HeaderValue};
use prost::Message as PbMsg;

/// Some re-exported crates that might be useful while writing software with `hrpc`.
pub mod exports {
    pub use bytes;
    pub use futures_util;
    pub use prost;
    pub use tracing;

    pub use http;
    pub use http_body;
    pub use hyper;
    pub use tower;
    pub use tower_http;

    #[cfg(feature = "client")]
    pub use rustls_native_certs;
    pub use tokio_tungstenite::{self, tungstenite};
}

/// Common client types and functions.
#[cfg(feature = "client")]
pub mod client;
/// Common server types and functions.
#[cfg(feature = "server")]
pub mod server;

/// Body utitilies and types.
pub mod body;
/// The hRPC generated protocol.
pub mod proto;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
/// HTTP request used by hRPC.
pub type HttpRequest = http::Request<HyperBody>;
/// HTTP response used by hRPC.
pub type HttpResponse = http::Response<BoxBody>;

/// The hRPC protobuf mimetype.
pub const HRPC_HEADER: &[u8] = b"application/hrpc";

pub(crate) fn hrpc_header_value() -> HeaderValue {
    unsafe {
        http::HeaderValue::from_maybe_shared_unchecked(bytes::Bytes::from_static(HRPC_HEADER))
    }
}

#[derive(Debug)]
/// A hRPC request.
pub struct Request<T> {
    body: HyperBody,
    header_map: HeaderMap,
    message: PhantomData<T>,
}

impl Request<()> {
    /// Create an empty request.
    ///
    /// This is useful for hRPC socket requests, since they don't send any messages.
    pub fn empty() -> Request<()> {
        Self::new_body(HyperBody::empty())
    }
}

impl<T> Request<T> {
    pub(crate) fn new_body(body: HyperBody) -> Self {
        Self {
            body,
            message: PhantomData,
            header_map: {
                #[allow(clippy::mutable_key_type)]
                let mut map: HeaderMap = HeaderMap::with_capacity(1);
                map.insert(http::header::CONTENT_TYPE, hrpc_header_value());
                map
            },
        }
    }

    /// Get a reference to the inner header map.
    pub fn header_map(&self) -> &HeaderMap {
        &self.header_map
    }

    /// Get a mutable reference to the inner header map.
    pub fn header_map_mut(&mut self) -> &mut HeaderMap {
        &mut self.header_map
    }
}

impl<T: PbMsg> Request<T> {
    /// Create a new request with the specified message.
    ///
    /// This adds the [`HRPC_HEADER`] to the [`http::header::CONTENT_TYPE`]
    /// header for hRPC unary requests.
    pub fn new(msg: T) -> Self {
        let encoded = encode_protobuf_message(msg).freeze();
        Self::new_body(HyperBody::from(encoded))
    }
}

/// Errors that can occur while decoding the body of a [`Request`].
#[derive(Debug)]
pub enum DecodeBodyError {
    /// The body contained an invalid protobuf message.
    InvalidProtoMessage(prost::DecodeError),
    /// An error occured while reading the body.
    InvalidBody(BoxError),
}

impl Display for DecodeBodyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtoMessage(err) => write!(
                f,
                "body was detected to be protobuf, but contains invalid protobuf message: {}",
                err
            ),
            Self::InvalidBody(err) => write!(f, "error occured while aggregating body: {}", err),
        }
    }
}

impl StdError for DecodeBodyError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidProtoMessage(err) => Some(err),
            Self::InvalidBody(err) => err.source(),
        }
    }
}

impl<T: PbMsg + Default> Request<T> {
    /// Extract the body from the request and decode it into the message.
    #[inline]
    pub async fn into_message(self) -> Result<T, DecodeBodyError> {
        decode_body(box_body(self.body)).await
    }
}

pub(crate) async fn decode_body<T: PbMsg + Default>(body: BoxBody) -> Result<T, DecodeBodyError> {
    let buf = hyper::body::aggregate(body)
        .await
        .map_err(DecodeBodyError::InvalidBody)?;
    let decoded = T::decode(buf).map_err(DecodeBodyError::InvalidProtoMessage)?;
    Ok(decoded)
}

/// hRPC response type.
pub struct Response<T> {
    data: T,
}

impl<T> Response<T> {
    /// Create a new hRPC response.
    pub fn new(data: T) -> Response<T> {
        Self { data }
    }

    /// Extract the message this response contains.
    pub fn into_message(self) -> T {
        self.data
    }
}

impl<T> From<T> for Response<T> {
    fn from(data: T) -> Self {
        Self { data }
    }
}

/// Trait used for blanket impls on generated protobuf types.
pub trait IntoRequest<T> {
    /// Convert this to a hRPC request.
    fn into_request(self) -> Request<T>;
}

impl<T: PbMsg> IntoRequest<T> for T {
    fn into_request(self) -> Request<Self> {
        Request::new(self)
    }
}

impl<T> IntoRequest<T> for Request<T> {
    fn into_request(self) -> Request<T> {
        self
    }
}

/// Trait used for converting any type to a Response type.
pub trait IntoResponse<T> {
    /// Convert this to a hRPC response.
    fn into_response(self) -> Response<T>;
}

impl<T> IntoResponse<T> for T {
    fn into_response(self) -> Response<T> {
        Response::new(self)
    }
}

impl<T> IntoResponse<T> for Response<T> {
    fn into_response(self) -> Response<T> {
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