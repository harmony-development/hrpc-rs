use tokio::sync::{
    mpsc::{self, UnboundedReceiver as MpscReceiver, UnboundedSender as MpscSender},
    oneshot::Sender as OneshotSender,
};

use crate::{request::BoxRequest, response::BoxResponse};

/// A mock sender.
#[derive(Clone)]
pub struct MockSender {
    pub(crate) inner: MpscSender<(BoxRequest, OneshotSender<BoxResponse>)>,
}

/// A mock receiver.
pub struct MockReceiver {
    pub(crate) inner: MpscReceiver<(BoxRequest, OneshotSender<BoxResponse>)>,
}

/// Create a new pair of mock channels.
pub fn new_mock_channels() -> (MockSender, MockReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();
    (MockSender { inner: tx }, MockReceiver { inner: rx })
}
