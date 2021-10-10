use std::{
    fmt::{self, Debug, Formatter},
    ops::Not,
    str::FromStr,
    sync::Arc,
};

use crate::{body::box_body, decode_body, DecodeBodyError};

use super::Request;

use bytes::BytesMut;
use error::*;
use http::{
    uri::{PathAndQuery, Scheme},
    HeaderMap, Method, Uri,
};
use socket::*;
use tokio_rustls::webpki::DNSNameRef;
use tokio_tungstenite::tungstenite;
use tower::Service;
use tower_http::{
    classify::{SharedClassifier, StatusInRangeAsFailures},
    decompression::{Decompression, DecompressionLayer},
    trace::{Trace, TraceLayer},
};

pub mod error;
pub mod socket;

/// A `hyper` HTTP client that supports HTTPS.
pub type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

/// Creates a new HttpClient that you can use.
pub fn http_client() -> HttpClient {
    let connector = hyper_rustls::HttpsConnector::with_native_roots();
    hyper::Client::builder().build(connector)
}

type SocketRequest = tungstenite::handshake::client::Request;
type ClientService = Trace<Decompression<HttpClient>, SharedClassifier<StatusInRangeAsFailures>>;

/// Generic client implementation with common methods.
#[derive(Clone)]
pub struct Client {
    inner: ClientService,
    server: Uri,
    buf: BytesMut,
    modify_request_headers: Arc<dyn Fn(&mut HeaderMap) + Send + Sync>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.inner)
            .field("server", &self.server)
            .field("buf", &self.buf)
            .finish()
    }
}

impl Client {
    fn make_endpoint(&self, scheme: Option<Scheme>, path: &str) -> ClientResult<Uri> {
        let path = PathAndQuery::from_str(path)
            .map_err(|err| ClientError::FailedRequestBuilder(err.into()))?;

        let mut parts = self.server.clone().into_parts();
        parts.path_and_query = Some(path);
        if let Some(scheme) = scheme {
            parts.scheme = Some(scheme);
        }

        let endpoint =
            Uri::from_parts(parts).map_err(|err| ClientError::FailedRequestBuilder(err.into()))?;

        Ok(endpoint)
    }

    fn new_service(inner: HttpClient, server: Uri) -> Self {
        let inner = tower::ServiceBuilder::new()
            .layer(TraceLayer::new(
                StatusInRangeAsFailures::new(400..=599).into_make_classifier(),
            ))
            .layer(DecompressionLayer::new())
            .service(inner);

        Self {
            inner,
            server,
            buf: BytesMut::new(),
            modify_request_headers: Arc::new(|_| ()),
        }
    }

    /// Creates a new client.
    pub fn new(server: Uri) -> ClientResult<Self> {
        Self::new_inner(http_client(), server)
    }

    /// Creates a new client using the provided HttpClient.
    pub fn new_inner(inner: HttpClient, server: Uri) -> ClientResult<Self> {
        if let Some("http" | "https") = server.scheme_str() {
            Ok(Self::new_service(inner, server))
        } else {
            Err(ClientError::InvalidUrl(InvalidUrlKind::InvalidScheme))
        }
    }

    /// Set the function to modify request headers with before sending the request.
    pub fn modify_request_headers_with(
        mut self,
        f: Arc<dyn Fn(&mut HeaderMap) + Send + Sync>,
    ) -> Self {
        self.modify_request_headers = f;
        self
    }

    /// Executes an unary request and returns the decoded response.
    pub async fn execute_request<Req: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        path: &str,
        req: Request<Req>,
    ) -> ClientResult<Resp> {
        let endpoint = self.make_endpoint(None, path)?;

        let request = {
            let Request {
                body,
                mut header_map,
                ..
            } = req;

            let mut request = http::Request::builder()
                .uri(endpoint.clone())
                .method(Method::POST);

            (self.modify_request_headers)(&mut header_map);
            *request.headers_mut().unwrap() = header_map;

            request
                .body(body)
                .map_err(ClientError::FailedRequestBuilder)?
        };

        let resp = self.inner.call(request).await?;
        let status = resp.status();

        let is_error = status.is_success().not();
        if is_error {
            let raw_error = hyper::body::to_bytes(resp.into_body()).await?;
            return Err(ClientError::EndpointError {
                raw_error,
                status,
                endpoint,
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
            let data = hyper::body::to_bytes(resp.into_body()).await?;
            return Err(ClientError::ContentNotSupported(data));
        }

        let raw = hyper::body::aggregate(resp.into_body()).await?;
        Resp::decode(raw)
            .map_err(|err| ClientError::MessageDecode(DecodeBodyError::InvalidProtoMessage(err)))
    }

    /// Connect a socket with the server and return it.
    pub async fn connect_socket<Req, Resp>(
        &self,
        path: &str,
        mut req: Request<()>,
    ) -> ClientResult<Socket<Req, Resp>>
    where
        Req: prost::Message + 'static,
        Resp: prost::Message + Default + 'static,
    {
        let ws_scheme = match self.server.scheme_str() {
            Some("http") => "ws",
            Some("https") => "wss",
            _ => unreachable!("scheme cant be anything other than http or https"),
        };
        let endpoint = self.make_endpoint(Some(ws_scheme.parse().unwrap()), path)?;

        let mut request = SocketRequest::get(endpoint).body(()).unwrap();
        (self.modify_request_headers)(&mut req.header_map);
        for (key, value) in req.header_map {
            if let Some(key) = key {
                request.headers_mut().entry(key).or_insert(value);
            }
        }

        use tungstenite::{client::IntoClientRequest, stream::Mode};
        let request = request.into_client_request()?;

        let domain = request
            .uri()
            .host()
            .map(str::to_string)
            .expect("must have host");
        let port = request
            .uri()
            .port_u16()
            .unwrap_or_else(|| match request.uri().scheme_str() {
                Some("wss") => 443,
                Some("ws") => 80,
                _ => unreachable!("scheme cant be anything other than ws or wss"),
            });

        let addr = format!("{}:{}", domain, port);
        let socket = tokio::net::TcpStream::connect(addr).await?;

        let mode = tungstenite::client::uri_mode(request.uri())?;
        let stream = match mode {
            Mode::Plain => tokio_tungstenite::MaybeTlsStream::Plain(socket),
            Mode::Tls => {
                let mut config = tokio_rustls::rustls::ClientConfig::new();
                config.root_store =
                    rustls_native_certs::load_native_certs().map_err(|(_, err)| err)?;
                let domain = DNSNameRef::try_from_ascii_str(&domain).map_err(|err| {
                    tungstenite::Error::Tls(tungstenite::error::TlsError::Dns(err))
                })?;
                let stream = tokio_rustls::TlsConnector::from(Arc::new(config));
                let connected = stream.connect(domain, socket).await?;
                tokio_tungstenite::MaybeTlsStream::Rustls(connected)
            }
        };

        let inner = tokio_tungstenite::client_async(request, stream).await?.0;

        Ok(Socket::new(inner))
    }

    /// Connect a socket with the server, send a message and return it.
    ///
    /// Used by the server streaming methods.
    pub async fn connect_socket_req<Req, Resp>(
        &self,
        path: &str,
        request: Request<Req>,
    ) -> ClientResult<Socket<Req, Resp>>
    where
        Req: prost::Message + Default + 'static,
        Resp: prost::Message + Default + 'static,
    {
        let Request {
            body, header_map, ..
        } = request;
        let socket = self
            .connect_socket(
                path,
                Request {
                    header_map,
                    ..Request::empty()
                },
            )
            .await?;
        let message = decode_body(box_body(body)).await?;
        socket.send_message(message).await?;

        Ok(socket)
    }
}
