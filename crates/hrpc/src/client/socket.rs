use bytes::{Bytes, BytesMut};
use futures_util::{Future, SinkExt, StreamExt};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_tungstenite::tungstenite::{self, error::Error as SocketError};
use tracing::debug;

use super::error::{ClientError, ClientResult};
use crate::DecodeBodyError;

type SenderChanWithReq<Req> = (Req, oneshot::Sender<Result<(), ClientError>>);
type WebSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

// This does not implement "close-on-drop" since socket instances may be sent across threads
// by the user. This is done to prevent user mistakes.
/// A hRPC socket.
#[derive(Debug)]
pub struct Socket<Req, Resp>
where
    Req: prost::Message,
    Resp: prost::Message + Default,
{
    rx: flume::Receiver<Result<Resp, ClientError>>,
    tx: mpsc::Sender<SenderChanWithReq<Req>>,
    close_chan: mpsc::Sender<()>,
}

impl<Req, Resp> Socket<Req, Resp>
where
    Req: prost::Message + 'static,
    Resp: prost::Message + Default + 'static,
{
    pub(super) fn new(mut ws: WebSocket) -> Self {
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
                    Some(res_msg) = ws.next() => {
                        let resp = match res_msg {
                            Ok(msg) => {
                                use tungstenite::Message;

                                match msg {
                                    Message::Binary(raw) => {
                                        Resp::decode(Bytes::from(raw))
                                            .map_err(|err| ClientError::MessageDecode(DecodeBodyError::InvalidProtoMessage(err)))
                                    }
                                    Message::Close(_) => {
                                        let _ = recv_msg_tx.send_async(Err(tungstenite::Error::ConnectionClosed.into())).await;
                                        let _ = ws.close(None).await;
                                        return;
                                    },
                                    Message::Ping(data) => {
                                        let pong_res = ws
                                            .send(tungstenite::Message::Pong(data))
                                            .await;
                                        if let Err(err) = pong_res {
                                            Err(ClientError::SocketError(err))
                                        } else {
                                            continue;
                                        }
                                    },
                                    Message::Pong(_) | Message::Text(_) => continue,
                                }
                            }
                            Err(err) => {
                                let is_capped = matches!(err, tungstenite::Error::Capacity(_));
                                let res = Err(ClientError::SocketError(err));
                                if !is_capped {
                                    let _ = recv_msg_tx.send_async(res).await;
                                    let _ = ws.close(None).await;
                                    return;
                                } else {
                                    res
                                }
                            },
                        };
                        if recv_msg_tx.send_async(resp).await.is_err() {
                            let _ = ws.close(None).await;
                            return;
                        }
                    }
                    Some((req, chan)) = send_msg_rx.recv() => {
                        let req = {
                            crate::encode_protobuf_message_to(&mut buf, req);
                            buf.to_vec()
                        };

                        if let Err(e) = ws.send(tungstenite::Message::binary(req)).await {
                            debug!("socket send error: {}", e);
                            let is_capped_or_queue_full = matches!(e, tungstenite::Error::SendQueueFull(_) | tungstenite::Error::Capacity(_));
                            let _ = chan.send(Err(ClientError::SocketError(e)));
                            // Don't close socket if only the send queue is full
                            // or our message is bigger than the default max capacity
                            if !is_capped_or_queue_full {
                                let _ = ws.close(None).await;
                                return;
                            }
                        } else {
                            if chan.send(Ok(())).is_err() {
                                let _ = ws.close(None).await;
                                return;
                            }
                        }
                    }
                    // If we get *anything*, it means that either the channel is closed
                    // or we got a close message
                    _ = close_chan_rx.recv() => {
                        if let Err(err) = ws.close(None).await {
                            let _ = recv_msg_tx.send_async(Err(ClientError::SocketError(err))).await;
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
    /// ## Errors
    /// - Returns [`SocketError::ConnectionClosed`] if the socket is closed normally.
    /// - Returns [`SocketError::AlreadyClosed`] if the socket is already closed.
    ///
    /// ## Notes
    /// This will block until getting a message if the socket is not closed.
    pub async fn receive_message(&self) -> ClientResult<Resp> {
        if self.is_closed() {
            Err(ClientError::SocketError(SocketError::ConnectionClosed))
        } else {
            self.rx
                .recv_async()
                .await
                .unwrap_or(Err(ClientError::SocketError(SocketError::ConnectionClosed)))
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
    pub async fn send_message(&self, req: Req) -> ClientResult<()> {
        let (req_tx, req_rx) = oneshot::channel();
        if self.is_closed() || self.tx.send((req, req_tx)).await.is_err() {
            Err(ClientError::SocketError(SocketError::ConnectionClosed))
        } else {
            req_rx
                .await
                .unwrap_or(Err(ClientError::SocketError(SocketError::ConnectionClosed)))
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
    ) -> JoinHandle<Result<T, ClientError>>
    where
        Handler: FnOnce(Self) -> HandlerFut + 'static,
        HandlerFut: Future<Output = Result<T, ClientError>> + Send + 'static,
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
    ) -> JoinHandle<Result<(), ClientError>>
    where
        ProcessFn: for<'a> Fn(&'a Self, Resp) -> ProcessFut + Send + Sync + 'static,
        ProcessFut: Future<Output = Result<Req, ClientError>> + Send,
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
    Req: prost::Message + 'static,
    Resp: prost::Message + Default + 'static,
{
    fn clone(&self) -> Self {
        Self {
            close_chan: self.close_chan.clone(),
            rx: self.rx.clone(),
            tx: self.tx.clone(),
        }
    }
}
