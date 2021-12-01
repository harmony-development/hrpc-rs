//! Tracing layer for hRPC services.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, FutureExt};
use tower::{Layer, Service};
use tracing::Span;

use crate::{proto::Error as HrpcError, request::BoxRequest, response::BoxResponse};

/// Layer for layering hRPC services with [`Trace`].
#[derive(Debug, Clone)]
pub struct TraceLayer<SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn> {
    span_fn: SpanFn,
    on_request: OnRequestFn,
    on_success: OnSuccessFn,
    on_error: OnErrorFn,
}

impl<SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
    TraceLayer<SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
where
    SpanFn: Fn(&BoxRequest) -> Span + Clone,
    OnRequestFn: Fn(&BoxRequest, &Span) + Clone,
    OnSuccessFn: Fn(&BoxResponse, &Span) + Clone,
    OnErrorFn: Fn(&BoxResponse, &Span, &HrpcError) + Clone,
{
    /// Create a new trace layer.
    pub fn new(
        span_fn: SpanFn,
        on_request: OnRequestFn,
        on_success: OnSuccessFn,
        on_error: OnErrorFn,
    ) -> Self {
        Self {
            span_fn,
            on_request,
            on_success,
            on_error,
        }
    }
}

type SpanFnPtr = fn(&BoxRequest) -> Span;
type OnRequestFnPtr = fn(&BoxRequest, &Span);
type OnSuccessFnPtr = fn(&BoxRequest, &Span);
type OnErrorFnPtr = fn(&BoxRequest, &Span, &HrpcError);

impl TraceLayer<SpanFnPtr, OnRequestFnPtr, OnSuccessFnPtr, OnErrorFnPtr> {
    /// Create a trace layer with a default configuration.
    pub fn default() -> Self {
        Self {
            span_fn: |req| tracing::info_span!("request", endpoint = %req.endpoint()),
            on_request: |_, _| tracing::info!("processing request"),
            on_success: |_, _| tracing::info!("request successful"),
            on_error: |_, _, err| tracing::info!("request failed: {}", err),
        }
    }
}

impl<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn> Layer<S>
    for TraceLayer<SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
where
    SpanFn: Fn(&BoxRequest) -> Span + Clone,
    OnRequestFn: Fn(&BoxRequest, &Span) + Clone,
    OnSuccessFn: Fn(&BoxResponse, &Span) + Clone,
    OnErrorFn: Fn(&BoxResponse, &Span, &HrpcError) + Clone,
{
    type Service = Trace<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>;

    fn layer(&self, inner: S) -> Self::Service {
        Trace::new(
            inner,
            self.span_fn.clone(),
            self.on_request.clone(),
            self.on_success.clone(),
            self.on_error.clone(),
        )
    }
}

/// Service that can be used to trace request and responses.
#[derive(Debug, Clone)]
pub struct Trace<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn> {
    inner: S,
    span_fn: SpanFn,
    on_request: OnRequestFn,
    on_success: OnSuccessFn,
    on_error: OnErrorFn,
}

impl<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
    Trace<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
where
    SpanFn: Fn(&BoxRequest) -> Span,
    OnRequestFn: Fn(&BoxRequest, &Span),
    OnSuccessFn: Fn(&BoxResponse, &Span) + Clone,
    OnErrorFn: Fn(&BoxResponse, &Span, &HrpcError) + Clone,
{
    /// Create a new trace service.
    pub fn new(
        inner: S,
        span_fn: SpanFn,
        on_request: OnRequestFn,
        on_success: OnSuccessFn,
        on_error: OnErrorFn,
    ) -> Self {
        Self {
            inner,
            span_fn,
            on_request,
            on_success,
            on_error,
        }
    }
}

impl<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn> Service<BoxRequest>
    for Trace<S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
where
    S: Service<BoxRequest, Response = BoxResponse>,
    SpanFn: Fn(&BoxRequest) -> Span,
    OnRequestFn: Fn(&BoxRequest, &Span),
    OnSuccessFn: Fn(&BoxResponse, &Span) + Clone,
    OnErrorFn: Fn(&BoxResponse, &Span, &HrpcError) + Clone,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = TraceFuture<S::Future, OnSuccessFn, OnErrorFn>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        let span = (self.span_fn)(&req);

        let fut = {
            let _guard = span.enter();
            (self.on_request)(&req, &span);
            Service::call(&mut self.inner, req)
        };

        TraceFuture {
            span,
            on_error: self.on_error.clone(),
            on_success: self.on_success.clone(),
            fut,
        }
    }
}

pin_project_lite::pin_project! {
    /// Future used by [`Trace`].
    pub struct TraceFuture<Fut, OnSuccessFn, OnErrorFn> {
        #[pin]
        fut: Fut,
        span: Span,
        on_success: OnSuccessFn,
        on_error: OnErrorFn,
    }
}

impl<Fut, FutErr, OnSuccessFn, OnErrorFn> Future for TraceFuture<Fut, OnSuccessFn, OnErrorFn>
where
    Fut: Future<Output = Result<BoxResponse, FutErr>>,
    OnSuccessFn: Fn(&BoxResponse, &Span),
    OnErrorFn: Fn(&BoxResponse, &Span, &HrpcError),
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut project = self.project();
        let _guard = project.span.enter();

        let resp = futures_util::ready!(project.fut.poll_unpin(cx)?);

        if let Some(err) = resp.extensions().get::<HrpcError>() {
            (project.on_error)(&resp, project.span, err);
        } else {
            (project.on_success)(&resp, project.span);
        }

        Poll::Ready(Ok(resp))
    }
}
