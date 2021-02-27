use super::*;

use bytes::{Bytes, BytesMut};
use reqwest::header::HeaderName;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use tungstenite::http::HeaderValue;
use url::Url;

#[doc(inline)]
pub use error::*;

type WebSocketStream = async_tungstenite::WebSocketStream<
    async_tungstenite::stream::Stream<
        async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
        async_tungstenite::tokio::TokioAdapter<
            tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
        >,
    >,
>;

type SocketRequest = tungstenite::handshake::client::Request;
type UnaryRequest = reqwest::Request;

/// A hRPC request.
pub struct Request<T> {
    message: T,
    header_map: HashMap<HeaderName, HeaderValue>,
}

impl<T> Request<T> {
    /// Create a new request with the specified message.
    ///
    /// This adds the default "content-type" header used for hRPC unary requests.
    pub fn new(message: T) -> Self {
        Self {
            message,
            header_map: {
                #[allow(clippy::mutable_key_type)]
                let mut map: HashMap<HeaderName, HeaderValue> = HashMap::with_capacity(1);
                map.insert(
                    "content-type".parse().unwrap(),
                    "application/hrpc".parse().unwrap(),
                );
                map
            },
        }
    }

    /// Create an empty request.
    ///
    /// This is useful for hRPC socket requests, since they don't send any messages.
    pub fn empty() -> Request<()> {
        Request {
            message: (),
            header_map: HashMap::new(),
        }
    }

    /// Change / add a header.
    pub fn header(mut self, key: HeaderName, value: HeaderValue) -> Self {
        self.header_map.insert(key, value);
        self
    }

    /// Change the contained message.
    pub fn message<S>(self, message: S) -> Request<S> {
        let Request {
            message: _,
            header_map,
        } = self;

        Request {
            message,
            header_map,
        }
    }
}

/// Trait used for blanket impls on generated protobuf types.
pub trait IntoRequest<T> {
    /// Convert this to a request.
    fn into_request(self) -> Request<T>;
}

impl<T> IntoRequest<T> for T {
    fn into_request(self) -> Request<Self> {
        Request::new(self)
    }
}

impl<T> IntoRequest<T> for Request<T> {
    fn into_request(self) -> Request<T> {
        self
    }
}

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
    pub async fn execute_request<Msg: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        path: &str,
        req: Request<Msg>,
    ) -> ClientResult<Resp> {
        let request = {
            encode_protobuf_message(&mut self.buf, req.message);
            let mut request = UnaryRequest::new(
                reqwest::Method::POST,
                self.server
                    .join(path)
                    .expect("failed to form request URL, something must be terribly wrong"),
            );
            *request.body_mut() = Some(self.buf.to_vec().into());
            for (key, value) in req.header_map {
                request.headers_mut().entry(key).or_insert(value);
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
        #[allow(clippy::blocks_in_if_conditions)]
        if resp
            .headers()
            .get("Content-Type")
            .map(|v| v.to_str().ok())
            .flatten()
            .map(|t| t.split(';').next())
            .flatten()
            .map_or(true, |t| {
                !matches!(t, "application/octet-stream" | "application/hrpc")
            })
        {
            return Err(ClientError::NonProtobuf(resp.bytes().await?));
        }

        let raw = resp.bytes().await?;
        Ok(Resp::decode(raw)?)
    }

    /// Connect a socket with the server and return it.
    pub async fn connect_socket<Msg, Resp>(
        &self,
        path: &str,
        req: Request<()>,
    ) -> ClientResult<Socket<Msg, Resp>>
    where
        Msg: prost::Message,
        Resp: prost::Message + Default,
    {
        let mut url = self.server.join(path).unwrap();
        url.set_scheme(match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            _ => unreachable!("scheme cant be anything other than http or https"),
        })
        .expect("failed to form websocket URL, something must be terribly wrong");

        let mut request = SocketRequest::get(url.into_string()).body(()).unwrap();
        for (key, value) in req.header_map {
            request.headers_mut().entry(key).or_insert(value);
        }

        let inner = async_tungstenite::tokio::connect_async(request).await?.0;
        Ok(Socket::new(inner))
    }
}

/// A websocket, wrapped for ease of use with protobuf messages.
pub struct Socket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    data: Arc<SocketData>,
    _msg: PhantomData<Msg>,
    _resp: PhantomData<Resp>,
}

struct SocketData {
    tx: futures_util::lock::BiLock<WebSocketStream>,
    rx: futures_util::lock::BiLock<WebSocketStream>,
    buf: Mutex<BytesMut>,
}

impl<Msg, Resp> Debug for Socket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Socket")
            .field("buf", &self.data.buf)
            .finish()
    }
}

impl<Msg, Resp> Socket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    fn new(inner: WebSocketStream) -> Self {
        let (tx, rx) = futures_util::lock::BiLock::new(inner);
        Self {
            data: Arc::new(SocketData {
                tx,
                rx,
                buf: Mutex::new(BytesMut::new()),
            }),
            _msg: Default::default(),
            _resp: Default::default(),
        }
    }

    /// Send a protobuf message over the websocket.
    pub async fn send_message(&self, msg: Msg) -> ClientResult<()> {
        use futures_util::SinkExt;

        encode_protobuf_message(&mut *self.data.buf.lock().await, msg);
        let msg = tungstenite::Message::Binary(self.data.buf.lock().await.to_vec());

        Ok(self.data.tx.lock().await.send(msg).await?)
    }

    /// Get a message from the websocket.
    pub async fn get_message(&self) -> Option<ClientResult<Resp>> {
        use futures_util::StreamExt;

        // Without this timeout and the sleep after the timeout expires it blocks and will result in a deadlock
        let raw = tokio::time::timeout(Duration::from_nanos(1), async {
            self.data.rx.lock().await.next().await
        })
        .await;

        let raw = match raw {
            Ok(raw) => raw?,
            Err(_) => {
                tokio::time::sleep(Duration::from_nanos(1)).await;
                return None;
            }
        };

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
        Ok(self.data.tx.lock().await.close(None).await?)
    }

    /// Split this socket into a read and write socket.
    pub fn split(self) -> (ReadSocket<Msg, Resp>, WriteSocket<Msg, Resp>) {
        let read = ReadSocket {
            inner: self.clone(),
        };
        let write = WriteSocket { inner: self };

        (read, write)
    }
}

impl<Msg, Resp> Clone for Socket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            _msg: PhantomData::default(),
            _resp: PhantomData::default(),
        }
    }
}

/// A read-only version of [`Socket`].
#[derive(Debug, Clone)]
pub struct ReadSocket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    inner: Socket<Msg, Resp>,
}

impl<Msg, Resp> ReadSocket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    /// Get a message from the websocket.
    pub async fn get_message(&self) -> Option<ClientResult<Resp>> {
        self.inner.get_message().await
    }
}

/// A write-only version of [`Socket`].
#[derive(Debug, Clone)]
pub struct WriteSocket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    inner: Socket<Msg, Resp>,
}

impl<Msg, Resp> WriteSocket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    /// Send a protobuf message over the websocket.
    pub async fn send_message(&self, msg: Msg) -> ClientResult<()> {
        self.inner.send_message(msg).await
    }

    /// Handle ping messages.
    pub async fn handle_ping(&self) -> ClientResult<()> {
        if let Some(res) = self.inner.get_message().await {
            res?;
        }
        Ok(())
    }

    /// Close this websocket.
    /// All subsequent message sending operations will return an "connection closed" error.
    pub async fn close(&self) -> ClientResult<()> {
        Ok(self.inner.close().await?)
    }
}

mod error {
    use bytes::Bytes;
    use std::fmt::{self, Display, Formatter};

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
        SocketError(async_tungstenite::tungstenite::Error),
        /// Occurs if the data server responded with can't be decoded as a protobuf response.
        MessageDecode(prost::DecodeError),
        /// Occurs if the data server responded with isn't a protobuf response.
        NonProtobuf(Bytes),
        /// Occurs if the given URL is invalid.
        InvalidUrl(InvalidUrlKind),
    }

    impl Display for ClientError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                ClientError::Reqwest(err) => {
                    write!(f, "an error occured within the HTTP client: {}", err)
                }
                ClientError::EndpointError {
                    raw_error: _,
                    status,
                    endpoint,
                } => write!(
                    f,
                    "endpoint {} returned an error with status code {}",
                    endpoint, status,
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

    impl From<async_tungstenite::tungstenite::Error> for ClientError {
        fn from(err: async_tungstenite::tungstenite::Error) -> Self {
            ClientError::SocketError(err)
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
}
