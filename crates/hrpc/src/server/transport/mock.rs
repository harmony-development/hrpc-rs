use std::convert::Infallible;

use futures_util::{future::BoxFuture, SinkExt, StreamExt};
use tower::Service;

use crate::{
    box_error,
    client::transport::SocketChannels,
    common::{socket::SocketMessage, transport::mock::MockReceiver},
    response::BoxResponse,
    server::{socket::SocketHandler, MakeRoutes},
};

use super::Transport;

/// Mock transport for the server.
pub struct Mock {
    rx: MockReceiver,
}

impl Mock {
    /// Create a new mock transport.
    pub fn new(rx: MockReceiver) -> Self {
        Self { rx }
    }
}

impl Transport for Mock {
    type Error = Infallible;

    fn serve<S>(mut self, mk_routes: S) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        S: MakeRoutes,
    {
        let mut svc = mk_routes
            .into_make_service()
            .call(())
            .into_inner()
            .unwrap()
            .unwrap();

        Box::pin(async move {
            while let Some((req, sender)) = self.rx.inner.recv().await {
                let fut = Service::call(&mut svc, req);

                tokio::spawn(async move {
                    let mut resp = fut.await.expect("cant fail");

                    if let Some(socket_handler) = resp.extensions_mut().remove::<SocketHandler>() {
                        let (client_tx, server_rx) =
                            futures_channel::mpsc::unbounded::<SocketMessage>();
                        let (server_tx, client_rx) = futures_channel::mpsc::unbounded();

                        {
                            let client_socket_chans = SocketChannels::new(
                                Box::pin(client_tx.sink_map_err(box_error)),
                                Box::pin(client_rx.map(Ok)),
                            );

                            let mut resp = BoxResponse::empty();
                            resp.extensions_mut().insert(client_socket_chans);

                            sender.send(resp).expect("sender dropped");
                        }

                        let fut = (socket_handler.inner)(
                            Box::pin(server_rx.map(Ok)),
                            Box::pin(server_tx.sink_map_err(box_error)),
                        );

                        tokio::spawn(fut);
                    } else {
                        sender.send(resp).expect("sender dropped");
                    }
                });
            }

            Ok(())
        })
    }
}
