use futures_util::{future::LocalBoxFuture, Sink, Stream};

use crate::{
    common::socket::{BoxedWsRx, BoxedWsTx, SocketMessage},
    proto::Error as HrpcError,
    request::BoxRequest,
    response::BoxResponse,
};

use super::error::ClientResult;

/// Client HTTP transport.
#[cfg(feature = "_common_http_client")]
pub mod http;

/// A type alias that represents a result of a call.
pub type CallResult<'a, T, TransportError> = LocalBoxFuture<'a, ClientResult<T, TransportError>>;

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
        tx: BoxedWsTx,
        /// Receiver part of the socket.
        rx: BoxedWsRx,
    },
}

impl TransportResponse {
    /// Create a new unary transport response.
    pub fn new_unary(resp: BoxResponse) -> Self {
        Self::Unary(resp)
    }

    /// Create a new socket transport response.
    pub fn new_socket(tx: BoxedWsTx, rx: BoxedWsRx) -> Self {
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
    pub fn extract_socket(self) -> (BoxedWsTx, BoxedWsRx) {
        match self {
            Self::Socket { tx, rx } => (tx, rx),
            _ => panic!("expected socket response"),
        }
    }
}

/// Function that helps you meet bounds for boxed socket streams and sinks.
pub fn box_socket_stream_sink<Tx, Rx>(tx: Tx, rx: Rx) -> (BoxedWsTx, BoxedWsRx)
where
    Tx: Sink<SocketMessage, Error = HrpcError> + Send + 'static,
    Rx: Stream<Item = Result<SocketMessage, HrpcError>> + Send + 'static,
{
    (Box::pin(tx), Box::pin(rx))
}
