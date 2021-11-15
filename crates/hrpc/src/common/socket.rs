#![allow(dead_code)]

use std::{hint::spin_loop, marker::PhantomData, pin::Pin};

use futures_util::{FutureExt, Sink, SinkExt, Stream, StreamExt};

use bytes::BytesMut;
use prost::Message as PbMsg;

use crate::proto::Error as HrpcError;

pub(crate) type BoxedWsRx =
    Pin<Box<dyn Stream<Item = Result<SocketMessage, HrpcError>> + Send + 'static>>;
pub(crate) type BoxedWsTx = Pin<Box<dyn Sink<SocketMessage, Error = HrpcError> + Send + 'static>>;
type PingTx = Pin<Box<dyn Sink<Vec<u8>, Error = HrpcError> + Send + 'static>>;
type PingRx = Pin<Box<dyn Stream<Item = Result<Vec<u8>, HrpcError>> + Send + 'static>>;

type EncodeMessageFn<Msg> = fn(&mut BytesMut, &Msg) -> Vec<u8>;
type DecodeMessageFn<Msg> = fn(Vec<u8>) -> Result<Msg, HrpcError>;

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
        match err {
            SocketError::Closed => hrpc_err
                .with_identifier("hrpcrs.socket-closed")
                .with_message("socket was closed"),
            SocketError::AlreadyClosed => hrpc_err
                .with_identifier("hrpcrs.socket-already-closed")
                .with_message("socket was already closed"),
        }
    }
}

/// A hRPC socket.
#[must_use = "sockets do nothing unless you use `.send_message(msg)` or `.receive_message()`"]
pub struct Socket<Req, Resp> {
    pub(crate) write: WriteSocket<Req>,
    pub(crate) read: ReadSocket<Resp>,
}

impl<Req, Resp> Socket<Req, Resp>
where
    Req: PbMsg,
    Resp: PbMsg + Default,
{
    pub(crate) fn new(
        ws_rx: BoxedWsRx,
        ws_tx: BoxedWsTx,
        encode_message: EncodeMessageFn<Req>,
        decode_message: DecodeMessageFn<Resp>,
    ) -> Self {
        Self {
            write: WriteSocket::new(ws_tx, encode_message),
            read: ReadSocket::new(ws_rx, decode_message),
        }
    }

    /// Receive a message from the socket.
    ///
    /// ## Notes
    /// - This will block until getting a message (unless an error occurs).
    /// - This handles responding to pings. You should call this even if you
    /// aren't going to receive any messages.
    pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
        self.read.receive_message().await
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, req: Req) -> Result<(), HrpcError> {
        self.write.send_message(req).await
    }

    /// Close the socket.
    pub async fn close(self) -> Result<(), HrpcError> {
        self.write.close().await
    }

    /// Split this socket into the write half and the read half.
    pub fn split(self) -> (WriteSocket<Req>, ReadSocket<Resp>) {
        (self.write, self.read)
    }
}

/// Read half of a socket.
#[must_use = "read sockets do nothing unless you use `.receive_message()`"]
pub struct ReadSocket<Resp> {
    ws_rx: BoxedWsRx,
    _resp: PhantomData<Resp>,
    decode_message: DecodeMessageFn<Resp>,
    ping_tx: Option<PingTx>,
}

impl<Resp: PbMsg + Default> ReadSocket<Resp> {
    pub(crate) fn new(ws_rx: BoxedWsRx, decode_message: DecodeMessageFn<Resp>) -> Self {
        Self {
            ws_rx,
            _resp: PhantomData,
            decode_message,
            ping_tx: None,
        }
    }

    /// Set ping sink to allow ping messages to be sent to wherever the sink points to.
    pub fn set_ping_sink<S>(mut self, ping_sink: S) -> Self
    where
        S: Sink<Vec<u8>, Error = HrpcError> + Send + 'static,
    {
        self.ping_tx = Some(Box::pin(ping_sink));
        self
    }

    /// Receive a message from the socket.
    ///
    /// ## Notes
    /// - This will block until getting a message (unless an error occurs).
    /// - This will send ping messages over the `ping sink` if one is set (see `ReadSocket::set_ping_sink`).
    pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
        loop {
            futures_util::select_biased! {
                maybe_res = self.ws_rx.next().fuse() => if let Some(res) = maybe_res {
                    match res? {
                        SocketMessage::Binary(data) => break (self.decode_message)(data),
                        SocketMessage::Ping(data) => if let Some(ping_tx) = &mut self.ping_tx {
                            ping_tx.send(data).await?;
                        },
                        SocketMessage::Close => break Err(SocketError::Closed.into()),
                        _ => spin_loop(),
                    }
                },
                default => spin_loop(),
            }
        }
    }
}

/// Write half of a socket.
#[must_use = "write sockets do nothing unless you use `.send_message(msg)`"]
pub struct WriteSocket<Req> {
    pub(crate) ws_tx: BoxedWsTx,
    pub(crate) buf: BytesMut,
    _req: PhantomData<Req>,
    encode_message: EncodeMessageFn<Req>,
    ping_rx: Option<PingRx>,
}

impl<Req: PbMsg> WriteSocket<Req> {
    pub(crate) fn new(ws_tx: BoxedWsTx, encode_message: EncodeMessageFn<Req>) -> Self {
        Self {
            ws_tx,
            buf: BytesMut::new(),
            _req: PhantomData,
            encode_message,
            ping_rx: None,
        }
    }

    /// Set ping stream to allow ping messages to be received from wherever the stream points to.
    pub fn set_ping_stream<S>(mut self, ping_stream: S) -> Self
    where
        S: Stream<Item = Result<Vec<u8>, HrpcError>> + Send + 'static,
    {
        self.ping_rx = Some(Box::pin(ping_stream));
        self
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, req: Req) -> Result<(), HrpcError> {
        let data = (self.encode_message)(&mut self.buf, &req);
        self.ws_tx.send(SocketMessage::Binary(data)).await
    }

    /// Close the socket.
    pub async fn close(mut self) -> Result<(), HrpcError> {
        self.ws_tx.send(SocketMessage::Close).await
    }

    /// Send a ping over the socket.
    pub async fn ping(&mut self) -> Result<(), HrpcError> {
        self.ws_tx.send(SocketMessage::Ping(Vec::new())).await
    }

    /// Handle pings received from the ping stream (if one is set).
    pub async fn handle_pings(&mut self) -> Result<(), HrpcError> {
        if let Some(ping_rx) = &mut self.ping_rx {
            while let Some(data) = ping_rx.next().await.transpose()? {
                self.ws_tx.send(SocketMessage::Pong(data)).await?;
            }
        }

        Ok(())
    }
}
