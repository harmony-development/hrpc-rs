//! A HTTP client transport implementation using [`hyper`].

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    ops::Not,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;
use futures_util::{future::BoxFuture, ready, Future, FutureExt, StreamExt, TryFutureExt};
use http::{
    header,
    uri::{PathAndQuery, Scheme},
    HeaderMap, Method, Uri,
};
use prost::Message;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
use tower::Service;

use crate::{
    client::{
        error::{ClientError, HrpcError},
        transport::{is_socket_request, SocketChannels, TransportError},
    },
    common::transport::{
        http::{
            content_header_value, version_header_name, version_header_value, ws_version,
            ws_version_header_value,
        },
        tokio_tungstenite::WebSocket,
    },
    request::{self, BoxRequest},
    response::BoxResponse,
    Response, HRPC_SPEC_VERSION,
};

use super::{check_uri, map_scheme_to_ws, InvalidServerUrl};

type SocketRequest = tungstenite::handshake::client::Request;
/// A `hyper` HTTP client that supports HTTPS.
pub type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

/// Creates a new [`HttpClient`] that you can use.
pub fn http_client(builder: &mut hyper::client::Builder) -> HttpClient {
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    builder.build(connector)
}

/// HTTP transport implemented using [`hyper`].
///
/// This client will:
/// - Adds [`http::HeaderMap`], [`http::StatusCode`], [`http::Version`],
/// [`http::Extensions`] to the [`Response`] returned.
/// - Looks for a [`http::HeaderMap`] in a [`Request`]s extensions, if it
/// exists the headers in there will be added to the HTTP request that
/// will be used. Headers added by the client by default will be overwrited,
/// so take care while inserting headers that are mentioned in [the spec].
///
/// [the spec]: https://github.com/harmony-development/hrpc/blob/main/protocol/SPEC.md
#[derive(Debug, Clone)]
pub struct Hyper {
    client: HttpClient,
    server: Uri,
}

impl Hyper {
    /// Create a new HTTP transport using the provided URI as server URI.
    pub fn new(server: Uri) -> Result<Self, HyperError> {
        Self::new_with_hyper(
            server,
            http_client(
                hyper::Client::builder().http2_keep_alive_interval(Some(Duration::from_secs(10))),
            ),
        )
    }

    /// Create a new HTTP transport using the provided URI as server URI, and
    /// the provided [`HttpClient`] as the underlying client.
    ///
    /// You can create a [`HttpClient`] using [`http_client`].
    pub fn new_with_hyper(server: Uri, hyper_client: HttpClient) -> Result<Self, HyperError> {
        Ok(Self {
            client: hyper_client,
            server: check_uri(server).map_err(HyperError::InvalidUrl)?,
        })
    }

    fn make_endpoint(&self, scheme: Option<Scheme>, path: &str) -> Result<Uri, HyperError> {
        let path = PathAndQuery::from_str(path)
            .map_err(http::Error::from)
            .map_err(HyperError::FailedRequestBuilder)?;

        let mut parts = self.server.clone().into_parts();
        parts.path_and_query = Some(path);
        if let Some(scheme) = scheme {
            parts.scheme = Some(scheme);
        }

        let endpoint = Uri::from_parts(parts)
            .map_err(http::Error::from)
            .map_err(HyperError::FailedRequestBuilder)?;

        Ok(endpoint)
    }
}

impl Service<BoxRequest> for Hyper {
    type Response = BoxResponse;

    type Error = TransportError<HyperError>;

    type Future = HyperCallFuture;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, mut req: BoxRequest) -> Self::Future {
        if is_socket_request(&req) {
            let ws_scheme = map_scheme_to_ws(self.server.scheme_str().expect("must have scheme"))
                .expect("scheme can't be anything other than https or http");
            let maybe_endpoint =
                self.make_endpoint(Some(ws_scheme.parse().unwrap()), req.endpoint());

            let endpoint = match maybe_endpoint {
                Ok(uri) => uri,
                Err(err) => return HyperCallFuture(HyperCallFutureInner::Err(Some(err.into()))),
            };

            let mut request = SocketRequest::get(endpoint).body(()).unwrap();

            // Insert default protocol (can be overwritten by users)
            request
                .headers_mut()
                .insert(header::SEC_WEBSOCKET_PROTOCOL, ws_version_header_value());

            if let Some(header_map) = req.extensions_mut().remove::<HeaderMap>() {
                for (key, value) in header_map {
                    if let Some(key) = key {
                        request.headers_mut().insert(key, value);
                    }
                }
            }

            let connect_fut = tokio_tungstenite::connect_async(request);

            HyperCallFuture(HyperCallFutureInner::Socket {
                connect_fut: Box::pin(connect_fut),
            })
        } else {
            let maybe_req_url = self.make_endpoint(None, req.endpoint());

            let req_url = match maybe_req_url {
                Ok(uri) => uri,
                Err(err) => return HyperCallFuture(HyperCallFutureInner::Err(Some(err.into()))),
            };

            let request::Parts {
                body,
                mut extensions,
                endpoint,
            } = req.into();

            let request = {
                let mut request = http::Request::builder().uri(req_url).method(Method::POST);

                let mut header_map = extensions.remove::<HeaderMap>().unwrap_or_default();
                // insert content type
                header_map.insert(header::CONTENT_TYPE, content_header_value());
                // insert spec version
                header_map.insert(version_header_name(), version_header_value());

                *request.headers_mut().unwrap() = header_map;

                let maybe_resp = request
                    .body(body.into())
                    .map_err(HyperError::FailedRequestBuilder);

                match maybe_resp {
                    Ok(resp) => resp,
                    Err(err) => {
                        return HyperCallFuture(HyperCallFutureInner::Err(Some(err.into())))
                    }
                }
            };

            let resp_fut = self.client.request(request);

            HyperCallFuture(HyperCallFutureInner::Unary {
                request_fut: resp_fut,
                error_fut: None,
                endpoint: Some(endpoint),
            })
        }
    }
}

enum HyperCallFutureInner {
    Err(Option<TransportError<HyperError>>),
    #[allow(clippy::type_complexity)]
    Socket {
        connect_fut: BoxFuture<
            'static,
            Result<
                (
                    WebSocketStream<MaybeTlsStream<TcpStream>>,
                    tungstenite::handshake::client::Response,
                ),
                tungstenite::Error,
            >,
        >,
    },
    Unary {
        request_fut: hyper::client::ResponseFuture,
        error_fut: Option<BoxFuture<'static, Result<Bytes, HyperError>>>,
        endpoint: Option<Cow<'static, str>>,
    },
}

/// Call future used by this transport.
pub struct HyperCallFuture(HyperCallFutureInner);

impl Future for HyperCallFuture {
    type Output = Result<BoxResponse, TransportError<HyperError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().0 {
            HyperCallFutureInner::Err(err) => Poll::Ready(Err(err
                .take()
                .expect("called call future again - this is a bug"))),
            HyperCallFutureInner::Socket { connect_fut } => {
                match ready!(connect_fut.poll_unpin(cx)) {
                    Ok((ws_stream, response)) => {
                        if !response
                            .headers()
                            .get(header::SEC_WEBSOCKET_PROTOCOL)
                            .and_then(|h| h.to_str().ok())
                            .map_or(false, |v| v.contains(&ws_version()))
                        {
                            return Poll::Ready(Err(HyperError::SocketInitError(
                                SocketInitError::InvalidProtocol,
                            )
                            .into()));
                        }

                        let (ws_tx, ws_rx) = WebSocket::new(ws_stream).split();
                        let chans = SocketChannels::new(ws_tx, ws_rx);

                        let mut resp = BoxResponse::empty();

                        let exts_mut = resp.extensions_mut();

                        let (parts, _) = response.into_parts();
                        exts_mut.insert(parts.status);
                        exts_mut.insert(parts.extensions);
                        exts_mut.insert(parts.headers);
                        exts_mut.insert(parts.version);

                        exts_mut.insert(chans);

                        Poll::Ready(Ok(resp))
                    }
                    Err(err) => Poll::Ready(Err(HyperError::SocketInitError(
                        SocketInitError::Tungstenite(err),
                    )
                    .into())),
                }
            }
            HyperCallFutureInner::Unary {
                request_fut,
                error_fut,
                endpoint,
            } => {
                if let Some(fut) = error_fut.as_mut() {
                    match ready!(fut.poll_unpin(cx)) {
                        Ok(raw_error) => {
                            let hrpc_error = HrpcError::decode(raw_error.as_ref())
                                .unwrap_or_else(|_| HrpcError::invalid_hrpc_error(raw_error));
                            return Poll::Ready(Err((ClientError::EndpointError {
                                hrpc_error,
                                endpoint: endpoint.take().expect(
                                    "hyper call future polled after copmletion - this is a bug",
                                ),
                            })
                            .into()));
                        }
                        Err(err) => {
                            return Poll::Ready(Err(err.into()));
                        }
                    }
                }
                match ready!(request_fut.poll_unpin(cx)) {
                    Ok(resp) => {
                        let status = resp.status();

                        if status.is_success().not() {
                            let fut = Box::pin(
                                hyper::body::to_bytes(resp.into_body()).map_err(HyperError::Http),
                            );
                            error_fut.replace(fut);
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }

                        // Handle non-protobuf successful responses
                        let is_hrpc = |t: &[u8]| {
                            t.eq_ignore_ascii_case(crate::HRPC_CONTENT_MIMETYPE.as_bytes())
                        };
                        if !resp
                            .headers()
                            .get(&http::header::CONTENT_TYPE)
                            .and_then(|t| t.as_bytes().split(|c| b';'.eq(c)).next())
                            .map_or(false, is_hrpc)
                        {
                            return Poll::Ready(Err(ClientError::ContentNotSupported.into()));
                        }

                        // check if the spec version matches
                        if !resp
                            .headers()
                            .get(version_header_name())
                            .map(|h| h.as_bytes())
                            .map_or(false, |v| {
                                v.eq_ignore_ascii_case(HRPC_SPEC_VERSION.as_bytes())
                            })
                        {
                            // TODO: parse the header properly and extract the version instead of just doing a contains
                            return Poll::Ready(Err(ClientError::IncompatibleSpecVersion.into()));
                        }

                        let (parts, body) = resp.into_parts();

                        let mut response = Response::new_with_body(body.into());

                        let exts_mut = response.extensions_mut();
                        exts_mut.insert(parts.status);
                        exts_mut.insert(parts.extensions);
                        exts_mut.insert(parts.headers);
                        exts_mut.insert(parts.version);

                        Poll::Ready(Ok(response))
                    }
                    Err(err) => Poll::Ready(Err(HyperError::Http(err).into())),
                }
            }
        }
    }
}

/// Errors that [`Hyper`] transport might produce.
#[derive(Debug)]
pub enum HyperError {
    /// Occurs if request creation fails.
    FailedRequestBuilder(http::Error),
    /// Occurs if hyper, the HTTP client, returns an error.
    Http(hyper::Error),
    /// Occurs if the given URL is invalid.
    InvalidUrl(InvalidServerUrl),
    /// Occurs if socket creation.
    SocketInitError(SocketInitError),
}

impl Display for HyperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedRequestBuilder(err) => write!(f, "failed to build request: {}", err),
            Self::Http(err) => write!(f, "HTTP error: {}", err),
            Self::InvalidUrl(err) => write!(f, "invalid URL: {}", err),
            Self::SocketInitError(err) => write!(f, "failed to create socket: {}", err),
        }
    }
}

impl StdError for HyperError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::FailedRequestBuilder(err) => Some(err),
            Self::Http(err) => Some(err),
            Self::SocketInitError(err) => Some(err),
            Self::InvalidUrl(err) => Some(err),
        }
    }
}

impl From<hyper::Error> for HyperError {
    fn from(err: hyper::Error) -> Self {
        HyperError::Http(err)
    }
}

impl From<HyperError> for TransportError<HyperError> {
    fn from(err: HyperError) -> Self {
        TransportError::Transport(err)
    }
}

/// Errors that can occur on socket creation.
#[derive(Debug)]
pub enum SocketInitError {
    /// Occurs if socket creation fails with a tungstenite error.
    Tungstenite(tungstenite::Error),
    /// Occurs if the server sent an invalid protocol (not `hrpc`).
    InvalidProtocol,
}

impl Display for SocketInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tungstenite(err) => write!(f, "tungstenite error: {}", err),
            Self::InvalidProtocol => {
                write!(f, "server sent incompatible protocol, expected 'hrpc'")
            }
        }
    }
}

impl StdError for SocketInitError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Tungstenite(err) => Some(err),
            _ => None,
        }
    }
}
