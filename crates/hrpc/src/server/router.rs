use std::{borrow::Cow, convert::Infallible};

use super::{
    service::{not_found, CallFuture, HrpcService},
    HrpcLayer,
};
use crate::{HttpRequest, HttpResponse};

use matchit::Node as Matcher;
use tower::{Layer, Service};

/// Builder type for inserting [`Handler`]s before building a [`RoutesFinalized`].
pub struct Routes {
    handlers: Vec<(Cow<'static, str>, HrpcService)>,
    any: Option<HrpcService>,
    all_layer: Option<HrpcLayer>,
}

impl Default for Routes {
    fn default() -> Self {
        Self::new()
    }
}

impl Routes {
    /// Create a new [`Routes`].
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            any: None,
            all_layer: None,
        }
    }

    /// Add a new route.
    pub fn route<S>(mut self, path: impl Into<Cow<'static, str>>, handler: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        self.handlers.push((path.into(), HrpcService::new(handler)));
        self
    }

    /// Layer the routes that were added until this.
    pub fn layer<L, S>(mut self, layer: L) -> Self
    where
        L: Layer<HrpcService, Service = S>,
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        for (path, handler) in self.handlers.drain(..).collect::<Vec<_>>() {
            let layered = handler.layer(&layer);
            self.handlers.push((path, layered));
        }
        self
    }

    /// Set layer for the finalized router service.
    pub fn layer_all<L, S>(mut self, layer: L) -> Self
    where
        L: Layer<HrpcService, Service = S> + Sync + Send + 'static,
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        self.all_layer = Some(HrpcLayer::new(layer));
        self
    }

    /// Combine this with another [`Routes`]. Note that this cannot combine the `any` handler.
    pub fn combine_with(mut self, other_routes: impl Into<Routes>) -> Self {
        let mut other_routes = other_routes.into();
        self.handlers.append(&mut other_routes.handlers);
        self.all_layer = if let Some(layer) = self.all_layer {
            let layer = if let Some(other_layer) = other_routes.all_layer {
                HrpcLayer::stack(layer, other_layer)
            } else {
                layer
            };
            Some(layer)
        } else {
            other_routes.all_layer
        };
        self
    }

    /// Set the service that will be used if no routes are matched.
    pub fn any<S>(mut self, handler: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        self.any = Some(HrpcService::new(handler));
        self
    }

    /// Build the routes.
    ///
    /// ## Panics
    /// - This can panic if one of the paths of the inserted routes is invalid.
    pub fn build(self) -> RoutesFinalized {
        let mut matcher = Matcher::new();

        for (path, handler) in self.handlers {
            matcher.insert(path, handler).expect("invalid route path");
        }

        let internal = RoutesInternal {
            matcher,
            any: self.any.unwrap_or_else(not_found),
        };

        let inner = match self.all_layer {
            Some(layer) => layer.layer(internal),
            None => HrpcService::new(internal),
        };

        RoutesFinalized { inner }
    }
}

struct RoutesInternal {
    matcher: Matcher<HrpcService>,
    any: HrpcService,
}

impl Service<HttpRequest> for RoutesInternal {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = CallFuture<'static>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // TODO: fix this when able to get all values inside a matcher
        Ok(()).into()
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        let path = req.uri().path();
        match self.matcher.at_mut(path) {
            Ok(matched) => Service::call(matched.value, req),
            Err(err) if err.tsr() => {
                let redirect_to = if let Some(without_tsr) = path.strip_suffix('/') {
                    Cow::Borrowed(without_tsr)
                } else {
                    Cow::Owned(format!("{}/", path))
                };
                match self.matcher.at_mut(redirect_to.as_ref()) {
                    Ok(matched) => Service::call(matched.value, req),
                    _ => Service::call(&mut self.any, req),
                }
            }
            _ => Service::call(&mut self.any, req),
        }
    }
}

/// Finalized [`Routes`], ready for serving as a [`Service`].
pub struct RoutesFinalized {
    inner: HrpcService,
}

impl Service<HttpRequest> for RoutesFinalized {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = CallFuture<'static>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        Service::call(&mut self.inner, req)
    }
}
