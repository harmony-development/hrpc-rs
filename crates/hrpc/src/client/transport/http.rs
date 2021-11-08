use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    ops::Not,
    str::FromStr,
    sync::Arc,
};

use bytes::BytesMut;
use futures_util::{future::BoxFuture, StreamExt};
use http::{
    header,
    uri::{PathAndQuery, Scheme},
    HeaderMap, Method, Uri,
};
use prost::Message;
use tokio_rustls::webpki::DNSNameRef;
use tokio_tungstenite::tungstenite;

use super::{
    super::error::{ClientError, ClientResult, HrpcError},
    Transport,
};
use crate::{
    client::socket::Socket,
    common::transport::http::{hrpc_header_value, WebSocket},
    request, Request, Response,
};

type SocketRequest = tungstenite::handshake::client::Request;
/// A `hyper` HTTP client that supports HTTPS.
pub type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

/// Creates a new [`HttpClient`] that you can use.
pub fn http_client() -> HttpClient {
    let connector = hyper_rustls::HttpsConnector::with_native_roots();
    hyper::Client::builder().build(connector)
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
        if let Some("https" | "http") = server.scheme_str() {
            Ok(Self {
                client: http_client(),
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
                header_map.insert(header::CONTENT_TYPE, hrpc_header_value());

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
                    || t.eq_ignore_ascii_case(crate::HRPC_HEADER))
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

            Ok(Response::new_with_body(resp.into_body().into()))
        })
    }

    fn call_socket<Req, Resp>(
        &mut self,
        mut req: Request<()>,
    ) -> BoxFuture<'_, ClientResult<Socket<Req, Resp>, Self::Error>>
    where
        Req: Message,
        Resp: Message + Default,
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

            if let Some(header_map) = req.extensions_mut().remove::<HeaderMap>() {
                for (key, value) in header_map {
                    if let Some(key) = key {
                        request.headers_mut().entry(key).or_insert(value);
                    }
                }
            }

            use tungstenite::{client::IntoClientRequest, stream::Mode};
            let request = request
                .into_client_request()
                .map_err(HyperError::SocketInitError)
                .map_err(ClientError::Transport)?;

            let domain = request
                .uri()
                .host()
                .map(str::to_string)
                .expect("must have host");
            let port =
                request
                    .uri()
                    .port_u16()
                    .unwrap_or_else(|| match request.uri().scheme_str() {
                        Some("wss") => 443,
                        Some("ws") => 80,
                        _ => unreachable!("scheme cant be anything other than ws or wss"),
                    });

            let addr = format!("{}:{}", domain, port);
            let socket = tokio::net::TcpStream::connect(addr).await?;

            let mode = tungstenite::client::uri_mode(request.uri())
                .map_err(HyperError::SocketInitError)
                .map_err(ClientError::Transport)?;
            let stream = match mode {
                Mode::Plain => tokio_tungstenite::MaybeTlsStream::Plain(socket),
                Mode::Tls => {
                    let mut config = tokio_rustls::rustls::ClientConfig::new();
                    config.root_store =
                        rustls_native_certs::load_native_certs().map_err(|(_, err)| err)?;
                    let domain = DNSNameRef::try_from_ascii_str(&domain)
                        .map_err(|err| {
                            tungstenite::Error::Tls(tungstenite::error::TlsError::Dns(err))
                        })
                        .map_err(HyperError::SocketInitError)
                        .map_err(ClientError::Transport)?;
                    let stream = tokio_rustls::TlsConnector::from(Arc::new(config));
                    let connected = stream.connect(domain, socket).await?;
                    tokio_tungstenite::MaybeTlsStream::Rustls(connected)
                }
            };

            let inner = tokio_tungstenite::client_async(request, stream)
                .await
                .map_err(HyperError::SocketInitError)
                .map_err(ClientError::Transport)?
                .0;

            let (ws_tx, ws_rx) = WebSocket::new(inner).split();

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
    /// Occurs if socket creation fails.
    SocketInitError(tungstenite::Error),
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

impl StdError for HyperError {}

impl From<hyper::Error> for HyperError {
    fn from(err: hyper::Error) -> Self {
        HyperError::Http(err)
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
