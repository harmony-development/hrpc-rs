use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use tower::{Layer, Service};

use crate::{request::BoxRequest, response::BoxResponse};

use super::transport::{TransportRequest, TransportResponse};

/// Layer for creating [`Modify`] instances.
#[derive(Debug, Clone)]
pub struct ModifyLayer<ReqFn, RespFn> {
    req_fn: ReqFn,
    resp_fn: RespFn,
}

impl<ReqFn, RespFn> ModifyLayer<ReqFn, RespFn> {
    /// Create a new layer.
    pub fn new(req_fn: ReqFn, resp_fn: RespFn) -> Self {
        Self { req_fn, resp_fn }
    }
}

impl<ReqFn, RespFn, S> Layer<S> for ModifyLayer<ReqFn, RespFn>
where
    ReqFn: Clone,
    RespFn: Clone,
{
    type Service = Modify<ReqFn, RespFn, S>;

    fn layer(&self, inner: S) -> Self::Service {
        Modify::new(inner, self.req_fn.clone(), self.resp_fn.clone())
    }
}

/// Service that lets you modify / inspect requests and responses.
///
/// **Note:** only unary responses can be modified for responses. This is because
/// there is no response to modify with a socket response.
#[derive(Debug, Clone)]
pub struct Modify<ReqFn, RespFn, S> {
    inner: S,
    req_fn: ReqFn,
    resp_fn: RespFn,
}

impl<ReqFn, RespFn, S> Modify<ReqFn, RespFn, S> {
    /// Create a new service by wrapping a given service.
    pub fn new(inner: S, req_fn: ReqFn, resp_fn: RespFn) -> Self {
        Self {
            inner,
            req_fn,
            resp_fn,
        }
    }
}

impl<ReqFn, RespFn, S> Service<TransportRequest> for Modify<ReqFn, RespFn, S>
where
    S: Service<TransportRequest, Response = TransportResponse>,
    S::Future: Unpin,
    ReqFn: FnMut(&mut BoxRequest),
    RespFn: FnMut(&mut BoxResponse) + Clone + Unpin,
{
    type Response = TransportResponse;

    type Error = S::Error;

    type Future = ModifyFuture<S::Future, RespFn>;

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
            resp_fn: self.resp_fn.clone(),
        }
    }
}

/// Future for [`Modify`].
pub struct ModifyFuture<Fut, RespFn> {
    fut: Fut,
    resp_fn: RespFn,
}

impl<Fut, Err, RespFn> Future for ModifyFuture<Fut, RespFn>
where
    Fut: Future<Output = Result<TransportResponse, Err>> + Unpin,
    RespFn: FnMut(&mut BoxResponse) + Unpin,
{
    type Output = Result<TransportResponse, Err>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.fut.poll_unpin(cx).map_ok(|mut resp| {
            match &mut resp {
                TransportResponse::Unary(resp) => (self.resp_fn)(resp),
                TransportResponse::Socket { .. } => {}
            }
            resp
        })
    }
}
