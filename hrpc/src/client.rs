use std::{
    fmt::{self, Debug, Formatter},
    sync::Arc,
};

use super::*;

use bytes::{Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use tokio_rustls::webpki::DNSNameRef;
use tracing::{debug, error};
use url::Url;

#[doc(inline)]
pub use error::*;

type WebSocketStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

type SocketRequest = tungstenite::handshake::client::Request;
type UnaryRequest = reqwest::Request;

/// Generic client implementation with common methods.
#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    server: Url,
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
    /// Creates a new client.
    pub fn new(inner: reqwest::Client, server: Url) -> ClientResult<Self> {
        if let "http" | "https" = server.scheme() {
            Ok(Self {
                inner,
                server,
                buf: BytesMut::new(),
                modify_request_headers: Arc::new(|_| ()),
            })
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

    /// Access to the inner HTTP client.
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Executes an unary request and returns the decoded response.
    pub async fn execute_request<Req: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        path: &str,
        mut req: Request<Req>,
    ) -> ClientResult<Resp> {
        let request = {
            encode_protobuf_message_to(&mut self.buf, req.message);
            let mut request = UnaryRequest::new(
                reqwest::Method::POST,
                self.server
                    .join(path)
                    .expect("failed to form request URL, something must be terribly wrong"),
            );
            *request.body_mut() = Some(self.buf.clone().freeze().into());
            (self.modify_request_headers)(&mut req.header_map);
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
        mut req: Request<()>,
    ) -> ClientResult<Socket<Req, Resp>>
    where
        Req: prost::Message + 'static,
        Resp: prost::Message + Default + 'static,
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
        let (message, headers, url) = request.into_parts();
        let socket = self
            .connect_socket(path, Request::from_parts(((), headers, url)))
            .await?;
        socket.send_message(message).await?;

        Ok(socket)
    }
}

/// A websocket, wrapped for ease of use with protobuf messages.
#[derive(Debug, Clone)]
pub struct Socket<Req, Resp>
where
    Req: prost::Message,
    Resp: prost::Message + Default,
{
    rx: flume::Receiver<Result<Resp, ClientError>>,
    tx: flume::Sender<Req>,
    close_chan: flume::Sender<()>,
}

impl<Req, Resp> Socket<Req, Resp>
where
    Req: prost::Message + 'static,
    Resp: prost::Message + Default + 'static,
{
    pub(super) fn new(mut ws: WebSocketStream) -> Self {
        let (recv_msg_tx, recv_msg_rx) = flume::bounded(64);
        let (send_msg_tx, send_msg_rx) = flume::bounded(64);
        let (close_chan_tx, close_chan_rx) = flume::bounded(1);
        tokio::spawn(async move {
            let mut buf = BytesMut::new();
            loop {
                tokio::select! {
                    Some(res_msg) = ws.next() => {
                        let resp = match res_msg {
                            Ok(msg) => {
                                use tungstenite::Message;

                                match msg {
                                    Message::Binary(raw) => {
                                        Resp::decode(Bytes::from(raw)).map_err(Into::into)
                                    }
                                    Message::Close(_) => {
                                        let _ = recv_msg_tx.send_async(Err(tungstenite::Error::ConnectionClosed.into())).await;
                                        let _ = ws.close(None).await;
                                        return;
                                    },
                                    Message::Ping(data) => {
                                        let pong_res = ws
                                            .send(tungstenite::Message::Pong(data))
                                            .await;
                                        if let Err(err) = pong_res {
                                            Err(ClientError::SocketError(err))
                                        } else {
                                            continue;
                                        }
                                    },
                                    Message::Pong(_) | Message::Text(_) => continue,
                                }
                            }
                            Err(err) => if let tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed = err {
                                let _ = recv_msg_tx.send_async(Err(ClientError::SocketError(err))).await;
                                let _ = ws.close(None).await;
                                return;
                            } else {
                                Err(ClientError::SocketError(err))
                            },
                        };
                        if recv_msg_tx.send_async(resp).await.is_err() {
                            let _ = ws.close(None).await;
                            return;
                        }
                    }
                    Ok(resp) = send_msg_rx.recv_async() => {
                        let resp = {
                            crate::encode_protobuf_message_to(&mut buf, resp);
                            buf.to_vec()
                        };

                        if let Err(e) = ws.send(tungstenite::Message::binary(resp)).await {
                            error!("socket send error: {}", e);
                            if let tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed = e {
                                let _ = recv_msg_tx.send_async(Err(ClientError::SocketError(e))).await;
                                let _ = ws.close(None).await;
                                return;
                            }
                        } else {
                            debug!("responded to server socket");
                        }
                    }
                    // If we get *anything*, it means that either the channel is closed
                    // or we got a close message
                    _ = close_chan_rx.recv_async() => {
                        if let Err(err) = ws.close(None).await {
                            let _ = recv_msg_tx.send_async(Err(ClientError::SocketError(err))).await;
                        }
                        return;
                    }
                    else => tokio::task::yield_now().await,
                }
            }
        });

        Self {
            rx: recv_msg_rx,
            tx: send_msg_tx,
            close_chan: close_chan_tx,
        }
    }

    /// Receive a message from the socket.
    ///
    /// Returns a connection closed error if the socket is closed.
    /// This will block until getting a message if the socket is not closed.
    pub async fn receive_message(&self) -> ClientResult<Resp> {
        if self.is_closed() {
            Err(ClientError::SocketError(
                tungstenite::Error::ConnectionClosed,
            ))
        } else {
            self.rx
                .recv_async()
                .await
                .unwrap_or(Err(ClientError::SocketError(
                    tungstenite::Error::ConnectionClosed,
                )))
        }
    }

    /// Send a message over the socket.
    ///
    /// This will block if the inner send buffer is filled.
    pub async fn send_message(&self, req: Req) -> ClientResult<()> {
        if self.is_closed() || self.tx.send_async(req).await.is_err() {
            Err(ClientError::SocketError(
                tungstenite::Error::ConnectionClosed,
            ))
        } else {
            Ok(())
        }
    }

    /// Return whether the socket is closed or not.
    pub fn is_closed(&self) -> bool {
        self.close_chan.is_disconnected()
    }

    /// Close the socket.
    pub async fn close(&self) {
        // We don't care about the error, it's closed either way
        let _ = self.close_chan.send_async(()).await;
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
