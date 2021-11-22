use std::convert::Infallible;

use futures_util::{future::LocalBoxFuture, Sink, Stream};

use crate::{
    common::socket::{BoxedSocketRx, BoxedSocketTx, SocketMessage},
    request::BoxRequest,
    response::BoxResponse,
    BoxError,
};

use super::error::ClientError;

/// Client HTTP transport.
#[cfg(feature = "_common_http_client")]
pub mod http;

/// A type alias that represents a result of a call.
pub type CallResult<'a, T, Err> = LocalBoxFuture<'a, Result<T, TransportError<Err>>>;

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
                ClientError::Io(err) => ClientError::Io(err),
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
                ClientError::Io(err) => ClientError::Io(err),
                ClientError::MessageDecode(err) => ClientError::MessageDecode(err),
                ClientError::Transport(_) => unreachable!("infallible"),
            }),
        }
    }
}

/// A request that a transport can get.
pub enum TransportRequest {
    /// A unary request.
    Unary(BoxRequest),
    /// A socket request.
    Socket(BoxRequest),
}

/// A response that a transport can return.
///
/// Note: a transport MUST return the corresponding variant of response
/// for the request variant they got.
pub enum TransportResponse {
    /// A unary response.
    Unary(BoxResponse),
    /// A socket response.
    Socket {
        /// Sender part of the socket.
        tx: BoxedSocketTx,
        /// Receiver part of the socket.
        rx: BoxedSocketRx,
    },
}

impl TransportResponse {
    /// Create a new unary transport response.
    pub fn new_unary(resp: BoxResponse) -> Self {
        Self::Unary(resp)
    }

    /// Create a new socket transport response.
    pub fn new_socket(tx: BoxedSocketTx, rx: BoxedSocketRx) -> Self {
        Self::Socket { tx, rx }
    }

    /// Extracts a unary response from this transport response.
    ///
    /// # Panics
    /// - Panics if the transport response isn't a unary response.
    pub fn extract_unary(self) -> BoxResponse {
        match self {
            Self::Unary(req) => req,
            _ => panic!("expected unary response"),
        }
    }

    /// Extracts a socket response from this transport response.
    ///
    /// # Panics
    /// - Panics if the transport response isn't a socket response.
    pub fn extract_socket(self) -> (BoxedSocketTx, BoxedSocketRx) {
        match self {
            Self::Socket { tx, rx } => (tx, rx),
            _ => panic!("expected socket response"),
        }
    }
}

/// Function that helps you meet bounds for boxed socket streams and sinks.
pub fn box_socket_stream_sink<Tx, Rx>(tx: Tx, rx: Rx) -> (BoxedSocketTx, BoxedSocketRx)
where
    Tx: Sink<SocketMessage, Error = BoxError> + Send + 'static,
    Rx: Stream<Item = Result<SocketMessage, BoxError>> + Send + 'static,
{
    (Box::pin(tx), Box::pin(rx))
}
