use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::{request::BoxRequest, response::BoxResponse};

use super::transport::{TransportRequest, TransportResponse};

/// Function to modify a request.
pub type ModifyReq = fn(&mut BoxRequest);
/// Function to modify a response.
pub type ModifyResp = fn(&mut BoxResponse);

/// Layer for creating [`Modify`] instances.
/// Please see it's documentation for more information and limitations.
#[derive(Clone)]
pub struct ModifyLayer {
    req_fn: ModifyReq,
    resp_fn: ModifyResp,
}

impl ModifyLayer {
    /// Create a new layer.
    pub fn new(req_fn: ModifyReq, resp_fn: ModifyResp) -> Self {
        Self { req_fn, resp_fn }
    }

    /// Create a new layer that only modifies requests.
    pub fn new_request(f: ModifyReq) -> Self {
        Self::new(f, |_| ())
    }

    /// Create a new layer that only modifies responses.
    pub fn new_response(f: ModifyResp) -> Self {
        Self::new(|_| (), f)
    }
}

impl<S> Layer<S> for ModifyLayer {
    type Service = Modify<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Modify::new(inner, self.req_fn, self.resp_fn)
    }
}

/// Service that lets you modify / inspect requests and responses.
///
/// **Note:** only unary responses can be modified for responses. This is because
/// there is no response to modify with a socket response.
#[derive(Clone)]
pub struct Modify<S> {
    inner: S,
    req_fn: ModifyReq,
    resp_fn: ModifyResp,
}

impl<S> Modify<S> {
    /// Create a new service by wrapping a given service.
    pub fn new(inner: S, req_fn: ModifyReq, resp_fn: ModifyResp) -> Self {
        Self {
            inner,
            req_fn,
            resp_fn,
        }
    }
}

impl<S> Service<TransportRequest> for Modify<S>
where
    S: Service<TransportRequest, Response = TransportResponse>,
{
    type Response = TransportResponse;

    type Error = S::Error;

    type Future = ModifyFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, mut req: TransportRequest) -> Self::Future {
        match &mut req {
            TransportRequest::Socket(req) => (self.req_fn)(req),
            TransportRequest::Unary(req) => (self.req_fn)(req),
        }

        ModifyFuture {
            fut: Service::call(&mut self.inner, req),
            resp_fn: self.resp_fn,
        }
    }
}

pin_project! {
    /// Future for [`Modify`].
    pub struct ModifyFuture<Fut> {
        #[pin]
        fut: Fut,
        resp_fn: ModifyResp,
    }
}

impl<Fut, Err> Future for ModifyFuture<Fut>
where
    Fut: Future<Output = Result<TransportResponse, Err>>,
{
    type Output = Result<TransportResponse, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        this.fut.poll_unpin(cx).map_ok(|mut resp| {
            match &mut resp {
                TransportResponse::Unary(resp) => (this.resp_fn)(resp),
                TransportResponse::Socket { .. } => {}
            }
            resp
        })
    }
}
