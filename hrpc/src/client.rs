use std::sync::Arc;

use super::*;

use bytes::BytesMut;
use tokio_rustls::webpki::DNSNameRef;
use url::Url;

#[doc(inline)]
pub use error::*;

type WebSocketStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

type SocketRequest = tungstenite::handshake::client::Request;
type UnaryRequest = reqwest::Request;

/// Generic client implementation with common methods.
#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    server: Url,
    buf: BytesMut,
}

impl Client {
    /// Creates a new client.
    pub fn new(inner: reqwest::Client, server: Url) -> ClientResult<Self> {
        if let "http" | "https" = server.scheme() {
            Ok(Self {
                inner,
                server,
                buf: BytesMut::new(),
            })
        } else {
            Err(ClientError::InvalidUrl(InvalidUrlKind::InvalidScheme))
        }
    }

    /// Executes an unary request returns the decoded response.
    pub async fn execute_request<Req: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        path: &str,
        req: Request<Req>,
    ) -> ClientResult<Resp> {
        let request = {
            encode_protobuf_message(&mut self.buf, req.message);
            let mut request = UnaryRequest::new(
                reqwest::Method::POST,
                self.server
                    .join(path)
                    .expect("failed to form request URL, something must be terribly wrong"),
            );
            *request.body_mut() = Some(self.buf.clone().freeze().into());
            for (key, value) in req.header_map {
                if let Some(key) = key {
                    request.headers_mut().entry(key).or_insert(value);
                }
            }
            request
        };

        let resp = self.inner.execute(request).await?;

        // Handle errors
        if let Err(error) = resp.error_for_status_ref() {
            let raw_error = resp.bytes().await?;
            return Err(ClientError::EndpointError {
                raw_error,
                status: error.status().unwrap(), // We always generate the error from a response, so this is always safe
                endpoint: error
                    .url()
                    .map_or_else(|| "unknown".to_string(), ToString::to_string),
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
            return Err(ClientError::NonProtobuf(resp.bytes().await?));
        }

        let raw = resp.bytes().await?;
        Ok(Resp::decode(raw)?)
    }

    /// Connect a socket with the server and return it.
    pub async fn connect_socket<Req, Resp>(
        &self,
        path: &str,
        req: Request<()>,
    ) -> ClientResult<socket::Socket<Req, Resp>>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        let mut url = self.server.join(path).unwrap();
        url.set_scheme(match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            _ => unreachable!("scheme cant be anything other than http or https"),
        })
        .expect("failed to form websocket URL, something must be terribly wrong");

        let mut request = SocketRequest::get(Into::<String>::into(url))
            .body(())
            .unwrap();
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

        Ok(socket::Socket::new(inner))
    }

    /// Connect a socket with the server, send a message and return it.
    ///
    /// Used by the server streaming methods.
    pub async fn connect_socket_req<Req, Resp>(
        &self,
        path: &str,
        request: Request<Req>,
    ) -> ClientResult<socket::ReadSocket<Req, Resp>>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        let (message, headers) = request.into_parts();
        let mut socket = self
            .connect_socket(path, Request::from_parts(((), headers)))
            .await?;
        socket.send_message(message).await?;

        Ok(socket.read_only())
    }
}

/// Socket implementations.
pub mod socket {
    use super::{encode_protobuf_message, tungstenite, ClientResult, WebSocketStream};
    use bytes::{Bytes, BytesMut};
    use futures_util::{
        stream::{SplitSink, SplitStream},
        StreamExt,
    };
    use std::{
        fmt::{self, Debug, Formatter},
        marker::PhantomData,
        sync::Arc,
    };
    use tokio::sync::Mutex;

    /// A websocket, wrapped for ease of use with protobuf messages.
    pub struct Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        data: Arc<SocketData>,
        buf: BytesMut,
        _msg: PhantomData<Req>,
        _resp: PhantomData<Resp>,
    }

    struct SocketData {
        tx: Mutex<SplitSink<WebSocketStream, tungstenite::Message>>,
        rx: Mutex<SplitStream<WebSocketStream>>,
    }

    impl<Req, Resp> Debug for Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            f.debug_struct("Socket").field("buf", &self.buf).finish()
        }
    }

    impl<Req, Resp> Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        pub(super) fn new(inner: WebSocketStream) -> Self {
            let (tx, rx) = inner.split();
            Self {
                data: Arc::new(SocketData {
                    tx: Mutex::new(tx),
                    rx: Mutex::new(rx),
                }),
                buf: BytesMut::new(),
                _msg: Default::default(),
                _resp: Default::default(),
            }
        }

        /// Send a protobuf message over the websocket.
        pub async fn send_message(&mut self, msg: Req) -> ClientResult<()> {
            use futures_util::SinkExt;

            encode_protobuf_message(&mut self.buf, msg);
            let msg = tungstenite::Message::Binary(self.buf.to_vec());

            Ok(self.data.tx.lock().await.send(msg).await?)
        }

        /// Get a message from the websocket.
        ///
        /// This should be polled always in order to handle incoming ping messages.
        pub async fn get_message(&self) -> Option<ClientResult<Resp>> {
            let raw = self.data.rx.lock().await.next().await?;

            match raw {
                Ok(msg) => {
                    use futures_util::SinkExt;
                    use tungstenite::Message;

                    match msg {
                        Message::Binary(raw) => {
                            Some(Resp::decode(Bytes::from(raw)).map_err(Into::into))
                        }
                        Message::Close(_) => Some(Err(tungstenite::Error::ConnectionClosed.into())),
                        Message::Ping(data) => self
                            .data
                            .tx
                            .lock()
                            .await
                            .send(tungstenite::Message::Pong(data))
                            .await
                            .map_or_else(|err| Some(Err(err.into())), |_| None),
                        Message::Pong(_) | Message::Text(_) => None,
                    }
                }
                Err(err) => Some(Err(err.into())),
            }
        }

        /// Close this websocket.
        /// All subsequent message sending operations will return an "connection closed" error.
        pub async fn close(&self) -> ClientResult<()> {
            use futures_util::SinkExt;

            Ok(self
                .data
                .tx
                .lock()
                .await
                .send(tungstenite::Message::Close(None))
                .await?)
        }

        /// Converts this socket to a read-only socket.
        pub fn read_only(self) -> ReadSocket<Req, Resp> {
            ReadSocket { inner: self }
        }
    }

    impl<Req, Resp> Clone for Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        fn clone(&self) -> Self {
            Self {
                data: self.data.clone(),
                buf: BytesMut::with_capacity(self.buf.capacity()),
                _msg: PhantomData::default(),
                _resp: PhantomData::default(),
            }
        }
    }

    /// A read-only version of [`Socket`].
    #[derive(Debug, Clone)]
    pub struct ReadSocket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        inner: Socket<Req, Resp>,
    }

    impl<Req, Resp> ReadSocket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        /// Get a message from the websocket.
        ///
        /// This should be polled always in order to handle incoming ping messages.
        pub async fn get_message(&self) -> Option<ClientResult<Resp>> {
            self.inner.get_message().await
        }

        /// Close this websocket.
        pub async fn close(&self) -> ClientResult<()> {
            self.inner.close().await.map_err(Into::into)
        }
    }
}

mod error {
    use bytes::Bytes;
    use std::{
        error::Error as StdError,
        fmt::{self, Display, Formatter},
    };
    use tokio_tungstenite::tungstenite;

    /// Convenience type for `Client` operation result.
    pub type ClientResult<T> = Result<T, ClientError>;

    /// Errors that can occur within `Client` operation.
    #[derive(Debug)]
    pub enum ClientError {
        /// Occurs if reqwest, the HTTP client, returns an error.
        Reqwest(reqwest::Error),
        /// Occurs if an endpoint returns an error.
        EndpointError {
            raw_error: Bytes,
            status: reqwest::StatusCode,
            endpoint: String,
        },
        /// Occurs if a websocket returns an error.
        SocketError(tungstenite::Error),
        /// Occurs if the data server responded with can't be decoded as a protobuf response.
        MessageDecode(prost::DecodeError),
        /// Occurs if the data server responded with isn't a protobuf response.
        NonProtobuf(Bytes),
        /// Occurs if the given URL is invalid.
        InvalidUrl(InvalidUrlKind),
        /// Occurs if an IO error is returned.
        Io(std::io::Error),
    }

    impl Display for ClientError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                ClientError::Reqwest(err) => {
                    write!(f, "an error occured within the HTTP client: {}", err)
                }
                ClientError::EndpointError {
                    raw_error,
                    status,
                    endpoint,
                } => write!(
                    f,
                    "endpoint {} returned an error with status code {}: {:?}",
                    endpoint, status, raw_error,
                ),
                ClientError::SocketError(err) => {
                    write!(f, "an error occured within the websocket: {}", err)
                }
                ClientError::NonProtobuf(_) => {
                    write!(f, "server responded with a non protobuf response")
                }
                ClientError::MessageDecode(err) => write!(
                    f,
                    "failed to decode response data as protobuf response: {}",
                    err
                ),
                ClientError::InvalidUrl(err) => write!(f, "invalid base URL: {}", err),
                ClientError::Io(err) => write!(f, "io error: {}", err),
            }
        }
    }

    impl From<reqwest::Error> for ClientError {
        fn from(err: reqwest::Error) -> Self {
            ClientError::Reqwest(err)
        }
    }

    impl From<prost::DecodeError> for ClientError {
        fn from(err: prost::DecodeError) -> Self {
            ClientError::MessageDecode(err)
        }
    }

    impl From<tungstenite::Error> for ClientError {
        fn from(err: tungstenite::Error) -> Self {
            ClientError::SocketError(err)
        }
    }

    impl From<std::io::Error> for ClientError {
        fn from(err: std::io::Error) -> Self {
            ClientError::Io(err)
        }
    }

    impl StdError for ClientError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            match self {
                ClientError::InvalidUrl(err) => Some(err),
                ClientError::MessageDecode(err) => Some(err),
                ClientError::Reqwest(err) => Some(err),
                ClientError::SocketError(err) => Some(err),
                ClientError::Io(err) => Some(err),
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
}
