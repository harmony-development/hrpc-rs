#![allow(dead_code)]

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{lock::BiLock, Future, Sink, SinkExt, Stream, StreamExt};

use bytes::BytesMut;
use prost::Message as PbMsg;

use crate::proto::Error as HrpcError;

type EncodeMessageFn<Msg> = fn(&mut BytesMut, &Msg) -> Vec<u8>;
type DecodeMessageFn<Msg> = fn(Vec<u8>) -> Result<Msg, HrpcError>;

/// A boxed stream that yields socket messages.
pub type BoxedSocketRx =
    Pin<Box<dyn Stream<Item = Result<SocketMessage, HrpcError>> + Send + 'static>>;
/// A boxed sink that accepts socket messages.
pub type BoxedSocketTx = Pin<Box<dyn Sink<SocketMessage, Error = HrpcError> + Send + 'static>>;

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
        rx: BoxedSocketRx,
        tx: BoxedSocketTx,
        encode_message: EncodeMessageFn<Req>,
        decode_message: DecodeMessageFn<Resp>,
    ) -> Self {
        let (first_tx, second_tx) = BiLock::new(tx);
        Self {
            write: WriteSocket::new(first_tx, encode_message),
            read: ReadSocket::new(rx, second_tx, decode_message),
        }
    }

    /// Receive a message from the socket.
    ///
    /// ## Notes
    /// - This will block until getting a message (unless an error occurs).
    /// - This will handle ping messages.
    #[inline]
    pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
        self.read.receive_message().await
    }

    /// Send a message over the socket.
    #[inline]
    pub async fn send_message(&mut self, req: Req) -> Result<(), HrpcError> {
        self.write.send_message(req).await
    }

    /// Send a ping over the socket.
    #[inline]
    pub async fn ping(&mut self) -> Result<(), HrpcError> {
        self.write.ping().await
    }

    /// Close the socket.
    #[inline]
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
    rx: BoxedSocketRx,
    tx: BiLock<BoxedSocketTx>,
    decode_message: DecodeMessageFn<Resp>,
}

impl<Resp: PbMsg + Default> ReadSocket<Resp> {
    pub(crate) fn new(
        rx: BoxedSocketRx,
        tx: BiLock<BoxedSocketTx>,
        decode_message: DecodeMessageFn<Resp>,
    ) -> Self {
        Self {
            rx,
            tx,
            decode_message,
        }
    }

    /// Receive a message from the socket.
    ///
    /// ## Notes
    /// - This will block until getting a message (unless an error occurs).
    /// - This will handle ping messages.
    pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
        loop {
            let msg_fut = ReceiveMessageFuture {
                rx: &mut self.rx,
                decode_message: self.decode_message,
            };
            match msg_fut.await? {
                RecvMsg::Msg(resp) => break Ok(resp),
                RecvMsg::Ping(ping_data) => {
                    self.tx
                        .lock()
                        .await
                        .send(SocketMessage::Pong(ping_data))
                        .await?;
                }
            }
        }
    }
}

enum RecvMsg<Resp> {
    Ping(Vec<u8>),
    Msg(Resp),
}

struct ReceiveMessageFuture<'a, Resp> {
    rx: &'a mut BoxedSocketRx,
    decode_message: DecodeMessageFn<Resp>,
}

impl<'a, Resp> Future for ReceiveMessageFuture<'a, Resp> {
    type Output = Result<RecvMsg<Resp>, HrpcError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = futures_util::ready!(self.rx.poll_next_unpin(cx));
        match item {
            Some(res) => match res {
                Ok(msg) => match msg {
                    SocketMessage::Binary(data) => {
                        Poll::Ready((self.decode_message)(data).map(RecvMsg::Msg))
                    }
                    SocketMessage::Ping(data) => Poll::Ready(Ok(RecvMsg::Ping(data))),
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
    pub(crate) tx: BiLock<BoxedSocketTx>,
    pub(crate) buf: BytesMut,
    encode_message: EncodeMessageFn<Req>,
}

impl<Req: PbMsg> WriteSocket<Req> {
    pub(crate) fn new(tx: BiLock<BoxedSocketTx>, encode_message: EncodeMessageFn<Req>) -> Self {
        Self {
            tx,
            buf: BytesMut::new(),
            encode_message,
        }
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, req: Req) -> Result<(), HrpcError> {
        let data = (self.encode_message)(&mut self.buf, &req);
        self.tx.lock().await.send(SocketMessage::Binary(data)).await
    }

    /// Close the socket.
    pub async fn close(self) -> Result<(), HrpcError> {
        self.tx.lock().await.send(SocketMessage::Close).await
    }

    /// Send a ping over the socket.
    pub async fn ping(&mut self) -> Result<(), HrpcError> {
        self.tx
            .lock()
            .await
            .send(SocketMessage::Ping(Vec::new()))
            .await
    }
}
