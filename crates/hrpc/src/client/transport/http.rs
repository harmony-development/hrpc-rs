use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    ops::Not,
    str::FromStr,
    time::Duration,
};

use bytes::BytesMut;
use futures_util::{future::BoxFuture, StreamExt};
use http::{
    header,
    uri::{PathAndQuery, Scheme},
    HeaderMap, Method, Uri,
};
use prost::Message;
use tokio_tungstenite::tungstenite;

use super::{
    super::error::{ClientError, ClientResult, HrpcError},
    Transport,
};
use crate::{
    client::socket::Socket,
    common::transport::http::{
        content_header_value, version_header_name, version_header_value, ws_version,
        ws_version_header_value, WebSocket,
    },
    request, Request, Response, HRPC_SPEC_VERSION,
};

type SocketRequest = tungstenite::handshake::client::Request;
/// A `hyper` HTTP client that supports HTTPS.
pub type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

/// Creates a new [`HttpClient`] that you can use.
pub fn http_client(builder: &mut hyper::client::Builder) -> HttpClient {
    let connector = hyper_rustls::HttpsConnector::with_native_roots();
    builder.build(connector)
}

/// HTTP transport implemented using [`hyper`].
#[derive(Debug, Clone)]
pub struct Hyper {
    client: HttpClient,
    server: Uri,
    buf: BytesMut,
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
        if let Some("https" | "http") = server.scheme_str() {
            Ok(Self {
                client: hyper_client,
                server,
                buf: BytesMut::new(),
            })
        } else {
            Err(HyperError::InvalidUrl(InvalidUrlKind::InvalidScheme))
        }
    }

    fn make_endpoint(&self, scheme: Option<Scheme>, path: &str) -> Result<Uri, HyperError> {
        let path = PathAndQuery::from_str(path)
            .map_err(|err| HyperError::FailedRequestBuilder(err.into()))?;

        let mut parts = self.server.clone().into_parts();
        parts.path_and_query = Some(path);
        if let Some(scheme) = scheme {
            parts.scheme = Some(scheme);
        }

        let endpoint =
            Uri::from_parts(parts).map_err(|err| HyperError::FailedRequestBuilder(err.into()))?;

        Ok(endpoint)
    }
}

impl Transport for Hyper {
    type Error = HyperError;

    fn call_unary<'a, Req, Resp>(
        &'a mut self,
        req: Request<Req>,
    ) -> BoxFuture<'a, ClientResult<Response<Resp>, Self::Error>>
    where
        Req: prost::Message + 'a,
        Resp: prost::Message + Default,
    {
        Box::pin(async move {
            let endpoint = self
                .make_endpoint(None, req.endpoint())
                .map_err(ClientError::Transport)?;

            let request = {
                let request::Parts {
                    body,
                    mut extensions,
                    ..
                } = req.into();

                let mut request = http::Request::builder()
                    .uri(endpoint.clone())
                    .method(Method::POST);

                let mut header_map = extensions.remove::<HeaderMap>().unwrap_or_default();
                // insert content type
                header_map.insert(header::CONTENT_TYPE, content_header_value());
                // insert spec version
                header_map.insert(version_header_name(), version_header_value());

                *request.headers_mut().unwrap() = header_map;

                request
                    .body(body.into())
                    .map_err(HyperError::FailedRequestBuilder)
                    .map_err(ClientError::Transport)?
            };

            let resp = self
                .client
                .request(request)
                .await
                .map_err(HyperError::Http)
                .map_err(ClientError::Transport)?;
            let status = resp.status();

            let is_error = status.is_success().not();
            if is_error {
                let raw_error = hyper::body::to_bytes(resp.into_body())
                    .await
                    .map_err(HyperError::Http)
                    .map_err(ClientError::Transport)?;
                let hrpc_error = HrpcError::decode(raw_error.as_ref()).unwrap_or_else(|_| HrpcError {
                    human_message: "the server error was an invalid hRPC error, check more_details field for the error".to_string(),
                    identifier: "hrpcrs.invalid-hrpc-error".to_string(),
                    more_details: raw_error,
                });
                return Err(ClientError::EndpointError {
                    hrpc_error,
                    endpoint: Cow::Owned(endpoint.path().to_string()),
                });
            }

            // Handle non-protobuf successful responses
            let is_hrpc = |t: &[u8]| {
                !(t.eq_ignore_ascii_case(b"application/octet-stream")
                    || t.eq_ignore_ascii_case(crate::HRPC_CONTENT_MIMETYPE))
            };
            if resp
                .headers()
                .get(&http::header::CONTENT_TYPE)
                .map(|t| t.as_bytes().split(|c| b';'.eq(c)).next())
                .flatten()
                .map_or(true, is_hrpc)
            {
                let data = hyper::body::to_bytes(resp.into_body())
                    .await
                    .map_err(HyperError::Http)
                    .map_err(ClientError::Transport)?;
                return Err(ClientError::ContentNotSupported(data));
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
                return Err(ClientError::Transport(HyperError::IncompatibleSpecVersion));
            }

            Ok(Response::new_with_body(resp.into_body().into()))
        })
    }

    fn call_socket<Req, Resp>(
        &mut self,
        mut req: Request<()>,
    ) -> BoxFuture<'_, ClientResult<Socket<Req, Resp>, Self::Error>>
    where
        Req: Message + 'static,
        Resp: Message + Default + 'static,
    {
        Box::pin(async move {
            let ws_scheme = match self.server.scheme_str() {
                Some("http") => "ws",
                Some("https") => "wss",
                _ => unreachable!("scheme cant be anything other than http or https"),
            };
            let endpoint = self
                .make_endpoint(Some(ws_scheme.parse().unwrap()), req.endpoint())
                .map_err(ClientError::Transport)?;

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

            let (ws_stream, response) = tokio_tungstenite::connect_async(request)
                .await
                .map_err(SocketInitError::Tungstenite)
                .map_err(HyperError::SocketInitError)
                .map_err(ClientError::Transport)?;

            if !response
                .headers()
                .get(header::SEC_WEBSOCKET_PROTOCOL)
                .and_then(|h| h.to_str().ok())
                .map_or(false, |v| v.contains(&ws_version()))
            {
                return Err(ClientError::Transport(HyperError::SocketInitError(
                    SocketInitError::InvalidProtocol,
                )));
            }

            let (ws_tx, ws_rx) = WebSocket::new(ws_stream).split();

            Ok(Socket::new(Box::pin(ws_rx), Box::pin(ws_tx)))
        })
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
    InvalidUrl(InvalidUrlKind),
    /// Occurs if socket creation.
    SocketInitError(SocketInitError),
    /// Occurs if the spec implemented on server doesn't match ours.
    IncompatibleSpecVersion,
}

impl Display for HyperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedRequestBuilder(err) => write!(f, "failed to build request: {}", err),
            Self::Http(err) => write!(f, "HTTP error: {}", err),
            Self::InvalidUrl(err) => write!(f, "invalid URL: {}", err),
            Self::SocketInitError(err) => write!(f, "failed to create socket: {}", err),
            Self::IncompatibleSpecVersion => write!(f, "server has incompatible spec version",),
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
            _ => None,
        }
    }
}

impl From<hyper::Error> for HyperError {
    fn from(err: hyper::Error) -> Self {
        HyperError::Http(err)
    }
}

/// Errors that can occur on socket creation.
#[derive(Debug)]
pub enum SocketInitError {
    /// Occurs if socket creation fails with a tungstenite error.
    Tungstenite(tungstenite::Error),
    /// Occurs if the server has an incompatible spec version.
    IncompatibleSpecVersion,
    /// Occurs if the server sent an invalid protocol (not `hrpc`).
    InvalidProtocol,
}

impl Display for SocketInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tungstenite(err) => write!(f, "tungstenite error: {}", err),
            Self::IncompatibleSpecVersion => write!(f, "server has incompatible spec version",),
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

#[derive(Debug)]
/// Errors that can occur while parsing the URL given to `Client::new()`.
pub enum InvalidUrlKind {
    /// Occurs if URL scheme isn't `http` or `https`.
    InvalidScheme,
}

impl Display for InvalidUrlKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InvalidUrlKind::InvalidScheme => {
                write!(f, "invalid scheme, expected `http` or `https`")
            }
        }
    }
}

impl StdError for InvalidUrlKind {}
