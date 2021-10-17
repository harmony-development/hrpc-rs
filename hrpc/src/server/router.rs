use std::convert::Infallible;

use super::handler::{not_found, CallFuture, Handler};
use crate::{HttpRequest, HttpResponse};

use matchit::Node as Matcher;
use tower::{Layer, Service};

/// Builder type for inserting [`Handler`]s before building a [`RoutesFinalized`].
pub struct Routes {
    handlers: Vec<(String, Handler)>,
    any: Option<Handler>,
}

impl Default for Routes {
    fn default() -> Self {
        Self::new()
    }
}

impl Routes {
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

    pub fn combine_with(mut self, other_router_builder: impl Into<Routes>) -> Self {
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

    pub fn build(self) -> RoutesFinalized {
        let mut matcher = Matcher::new();

        for (path, handler) in self.handlers {
            matcher.insert(path, handler).expect("invalid route path");
        }

        RoutesFinalized {
            matcher,
            any: self.any.unwrap_or_else(not_found),
        }
    }
}

/// Finalized [`Routes`], ready for serving as a [`Service`].
pub struct RoutesFinalized {
    matcher: Matcher<Handler>,
    any: Handler,
}

impl Service<HttpRequest> for RoutesFinalized {
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
