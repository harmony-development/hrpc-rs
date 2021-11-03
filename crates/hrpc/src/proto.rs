use bytes::Bytes;

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
