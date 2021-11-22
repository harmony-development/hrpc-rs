use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Sink, SinkExt, Stream, StreamExt};
use ws_stream_wasm::{WsErr, WsMessage, WsStream};

use crate::{common::socket::SocketMessage, proto::Error as HrpcError};

/// Type that wraps a [`WsStream`] and implements [`Sink`] and [`Stream`]
/// for working with [`SocketMessage`]s.
pub struct WebSocket {
    stream: WsStream,
}

impl WebSocket {
    /// Create a new websocket from a stream.
    pub fn new(stream: WsStream) -> Self {
        Self { stream }
    }
}

impl Stream for WebSocket {
    type Item = Result<SocketMessage, HrpcError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream
            .poll_next_unpin(cx)
            .map(|a| a.map(SocketMessage::from).map(Ok))
    }
}

impl Sink<SocketMessage> for WebSocket {
    type Error = HrpcError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.stream.poll_ready_unpin(cx).map_err(HrpcError::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: SocketMessage) -> Result<(), Self::Error> {
        self.stream
            .start_send_unpin(WsMessage::try_from(item)?)
            .map_err(HrpcError::from)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.stream.poll_flush_unpin(cx).map_err(HrpcError::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.stream.poll_close_unpin(cx).map_err(HrpcError::from)
    }
}

impl TryFrom<SocketMessage> for WsMessage {
    type Error = HrpcError;

    fn try_from(msg: SocketMessage) -> Result<Self, Self::Error> {
        match msg {
            SocketMessage::Binary(data) => Ok(WsMessage::Binary(data)),
            SocketMessage::Text(data) => Ok(WsMessage::Text(data)),
            msg => Err((
                "hrpcrs.socket-error.ws-wasm",
                format!("invalid socket message passed: {:?}", msg),
            )
                .into()),
        }
    }
}

impl From<WsMessage> for SocketMessage {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Binary(data) => SocketMessage::Binary(data),
            WsMessage::Text(data) => SocketMessage::Text(data),
        }
    }
}

impl From<WsErr> for HrpcError {
    fn from(err: WsErr) -> Self {
        HrpcError::default()
            .with_identifier("hrpcrs.socket-error")
            .with_message(err.to_string())
    }
}
