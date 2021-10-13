use futures_util::future::BoxFuture;
use http::StatusCode;
use matchit::Node as Matcher;
use std::{convert::Infallible, future};
use tower::{service_fn, util::BoxService, Layer, Service};

use crate::{HttpRequest, HttpResponse};

use super::prelude::CustomError;

/// Call future used by Handler.
pub type CallFuture = BoxFuture<'static, Result<HttpResponse, Infallible>>;
pub type BoxedHandlerService = BoxService<HttpRequest, HttpResponse, Infallible>;
pub type BoxedMakeRouterService<T> = BoxService<T, BoxedHandlerService, Infallible>;

pub fn not_found() -> Handler {
    Handler::new(service_fn(|_| {
        future::ready(Ok((StatusCode::NOT_FOUND, "not found").as_error_response()))
    }))
}

/// hRPC handler service type.
pub struct Handler {
    svc: BoxService<HttpRequest, HttpResponse, Infallible>,
}

impl Handler {
    pub fn new<S>(svc: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        Self {
            svc: BoxService::new(svc),
        }
    }

    pub fn layer<L, S>(self, layer: L) -> Self
    where
        L: Layer<Self, Service = S>,
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        Self {
            svc: BoxService::new(layer.layer(self)),
        }
    }
}

impl Service<HttpRequest> for Handler {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = CallFuture;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        Service::call(&mut self.svc, req)
    }
}

pub struct RouterBuilder {
    handlers: Vec<(String, Handler)>,
    any: Option<Handler>,
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            any: None,
        }
    }

    pub fn route<S>(mut self, path: impl Into<String>, handler: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        self.handlers.push((path.into(), Handler::new(handler)));
        self
    }

    pub fn layer<L, S>(mut self, layer: L) -> Self
    where
        L: Layer<Handler, Service = S>,
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        for (path, handler) in self.handlers.drain(..).collect::<Vec<_>>() {
            let layered = handler.layer(&layer);
            self.handlers.push((path, layered));
        }
        self
    }

    pub fn combine_with(mut self, other_router_builder: impl Into<RouterBuilder>) -> Self {
        let mut orb = other_router_builder.into();
        self.handlers.append(&mut orb.handlers);
        self
    }

    pub fn any<S>(mut self, handler: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        self.any = Some(Handler::new(handler));
        self
    }

    pub fn build(self) -> Router {
        let mut matcher = Matcher::new();

        for (path, handler) in self.handlers {
            matcher.insert(path, handler).expect("invalid route path");
        }

        Router {
            matcher,
            any: self.any.unwrap_or_else(not_found),
        }
    }
}

pub struct Router {
    matcher: Matcher<Handler>,
    any: Handler,
}

impl Service<HttpRequest> for Router {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = CallFuture;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        let path = req.uri().path();
        if let Ok(matched) = self.matcher.at_mut(path) {
            Service::call(matched.value, req)
        } else {
            Service::call(&mut self.any, req)
        }
    }
}

pub trait MakeRouter: Send + 'static {
    fn make_router(&self) -> RouterBuilder;

    fn combine_with<Other>(self, other: Other) -> MakeRouterStack<Other, Self>
    where
        Other: MakeRouter,
        Self: Sized,
    {
        MakeRouterStack {
            outer: other,
            inner: self,
        }
    }

    fn into_make_service(self) -> IntoMakeService<Self>
    where
        Self: Sized,
    {
        IntoMakeService { mk_router: self }
    }

    fn layer<L>(self, layer: L) -> LayeredMakeRouter<L, Self>
    where
        L: Layer<Handler, Service = BoxedHandlerService> + Send + 'static,
        Self: Sized,
    {
        LayeredMakeRouter { inner: self, layer }
    }
}

pub struct LayeredMakeRouter<L, MkRouter>
where
    L: Layer<Handler, Service = BoxedHandlerService> + Send + 'static,
    MkRouter: MakeRouter,
{
    inner: MkRouter,
    layer: L,
}

impl<L, MkRouter> MakeRouter for LayeredMakeRouter<L, MkRouter>
where
    L: Layer<Handler, Service = BoxedHandlerService> + Send + 'static,
    MkRouter: MakeRouter,
{
    fn make_router(&self) -> RouterBuilder {
        let rb = MkRouter::make_router(&self.inner);
        rb.layer(&self.layer)
    }
}

pub struct MakeRouterStack<Outer, Inner>
where
    Outer: MakeRouter,
    Inner: MakeRouter,
{
    outer: Outer,
    inner: Inner,
}

impl<Outer, Inner> MakeRouter for MakeRouterStack<Outer, Inner>
where
    Outer: MakeRouter,
    Inner: MakeRouter,
{
    fn make_router(&self) -> RouterBuilder {
        let outer_rb = MakeRouter::make_router(&self.outer);
        let inner_rb = MakeRouter::make_router(&self.inner);
        outer_rb.combine_with(inner_rb)
    }
}

pub struct IntoMakeService<MkRouter: MakeRouter> {
    mk_router: MkRouter,
}

impl<T, MkRouter: MakeRouter> Service<T> for IntoMakeService<MkRouter> {
    type Response = Router;

    type Error = Infallible;

    type Future = future::Ready<Result<Router, Infallible>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _req: T) -> Self::Future {
        let router = MakeRouter::make_router(&self.mk_router).build();
        future::ready(Ok(router))
    }
}
