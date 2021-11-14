use bytes::Bytes;
use futures_util::{Sink, Stream, StreamExt, TryStreamExt};
use http::{header::HeaderName, HeaderMap, HeaderValue, StatusCode};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{
    tungstenite::{Error as WsError, Message as WsMessage},
    WebSocketStream,
};

use crate::{
    body::Body,
    common::socket::SocketMessage,
    proto::{Error as HrpcError, HrpcErrorIdentifier},
    BoxError, Request, Response, HRPC_CONTENT_MIMETYPE, HRPC_SPEC_VERSION,
};

/// The hRPC version header used in unary requests.
pub const HRPC_VERSION_HEADER: &str = "hrpc-version";

/// A boxed HTTP body. This is used to unify response bodies.
pub type BoxBody = http_body::combinators::BoxBody<Bytes, BoxError>;
/// A HTTP request.
pub type HttpRequest = http::Request<hyper::Body>;
/// A HTTP response.
pub type HttpResponse = http::Response<BoxBody>;

/// Convert a body with the correct attributes to a [`BoxBody`].
pub fn box_body<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    B::Error: Into<BoxError>,
{
    BoxBody::new(body.map_err(Into::into))
}

/// Create a header value for the hRPC content type.
pub fn content_header_value() -> HeaderValue {
    unsafe { HeaderValue::from_maybe_shared_unchecked(Bytes::from_static(HRPC_CONTENT_MIMETYPE)) }
}

/// Create the spec compliant WS protocol with hRPC version, as a header value.
pub fn ws_version_header_value() -> HeaderValue {
    unsafe { HeaderValue::from_maybe_shared_unchecked(ws_version().into_bytes()) }
}

/// Create the spec compliant WS protocol with hRPC version.
pub fn ws_version() -> String {
    format!("hrpc{}", HRPC_SPEC_VERSION)
}

/// Create a header value with hRPC version, for the [`HRPC_VERSION_HEADER`] header.
pub fn version_header_value() -> HeaderValue {
    unsafe {
        HeaderValue::from_maybe_shared_unchecked(Bytes::from_static(HRPC_SPEC_VERSION.as_bytes()))
    }
}

/// Create a header name for the hRPC version header (see [`HRPC_VERSION_HEADER`]).
pub fn version_header_name() -> HeaderName {
    HeaderName::from_static(HRPC_VERSION_HEADER)
}

/// Helper methods for working with `HeaderMap`.
pub trait HeaderMapExt {
    /// Check if a header is equal to a bytes array. Ignores casing.
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool;
    /// Check if a header contains a string. Ignores casing for the header value.
    fn header_contains_str(&self, key: &HeaderName, value: &str) -> bool;
}

impl HeaderMapExt for HeaderMap {
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool {
        self.get(key).map_or(false, |header| {
            header.as_bytes().eq_ignore_ascii_case(value)
        })
    }

    fn header_contains_str(&self, key: &HeaderName, pat: &str) -> bool {
        self.get(key).map_or(false, |header| {
            header
                .to_str()
                .map_or(false, |value| value.to_ascii_lowercase().contains(pat))
        })
    }
}

impl_exts::impl_exts!(Request<T>);
impl_exts::impl_exts!(Response<T>);

impl<T> Request<T> {
    /// Get an immutable reference to the HTTP method.
    #[inline]
    pub fn http_method(&self) -> Option<&http::Method> {
        self.extensions().get::<http::Method>()
    }

    /// Get an immutable reference to the HTTP version.
    #[inline]
    pub fn http_version(&self) -> Option<&http::Version> {
        self.extensions().get::<http::Version>()
    }

    /// Get an immutable reference to the HTTP URI.
    #[inline]
    pub fn http_uri(&self) -> Option<&http::Uri> {
        self.extensions().get::<http::Uri>()
    }
}

mod impl_exts {
    macro_rules! impl_exts {
        ($t:ty) => {
            impl<T> $t {
                /// Get an immutable reference to the header map.
                #[inline]
                pub fn header_map(&self) -> Option<&HeaderMap> {
                    self.extensions().get::<HeaderMap>()
                }

                /// Get a mutable reference to the header map.
                #[inline]
                pub fn header_map_mut(&mut self) -> Option<&mut HeaderMap> {
                    self.extensions_mut().get_mut::<HeaderMap>()
                }

                /// Get a mutable reference to the header map, inserting a new one
                /// if it doesn't already exist.
                #[inline]
                pub fn get_or_insert_header_map(&mut self) -> &mut HeaderMap {
                    if !self.extensions().contains::<HeaderMap>() {
                        self.extensions_mut().insert(HeaderMap::new());
                    }
                    self.extensions_mut().get_mut::<HeaderMap>().unwrap()
                }

                /// Get an immutable reference to the HTTP extensions.
                #[inline]
                pub fn http_extensions(&self) -> Option<&http::Extensions> {
                    self.extensions().get::<http::Extensions>()
                }

                /// Get a mutable reference to the HTTP extensions.
                #[inline]
                pub fn http_extensions_mut(&mut self) -> Option<&mut http::Extensions> {
                    self.extensions_mut().get_mut::<http::Extensions>()
                }

                /// Get a mutable reference to the HTTP extensions, inserting a new one
                /// if it doesn't already exist.
                #[inline]
                pub fn get_or_insert_http_extensions(&mut self) -> &mut http::Extensions {
                    if !self.extensions().contains::<http::Extensions>() {
                        self.extensions_mut().insert(http::Extensions::new());
                    }
                    self.extensions_mut().get_mut::<http::Extensions>().unwrap()
                }
            }
        };
    }

    pub(crate) use impl_exts;
}

// Trait impls

impl From<Body> for hyper::Body {
    fn from(body: Body) -> Self {
        hyper::Body::wrap_stream(body)
    }
}

impl From<hyper::Body> for Body {
    fn from(hbody: hyper::Body) -> Self {
        Body::new(
            hbody
                .into_stream()
                .map_err(|err| -> BoxError { Box::new(err) }),
        )
    }
}

impl http_body::Body for Body {
    type Data = Bytes;

    type Error = BoxError;

    fn poll_data(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Data, Self::Error>>> {
        self.poll_next_unpin(cx)
    }

    fn poll_trailers(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<Option<HeaderMap>, Self::Error>> {
        std::task::Poll::Ready(Ok(None))
    }
}

/// Wrapper over a [`tokio_tungstenite::WebSocketStream`] that produces
/// and takes [`SocketMessage`].
pub struct WebSocket<S> {
    inner: WebSocketStream<S>,
}

impl<S> WebSocket<S> {
    /// Create a new web socket by wrapping a [`tokio_tungstenite::WebSocketStream`].
    pub fn new(inner: WebSocketStream<S>) -> Self {
        Self { inner }
    }
}

impl<S> Stream for WebSocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Item = Result<SocketMessage, HrpcError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_next(cx)
            .map_ok(SocketMessage::from)
            .map_err(HrpcError::from)
    }
}

impl<S> Sink<SocketMessage> for WebSocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Error = HrpcError;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_ready(cx).map_err(HrpcError::from)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: SocketMessage,
    ) -> Result<(), Self::Error> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.start_send(item.into()).map_err(HrpcError::from)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_flush(cx).map_err(HrpcError::from)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_close(cx).map_err(HrpcError::from)
    }
}

impl From<WsMessage> for SocketMessage {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Binary(data) => Self::Binary(data),
            WsMessage::Close(_) => Self::Close,
            WsMessage::Text(data) => Self::Text(data),
            WsMessage::Pong(data) => Self::Pong(data),
            WsMessage::Ping(data) => Self::Ping(data),
        }
    }
}

impl From<SocketMessage> for WsMessage {
    fn from(msg: SocketMessage) -> WsMessage {
        match msg {
            SocketMessage::Binary(data) => Self::Binary(data),
            SocketMessage::Close => Self::Close(None),
            SocketMessage::Text(data) => Self::Text(data),
            SocketMessage::Pong(data) => Self::Pong(data),
            SocketMessage::Ping(data) => Self::Ping(data),
        }
    }
}

impl From<WsError> for HrpcError {
    fn from(err: WsError) -> Self {
        HrpcError::default()
            .with_identifier("hrpcrs.socket-error")
            .with_message(err.to_string())
    }
}

impl From<HrpcErrorIdentifier> for StatusCode {
    fn from(id: HrpcErrorIdentifier) -> Self {
        match id {
            HrpcErrorIdentifier::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            HrpcErrorIdentifier::NotFound => StatusCode::NOT_FOUND,
            HrpcErrorIdentifier::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            HrpcErrorIdentifier::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            HrpcErrorIdentifier::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
