use std::convert::Infallible;

use futures_util::{Sink, Stream};

use crate::{
    common::socket::{BoxedSocketRx, BoxedSocketTx, SocketMessage},
    BoxError, Request,
};

use super::error::ClientError;

/// Client HTTP transport.
#[cfg(feature = "_common_http_client")]
pub mod http;
/// The mock transport. Useful for testing.
#[cfg(feature = "mock_client")]
pub mod mock;

/// Error type that transports need to return.
pub enum TransportError<Err> {
    /// A transport specific error.
    Transport(Err),
    /// A generic client error. This can be used by transports to reduce
    /// duplicated error variants.
    GenericClient(ClientError<Infallible>),
}

impl<Err> From<TransportError<Err>> for ClientError<Err> {
    fn from(err: TransportError<Err>) -> Self {
        match err {
            TransportError::Transport(err) => ClientError::Transport(err),
            TransportError::GenericClient(err) => match err {
                ClientError::ContentNotSupported => ClientError::ContentNotSupported,
                ClientError::EndpointError {
                    hrpc_error,
                    endpoint,
                } => ClientError::EndpointError {
                    hrpc_error,
                    endpoint,
                },
                ClientError::IncompatibleSpecVersion => ClientError::IncompatibleSpecVersion,
                ClientError::MessageDecode(err) => ClientError::MessageDecode(err),
                ClientError::Transport(_) => unreachable!("infallible"),
            },
        }
    }
}

impl<Err> From<ClientError<Err>> for TransportError<Err> {
    fn from(err: ClientError<Err>) -> Self {
        match err {
            ClientError::Transport(err) => TransportError::Transport(err),
            other => TransportError::GenericClient(match other {
                ClientError::ContentNotSupported => ClientError::ContentNotSupported,
                ClientError::EndpointError {
                    hrpc_error,
                    endpoint,
                } => ClientError::EndpointError {
                    hrpc_error,
                    endpoint,
                },
                ClientError::IncompatibleSpecVersion => ClientError::IncompatibleSpecVersion,
                ClientError::MessageDecode(err) => ClientError::MessageDecode(err),
                ClientError::Transport(_) => unreachable!("infallible"),
            }),
        }
    }
}

/// Struct that should be used by transports to return socket channels
/// to generic client.
pub struct SocketChannels {
    pub(super) tx: BoxedSocketTx,
    pub(super) rx: BoxedSocketRx,
}

impl SocketChannels {
    /// Create a new socket channels.
    pub fn new<Tx, Rx>(tx: Tx, rx: Rx) -> Self
    where
        Tx: Sink<SocketMessage, Error = BoxError> + Send + Sync + 'static,
        Rx: Stream<Item = Result<SocketMessage, BoxError>> + Send + Sync + 'static,
    {
        Self {
            tx: Box::pin(tx),
            rx: Box::pin(rx),
        }
    }
}

/// Marker struct that marks a request as a socket request.
pub(super) struct SocketRequestMarker;

/// Returns whether a request is a socket request or not.
pub fn is_socket_request<T>(req: &Request<T>) -> bool {
    req.extensions().contains::<SocketRequestMarker>()
}
