use std::borrow::Cow;

use bytes::Bytes;
use futures_util::{Sink, Stream, StreamExt, TryStreamExt};
use http::{
    header::{self, HeaderName},
    HeaderMap, HeaderValue, Method, StatusCode,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{
    tungstenite::{Error as WsError, Message as WsMessage},
    WebSocketStream,
};

use crate::{
    body::Body,
    common::{extensions::Extensions, socket::SocketMessage},
    proto::{Error as HrpcError, HrpcErrorIdentifier},
    request, response, BoxError, Request, Response, HRPC_HEADER,
};

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
pub fn hrpc_header_value() -> HeaderValue {
    unsafe { HeaderValue::from_maybe_shared_unchecked(Bytes::from_static(HRPC_HEADER)) }
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

// Conversion function impls

impl<T> Response<T> {
    /// Convert this hRPC response into a unary HTTP response.
    pub fn into_unary_response(self) -> HttpResponse {
        let mut parts = response::Parts::from(self);

        let mut resp = http::Response::builder()
            .header(http::header::CONTENT_TYPE, hrpc_header_value())
            .header(http::header::ACCEPT, hrpc_header_value())
            .body(box_body(parts.body))
            .unwrap();

        if let Some(exts) = parts.extensions.remove::<http::Extensions>() {
            *resp.extensions_mut() = exts;
        }

        if let Some(headers) = parts.extensions.remove::<HeaderMap>() {
            resp.headers_mut().extend(headers);
        }

        resp.extensions_mut().insert(parts.extensions);

        resp
    }
}

impl<T> Request<T> {
    /// Try to create a [`Request`] from a unary [`HttpRequest`].
    pub fn from_unary_request(req: HttpRequest) -> Result<Request<T>, (StatusCode, &'static str)> {
        let (parts, body) = req.into_parts();

        if parts.method != Method::POST {
            return Err((StatusCode::METHOD_NOT_ALLOWED, "method must be POST"));
        }

        if !parts.headers.header_eq(&header::CONTENT_TYPE, HRPC_HEADER) {
            return Err((
                StatusCode::BAD_REQUEST,
                "request content type not supported",
            ));
        }

        let endpoint = Cow::Owned(parts.uri.path().to_string());

        let mut extensions = Extensions::new();
        extensions.insert(parts.extensions);
        extensions.insert(parts.headers);
        extensions.insert(parts.method);
        extensions.insert(parts.version);
        extensions.insert(parts.uri);

        let req = Request::from(request::Parts {
            body: body.into(),
            extensions,
            endpoint,
        });

        Ok(req)
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
