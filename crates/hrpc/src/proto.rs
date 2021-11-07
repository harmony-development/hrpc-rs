use std::{error::Error as StdError, fmt::Display, str::FromStr};

use bytes::Bytes;

use crate::{response::BoxResponse, BoxError, Response};

crate::include_proto!("hrpc.v1");

/// Represents a hRPC error identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HrpcErrorIdentifier {
    /// Endpoint was not implemented by server.
    NotImplemented,
    /// Reached resource quota or rate limited by server.
    ResourceExhausted,
    /// An error occured in server.
    InternalServerError,
    /// The server could not be reached, most likely means the server is down.
    Unavailable,
    /// Specified endpoint was not found on server.
    NotFound,
}

impl From<HrpcErrorIdentifier> for String {
    fn from(id: HrpcErrorIdentifier) -> Self {
        id.as_id().to_string()
    }
}

impl AsRef<str> for HrpcErrorIdentifier {
    fn as_ref(&self) -> &str {
        self.as_id()
    }
}

/// Error produced when trying to parse a string as a [`HrpcErrorIdentifier`].
#[derive(Debug)]
#[non_exhaustive]
pub struct NotHrpcErrorIdentifier;

impl FromStr for HrpcErrorIdentifier {
    type Err = NotHrpcErrorIdentifier;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        compare_if_ret::compare_if_ret! {
            s,
            Self::NotImplemented,
            Self::ResourceExhausted,
            Self::InternalServerError,
            Self::Unavailable,
            Self::NotFound,
        }
    }
}

mod compare_if_ret {
    macro_rules! compare_if_ret {
        ($var:ident, $variant:expr, $( $variant2:expr, )+) => {
            if $variant.compare($var) {
                return Ok($variant);
            } $(
                else if $variant2.compare($var) {
                    return Ok($variant2);
                }
            )* else {
                return Err(NotHrpcErrorIdentifier);
            }
        };
    }

    pub(crate) use compare_if_ret;
}

impl HrpcErrorIdentifier {
    /// Return the string version of this hRPC identifier.
    pub const fn as_id(&self) -> &'static str {
        match self {
            Self::InternalServerError => "hrpc.internal-server-error",
            Self::ResourceExhausted => "hrpc.resource-exhausted",
            Self::NotImplemented => "hrpc.not-implemented",
            Self::Unavailable => "hrpc.unavailable",
            Self::NotFound => "hrpc.not-found",
        }
    }

    /// Compare this hRPC identifier with some string identifier to see if they match.
    pub fn compare(&self, identifier: impl AsRef<str>) -> bool {
        identifier.as_ref() == self.as_id()
    }
}

impl Error {
    /// Create a new hRPC error representing a not implemented endpoint ([`HrpcErrorIdentifier::NotImplemented`]).
    pub fn new_not_implemented(message: impl Into<String>) -> Self {
        Self::default()
            .with_identifier(HrpcErrorIdentifier::NotImplemented)
            .with_message(message)
    }

    /// Create a new hRPC error representing resource exhaustion by a client ([`HrpcErrorIdentifier::ResourceExhausted`]).
    pub fn new_resource_exhausted(message: impl Into<String>) -> Self {
        Self::default()
            .with_identifier(HrpcErrorIdentifier::ResourceExhausted)
            .with_message(message)
    }

    /// Create a new hRPC error representing an internal server error ([`HrpcErrorIdentifier::InternalServerError`]).
    pub fn new_internal_server_error(message: impl Into<String>) -> Self {
        Self::default()
            .with_identifier(HrpcErrorIdentifier::InternalServerError)
            .with_message(message)
    }

    /// Create a new hRPC error representing a not found error ([`HrpcErrorIdentifier::NotFound`]).
    pub fn new_not_found(message: impl Into<String>) -> Self {
        Self::default()
            .with_identifier(HrpcErrorIdentifier::NotFound)
            .with_message(message)
    }

    /// Set the "more details" of this hRPC error.
    pub fn with_details(mut self, details: impl Into<Bytes>) -> Self {
        self.more_details = details.into();
        self
    }

    /// Set the "identifier" of this hRPC error.
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = identifier.into();
        self
    }

    /// Set the "human message" of this hRPC error.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.human_message = message.into();
        self
    }
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Self {
        Error::default()
            .with_identifier(HrpcErrorIdentifier::InternalServerError)
            .with_message(msg)
    }
}

impl From<(&'static str, &'static str)> for Error {
    fn from((id, msg): (&'static str, &'static str)) -> Self {
        Error::default().with_identifier(id).with_message(msg)
    }
}

impl From<BoxError> for Error {
    fn from(err: BoxError) -> Self {
        Error::default().with_message(err.to_string())
    }
}

impl From<(&'static str, BoxError)> for Error {
    fn from((id, err): (&'static str, BoxError)) -> Self {
        Error::default()
            .with_identifier(id)
            .with_message(err.to_string())
    }
}

impl From<Error> for Response<Error> {
    fn from(err: Error) -> Self {
        let mut resp = Response::new(&err);
        resp.extensions_mut().insert(err);
        resp
    }
}

impl From<Error> for BoxResponse {
    fn from(err: Error) -> Self {
        Response::<Error>::from(err).map::<()>()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error '{}': {}", self.identifier, self.human_message)
    }
}

impl StdError for Error {}
