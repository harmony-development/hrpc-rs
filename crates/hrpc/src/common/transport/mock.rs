use futures_channel::{
    mpsc::{self, UnboundedReceiver as MpscReceiver, UnboundedSender as MpscSender},
    oneshot::Sender as OneshotSender,
};

use crate::client::transport::{TransportRequest, TransportResponse};

/// A mock sender.
#[derive(Clone)]
pub struct MockSender {
    pub(crate) inner: MpscSender<(TransportRequest, OneshotSender<TransportResponse>)>,
}

/// A mock receiver.
pub struct MockReceiver {
    pub(crate) inner: MpscReceiver<(TransportRequest, OneshotSender<TransportResponse>)>,
}

/// Create a new pair of mock channels.
pub fn new_mock_channels() -> (MockSender, MockReceiver) {
    let (tx, rx) = mpsc::unbounded();
    (MockSender { inner: tx }, MockReceiver { inner: rx })
}
