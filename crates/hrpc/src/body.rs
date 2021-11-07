use std::{
    fmt::{self, Debug, Formatter},
    pin::Pin,
};

use bytes::{Buf, Bytes};
use futures_util::{Stream, StreamExt};

use super::BoxError;
use crate::common::buf::BufList;

/// Type of the item that a [`Body`] produces.
pub type BodyResult = Result<Bytes, BoxError>;

/// A request or response body.
pub struct Body {
    stream: Pin<Box<dyn Stream<Item = BodyResult> + Send + Sync + 'static>>,
}

impl Body {
    /// Create a new body by wrapping a stream.
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = BodyResult> + Send + Sync + 'static,
    {
        Self {
            stream: Box::pin(stream),
        }
    }

    /// Create a new, empty body.
    pub fn empty() -> Self {
        Self::new(futures_util::stream::empty())
    }

    /// Create a body using a single chunk of data.
    pub fn full<Data>(data: Data) -> Self
    where
        Data: Into<Bytes>,
    {
        Self::new(futures_util::stream::once(futures_util::future::ready(Ok(
            data.into(),
        ))))
    }

    /// Aggregate this body into a single [`Buf`].
    ///
    /// Note that this does not do any sort of size check for the body.
    pub async fn aggregate(mut self) -> Result<impl Buf, BoxError> {
        let mut bufs = BufList::new();

        while let Some(buf) = self.next().await {
            let buf = buf?;
            if buf.has_remaining() {
                bufs.push(buf);
            }
        }

        Ok(bufs)
    }
}

impl Stream for Body {
    type Item = BodyResult;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body").field("stream", &"<hidden>").finish()
    }
}
