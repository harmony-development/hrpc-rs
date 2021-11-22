use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use http::StatusCode;
use tower::{Layer, Service};

use crate::{proto::Error as HrpcError, request::BoxRequest, response::BoxResponse};

/// Layer for layering services with [`ErrorIdentifierToStatus`].
pub struct ErrorIdentifierToStatusLayer {
    to_status: ToStatus,
}

impl ErrorIdentifierToStatusLayer {
    /// Create a new layer using the provided [`ToStatus`] function.
    pub fn new(to_status: ToStatus) -> Self {
        Self { to_status }
    }
}

impl<S> Layer<S> for ErrorIdentifierToStatusLayer {
    type Service = ErrorIdentifierToStatus<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorIdentifierToStatus::new(self.to_status, inner)
    }
}

/// Type alias for a function that converts an error identifier to a [`StatusCode`].
///
/// Used in [`ErrorIdentifierToStatus`].
pub type ToStatus = fn(&str) -> Option<StatusCode>;

/// Service to set response status from possible errors.
pub struct ErrorIdentifierToStatus<S> {
    inner: S,
    to_status: ToStatus,
}

impl<S> ErrorIdentifierToStatus<S> {
    /// Create a new service by wrapping another service, and converting to
    /// status using the provided function.
    pub fn new(to_status: ToStatus, inner: S) -> Self {
        Self { inner, to_status }
    }
}

impl<S> Service<BoxRequest> for ErrorIdentifierToStatus<S>
where
    S: Service<BoxRequest, Response = BoxResponse, Error = HrpcError>,
    S::Future: Unpin,
{
    type Response = BoxResponse;

    type Error = HrpcError;

    type Future = ErrorIdentifierToStatusFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        ErrorIdentifierToStatusFuture {
            resp_fut: Service::call(&mut self.inner, req),
            to_status: self.to_status,
        }
    }
}

/// Future for [`ErrorIdentifierToStatus`].
pub struct ErrorIdentifierToStatusFuture<Fut> {
    resp_fut: Fut,
    to_status: ToStatus,
}

impl<Fut> Future for ErrorIdentifierToStatusFuture<Fut>
where
    Fut: Future<Output = Result<BoxResponse, HrpcError>> + Unpin,
{
    type Output = Result<BoxResponse, HrpcError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.resp_fut.poll_unpin(cx).map_ok(|mut resp| {
            if let Some(status) = resp
                .extensions()
                .get::<HrpcError>()
                .and_then(|err| (self.to_status)(&err.identifier))
            {
                resp.extensions_mut().insert(status);
            }
            resp
        })
    }
}
