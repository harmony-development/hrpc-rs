#![allow(dead_code)]

use std::pin::Pin;

use futures_util::{Sink, Stream};

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

pub use internal::*;

/// Socket types that have automatic ping handling.
#[cfg(not(feature = "disable_socket_auto_ping"))]
mod internal {
    use bytes::{Bytes, BytesMut};
    use futures_util::{Future, SinkExt, StreamExt};
    use tokio::{
        sync::{mpsc, oneshot},
        task::JoinHandle,
    };
    use tracing::debug;

    use super::*;

    type SenderChanWithReq<Req> = (Req, oneshot::Sender<Result<(), HrpcError>>);

    // This does not implement "close-on-drop" since socket instances may be sent across threads
    // by the user. This is done to prevent user mistakes.
    /// A hRPC socket.
    #[derive(Debug)]
    pub struct Socket<Req, Resp> {
        rx: flume::Receiver<Result<Resp, HrpcError>>,
        tx: mpsc::Sender<SenderChanWithReq<Req>>,
        close_chan: mpsc::Sender<()>,
    }

    impl<Req, Resp> Socket<Req, Resp>
    where
        Req: prost::Message + 'static,
        Resp: prost::Message + Default + 'static,
    {
        #[allow(dead_code)]
        pub(crate) fn new(mut ws_rx: BoxedWsRx, mut ws_tx: BoxedWsTx) -> Self {
            let (recv_msg_tx, recv_msg_rx) = flume::bounded(64);
            let (send_msg_tx, mut send_msg_rx): (
                mpsc::Sender<SenderChanWithReq<Req>>,
                mpsc::Receiver<SenderChanWithReq<Req>>,
            ) = mpsc::channel(64);
            let (close_chan_tx, mut close_chan_rx) = mpsc::channel(1);
            tokio::spawn(async move {
                let mut buf = BytesMut::new();
                loop {
                    tokio::select! {
                        Some(res_msg) = ws_rx.next() => {
                            let resp = match res_msg {
                                Ok(msg) => {
                                    match msg {
                                        SocketMessage::Binary(raw) => {
                                            Resp::decode(Bytes::from(raw))
                                                .map_err(|err| DecodeBodyError::InvalidProtoMessage(err).into())
                                        }
                                        SocketMessage::Close => {
                                            let _ = recv_msg_tx.send_async(Err(SocketError::Closed.into())).await;
                                            let _ = ws_tx.send(SocketMessage::Close).await;
                                            return;
                                        },
                                        SocketMessage::Ping(data) => {
                                            let pong_res = ws_tx
                                                .send(SocketMessage::Pong(data))
                                                .await;
                                            if let Err(err) = pong_res {
                                                Err(err)
                                            } else {
                                                continue;
                                            }
                                        },
                                        SocketMessage::Pong(_) | SocketMessage::Text(_) => continue,
                                    }
                                }
                                Err(err) => {
                                    let _ = recv_msg_tx.send_async(Err(err)).await;
                                    let _ = ws_tx.send(SocketMessage::Close).await;
                                    return;
                                },
                            };
                            if recv_msg_tx.send_async(resp).await.is_err() {
                                let _ = ws_tx.send(SocketMessage::Close).await;
                                return;
                            }
                        }
                        Some((req, chan)) = send_msg_rx.recv() => {
                            let req = {
                                crate::encode::encode_protobuf_message_to(&mut buf, &req);
                                // TODO: don't allocate here?
                                buf.to_vec()
                            };

                            if let Err(e) = ws_tx.send(SocketMessage::Binary(req)).await {
                                debug!("socket send error: {}", e);
                                let _ = ws_tx.send(SocketMessage::Close).await;
                                return;
                            } else if chan.send(Ok(())).is_err() {
                                let _ = ws_tx.send(SocketMessage::Close).await;
                                return;
                            }
                        }
                        // If we get *anything*, it means that either the channel is closed
                        // or we got a close message
                        _ = close_chan_rx.recv() => {
                            if let Err(err) = ws_tx.send(SocketMessage::Close).await {
                                let _ = recv_msg_tx.send_async(Err(err)).await;
                            }
                            return;
                        }
                        else => tokio::task::yield_now().await,
                    }
                }
            });

            Self {
                rx: recv_msg_rx,
                tx: send_msg_tx,
                close_chan: close_chan_tx,
            }
        }

        /// Receive a message from the socket.
        ///
        /// ## Notes
        /// - This will block until getting a message if the socket is not closed.
        /// - Cloning a [`Socket`] will NOT make you able to receive a message on all of the sockets.
        /// You will only receive a message on one of the sockets.
        pub async fn receive_message(&self) -> Result<Resp, HrpcError> {
            if self.is_closed() {
                Err(SocketError::AlreadyClosed.into())
            } else {
                self.rx
                    .recv_async()
                    .await
                    .unwrap_or_else(|_| Err(SocketError::Closed.into()))
            }
        }

        /// Send a message over the socket.
        ///
        /// ## Notes
        /// This will block if the inner send buffer is filled.
        pub async fn send_message(&self, req: Req) -> Result<(), HrpcError> {
            let (req_tx, req_rx) = oneshot::channel();
            if self.is_closed() || self.tx.send((req, req_tx)).await.is_err() {
                Err(SocketError::AlreadyClosed.into())
            } else {
                req_rx
                    .await
                    .unwrap_or_else(|_| Err(SocketError::Closed.into()))
            }
        }

        /// Return whether the socket is closed or not.
        pub fn is_closed(&self) -> bool {
            self.close_chan.is_closed()
        }

        /// Close the socket.
        pub async fn close(&self) {
            // We don't care about the error, it's closed either way
            let _ = self.close_chan.send(()).await;
        }

        /// Spawns a parallel task that processes a socket.
        pub fn spawn_task<T, Handler, HandlerFut>(
            &self,
            f: Handler,
        ) -> JoinHandle<Result<T, HrpcError>>
        where
            Handler: FnOnce(Self) -> HandlerFut + 'static,
            HandlerFut: Future<Output = Result<T, HrpcError>> + Send + 'static,
            T: Send + 'static,
        {
            let sock = self.clone();
            let fut = f(sock);
            tokio::spawn(fut)
        }

        /// Spawns a parallel task that processes response messages and produces
        /// request messages.
        pub fn spawn_process_task<ProcessFn, ProcessFut>(
            &self,
            f: ProcessFn,
        ) -> JoinHandle<Result<(), HrpcError>>
        where
            ProcessFn: for<'a> Fn(&'a Self, Resp) -> ProcessFut + Send + Sync + 'static,
            ProcessFut: Future<Output = Result<Req, HrpcError>> + Send,
        {
            let sock = self.clone();
            tokio::spawn(async move {
                loop {
                    let req = sock.receive_message().await?;
                    let resp = f(&sock, req).await?;
                    sock.send_message(resp).await?;
                }
            })
        }
    }

    impl<Req, Resp> Clone for Socket<Req, Resp> {
        fn clone(&self) -> Self {
            Self {
                close_chan: self.close_chan.clone(),
                rx: self.rx.clone(),
                tx: self.tx.clone(),
            }
        }
    }
}

/// Socket types that don't have automatic ping handling.
#[cfg(feature = "disable_socket_auto_ping")]
mod internal {
    use std::marker::PhantomData;

    use bytes::BytesMut;
    use futures_util::{SinkExt, StreamExt};
    use tokio::sync::mpsc::{self, error::TrySendError, Receiver, Sender};

    use super::*;

    /// A hRPC socket.
    pub struct Socket<Req, Resp> {
        write: WriteSocket<Req>,
        read: ReadSocket<Resp>,
    }

    impl<Req, Resp> Socket<Req, Resp>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        pub(crate) fn new(ws_rx: BoxedWsRx, ws_tx: BoxedWsTx) -> Self {
            let (tx, rx) = mpsc::channel(16);
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
    pub struct ReadSocket<Resp> {
        ws_rx: BoxedWsRx,
        tx_chan: Sender<SocketMessage>,
        _resp: PhantomData<Resp>,
    }

    impl<Resp: prost::Message + Default> ReadSocket<Resp> {
        pub(crate) fn new(ws_rx: BoxedWsRx, tx_chan: Sender<SocketMessage>) -> Self {
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
            fn handle_sock_res<T>(res: Result<(), TrySendError<T>>) -> Result<bool, HrpcError> {
                match res {
                    Ok(_) => Ok(false),
                    Err(err) => match err {
                        TrySendError::Closed(_) => {
                            Err(("hrpcrs.socket-error", "sender half dropped").into())
                        }
                        TrySendError::Full(_) => Ok(true),
                    },
                }
            }

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
                                if handle_sock_res(self.tx_chan.try_send(SocketMessage::Pong(data)))? {
                                    tracing::debug!("sender message queue is full, can't send pings (are you forgetting to call handle_pings on sender?)");
                                }
                                continue;
                            }
                            SocketMessage::Close => {
                                if handle_sock_res(self.tx_chan.try_send(SocketMessage::Close))? {
                                    tracing::debug!("sender message queue is full, can't send close (are you forgetting to call handle_pings on sender?)");
                                }
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
    pub struct WriteSocket<Req> {
        ws_tx: BoxedWsTx,
        buf: BytesMut,
        rx_chan: Receiver<SocketMessage>,
        _req: PhantomData<Req>,
    }

    impl<Req: prost::Message> WriteSocket<Req> {
        pub(crate) fn new(ws_tx: BoxedWsTx, rx_chan: Receiver<SocketMessage>) -> Self {
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
