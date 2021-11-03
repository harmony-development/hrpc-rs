use super::HttpResponse;
use crate::{body::box_body, encode_protobuf_message, proto::Error as HrpcError, DecodeBodyError};

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
};

use bytes::Bytes;
use http::StatusCode;

pub use tokio_tungstenite::tungstenite::Error as SocketError;

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: Debug + Send + Sync + 'static {
    /// Error message for humans.
    fn error_message(&self) -> Cow<'_, str> {
        format!("{:?}", self).into()
    }

    /// HTTP status code that will be used in client response.
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Error identifier.
    fn identifier(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    /// More details for this error.
    fn more_details(&self) -> Bytes {
        Bytes::new()
    }

    /// Create an hRPC protocol error from this custom error.
    fn as_hrpc_error(&self) -> HrpcError {
        HrpcError {
            human_message: self.error_message().into_owned(),
            identifier: self.identifier().into_owned(),
            more_details: self.more_details(),
        }
    }

    /// Create a response from this error.
    fn as_error_response(&self) -> HttpResponse {
        let error = self.as_hrpc_error();
        let encoded_error = encode_protobuf_message(error).freeze();

        let status = self.status();

        http::Response::builder()
            .status(status)
            .body(box_body(hyper::Body::from(encoded_error)))
            .unwrap()
    }
}

impl CustomError for std::convert::Infallible {}

impl CustomError for &'static str {
    fn error_message(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl CustomError for (StatusCode, &'static str) {
    fn error_message(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.1)
    }

    fn status(&self) -> StatusCode {
        self.0
    }
}

impl CustomError for String {
    fn error_message(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_str())
    }
}

impl CustomError for (StatusCode, String) {
    fn error_message(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.1.as_str())
    }

    fn status(&self) -> StatusCode {
        self.0
    }
}

impl CustomError for SocketError {
    fn error_message(&self) -> Cow<'_, str> {
        format!("{}", self).into()
    }

    fn identifier(&self) -> Cow<'_, str> {
        Cow::Borrowed("hrpcrs.socket-error")
    }
}

impl CustomError for DecodeBodyError {
    fn error_message(&self) -> Cow<'_, str> {
        self.to_string().into()
    }

    fn status(&self) -> StatusCode {
        match self {
            DecodeBodyError::InvalidBody(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DecodeBodyError::InvalidProtoMessage(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn identifier(&self) -> Cow<'_, str> {
        Cow::Borrowed("hrpcrs.decode-body-error")
    }
}

impl CustomError for HrpcError {
    fn error_message(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.human_message)
    }

    fn status(&self) -> StatusCode {
        match self.identifier.as_str() {
            "hrpc.resource-exhausted" => StatusCode::TOO_MANY_REQUESTS,
            "hrpc.not-implemented" => StatusCode::NOT_IMPLEMENTED,
            "hrpc.not-found" => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn identifier(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.identifier)
    }

    fn more_details(&self) -> Bytes {
        self.more_details.clone()
    }

    fn as_hrpc_error(&self) -> HrpcError {
        self.clone()
    }
}

/// Shorthand type for `Result<T, ServerError>.
pub type ServerResult<T> = Result<T, ServerError>;

/// A server error.
#[derive(Debug)]
pub struct ServerError {
    inner: Box<dyn CustomError>,
}

impl ServerError {
    /// Convert this error into a HTTP response.
    pub fn as_error_response(&self) -> HttpResponse {
        self.inner.as_error_response()
    }

    /// Get a reference to the inner [`CustomError`] for this error.
    #[inline]
    pub fn as_custom_error(&self) -> &dyn CustomError {
        self.inner.as_ref()
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.inner.error_message())
    }
}

impl StdError for ServerError {}

impl<Err: CustomError> From<Err> for ServerError {
    fn from(err: Err) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}
