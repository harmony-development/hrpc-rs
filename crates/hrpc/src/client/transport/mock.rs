use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    task::Poll,
};

use futures_util::{Future, FutureExt};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver};
use tower::Service;

use crate::common::transport::mock::MockSender;

use super::{TransportRequest, TransportResponse};

/// A client that uses a channel to send requests to a (possibly)
/// mock server.
#[derive(Clone)]
pub struct Mock {
    tx: MockSender,
}

impl Mock {
    /// Create a new mock client.
    pub fn new(tx: MockSender) -> Self {
        Self { tx }
    }
}

impl Service<TransportRequest> for Mock {
    type Response = TransportResponse;

    type Error = MockError;

    type Future = MockCallFuture;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: TransportRequest) -> Self::Future {
        let (resp_tx, resp_rx) = oneshot::channel();
        let send_res = self
            .tx
            .inner
            .send((req, resp_tx))
            .map_err(|err| MockError::Send(err.0 .0));

        match send_res {
            Ok(_) => MockCallFuture {
                inner: MockCallFutureInner::Recv(resp_rx),
            },
            Err(err) => MockCallFuture {
                inner: MockCallFutureInner::Err(Some(err)),
            },
        }
    }
}

enum MockCallFutureInner {
    Recv(OneshotReceiver<TransportResponse>),
    Err(Option<MockError>),
}

/// Future used by [`Mock`].
pub struct MockCallFuture {
    inner: MockCallFutureInner,
}

impl Future for MockCallFuture {
    type Output = Result<TransportResponse, MockError>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().inner {
            MockCallFutureInner::Err(err) => {
                Poll::Ready(Err(err.take().expect("future polled after completion")))
            }
            MockCallFutureInner::Recv(rx) => rx.poll_unpin(cx).map_err(|_| MockError::Receive),
        }
    }
}

/// Errors this client can return.
#[derive(Debug)]
pub enum MockError {
    /// Occurs if receiving a response fails. Only happens if sender end of the
    /// channel is dropped.
    Receive,
    /// Occurs if sending a request fails. Only happens if receiver end of the
    /// channel is dropped.
    Send(TransportRequest),
}

impl Display for MockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MockError::Receive => f.write_str("failed to receive response"),
            MockError::Send(_) => f.write_str("failed to send request"),
        }
    }
}

impl StdError for MockError {}
