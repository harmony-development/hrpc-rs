#![allow(dead_code)]

use std::pin::Pin;

use futures_util::{Sink, Stream};

use crate::proto::Error as HrpcError;

pub(crate) type BoxedWsRx =
    Pin<Box<dyn Stream<Item = Result<SocketMessage, HrpcError>> + Send + 'static>>;
pub(crate) type BoxedWsTx = Pin<Box<dyn Sink<SocketMessage, Error = HrpcError> + Send + 'static>>;

/// Generic socket message.
#[derive(Debug)]
pub enum SocketMessage {
    /// Binary message.
    Binary(Vec<u8>),
    /// Text message.
    Text(String),
    /// Ping message.
    Ping(Vec<u8>),
    /// Pong message.
    Pong(Vec<u8>),
    /// Close message.
    Close,
}

#[derive(Debug)]
pub(crate) enum SocketError {
    Closed,
    AlreadyClosed,
}

impl From<SocketError> for HrpcError {
    fn from(err: SocketError) -> Self {
        let hrpc_err = HrpcError::default();
        // TODO: add messages probably
        match err {
            SocketError::Closed => hrpc_err.with_identifier("hrpcrs.socket-closed"),
            SocketError::AlreadyClosed => hrpc_err.with_identifier("hrpcrs.socket-already-closed"),
        }
    }
}
