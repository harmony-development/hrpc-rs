use std::{
    borrow::Cow, convert::Infallible, mem::ManuallyDrop, ops::DerefMut, ptr::NonNull, task::Poll,
};

use crate::{request::BoxRequest, response::BoxResponse};

use super::{
    service::{not_found, CallFuture, HrpcService},
    HrpcLayer,
};

use matchit::Node as Matcher;
use tower::{Layer, Service};
use tracing::Span;

/// Builder type for inserting [`HrpcService`]s before building a [`RoutesFinalized`].
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
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        self.handlers.push((path.into(), HrpcService::new(handler)));
        self
    }

    /// Layer the routes that were added until this was called.
    pub fn layer<L, S>(mut self, layer: L) -> Self
    where
        L: Layer<HrpcService, Service = S>,
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        for (path, handler) in self.handlers.drain(..).collect::<Vec<_>>() {
            let layered = handler.layer(&layer);
            self.handlers.push((path, layered));
        }
        self
    }

    /// Set layer for the finalized router service. This applies to all routes.
    pub fn layer_all<L, S>(mut self, layer: L) -> Self
    where
        L: Layer<HrpcService, Service = S> + Sync + Send + 'static,
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        let outer = HrpcLayer::new(layer);
        self.all_layer = match self.all_layer {
            Some(inner) => Some(HrpcLayer::stack(inner, outer)),
            None => Some(outer),
        };
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

    /// Set the handler that will be used if no routes are matched.
    pub fn any<S>(mut self, handler: S) -> Self
    where
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
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
        let mut all_routes = Vec::with_capacity(self.handlers.len());

        for (path, handler) in self.handlers {
            let manual_drop = ManuallyDrop::new(handler);
            let mut_ptr = Box::leak(Box::new(manual_drop)) as *mut ManuallyDrop<HrpcService>;
            let handler_ptr = NonNull::new(mut_ptr).unwrap();
            matcher
                .insert(path, handler_ptr)
                .expect("invalid route path");
            all_routes.push(handler_ptr);
        }

        let internal = RoutesInternal {
            matcher,
            all_routes,
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
    matcher: Matcher<NonNull<ManuallyDrop<HrpcService>>>,
    all_routes: Vec<NonNull<ManuallyDrop<HrpcService>>>,
    any: HrpcService,
}

// SAFETY: this impl is safe since our NonNull is not aliased while `Send`ing
unsafe impl Send for RoutesInternal {}

impl Drop for RoutesInternal {
    fn drop(&mut self) {
        for route_ptr in &mut self.all_routes {
            // SAFETY: this is required since we are manually dropping the items
            unsafe { ManuallyDrop::drop(route_ptr.as_mut()) }
        }
    }
}

impl Service<BoxRequest> for RoutesInternal {
    type Response = BoxResponse;

    type Error = Infallible;

    type Future = CallFuture<'static>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        for route_ptr in &mut self.all_routes {
            // SAFETY: our ptr is always initialized
            let route = unsafe { route_ptr.as_mut() };
            match Service::poll_ready(route.deref_mut(), cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                _ => continue,
            }
        }
        Service::poll_ready(&mut self.any, cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        let path = req.endpoint();

        match self.matcher.at_mut(path) {
            Ok(matched) => Service::call(
                // SAFETY: our ptr is always initialized
                unsafe { matched.value.as_mut().deref_mut() },
                req,
            ),
            Err(err) if err.tsr() => {
                let redirect_to = if let Some(without_tsr) = path.strip_suffix('/') {
                    Cow::Borrowed(without_tsr)
                } else {
                    Cow::Owned(format!("{}/", path))
                };
                Span::current().record("redirected_to", &redirect_to.as_ref());
                match self.matcher.at_mut(redirect_to.as_ref()) {
                    Ok(matched) => {
                        Service::call(
                            // SAFETY: our ptr is always initialized
                            unsafe { matched.value.as_mut().deref_mut() },
                            req,
                        )
                    }
                    _ => Service::call(&mut self.any, req),
                }
            }
            _ => Service::call(&mut self.any, req),
        }
    }
}

/// Finalized [`Routes`], ready for serving as a [`Service`].
pub struct RoutesFinalized {
    pub(crate) inner: HrpcService,
}

impl Service<BoxRequest> for RoutesFinalized {
    type Response = BoxResponse;

    type Error = Infallible;

    type Future = CallFuture<'static>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        Service::call(&mut self.inner, req)
    }
}
