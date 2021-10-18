use std::{convert::Infallible, future, net::ToSocketAddrs};

use tower::{Layer, Service};

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
        Server,
    };
    pub use crate::{body::box_body, Request as HrpcRequest, Response as HrpcResponse};
    pub use crate::{HttpRequest, HttpResponse};
    pub use tower::{layer::util::Identity, Layer};
}

/// Prelude that exports commonly used server types.
pub mod prelude {
    pub use super::{
        error::{CustomError, ServerResult},
        handler::HrpcLayer,
        socket::Socket,
        Server,
    };
    pub use crate::{IntoResponse, Request, Response};
    pub use async_trait::async_trait;
    pub use http::StatusCode;
}

/// The core trait of `hrpc-rs` servers. This trait acts as a `tower::MakeService`,
/// it can produce a set of [`Routes`] and can be combined with other [`Server`]s.
#[async_trait::async_trait]
pub trait Server: Send + 'static {
    /// Creates a [`Routes`], which will be used to build a [`RoutesFinalized`] instance.
    fn make_routes(&self) -> Routes;

    /// Combines this server with another server.
    fn combine_with<Other, OtherSvc>(self, other: Other) -> ServerStack<Other, Self>
    where
        Other: Server,
        Self: Sized,
    {
        ServerStack {
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

    /// Layers this server with a layer that transforms handlers.
    fn layer<L>(self, layer: L) -> LayeredServer<L, Self>
    where
        L: Layer<Handler, Service = Handler> + Send + 'static,
        Self: Sized,
    {
        LayeredServer { inner: self, layer }
    }

    /// Serves this server. See [`utils::serve`] for more information.
    async fn serve<A>(self, address: A) -> Result<(), hyper::Error>
    where
        A: ToSocketAddrs + Send,
        A::Iter: Send,
        Self: Sized,
    {
        utils::serve(self, address).await
    }
}

/// Type that layers the handlers that are produced by a [`Server`].
pub struct LayeredServer<L, S>
where
    L: Layer<Handler, Service = Handler> + Send + 'static,
    S: Server,
{
    inner: S,
    layer: L,
}

impl<L, S> Clone for LayeredServer<L, S>
where
    L: Layer<Handler, Service = Handler> + Clone + Send + 'static,
    S: Server + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            layer: self.layer.clone(),
        }
    }
}

impl<L, S> Server for LayeredServer<L, S>
where
    L: Layer<Handler, Service = Handler> + Send + 'static,
    S: Server,
{
    fn make_routes(&self) -> Routes {
        let rb = Server::make_routes(&self.inner);
        rb.layer(&self.layer)
    }
}

/// Type that contains two [`Server`]s and stacks (combines) them.
pub struct ServerStack<Outer, Inner>
where
    Outer: Server,
    Inner: Server,
{
    outer: Outer,
    inner: Inner,
}

impl<Outer, Inner> Clone for ServerStack<Outer, Inner>
where
    Outer: Server + Clone,
    Inner: Server + Clone,
{
    fn clone(&self) -> Self {
        Self {
            outer: self.outer.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<Outer, Inner> Server for ServerStack<Outer, Inner>
where
    Outer: Server,
    Inner: Server,
{
    fn make_routes(&self) -> Routes {
        let outer_rb = Server::make_routes(&self.outer);
        let inner_rb = Server::make_routes(&self.inner);
        outer_rb.combine_with(inner_rb)
    }
}

/// Type that contains a [`Server`] and implements `tower::MakeService` to
/// create [`RoutesFinalized`] instances.
pub struct IntoMakeService<S: Server> {
    mk_router: S,
}

impl<S: Server + Clone> Clone for IntoMakeService<S> {
    fn clone(&self) -> Self {
        Self {
            mk_router: self.mk_router.clone(),
        }
    }
}

impl<T, S: Server> Service<T> for IntoMakeService<S> {
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
