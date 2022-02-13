use std::{error::Error as StdError, fmt::Display};

use futures_util::future::BoxFuture;
use tower::{util::BoxCloneService, Service, ServiceExt};

use crate::{box_error, request::BoxRequest, response::BoxResponse, BoxError};

use super::transport::TransportError;

/// A type erased transport. This is useful for storing transports or clients
/// and swapping them at runtime.
#[derive(Clone)]
pub struct BoxedTransport {
    inner: BoxCloneService<BoxRequest, BoxResponse, TransportError<BoxedTransportError>>,
}

impl BoxedTransport {
    /// Create a new boxed transport by wrapping any transport.
    pub fn new<Svc, SvcErr>(svc: Svc) -> Self
    where
        Svc: Service<BoxRequest, Response = BoxResponse, Error = TransportError<SvcErr>>
            + Send
            + Clone
            + 'static,
        Svc::Future: Send,
        SvcErr: StdError + Sync + Send + 'static,
    {
        let svc = svc.map_err(|err| match err {
            TransportError::GenericClient(err) => TransportError::GenericClient(err),
            TransportError::Transport(err) => {
                TransportError::Transport(BoxedTransportError::new(err))
            }
        });
        Self {
            inner: BoxCloneService::new(svc),
        }
    }
}

impl Service<BoxRequest> for BoxedTransport {
    type Response = BoxResponse;

    type Error = TransportError<BoxedTransportError>;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        Service::call(&mut self.inner, req)
    }
}

/// A type erased boxed transport error.
#[derive(Debug)]
pub struct BoxedTransportError {
    inner: BoxError,
}

impl BoxedTransportError {
    /// Create a new boxed transport error.
    pub fn new<Err>(err: Err) -> Self
    where
        Err: StdError + Send + Sync + 'static,
    {
        Self {
            inner: box_error(err),
        }
    }

    /// Extract the inner box error from this error.
    pub fn into_box_error(self) -> BoxError {
        self.inner
    }
}

impl Display for BoxedTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl StdError for BoxedTransportError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.inner.as_ref())
    }
}
