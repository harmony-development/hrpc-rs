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

    /// Change the span function that will be used.
    pub fn span_fn<NewSpanFn>(
        self,
        span_fn: NewSpanFn,
    ) -> TraceLayer<NewSpanFn, OnRequestFn, OnSuccessFn, OnErrorFn>
    where
        NewSpanFn: Fn(&BoxRequest) -> Span + Clone,
    {
        TraceLayer {
            span_fn,
            on_error: self.on_error,
            on_request: self.on_request,
            on_success: self.on_success,
        }
    }

    /// Change the on request function that will be used.
    pub fn on_request<NewOnRequestFn>(
        self,
        on_request: NewOnRequestFn,
    ) -> TraceLayer<SpanFn, NewOnRequestFn, OnSuccessFn, OnErrorFn>
    where
        NewOnRequestFn: Fn(&BoxRequest, &Span) + Clone,
    {
        TraceLayer {
            span_fn: self.span_fn,
            on_error: self.on_error,
            on_request,
            on_success: self.on_success,
        }
    }

    /// Change the on success function that will be used.
    pub fn on_success<NewOnSuccessFn>(
        self,
        on_success: NewOnSuccessFn,
    ) -> TraceLayer<SpanFn, OnRequestFn, NewOnSuccessFn, OnErrorFn>
    where
        NewOnSuccessFn: Fn(&BoxResponse, &Span) + Clone,
    {
        TraceLayer {
            span_fn: self.span_fn,
            on_error: self.on_error,
            on_request: self.on_request,
            on_success,
        }
    }

    /// Change the on error function that will be used.
    pub fn on_error<NewOnErrorFn>(
        self,
        on_error: NewOnErrorFn,
    ) -> TraceLayer<SpanFn, OnRequestFn, OnSuccessFn, NewOnErrorFn>
    where
        NewOnErrorFn: Fn(&BoxResponse, &Span, &HrpcError) + Clone,
    {
        TraceLayer {
            span_fn: self.span_fn,
            on_error,
            on_request: self.on_request,
            on_success: self.on_success,
        }
    }
}

type SpanFnPtr = fn(&BoxRequest) -> Span;
type OnRequestFnPtr = fn(&BoxRequest, &Span);
type OnSuccessFnPtr = fn(&BoxResponse, &Span);
type OnErrorFnPtr = fn(&BoxResponse, &Span, &HrpcError);

impl TraceLayer<SpanFnPtr, OnRequestFnPtr, OnSuccessFnPtr, OnErrorFnPtr> {
    /// Create a trace layer with a default configuration that logs on the info
    /// level of `tracing`. Errors are logged with error level.
    pub fn default() -> Self {
        Self {
            span_fn: |req| tracing::info_span!("request", endpoint = %req.endpoint()),
            on_request: |_, _| tracing::info!("processing request"),
            on_success: |_, _| tracing::info!("request successful"),
            on_error: |_, _, err| tracing::error!("request failed: {}", err),
        }
    }

    /// Create a trace layer with a default configuration that logs on the debug
    /// level of `tracing`. Errors are logged with error level.
    pub fn default_debug() -> Self {
        Self {
            span_fn: |req| tracing::debug_span!("request", endpoint = %req.endpoint()),
            on_request: |_, _| tracing::debug!("processing request"),
            on_success: |_, _| tracing::debug!("request successful"),
            on_error: |_, _, err| tracing::error!("request failed: {}", err),
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
