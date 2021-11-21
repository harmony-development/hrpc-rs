//! A HTTP client transport implementation using [`hyper`].

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    ops::Not,
    str::FromStr,
    task::{Context, Poll},
    time::Duration,
};

use futures_util::StreamExt;
use http::{
    header,
    uri::{PathAndQuery, Scheme},
    HeaderMap, Method, Uri,
};
use prost::Message;
use tokio_tungstenite::tungstenite;
use tower::Service;

use crate::{
    client::{
        error::{ClientError, HrpcError},
        transport::{box_socket_stream_sink, CallResult, TransportRequest, TransportResponse},
    },
    common::transport::{
        http::{
            content_header_value, version_header_name, version_header_value, ws_version,
            ws_version_header_value,
        },
        tokio_tungstenite::WebSocket,
    },
    request, Response, HRPC_SPEC_VERSION,
};

use super::{check_uri, map_scheme_to_ws, InvalidServerUrl};

type SocketRequest = tungstenite::handshake::client::Request;
/// A `hyper` HTTP client that supports HTTPS.
pub type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

/// Creates a new [`HttpClient`] that you can use.
pub fn http_client(builder: &mut hyper::client::Builder) -> HttpClient {
    let connector = hyper_rustls::HttpsConnector::with_native_roots();
    builder.build(connector)
}

/// HTTP transport implemented using [`hyper`].
///
/// This client will:
/// - Adds [`http::HeaderMap`], [`http::StatusCode`], [`http::Version`],
/// [`http::Extensions`] to the [`Response`] returned in unary requests.
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

impl Service<TransportRequest> for Hyper {
    type Response = TransportResponse;

    type Error = ClientError<HyperError>;

    type Future = CallResult<'static, TransportResponse, HyperError>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: TransportRequest) -> Self::Future {
        match req {
            TransportRequest::Unary(req) => {
                let maybe_req_url = self.make_endpoint(None, req.endpoint());
                let client = self.client.clone();
                Box::pin(async move {
                    let req_url = maybe_req_url?;

                    let request::Parts {
                        body,
                        mut extensions,
                        endpoint,
                    } = req.into();

                    let request = {
                        let mut request =
                            http::Request::builder().uri(req_url).method(Method::POST);

                        let mut header_map = extensions.remove::<HeaderMap>().unwrap_or_default();
                        // insert content type
                        header_map.insert(header::CONTENT_TYPE, content_header_value());
                        // insert spec version
                        header_map.insert(version_header_name(), version_header_value());

                        *request.headers_mut().unwrap() = header_map;

                        request
                            .body(body.into())
                            .map_err(HyperError::FailedRequestBuilder)?
                    };

                    let resp = client.request(request).await.map_err(HyperError::Http)?;

                    let status = resp.status();

                    if status.is_success().not() {
                        let raw_error = hyper::body::to_bytes(resp.into_body())
                            .await
                            .map_err(HyperError::Http)?;
                        let hrpc_error = HrpcError::decode(raw_error.as_ref())
                            .unwrap_or_else(|_| HrpcError::invalid_hrpc_error(raw_error));
                        return Err(ClientError::EndpointError {
                            hrpc_error,
                            endpoint,
                        });
                    }

                    // Handle non-protobuf successful responses
                    let is_hrpc =
                        |t: &[u8]| t.eq_ignore_ascii_case(crate::HRPC_CONTENT_MIMETYPE.as_bytes());
                    if !resp
                        .headers()
                        .get(&http::header::CONTENT_TYPE)
                        .and_then(|t| t.as_bytes().split(|c| b';'.eq(c)).next())
                        .map_or(false, is_hrpc)
                    {
                        return Err(ClientError::ContentNotSupported);
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
                        return Err(ClientError::IncompatibleSpecVersion);
                    }

                    let (parts, body) = resp.into_parts();

                    let mut response = Response::new_with_body(body.into());

                    let exts_mut = response.extensions_mut();
                    exts_mut.insert(parts.status);
                    exts_mut.insert(parts.extensions);
                    exts_mut.insert(parts.headers);
                    exts_mut.insert(parts.version);

                    Ok(TransportResponse::new_unary(response))
                })
            }
            TransportRequest::Socket(mut req) => {
                let ws_scheme =
                    map_scheme_to_ws(self.server.scheme_str().expect("must have scheme"))
                        .expect("scheme can't be anything other than https or http");
                let maybe_endpont =
                    self.make_endpoint(Some(ws_scheme.parse().unwrap()), req.endpoint());

                Box::pin(async move {
                    let endpoint = maybe_endpont?;

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
                        .map_err(HyperError::SocketInitError)?;

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
                    let (tx, rx) = box_socket_stream_sink(ws_tx, ws_rx);

                    Ok(TransportResponse::new_socket(tx, rx))
                })
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

impl From<HyperError> for ClientError<HyperError> {
    fn from(err: HyperError) -> Self {
        ClientError::Transport(err)
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
