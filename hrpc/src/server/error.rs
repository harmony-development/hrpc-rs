use super::HttpResponse;
use crate::{body::box_body, BoxError, DecodeBodyError};

use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
};

use bytes::Bytes;
use http::StatusCode;

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: Debug + Send + Sync + 'static {
    /// Status code and message that will be used in client response.
    fn as_status_message(&self) -> (StatusCode, Bytes);

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

/// Socket error.
pub type SocketError = tokio_tungstenite::tungstenite::Error;

/// Shorthand type for `Result<T, ServerError>.
pub type ServerResult<T> = Result<T, ServerError>;

#[doc(hidden)]
#[derive(Debug)]
pub enum ServerError {
    EmptyHeaders,
    EmptyBody,
    MethodNotPost,
    UnsupportedRequestType(Bytes),
    MessageDecode(prost::DecodeError),
    BodyError(BoxError),
    SocketError(SocketError),
    DecodeBodyError(DecodeBodyError),
    Custom(Box<dyn CustomError>),
}

impl ServerError {
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
            Self::MessageDecode(_) | Self::BodyError(_) => StatusCode::BAD_REQUEST,
            Self::EmptyBody | Self::EmptyHeaders | Self::SocketError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
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
            Self::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
            Self::Custom(err) => write!(f, "error occured: {:?}", err),
            Self::BodyError(err) => write!(f, "body error: {}", err),
            Self::EmptyBody => f.write_str("request body has already been extracted"),
            Self::EmptyHeaders => f.write_str("request headers has already been extracted"),
            Self::MethodNotPost => f.write_str("request method must be POST"),
            Self::UnsupportedRequestType(ty) => write!(f, "request type not supported: {:?}", ty),
            Self::SocketError(err) => write!(f, "websocket error: {}", err),
            Self::DecodeBodyError(err) => write!(f, "failed to decode body: {}", err),
        }
    }
}

impl StdError for ServerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::MessageDecode(err) => Some(err),
            Self::SocketError(err) => Some(err),
            Self::DecodeBodyError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<prost::DecodeError> for ServerError {
    fn from(err: prost::DecodeError) -> Self {
        Self::MessageDecode(err)
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
