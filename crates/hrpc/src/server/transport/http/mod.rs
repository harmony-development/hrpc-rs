use futures_util::future::BoxFuture;
use std::{convert::Infallible, future, net::ToSocketAddrs, time::Duration};
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service,
};

use self::utils::HrpcServiceToHttp;

use super::Transport;
use crate::{
    common::transport::http::{HttpRequest, HttpResponse},
    server::MakeRoutes,
};

/// Utilities for working with this transport.
pub mod utils;
mod ws;

/// A transport based on [`hyper`].
#[non_exhaustive]
pub struct Hyper<Addr, L> {
    addr: Addr,
    layer: L,
}

impl<Addr: ToSocketAddrs> Hyper<Addr, Identity> {
    /// Create a new `hyper` transport.
    pub fn new(addr: Addr) -> Self {
        Self {
            addr,
            layer: Identity::new(),
        }
    }
}

impl<Addr, L> Hyper<Addr, L> {
    /// Layer this `hyper` server with a [`Layer`].
    pub fn layer<Layer>(self, layer: Layer) -> Hyper<Addr, Stack<Layer, L>> {
        Hyper {
            addr: self.addr,
            layer: Stack::new(layer, self.layer),
        }
    }
}

impl<Addr, L, S> Transport for Hyper<Addr, L>
where
    Addr: ToSocketAddrs,
    L: Layer<HrpcServiceToHttp, Service = S> + Clone + Send + 'static,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Error = hyper::Error;

    fn serve<M>(self, mk_routes: M) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        M: MakeRoutes,
    {
        let mut addrs = self
            .addr
            .to_socket_addrs()
            .expect("could not convert to socket address");

        let mut successful_addr = addrs.next().expect("no socket address provided");
        let mut builder = hyper::Server::try_bind(&successful_addr);
        for addr in addrs {
            builder = if builder.is_err() {
                successful_addr = addr;
                hyper::Server::try_bind(&successful_addr)
            } else {
                break;
            };
        }

        match builder {
            Ok(builder) => {
                let service =
                    utils::MakeRoutesToHttp::new(mk_routes.into_make_service()).layer(self.layer);

                let server = builder
                    .http1_keepalive(true)
                    .http2_keep_alive_interval(Some(Duration::from_secs(10)))
                    .http2_keep_alive_timeout(Duration::from_secs(20))
                    .serve(service);

                tracing::info!("serving at {}", successful_addr);

                Box::pin(server)
            }
            Err(err) => Box::pin(future::ready(Err(err))),
        }
    }
}
