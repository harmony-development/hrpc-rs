use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

pub use crate::{decode::DecodeBodyError, proto::Error as HrpcError};
pub use std::io::Error as IoError;

/// Convenience type for `Client` operation result.
pub type ClientResult<T, TransportError> = Result<T, ClientError<TransportError>>;

/// Errors that can occur within `Client` operation.
#[derive(Debug)]
pub enum ClientError<TransportError> {
    /// Occurs if an endpoint returns an error.
    EndpointError {
        /// The hRPC error.
        hrpc_error: HrpcError,
        /// The endpoint for which this error happened.
        endpoint: Cow<'static, str>,
    },
    /// Occurs if the data server responded with could not be decoded.
    MessageDecode(DecodeBodyError),
    /// Occurs if the data server responded with is not supported for decoding.
    ContentNotSupported,
    /// Occures if the underlying transport yields an error.
    Transport(TransportError),
    /// Occurs if the spec implemented on server doesn't match ours.
    IncompatibleSpecVersion,
}

impl<TransportError: StdError> Display for ClientError<TransportError> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ClientError::EndpointError {
                hrpc_error,
                endpoint,
            } => write!(
                f,
                "endpoint {} returned an error '{}': {}",
                endpoint, hrpc_error.identifier, hrpc_error.human_message,
            ),
            ClientError::ContentNotSupported => {
                write!(f, "server responded with a non protobuf response")
            }
            ClientError::MessageDecode(err) => write!(
                f,
                "failed to decode response data as protobuf response: {}",
                err
            ),
            ClientError::Transport(err) => write!(f, "transport error: {}", err),
            ClientError::IncompatibleSpecVersion => {
                write!(f, "server hrpc version is incompatible with ours")
            }
        }
    }
}

impl<TransportError> From<DecodeBodyError> for ClientError<TransportError> {
    fn from(err: DecodeBodyError) -> Self {
        ClientError::MessageDecode(err)
    }
}

impl<TransportError: StdError + 'static> StdError for ClientError<TransportError> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ClientError::MessageDecode(err) => Some(err),
            ClientError::Transport(err) => Some(err),
            ClientError::EndpointError { hrpc_error, .. } => Some(hrpc_error),
            _ => None,
        }
    }
}
