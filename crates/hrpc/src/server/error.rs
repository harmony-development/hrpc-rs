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

    /// Try to downcast this error into another error. Most commonly this can
    /// be used for downcasting to [`SocketError`] or [`DecodeBodyError`].
    ///
    /// # Example
    /// ```rust
    /// # use hrpc::server::error::{ServerError, SocketError};
    /// let err = ServerError::from(SocketError::ConnectionClosed);
    /// assert_eq!(err.downcast_into::<SocketError>(), Ok(SocketError::ConnectionClosed));
    /// ```
    pub fn downcast_into<T: 'static>(self) -> Result<T, Self> {
        let err = self;

        super::utils::downcast::if_downcast_into!(Self, T, err, {
            return Ok(err);
        });

        Err(err)
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
