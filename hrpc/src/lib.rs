use bytes::{Bytes, BytesMut};
use std::marker::PhantomData;
use tokio::net::TcpStream;
use url::Url;

pub use bytes;
pub use prost;
pub use reqwest;
pub use tokio_tungstenite::{self, tungstenite};
pub use url;

#[doc(inline)]
pub use error::*;

/// Generic client implementation with common methods.
#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    authorization: Option<String>,
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
                authorization: None,
                buf: BytesMut::new(),
            })
        } else {
            Err(ClientError::InvalidUrl(InvalidUrlKind::InvalidScheme))
        }
    }

    /// Sets the authorization token of this client.
    pub fn set_auth_token(&mut self, authorization: Option<String>) {
        self.authorization = authorization;
    }

    /// Makes a unary request from a protobuf message and the given endpoint path.
    pub fn make_request(
        &mut self,
        msg: impl prost::Message,
        path: &str,
    ) -> ClientResult<reqwest::Request> {
        self.buf
            .reserve(msg.encoded_len().saturating_sub(self.buf.len()));
        self.buf.clear();
        msg.encode(&mut self.buf)
            .expect("failed to encode protobuf message, something must be terribly wrong");

        let mut req = self.inner.post(
            self.server
                .join(path)
                .expect("failed to form request URL, something must be terribly wrong"),
        );
        req = req.body(self.buf.to_vec());
        if let Some(auth) = self.authorization.as_deref() {
            req = req.bearer_auth(auth);
        }

        Ok(req.build()?)
    }

    /// Executes an unary request returns the decoded response.
    pub async fn execute_request<Msg: prost::Message + Default>(
        &mut self,
        req: reqwest::Request,
    ) -> ClientResult<Msg> {
        let resp = self.inner.execute(req).await?;
        let raw = resp.bytes().await?;
        Ok(Msg::decode(raw)?)
    }

    pub async fn connect_socket<Msg, Resp>(&self, path: &str) -> ClientResult<Socket<Msg, Resp>>
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

        let request = if let Some(auth) = self.authorization.as_deref() {
            tungstenite::handshake::client::Request::get(url.to_string())
                .header("Authorization", auth)
                .body(())
                .unwrap()
        } else {
            tungstenite::handshake::client::Request::get(url.to_string())
                .body(())
                .unwrap()
        };

        let inner = tokio_tungstenite::connect_async(request).await?.0;
        Ok(Socket::new(inner))
    }
}

/// A message returned from the [`Socket::get_message()`] method of a [`Socket`].
#[derive(Debug)]
pub enum SocketMessage<Msg: prost::Message + Default> {
    /// A protobuf message.
    Protobuf(Msg),
    /// A text message.
    Text(String),
    /// A ping, should be responded with a pong.
    Ping(Bytes),
    /// A pong.
    Pong(Bytes),
}

/// A websocket, wrapped for ease of use with protobuf messages.
#[derive(Debug)]
pub struct Socket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    inner: tokio_tungstenite::WebSocketStream<TcpStream>,
    buf: BytesMut,
    _msg: PhantomData<Msg>,
    _resp: PhantomData<Resp>,
}

impl<Msg, Resp> Socket<Msg, Resp>
where
    Msg: prost::Message,
    Resp: prost::Message + Default,
{
    fn new(inner: tokio_tungstenite::WebSocketStream<TcpStream>) -> Self {
        Self {
            inner,
            buf: BytesMut::new(),
            _msg: Default::default(),
            _resp: Default::default(),
        }
    }

    /// Send a protobuf message over the websocket.
    pub async fn send_message(&mut self, msg: Msg) -> ClientResult<()> {
        use futures_util::SinkExt;

        self.buf
            .reserve(msg.encoded_len().saturating_sub(self.buf.len()));
        self.buf.clear();
        msg.encode(&mut self.buf)
            .expect("failed to encode protobuf message, something must be terribly wrong");

        Ok(self
            .inner
            .send(tungstenite::Message::Binary(self.buf.to_vec()))
            .await?)
    }

    /// Get a message from the websocket.
    pub async fn get_message(&mut self) -> Option<ClientResult<SocketMessage<Resp>>> {
        use futures_util::StreamExt;

        let raw = self.inner.next().await?;

        match raw {
            Ok(msg) => {
                use tungstenite::Message;

                match msg {
                    Message::Binary(raw) => Some(
                        Resp::decode(Bytes::from(raw))
                            .map(SocketMessage::Protobuf)
                            .map_err(Into::into),
                    ),
                    Message::Close(_) => Some(Err(tungstenite::Error::ConnectionClosed.into())),
                    Message::Text(msg) => Some(Ok(SocketMessage::Text(msg))),
                    Message::Ping(ping) => Some(Ok(SocketMessage::Ping(ping.into()))),
                    Message::Pong(pong) => Some(Ok(SocketMessage::Pong(pong.into()))),
                }
            }
            Err(err) => Some(Err(err.into())),
        }
    }

    /// Sends a "ping" message over the websocket and returns the payload as bytes.
    pub async fn ping(&mut self) -> ClientResult<Bytes> {
        use futures_util::SinkExt;

        let bytes = Bytes::from_static(b"ping");
        self.inner
            .send(tungstenite::Message::Ping(bytes.to_vec()))
            .await?;

        Ok(bytes)
    }

    /// Close and drop this websocket.
    pub async fn close(mut self) -> ClientResult<()> {
        Ok(self.inner.close(None).await?)
    }
}

mod error {
    use std::fmt::{self, Display, Formatter};

    /// Convenience type for [`Client`] operation result.
    pub type ClientResult<T> = Result<T, ClientError>;

    /// Errors that can occur within [`Client`] operation.
    #[derive(Debug)]
    pub enum ClientError {
        /// Occurs if reqwest, the HTTP client, returns an error.
        Reqwest(reqwest::Error),
        /// Occurs if a websocket returns an error.
        SocketError(tokio_tungstenite::tungstenite::Error),
        /// Occurs if the data server responded with can't be decoded as a protobuf response.
        MessageDecode(prost::DecodeError),
        /// Occurs if the given URL is invalid.
        InvalidUrl(InvalidUrlKind),
    }

    impl Display for ClientError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                ClientError::Reqwest(err) => write!(
                    f,
                    "an error occured within the HTTP client, or the server returned an error: {}",
                    err
                ),
                ClientError::SocketError(err) => {
                    write!(f, "an error occured within the websocket: {}", err)
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

    impl From<tokio_tungstenite::tungstenite::Error> for ClientError {
        fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
            ClientError::SocketError(err)
        }
    }

    #[derive(Debug)]
    pub enum InvalidUrlKind {
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
