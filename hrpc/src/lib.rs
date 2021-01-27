pub use bytes;
pub use prost;
pub use reqwest;
pub use url;

#[doc(inline)]
pub use error::*;

/// Generic client implementation with common methods.
#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    authorization: Option<String>,
    server: url::Url,
    buf: bytes::BytesMut,
}

impl Client {
    /// Creates a new client.
    pub fn new(inner: reqwest::Client, server: url::Url) -> ClientResult<Self> {
        if let "http" | "https" = server.scheme() {
            Ok(Self {
                inner,
                server,
                authorization: None,
                buf: bytes::BytesMut::new(),
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
                    "an error occured with the HTTP client, or the server returned an error: {}",
                    err
                ),
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
