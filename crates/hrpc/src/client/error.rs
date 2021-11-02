use bytes::Bytes;
use http::{StatusCode, Uri};
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

pub use crate::{proto::Error as HrpcError, DecodeBodyError};
pub use http::Error as HttpError;
pub use hyper::Error as HyperError;
pub use std::io::Error as IoError;
pub use tokio_tungstenite::tungstenite::Error as SocketError;

/// Convenience type for `Client` operation result.
pub type ClientResult<T> = Result<T, ClientError>;

/// Errors that can occur within `Client` operation.
#[derive(Debug)]
pub enum ClientError {
    /// Occurs if request creation fails.
    FailedRequestBuilder(HttpError),
    /// Occurs if hyper, the HTTP client, returns an error.
    Http(HyperError),
    /// Occurs if an endpoint returns an error.
    EndpointError {
        /// The hRPC error.
        hrpc_error: HrpcError,
        /// The HTTP status of the error.
        status: StatusCode,
        /// The endpoint for which this error happened.
        endpoint: Uri,
    },
    /// Occurs if a websocket returns an error.
    SocketError(SocketError),
    /// Occurs if the data server responded with could not be decoded.
    MessageDecode(DecodeBodyError),
    /// Occurs if the data server responded with is not supported for decoding.
    ContentNotSupported(Bytes),
    /// Occurs if the given URL is invalid.
    InvalidUrl(InvalidUrlKind),
    /// Occurs if an IO error is returned.
    Io(IoError),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ClientError::FailedRequestBuilder(err) => {
                write!(f, "error occured while building a request: {}", err)
            }
            ClientError::Http(err) => {
                write!(f, "an error occured within the HTTP client: {}", err)
            }
            ClientError::EndpointError {
                hrpc_error,
                status,
                endpoint,
            } => write!(
                f,
                "endpoint {} returned an error with status code {}, error identifier '{}': {}",
                endpoint, status, hrpc_error.identifier, hrpc_error.human_message,
            ),
            ClientError::SocketError(err) => {
                write!(f, "an error occured within the websocket: {}", err)
            }
            ClientError::ContentNotSupported(_) => {
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

impl From<hyper::Error> for ClientError {
    fn from(err: hyper::Error) -> Self {
        ClientError::Http(err)
    }
}

impl From<DecodeBodyError> for ClientError {
    fn from(err: DecodeBodyError) -> Self {
        ClientError::MessageDecode(err)
    }
}

impl From<SocketError> for ClientError {
    fn from(err: SocketError) -> Self {
        ClientError::SocketError(err)
    }
}

impl From<IoError> for ClientError {
    fn from(err: IoError) -> Self {
        ClientError::Io(err)
    }
}

impl From<tower_http::BodyOrIoError<hyper::Error>> for ClientError {
    fn from(err: tower_http::BodyOrIoError<hyper::Error>) -> Self {
        match err {
            tower_http::BodyOrIoError::Body(err) => err.into(),
            tower_http::BodyOrIoError::Io(err) => err.into(),
        }
    }
}

impl StdError for ClientError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ClientError::InvalidUrl(err) => Some(err),
            ClientError::MessageDecode(err) => Some(err),
            ClientError::Http(err) => Some(err),
            ClientError::SocketError(err) => Some(err),
            ClientError::Io(err) => Some(err),
            ClientError::FailedRequestBuilder(err) => Some(err),
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
