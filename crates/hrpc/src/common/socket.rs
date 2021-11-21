#![allow(dead_code)]

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{ready, Future, Sink, SinkExt, Stream, StreamExt};

use bytes::BytesMut;
use prost::Message as PbMsg;

use crate::proto::Error as HrpcError;

type PingTx = Pin<Box<dyn Sink<Vec<u8>, Error = HrpcError> + Send + 'static>>;
type PingRx = Pin<Box<dyn Stream<Item = Result<Vec<u8>, HrpcError>> + Send + 'static>>;

type EncodeMessageFn<Msg> = fn(&mut BytesMut, &Msg) -> Vec<u8>;
type DecodeMessageFn<Msg> = fn(Vec<u8>) -> Result<Msg, HrpcError>;

/// A boxed stream that yields socket messages.
pub type BoxedWsRx = Pin<Box<dyn Stream<Item = Result<SocketMessage, HrpcError>> + Send + 'static>>;
/// A boxed sink that accepts socket messages.
pub type BoxedWsTx = Pin<Box<dyn Sink<SocketMessage, Error = HrpcError> + Send + 'static>>;

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
///
/// Sockets by default **do not** handle pings. You must handle pings manually.
/// See [`ReadSocket`]'s `set_ping_sink` and [`WriteSocket`]'s `set_ping_stream`
/// methods. After setting them, you can use the [`WriteSocket`]'s `handle_pings`
/// method to handle them. Note that all of the methods mentioned are also
/// available on this type.
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
    /// - This will send ping messages over the `ping sink` if one is set (see `ReadSocket::set_ping_sink`).
    pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
        self.read.receive_message().await
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, req: Req) -> Result<(), HrpcError> {
        self.write.send_message(req).await
    }

    /// Set ping sink to allow ping messages to be sent to wherever the sink points to.
    pub fn set_ping_sink<S>(mut self, ping_sink: S) -> Self
    where
        S: Sink<Vec<u8>, Error = HrpcError> + Send + 'static,
    {
        self.read.ping_tx = Some(Box::pin(ping_sink));
        self
    }

    /// Set ping stream to allow ping messages to be received from wherever the stream points to.
    pub fn set_ping_stream<S>(mut self, ping_stream: S) -> Self
    where
        S: Stream<Item = Result<Vec<u8>, HrpcError>> + Send + 'static,
    {
        self.write.ping_rx = Some(Box::pin(ping_stream));
        self
    }

    /// Handle pings received from the ping stream (if one is set).
    pub async fn handle_pings(&mut self) -> Result<(), HrpcError> {
        self.write.handle_pings().await
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
    decode_message: DecodeMessageFn<Resp>,
    ping_tx: Option<PingTx>,
}

impl<Resp: PbMsg + Default> ReadSocket<Resp> {
    pub(crate) fn new(ws_rx: BoxedWsRx, decode_message: DecodeMessageFn<Resp>) -> Self {
        Self {
            ws_rx,
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
    pub fn receive_message(&mut self) -> impl Future<Output = Result<Resp, HrpcError>> + '_ {
        ReceiveMessageFuture {
            stream: &mut self.ws_rx,
            ping_tx: &mut self.ping_tx,
            decode_message: self.decode_message,
        }
    }
}

struct ReceiveMessageFuture<'a, Resp> {
    stream: &'a mut BoxedWsRx,
    ping_tx: &'a mut Option<PingTx>,
    decode_message: DecodeMessageFn<Resp>,
}

impl<'a, Resp> Future for ReceiveMessageFuture<'a, Resp> {
    type Output = Result<Resp, HrpcError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = futures_util::ready!(self.stream.poll_next_unpin(cx));
        match item {
            Some(res) => match res {
                Ok(msg) => match msg {
                    SocketMessage::Binary(data) => Poll::Ready((self.decode_message)(data)),
                    SocketMessage::Ping(data) => {
                        if let Some(ping_tx) = &mut self.ping_tx {
                            let res = ready!(ping_tx.poll_ready_unpin(cx));
                            match res {
                                Ok(_) => ping_tx
                                    .start_send_unpin(data)
                                    .map_or_else(|err| Poll::Ready(Err(err)), |_| Poll::Pending),
                                Err(err) => Poll::Ready(Err(err)),
                            }
                        } else {
                            Poll::Pending
                        }
                    }
                    SocketMessage::Close => Poll::Ready(Err(SocketError::Closed.into())),
                    _ => Poll::Pending,
                },
                Err(err) => Poll::Ready(Err(err)),
            },
            None => Poll::Pending,
        }
    }
}

/// Write half of a socket.
#[must_use = "write sockets do nothing unless you use `.send_message(msg)`"]
pub struct WriteSocket<Req> {
    pub(crate) ws_tx: BoxedWsTx,
    pub(crate) buf: BytesMut,
    encode_message: EncodeMessageFn<Req>,
    ping_rx: Option<PingRx>,
}

impl<Req: PbMsg> WriteSocket<Req> {
    pub(crate) fn new(ws_tx: BoxedWsTx, encode_message: EncodeMessageFn<Req>) -> Self {
        Self {
            ws_tx,
            buf: BytesMut::new(),
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
