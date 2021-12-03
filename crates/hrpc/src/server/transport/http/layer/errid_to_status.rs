use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use http::StatusCode;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::{proto::Error as HrpcError, request::BoxRequest, response::BoxResponse};

/// Layer for layering services with [`ErrorIdentifierToStatus`].
#[derive(Clone)]
pub struct ErrorIdentifierToStatusLayer<ToStatus> {
    to_status: ToStatus,
}

impl<ToStatus> ErrorIdentifierToStatusLayer<ToStatus>
where
    ToStatus: Fn(&str) -> Option<StatusCode>,
{
    /// Create a new layer using the provided [`ToStatus`] function.
    pub fn new(to_status: ToStatus) -> Self {
        Self { to_status }
    }
}

impl<ToStatus, S> Layer<S> for ErrorIdentifierToStatusLayer<ToStatus>
where
    ToStatus: Clone,
{
    type Service = ErrorIdentifierToStatus<ToStatus, S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorIdentifierToStatus::new(self.to_status.clone(), inner)
    }
}

/// Service to set response status from possible errors.
#[derive(Clone)]
pub struct ErrorIdentifierToStatus<ToStatus, S> {
    inner: S,
    to_status: ToStatus,
}

impl<ToStatus, S> ErrorIdentifierToStatus<ToStatus, S> {
    /// Create a new service by wrapping another service, and converting to
    /// status using the provided function.
    pub fn new(to_status: ToStatus, inner: S) -> Self {
        Self { inner, to_status }
    }
}

impl<ToStatus, S> Service<BoxRequest> for ErrorIdentifierToStatus<ToStatus, S>
where
    S: Service<BoxRequest, Response = BoxResponse>,
    ToStatus: Fn(&str) -> Option<StatusCode> + Clone,
{
    type Response = BoxResponse;

    type Error = S::Error;

    type Future = ErrorIdentifierToStatusFuture<ToStatus, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        ErrorIdentifierToStatusFuture {
            resp_fut: Service::call(&mut self.inner, req),
            to_status: self.to_status.clone(),
        }
    }
}

pin_project! {
    /// Future for [`ErrorIdentifierToStatus`].
    pub struct ErrorIdentifierToStatusFuture<ToStatus, Fut> {
        #[pin]
        resp_fut: Fut,
        to_status: ToStatus,
    }
}

impl<ToStatus, Fut, Err> Future for ErrorIdentifierToStatusFuture<ToStatus, Fut>
where
    Fut: Future<Output = Result<BoxResponse, Err>>,
    ToStatus: Fn(&str) -> Option<StatusCode>,
{
    type Output = Result<BoxResponse, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        this.resp_fut.poll_unpin(cx).map_ok(|mut resp| {
            if let Some(status) = resp
                .extensions()
                .get::<HrpcError>()
                .and_then(|err| (this.to_status)(&err.identifier))
            {
                resp.extensions_mut().insert(status);
            }
            resp
        })
    }
}
