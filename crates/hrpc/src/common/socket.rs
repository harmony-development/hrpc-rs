#![allow(dead_code)]

use std::marker::PhantomData;
use std::pin::Pin;

use bytes::BytesMut;
use futures_util::{Sink, Stream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::{decode::DecodeBodyError, proto::Error as HrpcError};

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

pub use manual_ping::*;

// TODO: implement this
/// Socket types that have automatic ping handling.
pub mod auto_ping {
    // empty
}

/// Socket types that don't have automatic ping handling.
pub mod manual_ping {
    use super::*;
    /// A hRPC socket.
    pub struct Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        write: WriteSocket<Req>,
        read: ReadSocket<Resp>,
    }

    impl<Req, Resp> Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        pub(crate) fn new(ws_rx: BoxedWsRx, ws_tx: BoxedWsTx) -> Self {
            let (tx, rx) = unbounded_channel();
            Self {
                write: WriteSocket::new(ws_tx, rx),
                read: ReadSocket::new(ws_rx, tx),
            }
        }

        /// Receive a message from the socket.
        ///
        /// ## Notes
        /// - This will block until getting a message (unless an error occurs).
        /// - This handles responding to pings. You should call this even if you
        /// aren't going to receive any messages.
        pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
            loop {
                tokio::select! {
                    res = self.read.receive_message() => {
                        return res;
                    }
                    Err(err) = self.write.handle_pings() => {
                        return Err(err);
                    }
                    else => tokio::task::yield_now().await,
                }
            }
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
    pub struct ReadSocket<Resp: prost::Message + Default> {
        ws_rx: BoxedWsRx,
        tx_chan: UnboundedSender<SocketMessage>,
        _resp: PhantomData<Resp>,
    }

    impl<Resp: prost::Message + Default> ReadSocket<Resp> {
        pub(crate) fn new(ws_rx: BoxedWsRx, tx_chan: UnboundedSender<SocketMessage>) -> Self {
            Self {
                ws_rx,
                tx_chan,
                _resp: PhantomData,
            }
        }

        /// Receive a message from the socket.
        ///
        /// ## Notes
        /// - This will block until getting a message (unless an error occurs).
        /// - This handles responding to pings. You should call this even if you
        /// aren't going to receive any messages.
        pub async fn receive_message(&mut self) -> Result<Resp, HrpcError> {
            loop {
                tokio::select! {
                    biased;
                    Some(res) = self.ws_rx.next() => {
                        match res? {
                            SocketMessage::Binary(data) => {
                                return Resp::decode(data.as_slice()).map_err(|err| {
                                    HrpcError::from(DecodeBodyError::InvalidProtoMessage(err))
                                });
                            }
                            SocketMessage::Ping(data) => {
                                self.tx_chan.send(SocketMessage::Pong(data))
                                    .map_err(|_| ("hrpcrs.socket-error", "sender half dropped"))?;
                                continue;
                            }
                            SocketMessage::Close => {
                                self.tx_chan.send(SocketMessage::Close)
                                    .map_err(|_| ("hrpcrs.socket-error", "sender half dropped"))?;
                                return Err(SocketError::Closed.into());
                            }
                            _ => tokio::task::yield_now().await,
                        }
                    }
                    else => tokio::task::yield_now().await,
                }
            }
        }
    }

    /// Write half of a socket.
    pub struct WriteSocket<Req: prost::Message> {
        ws_tx: BoxedWsTx,
        buf: BytesMut,
        rx_chan: UnboundedReceiver<SocketMessage>,
        _req: PhantomData<Req>,
    }

    impl<Req: prost::Message> WriteSocket<Req> {
        pub(crate) fn new(ws_tx: BoxedWsTx, rx_chan: UnboundedReceiver<SocketMessage>) -> Self {
            Self {
                ws_tx,
                buf: BytesMut::new(),
                rx_chan,
                _req: PhantomData,
            }
        }

        /// Send a message over the socket.
        pub async fn send_message(&mut self, req: Req) -> Result<(), HrpcError> {
            let req = {
                crate::encode::encode_protobuf_message_to(&mut self.buf, &req);
                // TODO: don't allocate here?
                self.buf.to_vec()
            };

            self.ws_tx.send(SocketMessage::Binary(req)).await
        }

        /// Close the socket.
        pub async fn close(mut self) -> Result<(), HrpcError> {
            self.ws_tx.send(SocketMessage::Close).await
        }

        /// Handle pings. This should be continously called.
        pub async fn handle_pings(&mut self) -> Result<(), HrpcError> {
            match self.rx_chan.recv().await {
                Some(msg) => self.ws_tx.send(msg).await,
                None => Err(("hrpcrs.socket-error", "receiver half dropped").into()),
            }
        }
    }
}
