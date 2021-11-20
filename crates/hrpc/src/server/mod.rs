use std::convert::Infallible;

use tower::{Layer, Service as TowerService};

use router::Routes;
use service::HrpcService;

use crate::{common::future, request::BoxRequest, response::BoxResponse};

use self::router::RoutesFinalized;

/// Error types used by hRPC.
pub mod error;
/// The router used by hRPC.
pub mod router;
/// Handler type and handlers used by hRPC.
pub mod service;
/// Socket used by hRPC for "streaming" RPCs.
pub mod socket;
/// Transports for hRPC services.
pub mod transport;
/// Other useful types, traits and functions used by hRPC.
pub mod utils;

mod macros;

pub use service::HrpcLayer;

// Prelude used by generated code. It is not meant to be used by users.
#[doc(hidden)]
pub mod gen_prelude {
    pub use super::{
        error::ServerResult,
        router::Routes,
        service::{unary_handler, ws_handler, HrpcLayer, HrpcService},
        socket::Socket,
        transport::Transport,
        MakeRoutes,
    };
    pub use crate::{BoxError, Request as HrpcRequest, Response as HrpcResponse};
    pub use bytes::Bytes;
    pub use futures_util::future::BoxFuture;
    pub use std::{convert::Infallible, future::Future, net::ToSocketAddrs, sync::Arc};
    pub use tower::{layer::util::Identity, Layer, Service as TowerService};
}

/// Prelude that exports commonly used server types.
pub mod prelude {
    pub use super::{
        error::{HrpcError, ServerResult},
        service::{HrpcLayer, HrpcLayerExt},
        socket::Socket,
        transport::Transport,
        MakeRoutes,
    };
    pub use crate::{make_handler, response::IntoResponse, Request, Response};
    pub use hrpc_proc_macro::handler;
}

/// The core trait of `hrpc-rs` servers. This trait acts as a `tower::MakeService`,
/// it can produce a set of [`Routes`] and can be combined with other [`MakeRoutes`]s.
///
/// Not to be confused with [`tower::Service`].
pub trait MakeRoutes: Send + 'static {
    /// Creates a [`Routes`], which will be used to build a [`RoutesFinalized`] instance.
    fn make_routes(&self) -> Routes;

    /// Combines this server with another server.
    fn combine_with<Other>(self, other: Other) -> ServiceStack<Other, Self>
    where
        Other: MakeRoutes,
        Self: Sized,
    {
        ServiceStack {
            outer: other,
            inner: self,
        }
    }

    /// Turns this server into a type that implements `MakeService`, so that
    /// it can be used with [`hyper`].
    fn into_make_service(self) -> IntoMakeService<Self>
    where
        Self: Sized,
    {
        IntoMakeService { mk_router: self }
    }

    /// Layers this server with a layer.
    ///
    /// If your layer does not implement [`Clone`], you can wrap it using
    /// [`HrpcLayer::new`].
    fn layer<S, L>(self, layer: L) -> LayeredService<S, L, Self>
    where
        L: Layer<HrpcService, Service = S> + Clone + Sync + Send + 'static,
        S: tower::Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
        Self: Sized,
    {
        LayeredService { inner: self, layer }
    }
}

/// Type that layers the handlers that are produced by a [`MakeRoutes`].
pub struct LayeredService<S, L, M>
where
    L: Layer<HrpcService, Service = S> + Clone + Sync + Send + 'static,
    S: tower::Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
    M: MakeRoutes,
{
    inner: M,
    layer: L,
}

impl<S, L, M> Clone for LayeredService<S, L, M>
where
    L: Layer<HrpcService, Service = S> + Clone + Sync + Send + 'static,
    S: tower::Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
    M: MakeRoutes + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            layer: self.layer.clone(),
        }
    }
}

impl<S, L, M> MakeRoutes for LayeredService<S, L, M>
where
    L: Layer<HrpcService, Service = S> + Clone + Sync + Send + 'static,
    S: tower::Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
    M: MakeRoutes,
{
    fn make_routes(&self) -> Routes {
        let rb = MakeRoutes::make_routes(&self.inner);
        rb.layer_all(self.layer.clone())
    }
}

/// Type that contains two [`MakeRoutes`]s and stacks (combines) them.
pub struct ServiceStack<Outer, Inner>
where
    Outer: MakeRoutes,
    Inner: MakeRoutes,
{
    outer: Outer,
    inner: Inner,
}

impl<Outer, Inner> Clone for ServiceStack<Outer, Inner>
where
    Outer: MakeRoutes + Clone,
    Inner: MakeRoutes + Clone,
{
    fn clone(&self) -> Self {
        Self {
            outer: self.outer.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<Outer, Inner> MakeRoutes for ServiceStack<Outer, Inner>
where
    Outer: MakeRoutes,
    Inner: MakeRoutes,
{
    fn make_routes(&self) -> Routes {
        let outer_rb = MakeRoutes::make_routes(&self.outer);
        let inner_rb = MakeRoutes::make_routes(&self.inner);
        outer_rb.combine_with(inner_rb)
    }
}

/// Type that contains a [`MakeRoutes`] and implements `tower::MakeService` to
/// create [`RoutesFinalized`] instances.
pub struct IntoMakeService<S: MakeRoutes> {
    mk_router: S,
}

impl<S: MakeRoutes + Clone> Clone for IntoMakeService<S> {
    fn clone(&self) -> Self {
        Self {
            mk_router: self.mk_router.clone(),
        }
    }
}

impl<T, S: MakeRoutes> TowerService<T> for IntoMakeService<S> {
    type Response = RoutesFinalized;

    type Error = Infallible;

    type Future = future::Ready<Result<RoutesFinalized, Infallible>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _req: T) -> Self::Future {
        future::ready(Ok(self.mk_router.make_routes().build()))
    }
}

#[cfg(test)]
mod tests {
    use tower::layer::util::Identity;

    use crate::server::{router::Routes, HrpcLayer, MakeRoutes};

    struct TestServer;

    impl MakeRoutes for TestServer {
        fn make_routes(&self) -> Routes {
            Routes::new()
        }
    }

    #[test]
    fn layered_identity() {
        let s = TestServer;

        // we can't poll it, and we don't want to anyways
        let _ = s.layer(HrpcLayer::new(Identity::new()));
    }
}
