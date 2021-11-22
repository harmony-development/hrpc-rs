use futures_util::{Sink, SinkExt, Stream, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{
    tungstenite::{Error as WsError, Message as WsMessage},
    WebSocketStream,
};

use crate::{common::socket::SocketMessage, proto::Error as HrpcError, BoxError};

/// Wrapper over a [`tokio_tungstenite::WebSocketStream`] that produces
/// and takes [`SocketMessage`].
pub struct WebSocket<S> {
    inner: WebSocketStream<S>,
}

impl<S> WebSocket<S> {
    /// Create a new web socket by wrapping a [`tokio_tungstenite::WebSocketStream`].
    pub fn new(inner: WebSocketStream<S>) -> Self {
        Self { inner }
    }
}

impl<S> Stream for WebSocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Item = Result<SocketMessage, BoxError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner
            .poll_next_unpin(cx)
            .map_ok(SocketMessage::from)
            .map_err(BoxError::from)
    }
}

impl<S> Sink<SocketMessage> for WebSocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Error = BoxError;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready_unpin(cx).map_err(BoxError::from)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: SocketMessage,
    ) -> Result<(), Self::Error> {
        self.inner
            .start_send_unpin(item.into())
            .map_err(BoxError::from)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_flush_unpin(cx).map_err(BoxError::from)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_close_unpin(cx).map_err(BoxError::from)
    }
}

impl From<WsMessage> for SocketMessage {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Binary(data) => Self::Binary(data),
            WsMessage::Close(_) => Self::Close,
            WsMessage::Text(data) => Self::Text(data),
            WsMessage::Pong(data) => Self::Pong(data),
            WsMessage::Ping(data) => Self::Ping(data),
        }
    }
}

impl From<SocketMessage> for WsMessage {
    fn from(msg: SocketMessage) -> WsMessage {
        match msg {
            SocketMessage::Binary(data) => Self::Binary(data),
            SocketMessage::Close => Self::Close(None),
            SocketMessage::Text(data) => Self::Text(data),
            SocketMessage::Pong(data) => Self::Pong(data),
            SocketMessage::Ping(data) => Self::Ping(data),
        }
    }
}

impl From<WsError> for HrpcError {
    fn from(err: WsError) -> Self {
        HrpcError::default()
            .with_identifier("hrpcrs.socket-error")
            .with_message(err.to_string())
    }
}
