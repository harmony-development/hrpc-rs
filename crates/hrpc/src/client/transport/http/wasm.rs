//! A HTTP transport that compiles to WASM and works on the Web.
use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    ops::Not,
    task::{Context, Poll},
};

use bytes::Buf;
use futures_util::StreamExt;
use http::{header, HeaderMap, Uri};
use js_sys::Uint8Array;
use prost::Message;
use reqwasm::http::Request as WasmRequest;
use tower::Service;
use wasm_bindgen::UnwrapThrowExt;

use super::{check_uri, map_scheme_to_ws, InvalidServerUrl};
use crate::{
    body::Body,
    client::{
        transport::{box_socket_stream_sink, CallResult, TransportRequest, TransportResponse},
        ClientError,
    },
    common::transport::{
        http::{ws_version, HRPC_VERSION_HEADER},
        ws_wasm::WebSocket,
    },
    proto::Error as HrpcError,
    request, BoxError, Response, HRPC_CONTENT_MIMETYPE, HRPC_SPEC_VERSION,
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
/// - (For streaming requests) Look for [`SocketProtocols`] in a [`Request`]s
/// extensions, if it exists it will be used to set the protocols of the
/// WebSocket that will be created. This will overwrite the protocols added
/// by this client.
///
/// # Limitations
///
/// - This client does not support setting headers for streaming requests,
/// due to Web's `WebSocket` API's limitations.
/// - This client does not add a [`HeaderMap`] to the returned [`Response`]'s
/// extensions. This is due to https://github.com/rustwasm/wasm-bindgen/pull/1913
/// not being in `wasm-bindgen`.
/// - If inserting [`HeaderMap`] in [`Request`] extensions to add headers,
/// keep in mind that **header values that are not valid strings won't be added**.
/// See [`HeaderValue`]'s `to_str` method for information about a "valid string".
///
/// [the spec]: https://github.com/harmony-development/hrpc/blob/main/protocol/SPEC.md
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
    /// Not checking spec version can save some allocations.
    pub fn check_spec_version(mut self, enabled: bool) -> Self {
        self.check_spec_version = enabled;
        self
    }
}

impl Service<TransportRequest> for Wasm {
    type Response = TransportResponse;

    type Error = ClientError<WasmError>;

    type Future = CallResult<'static, TransportResponse, WasmError>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: TransportRequest) -> Self::Future {
        match req {
            TransportRequest::Socket(req) => {
                let request::Parts {
                    mut extensions,
                    endpoint,
                    ..
                } = req.into();

                let scheme =
                    map_scheme_to_ws(self.server.scheme_str().expect_throw("must have scheme"))
                        .expect_throw("scheme can't be anything other than https or http");
                let url = format!(
                    "{}://{}:{}/{}",
                    scheme,
                    self.server
                        .host()
                        .expect("expected host on server URI, this is a bug"),
                    self.server
                        .port_u16()
                        .expect("expected port on server URI, this is a bug"),
                    endpoint.trim_start_matches('/'),
                );

                let sock_protocols = extensions
                    .remove::<SocketProtocols>()
                    .map_or_else(|| vec![Cow::Owned(ws_version())], |s| s.protocols);

                Box::pin(async move {
                    let protocols = Some(sock_protocols.iter().map(|s| s.as_ref()).collect());
                    let (_, ws_stream) = ws_stream_wasm::WsMeta::connect(url, protocols)
                        .await
                        .map_err(WasmError::SocketInitError)?;
                    let ws = WebSocket::new(ws_stream);
                    let (ws_tx, ws_rx) = ws.split();
                    let (tx, rx) = box_socket_stream_sink(ws_tx, ws_rx);

                    Ok(TransportResponse::new_socket(tx, rx))
                })
            }
            TransportRequest::Unary(req) => {
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

                Box::pin(async move {
                    let data = body.aggregate().await.map_err(WasmError::BodyError)?;

                    request = request.body({
                        let buf = Uint8Array::new_with_length(
                            data.remaining()
                                .try_into()
                                .expect_throw("can't send data bigger than a u32"),
                        );
                        let mut offset = 0;
                        while data.has_remaining() {
                            let chunk = data.chunk();
                            unsafe { buf.set(&Uint8Array::view(chunk), offset) }
                            let chunk_len: u32 = chunk
                                .len()
                                .try_into()
                                .expect_throw("can't send data bigger than a u32");
                            offset += chunk_len;
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
                        return Err(ClientError::EndpointError {
                            hrpc_error,
                            endpoint,
                        });
                    }

                    let content_type = response
                        .headers()
                        .get(header::CONTENT_TYPE.as_str())
                        .expect_throw("header name is valid");
                    if !content_type
                        .as_ref()
                        .and_then(|v| v.split(';').next())
                        .map_or(false, |v| v == HRPC_CONTENT_MIMETYPE)
                    {
                        return Err(ClientError::ContentNotSupported);
                    }

                    if check_spec_version {
                        let hrpc_version = response
                            .headers()
                            .get(HRPC_VERSION_HEADER)
                            .expect_throw("header name is valid");
                        if hrpc_version
                            .as_ref()
                            .map_or(false, |v| v == HRPC_SPEC_VERSION)
                        {
                            return Err(ClientError::IncompatibleSpecVersion);
                        }
                    }

                    let data = response.binary().await.map_err(WasmError::HttpError)?;

                    let mut resp = Response::new_with_body(Body::full(data));

                    resp.extensions_mut().insert(status);

                    Ok(TransportResponse::new_unary(resp))
                })
            }
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

impl From<WasmError> for ClientError<WasmError> {
    fn from(err: WasmError) -> Self {
        ClientError::Transport(err)
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
