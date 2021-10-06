use crate::DecodeBodyError;

use super::Body;

use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
};

use axum::{
    body::{box_body, BoxBody},
    extract::ws::rejection::WebSocketUpgradeRejection,
    response::IntoResponse,
};
use bytes::Bytes;
use http::{Response, StatusCode};

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: StdError + Debug + Display + Send + Sync + 'static {
    /// Status code that will be used in client response.
    fn code(&self) -> StatusCode;
    /// Message that will be used in client response.
    fn message(&self) -> Vec<u8>;
}

impl CustomError for std::convert::Infallible {
    fn code(&self) -> StatusCode {
        unreachable!()
    }

    fn message(&self) -> Vec<u8> {
        unreachable!()
    }
}

pub type BodyError = hyper::Error;
pub type SocketError = tokio_tungstenite::tungstenite::Error;

#[doc(hidden)]
#[derive(Debug)]
pub enum ServerError {
    EmptyHeaders,
    EmptyBody,
    MethodNotPost,
    UnsupportedRequestType(Bytes),
    MessageDecode(prost::DecodeError),
    BodyError(BodyError),
    SocketError(SocketError),
    DecodeBodyError(DecodeBodyError),
    WsUpgradeRejection(WebSocketUpgradeRejection),
    Custom(Box<dyn CustomError>),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
            Self::Custom(err) => write!(f, "error occured: {}", err),
            Self::BodyError(err) => write!(f, "body error: {}", err),
            Self::EmptyBody => f.write_str("request body has already been extracted"),
            Self::EmptyHeaders => f.write_str("request headers has already been extracted"),
            Self::MethodNotPost => f.write_str("request method must be POST"),
            Self::UnsupportedRequestType(ty) => write!(f, "request type not supported: {:?}", ty),
            Self::WsUpgradeRejection(err) => write!(f, "could not upgrade to ws: {}", err),
            Self::SocketError(err) => write!(f, "websocket error: {}", err),
            Self::DecodeBodyError(err) => write!(f, "failed to decode body: {}", err),
        }
    }
}

impl StdError for ServerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::MessageDecode(err) => Some(err),
            Self::Custom(err) => err.source(),
            Self::BodyError(err) => Some(err),
            Self::SocketError(err) => Some(err),
            Self::WsUpgradeRejection(err) => Some(err),
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

impl From<BodyError> for ServerError {
    fn from(err: BodyError) -> Self {
        Self::BodyError(err)
    }
}

impl From<SocketError> for ServerError {
    fn from(err: SocketError) -> Self {
        Self::SocketError(err)
    }
}

impl From<WebSocketUpgradeRejection> for ServerError {
    fn from(err: WebSocketUpgradeRejection) -> Self {
        Self::WsUpgradeRejection(err)
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

impl IntoResponse for ServerError {
    type Body = BoxBody;

    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        let body: Body = if let Self::Custom(err) = &self {
            err.message().into()
        } else {
            json_err_bytes(self.to_string()).into()
        };
        let status = match self {
            Self::Custom(err) => err.code(),
            Self::MessageDecode(_) | Self::BodyError(_) => StatusCode::BAD_REQUEST,
            Self::EmptyBody | Self::EmptyHeaders | Self::SocketError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::MethodNotPost => StatusCode::METHOD_NOT_ALLOWED,
            Self::UnsupportedRequestType(_) => StatusCode::BAD_REQUEST,
            Self::WsUpgradeRejection(err) => return err.into_response().map(box_body),
            Self::DecodeBodyError(_) => StatusCode::BAD_REQUEST,
        };

        Response::builder()
            .status(status)
            .body(box_body(body))
            .unwrap()
    }
}

/// Creates a JSON error response from a message.
pub fn json_err_bytes(msg: impl Into<String>) -> Vec<u8> {
    let mut msg = msg.into();
    msg.insert_str(0, r#"{{ "message": "#);
    msg.push_str(r#" }}"#);
    msg.into_bytes()
}
