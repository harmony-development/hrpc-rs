use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::{request::BoxRequest, response::BoxResponse};

/// Layer for creating [`Modify`] instances.
#[derive(Clone)]
pub struct ModifyLayer<ModifyReq, ModifyResp> {
    req_fn: ModifyReq,
    resp_fn: ModifyResp,
}

impl<ModifyReq, ModifyResp> ModifyLayer<ModifyReq, ModifyResp> {
    /// Create a new layer.
    pub fn new(req_fn: ModifyReq, resp_fn: ModifyResp) -> Self {
        Self { req_fn, resp_fn }
    }
}

impl<ModifyReq> ModifyLayer<ModifyReq, fn(&mut BoxResponse)> {
    /// Create a new layer that only modifies requests.
    pub fn new_request(f: ModifyReq) -> Self {
        Self::new(f, |_| ())
    }
}

impl<ModifyResp> ModifyLayer<fn(&mut BoxRequest), ModifyResp> {
    /// Create a new layer that only modifies responses.
    pub fn new_response(f: ModifyResp) -> Self {
        Self::new(|_| (), f)
    }
}

impl<ModifyReq, ModifyResp, S> Layer<S> for ModifyLayer<ModifyReq, ModifyResp>
where
    ModifyReq: Clone,
    ModifyResp: Clone,
{
    type Service = Modify<ModifyReq, ModifyResp, S>;

    fn layer(&self, inner: S) -> Self::Service {
        Modify::new(inner, self.req_fn.clone(), self.resp_fn.clone())
    }
}

/// Service that lets you modify / inspect requests and responses.
#[derive(Clone)]
pub struct Modify<ModifyReq, ModifyResp, S> {
    inner: S,
    req_fn: ModifyReq,
    resp_fn: ModifyResp,
}

impl<ModifyReq, ModifyResp, S> Modify<ModifyReq, ModifyResp, S> {
    /// Create a new service by wrapping a given service.
    pub fn new(inner: S, req_fn: ModifyReq, resp_fn: ModifyResp) -> Self {
        Self {
            inner,
            req_fn,
            resp_fn,
        }
    }
}

impl<ModifyReq, ModifyResp, S> Service<BoxRequest> for Modify<ModifyReq, ModifyResp, S>
where
    S: Service<BoxRequest, Response = BoxResponse>,
    ModifyReq: Fn(&mut BoxRequest),
    ModifyResp: Fn(&mut BoxResponse) + Clone,
{
    type Response = BoxResponse;

    type Error = S::Error;

    type Future = ModifyFuture<ModifyResp, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, mut req: BoxRequest) -> Self::Future {
        (self.req_fn)(&mut req);

        ModifyFuture {
            fut: Service::call(&mut self.inner, req),
            resp_fn: self.resp_fn.clone(),
        }
    }
}

pin_project! {
    /// Future for [`Modify`].
    pub struct ModifyFuture<ModifyResp, Fut> {
        #[pin]
        fut: Fut,
        resp_fn: ModifyResp,
    }
}

impl<ModifyResp, Fut, Err> Future for ModifyFuture<ModifyResp, Fut>
where
    Fut: Future<Output = Result<BoxResponse, Err>>,
    ModifyResp: Fn(&mut BoxResponse),
{
    type Output = Result<BoxResponse, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        this.fut.poll_unpin(cx).map_ok(|mut resp| {
            (this.resp_fn)(&mut resp);
            resp
        })
    }
}
