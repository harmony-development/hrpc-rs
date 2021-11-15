use futures_util::{Sink, Stream};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{
    tungstenite::{Error as WsError, Message as WsMessage},
    WebSocketStream,
};

use crate::{common::socket::SocketMessage, proto::Error as HrpcError};

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
    type Item = Result<SocketMessage, HrpcError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_next(cx)
            .map_ok(SocketMessage::from)
            .map_err(HrpcError::from)
    }
}

impl<S> Sink<SocketMessage> for WebSocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Error = HrpcError;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_ready(cx).map_err(HrpcError::from)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: SocketMessage,
    ) -> Result<(), Self::Error> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.start_send(item.into()).map_err(HrpcError::from)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_flush(cx).map_err(HrpcError::from)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let s = &mut self.inner;
        futures_util::pin_mut!(s);
        s.poll_close(cx).map_err(HrpcError::from)
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
