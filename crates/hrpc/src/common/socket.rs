#![allow(dead_code)]

use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    pin::Pin,
    sync::atomic::AtomicUsize,
    task::{Context, Poll},
};

use futures_util::{lock::BiLock, Future, Sink, SinkExt, Stream, StreamExt};

use bytes::BytesMut;
use prost::Message as PbMsg;

use crate::{decode::DecodeBodyError, proto::Error as HrpcError, BoxError};

pub(crate) enum DecodeResult<Msg> {
    Msg(Msg),
    Error(HrpcError),
}

type EncodeMessageFn<Msg> = fn(&mut BytesMut, &Msg) -> Vec<u8>;
type DecodeMessageFn<Msg> = fn(Vec<u8>) -> Result<DecodeResult<Msg>, DecodeBodyError>;

/// A boxed stream that yields socket messages.
pub type BoxedSocketRx =
    Pin<Box<dyn Stream<Item = Result<SocketMessage, BoxError>> + Send + 'static>>;
/// A boxed sink that accepts socket messages.
pub type BoxedSocketTx = Pin<Box<dyn Sink<SocketMessage, Error = BoxError> + Send + 'static>>;

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

/// Errors that can occur during a socket's operation.
#[derive(Debug)]
pub enum SocketError {
    /// Currently only occurs if a server sends an error to a client.
    /// This means that this will never occur on a server, only on clients.
    Protocol(HrpcError),
    /// Occurs if a message failed to decode.
    MessageDecode(DecodeBodyError),
    /// Occurs if the underlying transport for this socket yields an error.
    Transport(BoxError),
}

impl From<SocketError> for HrpcError {
    fn from(err: SocketError) -> Self {
        match err {
            SocketError::MessageDecode(err) => err.into(),
            SocketError::Protocol(err) => err,
            SocketError::Transport(err) => ("hrpcrs.socket-error", err).into(),
        }
    }
}

impl Display for SocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SocketError::MessageDecode(err) => write!(f, "message decode error: {}", err),
            SocketError::Protocol(err) => write!(f, "{}", err),
            SocketError::Transport(err) => write!(f, "transport error: {}", err),
        }
    }
}

impl StdError for SocketError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SocketError::MessageDecode(err) => Some(err),
            SocketError::Protocol(err) => Some(err),
            SocketError::Transport(err) => Some(err.as_ref()),
        }
    }
}

/// Error used when socket halfs could not be combined.
#[derive(Debug)]
#[non_exhaustive]
pub struct CombineError;

impl Display for CombineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("cannot combine sockets, they aren't split from the same socket!")
    }
}

impl StdError for CombineError {}

/// A hRPC socket.
///
/// Sockets only handle pings if you call `.receive_message()` continously.
/// See the respective socket transports used for more information on their behaviour.
/// The socket transports are located in [`crate::common::transport`].
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
        static SOCKET_ID: AtomicUsize = AtomicUsize::new(0);

        let socket_id = SOCKET_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (first_tx, second_tx) = BiLock::new(tx);
        Self {
            write: WriteSocket::new(first_tx, encode_message, socket_id),
            read: ReadSocket::new(rx, second_tx, decode_message, socket_id),
        }
    }

    /// Receive a message from the socket.
    ///
    /// ## Notes
    /// - This will block until getting a message (unless an error occurs).
    /// - This will handle ping messages.
    #[inline]
    pub async fn receive_message(&mut self) -> Result<Resp, SocketError> {
        self.read.receive_message().await
    }

    /// Send a message over the socket.
    #[inline]
    pub async fn send_message(&mut self, req: Req) -> Result<(), SocketError> {
        self.write.send_message(req).await
    }

    /// Send a ping over the socket.
    #[inline]
    pub async fn ping(&mut self) -> Result<(), SocketError> {
        self.write.ping().await
    }

    /// Close the socket.
    #[inline]
    pub async fn close(self) -> Result<(), SocketError> {
        self.write.close().await
    }

    /// Split this socket into the write half and the read half.
    pub fn split(self) -> (WriteSocket<Req>, ReadSocket<Resp>) {
        (self.write, self.read)
    }

    /// Combine a write and read half back to a socket.
    ///
    /// # Errors
    /// - Returns [`CombineError`] if the socket halfs aren't split from the same socket.
    pub fn combine(write: WriteSocket<Req>, read: ReadSocket<Resp>) -> Result<Self, CombineError> {
        (write.socket_id == read.socket_id)
            .then(|| Self { write, read })
            .ok_or(CombineError)
    }
}

/// Read half of a socket.
#[must_use = "read sockets do nothing unless you use `.receive_message()`"]
pub struct ReadSocket<Resp> {
    rx: BoxedSocketRx,
    tx: BiLock<BoxedSocketTx>,
    decode_message: DecodeMessageFn<Resp>,
    socket_id: usize,
}

impl<Resp: PbMsg + Default> ReadSocket<Resp> {
    pub(crate) fn new(
        rx: BoxedSocketRx,
        tx: BiLock<BoxedSocketTx>,
        decode_message: DecodeMessageFn<Resp>,
        socket_id: usize,
    ) -> Self {
        Self {
            rx,
            tx,
            decode_message,
            socket_id,
        }
    }

    /// Receive a message from the socket.
    ///
    /// ## Notes
    /// - This will block until getting a message (unless an error occurs).
    /// - This will handle ping messages.
    pub async fn receive_message(&mut self) -> Result<Resp, SocketError> {
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
                        .await
                        .map_err(SocketError::Transport)?;
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
    type Output = Result<RecvMsg<Resp>, SocketError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let item = futures_util::ready!(self.rx.poll_next_unpin(cx));
        match item {
            Some(res) => match res {
                Ok(msg) => match msg {
                    SocketMessage::Binary(data) => Poll::Ready(match (self.decode_message)(data) {
                        Ok(res) => match res {
                            DecodeResult::Msg(resp) => Ok(RecvMsg::Msg(resp)),
                            DecodeResult::Error(err) => Err(SocketError::Protocol(err)),
                        },
                        Err(err) => Err(SocketError::MessageDecode(err)),
                    }),
                    SocketMessage::Ping(data) => Poll::Ready(Ok(RecvMsg::Ping(data))),
                    SocketMessage::Close => {
                        Poll::Ready(Err(SocketError::Transport(Box::new(HrpcError::from((
                            "hrpcrs.socket-error",
                            "socket is closed by the other end",
                        ))))))
                    }
                    _ => Poll::Pending,
                },
                Err(err) => Poll::Ready(Err(SocketError::Transport(err))),
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
    socket_id: usize,
}

impl<Req: PbMsg> WriteSocket<Req> {
    pub(crate) fn new(
        tx: BiLock<BoxedSocketTx>,
        encode_message: EncodeMessageFn<Req>,
        socket_id: usize,
    ) -> Self {
        Self {
            tx,
            buf: BytesMut::new(),
            encode_message,
            socket_id,
        }
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, req: Req) -> Result<(), SocketError> {
        let data = (self.encode_message)(&mut self.buf, &req);
        self.tx
            .lock()
            .await
            .send(SocketMessage::Binary(data))
            .await
            .map_err(SocketError::Transport)
    }

    /// Close the socket.
    pub async fn close(self) -> Result<(), SocketError> {
        self.tx
            .lock()
            .await
            .send(SocketMessage::Close)
            .await
            .map_err(SocketError::Transport)
    }

    /// Send a ping over the socket.
    pub async fn ping(&mut self) -> Result<(), SocketError> {
        self.tx
            .lock()
            .await
            .send(SocketMessage::Ping(Vec::new()))
            .await
            .map_err(SocketError::Transport)
    }
}
