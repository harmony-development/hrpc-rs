use super::HttpResponse;
use crate::{body::box_body, DecodeBodyError};

use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
};

use bytes::Bytes;
use http::StatusCode;

pub use tokio_tungstenite::tungstenite::Error as SocketError;

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: Debug + Send + Sync + 'static {
    /// Status code and message that will be used in client response.
    fn as_status_message(&self) -> (StatusCode, Bytes);

    /// Create a response from this error.
    fn as_error_response(&self) -> HttpResponse {
        let (status, message) = self.as_status_message();
        http::Response::builder()
            .status(status)
            .body(box_body(hyper::Body::from(message)))
            .unwrap()
    }
}

impl CustomError for std::convert::Infallible {
    fn as_status_message(&self) -> (StatusCode, Bytes) {
        unreachable!()
    }
}

impl CustomError for &'static str {
    fn as_status_message(&self) -> (StatusCode, Bytes) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_err_bytes(*self).into(),
        )
    }
}

impl CustomError for (StatusCode, &'static str) {
    fn as_status_message(&self) -> (StatusCode, Bytes) {
        (self.0, json_err_bytes(self.1).into())
    }
}

/// Shorthand type for `Result<T, ServerError>.
pub type ServerResult<T> = Result<T, ServerError>;

/// A server error.
#[derive(Debug)]
pub enum ServerError {
    MethodNotPost,
    UnsupportedRequestType(Bytes),
    SocketError(SocketError),
    DecodeBodyError(DecodeBodyError),
    Custom(Box<dyn CustomError>),
}

impl ServerError {
    /// Convert this error into a response.
    pub fn into_response(self) -> HttpResponse {
        let (status, message) = self.into_status_message();

        http::Response::builder()
            .status(status)
            .body(box_body(hyper::Body::from(message)))
            .unwrap()
    }

    fn into_status_message(self) -> (StatusCode, Bytes) {
        if let Self::Custom(err) = self {
            return err.as_status_message();
        }
        let code = match &self {
            Self::Custom(_) => unreachable!(),
            Self::SocketError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MethodNotPost => StatusCode::METHOD_NOT_ALLOWED,
            Self::UnsupportedRequestType(_) => StatusCode::BAD_REQUEST,
            Self::DecodeBodyError(_) => StatusCode::BAD_REQUEST,
        };
        (code, json_err_bytes(self.to_string()).into())
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Custom(err) => write!(f, "error occured: {:?}", err),
            Self::MethodNotPost => f.write_str("request method must be POST"),
            Self::UnsupportedRequestType(ty) => write!(f, "request type not supported: {:?}", ty),
            Self::SocketError(err) => write!(f, "websocket error: {}", err),
            Self::DecodeBodyError(err) => write!(f, "failed to decode request body: {}", err),
        }
    }
}

impl StdError for ServerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::SocketError(err) => Some(err),
            Self::DecodeBodyError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<SocketError> for ServerError {
    fn from(err: SocketError) -> Self {
        Self::SocketError(err)
    }
}

impl<Err: CustomError> From<Err> for ServerError {
    fn from(err: Err) -> Self {
        Self::Custom(Box::new(err))
    }
}

impl From<DecodeBodyError> for ServerError {
    fn from(err: DecodeBodyError) -> Self {
        Self::DecodeBodyError(err)
    }
}

/// Creates a JSON error response from a message.
pub fn json_err_bytes(msg: impl Into<String>) -> Vec<u8> {
    let mut msg = msg.into();
    msg.insert_str(0, r#"{{ "message": "#);
    msg.push_str(r#" }}"#);
    msg.into_bytes()
}
