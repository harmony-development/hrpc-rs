use std::time::Duration;

use bytes::BytesMut;
use futures_util::{future::BoxFuture, Future, Sink, SinkExt, Stream, StreamExt};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::{debug, error};

use super::error::{HrpcError, ServerResult};
use crate::{
    common::socket::{BoxedWsRx, BoxedWsTx, SocketError, SocketMessage},
    decode::DecodeBodyError,
};

type SenderChanWithReq<Resp> = (Resp, oneshot::Sender<ServerResult<()>>);

// This does not implement "close-on-drop" since socket instances may be sent across threads
// by the user. This is done to prevent user mistakes.
/// A hRPC socket.
#[derive(Debug)]
pub struct Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    rx: flume::Receiver<ServerResult<Req>>,
    tx: mpsc::Sender<SenderChanWithReq<Resp>>,
    close_chan: mpsc::Sender<()>,
}

impl<Req, Resp> Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    pub(crate) fn new<WsRx, WsTx>(ws_rx: WsRx, ws_tx: WsTx) -> Self
    where
        WsRx: Stream<Item = ServerResult<SocketMessage>> + Send + 'static,
        WsTx: Sink<SocketMessage, Error = HrpcError> + Send + 'static,
    {
        let (recv_msg_tx, recv_msg_rx) = flume::bounded(64);
        let (send_msg_tx, mut send_msg_rx): (
            mpsc::Sender<SenderChanWithReq<Resp>>,
            mpsc::Receiver<SenderChanWithReq<Resp>>,
        ) = mpsc::channel(64);
        let (close_chan_tx, mut close_chan_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let mut buf = BytesMut::new();
            // Send a ping every 5 seconds
            let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
            futures_util::pin_mut!(ws_rx);
            futures_util::pin_mut!(ws_tx);
            loop {
                tokio::select! {
                    Some(res_msg) = ws_rx.next() => {
                        let resp = match res_msg {
                            Ok(msg) => {
                                match msg {
                                    SocketMessage::Binary(data) => {
                                        Req::decode(data.as_slice())
                                            .map_err(|err| HrpcError::from(DecodeBodyError::InvalidProtoMessage(err)))
                                    }
                                    SocketMessage::Ping(data) => {
                                        if let Err(err) = ws_tx.send(SocketMessage::Pong(data)).await {
                                            error!("error sending websocket pong: {}", err);
                                        }
                                        continue;
                                    }
                                    SocketMessage::Close => {
                                        let _ = recv_msg_tx.send_async(Err(HrpcError::from(SocketError::Closed))).await;
                                        let _ = ws_tx.send(SocketMessage::Close).await;
                                        return;
                                    },
                                    _ => continue,
                                }
                            }
                            Err(err) => {
                                let _ = recv_msg_tx.send_async(Err(err)).await;
                                let _ = ws_tx.send(SocketMessage::Close).await;
                                return;
                            }
                        };
                        if recv_msg_tx.send_async(resp).await.is_err() {
                            let _ = ws_tx.send(SocketMessage::Close).await;
                            return;
                        }
                    }
                    Some((resp, chan)) = send_msg_rx.recv() => {
                        let resp = {
                            crate::encode::encode_protobuf_message_to(&mut buf, &resp);
                            // TODO: don't allocate here?
                            buf.to_vec()
                        };

                        if let Err(err) = ws_tx.send(SocketMessage::Binary(resp)).await {
                            debug!("socket send error: {}", err);
                            let _ = chan.send(Err(err));
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
                    _ = ping_interval.tick() => {
                        if let Err(err) = ws_tx.send(SocketMessage::Ping(Vec::new())).await {
                            let _ = recv_msg_tx.send_async(Err(err)).await;
                            return;
                        }
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
    pub async fn receive_message(&self) -> ServerResult<Req> {
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
    /// - This will block if the inner send buffer is filled.
    pub async fn send_message(&self, resp: Resp) -> ServerResult<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        if self.is_closed() || self.tx.send((resp, resp_tx)).await.is_err() {
            Err(SocketError::AlreadyClosed.into())
        } else {
            resp_rx
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
    pub fn spawn_task<T, Handler, HandlerFut>(&self, f: Handler) -> JoinHandle<ServerResult<T>>
    where
        Handler: FnOnce(Self) -> HandlerFut + 'static,
        HandlerFut: Future<Output = ServerResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        let sock = self.clone();
        let fut = f(sock);
        tokio::spawn(fut)
    }

    /// Spawns a parallel task that processes request messages and produces
    /// response messages.
    pub fn spawn_process_task<ProcessFn, ProcessFut>(
        &self,
        f: ProcessFn,
    ) -> JoinHandle<ServerResult<()>>
    where
        ProcessFn: for<'a> Fn(&'a Self, Req) -> ProcessFut + Send + Sync + 'static,
        ProcessFut: Future<Output = ServerResult<Resp>> + Send,
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

impl<Req, Resp> Clone for Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    fn clone(&self) -> Self {
        Self {
            close_chan: self.close_chan.clone(),
            rx: self.rx.clone(),
            tx: self.tx.clone(),
        }
    }
}

pub(crate) struct SocketHandler {
    #[allow(dead_code)]
    pub(crate) inner:
        Box<dyn FnOnce(BoxedWsRx, BoxedWsTx) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
}
