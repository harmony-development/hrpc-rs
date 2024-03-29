//! A HTTP transport that compiles to WASM and works on the Web.
use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    ops::Not,
    task::{Context, Poll},
};

use bytes::{Buf, Bytes, BytesMut};
use futures_util::{future::LocalBoxFuture, Future, FutureExt, Stream, StreamExt};
use http::{header, HeaderMap, HeaderValue, Uri};
use js_sys::Uint8Array;
use prost::Message;
use reqwasm::http::Request as WasmRequest;
use tower::Service;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use super::{check_uri, map_scheme_to_ws, InvalidServerUrl};
use crate::{
    body::Body,
    client::{
        transport::{is_socket_request, SocketChannels, TransportError},
        ClientError,
    },
    common::transport::{
        http::{version_header_name, ws_version, HRPC_VERSION_HEADER},
        ws_wasm::WebSocket,
    },
    proto::Error as HrpcError,
    request::{self, BoxRequest},
    response::{self, BoxResponse},
    BoxError, Response, HRPC_CONTENT_MIMETYPE, HRPC_SPEC_VERSION,
};

/// HTTP client that compiles to WASM and works on the Web.
///
/// This client will:
///
/// - (For unary requests) Look for a [`http::HeaderMap`] in a [`Request`]s
/// extensions, if it exists the headers in there will be added to the HTTP
/// request that will be used. Headers added by the client by default will
/// be overwrited, so take care while inserting headers that are mentioned
/// in [the spec].
/// - Adds [`http::StatusCode`], to the [`Response`] extension returned in
/// unary requests. Also adds [`HRPC_VERSION_HEADER`] and [`header::CONTENT_TYPE`]
/// to a [`http::HeaderMap`] in the response's extensions. See limitations
/// as for why all of the headers aren't added.
/// - (For streaming requests) Look for [`SocketProtocols`] in a [`Request`]s
/// extensions, if it exists it will be used to set the protocols of the
/// WebSocket that will be created. This will overwrite the protocols added
/// by this client.
///
/// # Limitations
///
/// - This client does not support setting headers for streaming requests,
/// due to Web's `WebSocket` API's limitations.
/// - This client does not add all of the headers of the response to the [`HeaderMap`]
/// of the returned [`Response`]'s extensions. This is due to
/// https://github.com/rustwasm/wasm-bindgen/pull/1913 not being in `wasm-bindgen`.
/// - If inserting [`HeaderMap`] in [`Request`] extensions to add headers,
/// keep in mind that **header values that are not valid strings won't be added**.
/// See [`HeaderValue`]'s `to_str` method for information about a "valid string".
///
/// [the spec]: https://github.com/harmony-development/hrpc/blob/main/protocol/SPEC.md
#[derive(Debug, Clone)]
pub struct Wasm {
    server: Uri,
    check_spec_version: bool,
}

impl Wasm {
    /// Create a new client.
    pub fn new(server: Uri) -> Result<Self, WasmError> {
        Ok(Self {
            server: check_uri(server).map_err(WasmError::InvalidServerUrl)?,
            check_spec_version: true,
        })
    }

    /// Set whether to check for spec version.
    ///
    /// Note that this only affects unary requests.
    pub fn check_spec_version(mut self, enabled: bool) -> Self {
        self.check_spec_version = enabled;
        self
    }
}

impl Service<BoxRequest> for Wasm {
    type Response = BoxResponse;

    type Error = TransportError<WasmError>;

    type Future = CallFuture;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        if is_socket_request(&req) {
            let request::Parts {
                mut extensions,
                endpoint,
                ..
            } = req.into();

            let scheme =
                map_scheme_to_ws(self.server.scheme_str().expect_throw("must have scheme"))
                    .expect_throw("scheme can't be anything other than https or http");
            let port = self
                .server
                .port()
                .map_or_else(String::new, |port| format!(":{}", port.as_str()));
            let host = self
                .server
                .host()
                .expect_throw("expected host on server URI, this is a bug");
            let path = endpoint.trim_start_matches('/');
            let url = format!("{}://{}{}/{}", scheme, host, port, path);

            let sock_protocols = extensions
                .remove::<SocketProtocols>()
                .map_or_else(|| vec![Cow::Owned(ws_version())], |s| s.protocols);

            let inner = Box::pin(async move {
                let protocols = Some(sock_protocols.iter().map(|s| s.as_ref()).collect());
                let (_, ws_stream) = ws_stream_wasm::WsMeta::connect(url, protocols)
                    .await
                    .map_err(WasmError::SocketInitError)?;

                let ws = WebSocket::new(ws_stream);
                let (ws_tx, ws_rx) = ws.split();
                let chans = SocketChannels::new(ws_tx, ws_rx);

                let mut resp = BoxResponse::empty();
                resp.extensions_mut().insert(chans);

                Ok(resp)
            });

            CallFuture { inner }
        } else {
            let request::Parts {
                body,
                mut extensions,
                endpoint,
            } = req.into();

            let req_url = format!("{}{}", self.server, endpoint.trim_start_matches('/'));

            let mut request = WasmRequest::post(req_url.as_str());
            request = request
                .header(HRPC_VERSION_HEADER, HRPC_SPEC_VERSION)
                .header(header::CONTENT_TYPE.as_str(), HRPC_CONTENT_MIMETYPE);

            if let Some(header_map) = extensions.remove::<HeaderMap>() {
                for (key, value) in header_map.iter() {
                    if let Ok(value) = value.to_str() {
                        request = request.header(key.as_str(), value);
                    }
                }
            }

            let check_spec_version = self.check_spec_version;

            let inner = Box::pin(async move {
                let mut data = body.aggregate().await.map_err(WasmError::BodyError)?;

                request = request.body({
                    let buf = Uint8Array::new_with_length(
                        data.remaining()
                            .try_into()
                            .expect_throw("can't send data bigger than a u32"),
                    );
                    let mut offset = 0;
                    while data.has_remaining() {
                        let chunk_len = {
                            let chunk = data.chunk();
                            unsafe { buf.set(&Uint8Array::view(chunk), offset) }
                            chunk.len()
                        };
                        let chunk_len_js: u32 = chunk_len
                            .try_into()
                            .expect_throw("can't send data bigger than a u32");
                        offset += chunk_len_js;
                        data.advance(chunk_len);
                    }
                    buf
                });

                let response = request.send().await.map_err(WasmError::HttpError)?;

                let status = http::StatusCode::from_u16(response.status())
                    .expect_throw("got invalid status code from response");

                if status.is_success().not() {
                    let raw_error = response.binary().await.map_err(WasmError::HttpError)?;
                    let hrpc_error = HrpcError::decode(raw_error.as_ref())
                        .unwrap_or_else(|_| HrpcError::invalid_hrpc_error(raw_error));
                    return Err((ClientError::EndpointError {
                        hrpc_error,
                        endpoint,
                    })
                    .into());
                }

                let mut resp = Response::empty();

                let content_type = response
                    .headers()
                    .get(header::CONTENT_TYPE.as_str())
                    .expect_throw("header name is valid");

                if !content_type
                    .split(';')
                    .next()
                    .map_or(false, |v| v == HRPC_CONTENT_MIMETYPE)
                {
                    return Err(ClientError::ContentNotSupported.into());
                }

                if let Ok(value) = HeaderValue::from_str(&content_type) {
                    resp.get_or_insert_header_map()
                        .insert(header::CONTENT_TYPE, value);
                }

                let hrpc_version = response
                    .headers()
                    .get(HRPC_VERSION_HEADER)
                    .expect_throw("header name is valid");

                if check_spec_version && hrpc_version.trim() != HRPC_SPEC_VERSION {
                    tracing::debug!(
                        "incompatible spec version {:?} (ours is {})",
                        hrpc_version,
                        HRPC_SPEC_VERSION
                    );
                    return Err(ClientError::IncompatibleSpecVersion(hrpc_version).into());
                }

                if let Ok(value) = HeaderValue::from_str(&hrpc_version) {
                    resp.get_or_insert_header_map()
                        .insert(version_header_name(), value);
                }

                resp.extensions_mut().insert(status);

                let body = wasm_streams::ReadableStream::from_raw(
                    response
                        .body()
                        .expect_throw("response body was used before we used it -- this is a bug")
                        .dyn_into()
                        .expect_throw("failed to get body from response"),
                );

                let body = body.into_stream().map(|buf_js| {
                    let buffer = Uint8Array::new(&buf_js.map_err(|_| {
                        Box::new(HrpcError::from((
                            "hrpcrs.wasm.body-error",
                            "error occured while streaming response body",
                        )))
                    })?);
                    let fill_len = buffer.length() as usize;
                    let mut bytes = BytesMut::with_capacity(fill_len);
                    // Safety: this is safe because `copy_to` doesn't read
                    // anything from `bytes`, so we don't need to initialize it
                    unsafe {
                        bytes.set_len(fill_len);
                        buffer.copy_to(&mut bytes);
                    }
                    Ok(bytes.freeze())
                });

                let resp = Response::from(response::Parts {
                    body: Body::new(SendSyncBody { inner: body }),
                    ..response::Parts::from(resp)
                });

                Ok(resp)
            });

            CallFuture { inner }
        }
    }
}

/// Type that can be inserted into a [`Request`]'s extensions to set a socket's
/// protocols.
pub struct SocketProtocols {
    protocols: Vec<Cow<'static, str>>,
}

impl SocketProtocols {
    /// Create a new [`SocketProtocols`].
    pub fn new(protocols: impl Into<Vec<Cow<'static, str>>>) -> Self {
        Self {
            protocols: protocols.into(),
        }
    }
}

/// Errors that can occur while using [`Wasm`] client.
#[derive(Debug)]
pub enum WasmError {
    /// Occurs if the URL passed to [`Wasm`] is invalid.
    InvalidServerUrl(InvalidServerUrl),
    /// Occurs if [`reqwasm`] returns an error.
    HttpError(reqwasm::Error),
    /// Occurs if an error happens while aggregating a request body.
    BodyError(BoxError),
    /// Occurs if an error happens while initializing a socket.
    SocketInitError(ws_stream_wasm::WsErr),
}

impl From<WasmError> for TransportError<WasmError> {
    fn from(err: WasmError) -> Self {
        TransportError::Transport(err)
    }
}

impl Display for WasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WasmError::InvalidServerUrl(err) => write!(f, "invalid server url passed: {}", err),
            WasmError::HttpError(err) => {
                write!(f, "HTTP error: {}", err)
            }
            WasmError::BodyError(err) => write!(f, "error while aggregating body: {}", err),
            WasmError::SocketInitError(err) => {
                write!(f, "error while initializing socket: {}", err)
            }
        }
    }
}

impl StdError for WasmError {}

struct SendSyncBody<S> {
    inner: S,
}

impl<S> Stream for SendSyncBody<S>
where
    S: Stream<Item = Result<Bytes, BoxError>> + Unpin,
{
    type Item = S::Item;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

// SAFETY: this is safe on WASM since it is single-threaded
unsafe impl<S> Send for SendSyncBody<S> {}
unsafe impl<S> Sync for SendSyncBody<S> {}

/// Call future for [`Wasm`].
pub struct CallFuture {
    inner: LocalBoxFuture<'static, Result<BoxResponse, TransportError<WasmError>>>,
}

impl Future for CallFuture {
    type Output = Result<BoxResponse, TransportError<WasmError>>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_unpin(cx)
    }
}

// SAFETY: this is safe on WASM since it is single-threaded
unsafe impl Send for CallFuture {}
