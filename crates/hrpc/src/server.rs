use std::{convert::Infallible, future, net::ToSocketAddrs};

use tower::{Layer, Service as TowerService};

use crate::{HttpRequest, HttpResponse};
use handler::Handler;
use router::Routes;

use self::router::RoutesFinalized;

/// Error types used by hRPC.
pub mod error;
/// Handler type and handlers used by hRPC.
pub mod handler;
/// The router used by hRPC.
pub mod router;
/// Socket used by hRPC for "streaming" RPCs.
pub mod socket;
/// Other useful types, traits and functions used by hRPC.
pub mod utils;

mod macros;
pub(crate) mod ws;

pub use handler::HrpcLayer;

// Prelude used by generated code. It is not meant to be used by users.
#[doc(hidden)]
pub mod gen_prelude {
    pub use super::{
        error::ServerResult,
        handler::{unary_handler, ws_handler, Handler, HrpcLayer},
        router::Routes,
        socket::Socket,
        utils::serve,
        Service,
    };
    pub use crate::{
        body::box_body, BoxError, HttpRequest, HttpResponse, Request as HrpcRequest,
        Response as HrpcResponse,
    };
    pub use bytes::Bytes;
    pub use futures_util::future::BoxFuture;
    pub use http::Response as _HttpResponse;
    pub use http_body::Body as HttpBody;
    pub use std::{convert::Infallible, future::Future, sync::Arc};
    pub use tower::{layer::util::Identity, Layer, Service as TowerService};
}

/// Prelude that exports commonly used server types.
pub mod prelude {
    pub use super::{
        error::{CustomError, ServerResult},
        handler::HrpcLayer,
        socket::Socket,
        Service,
    };
    pub use crate::{IntoResponse, Request, Response};
    pub use async_trait::async_trait;
    pub use http::StatusCode;
}

/// The core trait of `hrpc-rs` servers. This trait acts as a `tower::MakeService`,
/// it can produce a set of [`Routes`] and can be combined with other [`Service`]s.
#[async_trait::async_trait]
pub trait Service: Send + 'static {
    /// Creates a [`Routes`], which will be used to build a [`RoutesFinalized`] instance.
    fn make_routes(&self) -> Routes;

    /// Combines this server with another server.
    fn combine_with<Other>(self, other: Other) -> ServiceStack<Other, Self>
    where
        Other: Service,
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
    fn layer<L>(self, layer: L) -> LayeredService<L, Self>
    where
        L: Layer<Handler, Service = Handler> + Clone + Sync + Send + 'static,
        Self: Sized,
    {
        LayeredService { inner: self, layer }
    }

    /// Serves this service. See [`utils::serve`] for more information.
    async fn serve<A>(self, address: A) -> Result<(), hyper::Error>
    where
        A: ToSocketAddrs + Send,
        A::Iter: Send,
        Self: Sized,
    {
        utils::serve(self, address).await
    }
}

/// Type that layers the handlers that are produced by a [`Service`].
pub struct LayeredService<L, S>
where
    L: Layer<Handler, Service = Handler> + Clone + Sync + Send + 'static,
    S: Service,
{
    inner: S,
    layer: L,
}

impl<L, S> Clone for LayeredService<L, S>
where
    L: Layer<Handler, Service = Handler> + Clone + Sync + Send + 'static,
    S: Service + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            layer: self.layer.clone(),
        }
    }
}

impl<L, S> Service for LayeredService<L, S>
where
    L: Layer<Handler, Service = Handler> + Clone + Sync + Send + 'static,
    S: Service,
{
    fn make_routes(&self) -> Routes {
        let rb = Service::make_routes(&self.inner);
        rb.layer_all(self.layer.clone())
    }
}

/// Type that contains two [`Service`]s and stacks (combines) them.
pub struct ServiceStack<Outer, Inner>
where
    Outer: Service,
    Inner: Service,
{
    outer: Outer,
    inner: Inner,
}

impl<Outer, Inner> Clone for ServiceStack<Outer, Inner>
where
    Outer: Service + Clone,
    Inner: Service + Clone,
{
    fn clone(&self) -> Self {
        Self {
            outer: self.outer.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<Outer, Inner> Service for ServiceStack<Outer, Inner>
where
    Outer: Service,
    Inner: Service,
{
    fn make_routes(&self) -> Routes {
        let outer_rb = Service::make_routes(&self.outer);
        let inner_rb = Service::make_routes(&self.inner);
        outer_rb.combine_with(inner_rb)
    }
}

/// Type that contains a [`Service`] and implements `tower::MakeService` to
/// create [`RoutesFinalized`] instances.
pub struct IntoMakeService<S: Service> {
    mk_router: S,
}

impl<S: Service + Clone> Clone for IntoMakeService<S> {
    fn clone(&self) -> Self {
        Self {
            mk_router: self.mk_router.clone(),
        }
    }
}

impl<T, S: Service> TowerService<T> for IntoMakeService<S> {
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

    use crate::server::{router::Routes, HrpcLayer, Service};

    use super::utils::recommended_layers;

    struct TestServer;

    impl Service for TestServer {
        fn make_routes(&self) -> Routes {
            Routes::new()
        }
    }

    #[test]
    fn layered_identity() {
        let s = TestServer;

        // we can't poll it, and we don't want to anyways
        let _ = s
            .layer(HrpcLayer::new(Identity::new()))
            .serve("127.0.0.1:2289");
    }

    #[test]
    fn layered_recommended() {
        let s = TestServer;

        // we can't poll it, and we don't want to anyways
        let _ = s
            .layer(recommended_layers(|_| true))
            .serve("127.0.0.1:2289");
    }
}
