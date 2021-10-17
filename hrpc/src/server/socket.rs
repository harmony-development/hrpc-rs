use std::time::Duration;

use bytes::BytesMut;
use futures_util::Future;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_tungstenite::tungstenite::Error as SocketError;
use tracing::{debug, error};

use crate::DecodeBodyError;

use super::{
    error::ServerError,
    ws::{WebSocket, WsMessage},
};

type SenderChanWithReq<Resp> = (Resp, oneshot::Sender<Result<(), ServerError>>);

// This does not implement "close-on-drop" since socket instances may be sent across threads
// by the user. This is done to prevent user mistakes.
/// A hRPC socket.
#[derive(Debug)]
pub struct Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    rx: flume::Receiver<Result<Req, ServerError>>,
    tx: mpsc::Sender<SenderChanWithReq<Resp>>,
    close_chan: mpsc::Sender<()>,
}

impl<Req, Resp> Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    pub(crate) fn new(mut ws: WebSocket) -> Self {
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
            loop {
                tokio::select! {
                    Some(res_msg) = ws.recv() => {
                        let resp = match res_msg {
                            Ok(msg) => {
                                match msg {
                                    WsMessage::Binary(data) => {
                                        Req::decode(data.as_slice())
                                            .map_err(|err| ServerError::DecodeBodyError(DecodeBodyError::InvalidProtoMessage(err)))
                                    }
                                    WsMessage::Ping(data) => {
                                        if let Err(err) = ws.send(WsMessage::Pong(data)).await {
                                            error!("error sending websocket pong: {}", err);
                                        }
                                        continue;
                                    }
                                    WsMessage::Close(_) => {
                                        let _ = recv_msg_tx.send_async(Err(SocketError::ConnectionClosed.into())).await;
                                        let _ = ws.close().await;
                                        return;
                                    },
                                    _ => continue,
                                }
                            }
                            Err(err) => {
                                let _ = recv_msg_tx.send_async(Err(err)).await;
                                let _ = ws.close().await;
                                return;
                            }
                        };
                        if recv_msg_tx.send_async(resp).await.is_err() {
                            let _ = ws.close().await;
                            return;
                        }
                    }
                    Some((resp, chan)) = send_msg_rx.recv() => {
                        let resp = {
                            crate::encode_protobuf_message_to(&mut buf, resp);
                            buf.to_vec()
                        };

                        if let Err(err) = ws.send(WsMessage::Binary(resp)).await {
                            debug!("socket send error: {}", err);
                            let is_capped_or_queue_full = matches!(err, ServerError::SocketError(SocketError::Capacity(_) | SocketError::SendQueueFull(_)));
                            if !is_capped_or_queue_full {
                                let _ = chan.send(Err(err));
                                let _ = ws.close().await;
                                return;
                            }
                        } else {
                            debug!("responded to client socket");
                            if chan.send(Ok(())).is_err() {
                                let _ = ws.close().await;
                                return;
                            }
                        }
                    }
                    // If we get *anything*, it means that either the channel is closed
                    // or we got a close message
                    _ = close_chan_rx.recv() => {
                        if let Err(err) = ws.close().await {
                            let _ = recv_msg_tx.send_async(Err(err)).await;
                        }
                        return;
                    }
                    _ = ping_interval.tick() => {
                        if let Err(err) = ws.send(WsMessage::Ping(Vec::new())).await {
                            let _ = recv_msg_tx.send_async(Err(err)).await;
                            return;
                        }
                    }
                    // TODO(yusdacra): we can just use std::hint::spin_loop() here?
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
    /// ## Errors
    /// - Returns [`SocketError::ConnectionClosed`] if the socket is closed normally.
    /// - Returns [`SocketError::AlreadyClosed`] if the socket is already closed.
    ///
    /// ## Notes
    /// This will block until getting a message if the socket is not closed.
    pub async fn receive_message(&self) -> Result<Req, ServerError> {
        if self.is_closed() {
            Err(SocketError::AlreadyClosed.into())
        } else {
            self.rx
                .recv_async()
                .await
                .unwrap_or_else(|_| Err(SocketError::ConnectionClosed.into()))
        }
    }

    /// Send a message over the socket.
    ///
    /// ## Errors
    /// - Returns [`SocketError::ConnectionClosed`] if the socket is closed normally.
    /// - Returns [`SocketError::AlreadyClosed`] if the socket is already closed.
    ///
    /// ## Notes
    /// This will block if the inner send buffer is filled.
    pub async fn send_message(&self, resp: Resp) -> Result<(), ServerError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        if self.is_closed() || self.tx.send((resp, resp_tx)).await.is_err() {
            Err(SocketError::AlreadyClosed.into())
        } else {
            resp_rx
                .await
                .unwrap_or_else(|_| Err(SocketError::ConnectionClosed.into()))
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
    ) -> JoinHandle<Result<T, ServerError>>
    where
        Handler: FnOnce(Self) -> HandlerFut + 'static,
        HandlerFut: Future<Output = Result<T, ServerError>> + Send + 'static,
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
    ) -> JoinHandle<Result<(), ServerError>>
    where
        ProcessFn: for<'a> Fn(&'a Self, Req) -> ProcessFut + Send + Sync + 'static,
        ProcessFut: Future<Output = Result<Resp, ServerError>> + Send,
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
