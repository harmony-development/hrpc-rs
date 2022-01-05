//! Layer for retrying rate limited requests.
//!
//! # Limitations
//!
//! - This layer is not supported on WASM platforms.
//! - This layer will drop all extensions that aren't cloned using the
//! `clone_extensions_fn` method on the layer. The method can be used
//! to set a function that will extract the values you want to clone from
//! the original extensions into the new extensions.

use std::{
    borrow::Cow,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;
use futures_util::{Future, FutureExt, StreamExt};
use prost::Message;
use tower::{Layer, Service};

use crate::{
    body::Body,
    client::{
        prelude::ClientError,
        transport::{SocketRequestMarker, TransportError},
    },
    common::extensions::Extensions,
    proto::{Error as HrpcError, HrpcErrorIdentifier, RetryInfo},
    request::{self, BoxRequest},
};

type CloneExtensionsFn = fn(&Extensions, &mut Extensions);

/// Layer that creates [`Backoff`] services.
#[derive(Clone)]
pub struct BackoffLayer {
    clone_exts: CloneExtensionsFn,
    max_retries: usize,
}

impl BackoffLayer {
    /// Set a function to extract extensions from a request and add it to a new request.
    ///
    /// This is needed so that user extensions can be added for new requests that
    /// are created for retry.
    pub fn clone_extensions_fn(mut self, f: CloneExtensionsFn) -> Self {
        self.clone_exts = f;
        self
    }

    /// Set max retry count.
    pub fn max_retries(mut self, num: usize) -> Self {
        self.max_retries = num;
        self
    }
}

impl<S> Layer<S> for BackoffLayer {
    type Service = Backoff<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Backoff {
            inner,
            clone_exts: self.clone_exts,
            max_retries: self.max_retries,
        }
    }
}

/// Retries ratelimited requests.
#[derive(Clone)]
pub struct Backoff<S> {
    inner: S,
    clone_exts: CloneExtensionsFn,
    max_retries: usize,
}

impl<S> Backoff<S> {
    /// Create a new backoff service by wrapping a client.
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            clone_exts: |_, _| {},
            max_retries: 5,
        }
    }

    /// Set a function to extract extensions from a request and add it to a new request.
    ///
    /// This is needed so that user extensions can be added for new requests that
    /// are created for retry.
    pub fn clone_extensions_fn(mut self, f: CloneExtensionsFn) -> Self {
        self.clone_exts = f;
        self
    }

    /// Set max retry count.
    pub fn max_retries(mut self, num: usize) -> Self {
        self.max_retries = num;
        self
    }
}

impl<S, Err> Service<BoxRequest> for Backoff<S>
where
    S: Service<BoxRequest, Error = TransportError<Err>> + Clone,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BackoffFuture<Err, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        BackoffFuture::new(self.inner.clone(), self.clone_exts, req, self.max_retries)
    }
}

struct RequestFactory {
    body: Bytes,
    extensions: Extensions,
    endpoint: Cow<'static, str>,
    clone_exts: CloneExtensionsFn,
}

impl RequestFactory {
    fn from_req(req: BoxRequest, clone_exts: CloneExtensionsFn) -> Result<Self, BoxRequest> {
        let request::Parts {
            mut body,
            endpoint,
            extensions,
        } = req.into();

        let maybe_body = body
            .next()
            .now_or_never()
            .flatten()
            .transpose()
            .ok()
            .flatten();
        let body = match maybe_body {
            Some(b) => b,
            None => {
                return Err(BoxRequest::from(request::Parts {
                    body,
                    endpoint,
                    extensions,
                }));
            }
        };

        Ok(Self {
            clone_exts,
            body,
            endpoint,
            extensions,
        })
    }

    fn make_req(&self) -> BoxRequest {
        let mut extensions = Extensions::new();
        (self.clone_exts)(&self.extensions, &mut extensions);

        if let Some(marker) = self.extensions.get::<SocketRequestMarker>().cloned() {
            extensions.insert(marker);
        }

        let parts = request::Parts {
            body: Body::full(self.body.clone()),
            endpoint: self.endpoint.clone(),
            extensions,
        };

        BoxRequest::from(parts)
    }
}

pin_project_lite::pin_project! {
    /// Future for [`Backoff`] service.
    pub struct BackoffFuture<Err, S: Service<BoxRequest, Error = TransportError<Err>>> {
        maybe_request_factory: Result<RequestFactory, BoxRequest>,
        service: S,
        max_retries: usize,
        retried: usize,
        req_fut: Option<Pin<Box<S::Future>>>,
        wait: Option<Pin<BoxedSleeper>>,
    }
}

impl<Err, S: Service<BoxRequest, Error = TransportError<Err>>> BackoffFuture<Err, S> {
    fn new(
        service: S,
        clone_exts_fn: CloneExtensionsFn,
        request: BoxRequest,
        max_retries: usize,
    ) -> Self {
        Self {
            max_retries,
            retried: 0,
            req_fut: None,
            maybe_request_factory: RequestFactory::from_req(request, clone_exts_fn),
            service,
            wait: None,
        }
    }
}

impl<Err, S: Service<BoxRequest, Error = TransportError<Err>>> Future for BackoffFuture<Err, S> {
    type Output = <<S as Service<BoxRequest>>::Future as Future>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if let Some(req_fut) = this.req_fut.as_mut().map(|pin| pin.as_mut()) {
            let resp = futures_util::ready!(req_fut.poll(cx));
            if let (
                true,
                Err(TransportError::GenericClient(ClientError::EndpointError {
                    hrpc_error, ..
                })),
            ) = ((this.retried < this.max_retries), &resp)
            {
                // if rate limited error, wait and try again
                if HrpcErrorIdentifier::ResourceExhausted.compare(&hrpc_error.identifier) {
                    // try to decode the retry info, if we can't we default to 5 seconds
                    let retry_after = RetryInfo::decode(hrpc_error.details.clone())
                        .map_or(5, |info| info.retry_after);
                    *this.wait =
                        Some(sleeper::sleep(Duration::from_secs(retry_after.into())).into());
                }
            }
            // otherwise return the result
            return Poll::Ready(resp);
        }

        // wait until ratelimit is gone
        if let Some(sleep) = this.wait.as_mut().map(|pin| pin.as_mut()) {
            futures_util::ready!(sleep.poll(cx));
        }

        match &this.maybe_request_factory {
            Ok(request_factory) => {
                // create a new request future, and increase our retried count
                let req = request_factory.make_req();
                *this.req_fut = Some(Box::pin(Service::call(&mut this.service, req)));
                *this.retried += 1;

                // wake is needed here since we want it to poll the request future
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            Err(req) => {
                Poll::Ready(Err(TransportError::GenericClient(ClientError::EndpointError {
                    endpoint: Cow::Owned(req.endpoint().to_string()),
                    hrpc_error: HrpcError::default()
                        .with_message("can't do request because no body was immediately available; this might be a bug in the backoff layer")
                        .with_identifier("hrpcrs.client.backoff-no-immediate-body"),
                })))
            }
        }
    }
}

trait Sleeper {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;
}
type BoxedSleeper = Box<dyn Sleeper + Send>;

#[cfg(feature = "tokio")]
mod sleeper {
    use super::*;

    use ::tokio::time::Sleep;

    pub(super) fn sleep(duration: Duration) -> BoxedSleeper {
        Box::new(::tokio::time::sleep(duration))
    }

    impl Sleeper for Sleep {
        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<()> {
            Future::poll(self, cx)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod sleeper {
    use super::*;

    pub(super) fn sleep(_duration: Duration) -> BoxedSleeper {
        panic!("backoff layer is not supported on WASM: can't use Sleep")
    }

    struct Sleep;

    impl Sleeper for Sleep {
        fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> std::task::Poll<()> {
            panic!("backoff layer is not supported on WASM: can't use Sleep")
        }
    }
}
