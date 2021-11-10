use futures_util::future::BoxFuture;
use std::{
    convert::Infallible,
    io,
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
};
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

/// A transport based on [`hyper`] that supports TLS.
#[non_exhaustive]
pub struct Hyper<L> {
    addr: SocketAddr,
    layer: L,
    tls: Option<(PathBuf, PathBuf)>,
}

impl Hyper<Identity> {
    /// Create a new `hyper` transport.
    pub fn new<Addr: ToSocketAddrs>(addr: Addr) -> Result<Self, io::Error> {
        Ok(Self {
            addr: addr.to_socket_addrs()?.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "no socket addresses passed")
            })?,
            layer: Identity::new(),
            tls: None,
        })
    }
}

impl<L> Hyper<L> {
    /// Layer this `hyper` server with a [`Layer`].
    pub fn layer<Layer>(self, layer: Layer) -> Hyper<Stack<Layer, L>> {
        Hyper {
            addr: self.addr,
            layer: Stack::new(layer, self.layer),
            tls: self.tls,
        }
    }

    /// Configure TLS for this server with paths to certificate and private key
    /// files.
    pub fn configure_tls_files(
        mut self,
        cert_path: impl Into<PathBuf>,
        key_path: impl Into<PathBuf>,
    ) -> Self {
        self.tls = Some((cert_path.into(), key_path.into()));
        self
    }
}

impl<L, S> Transport for Hyper<L>
where
    L: Layer<HrpcServiceToHttp, Service = S> + Clone + Send + 'static,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Error = std::io::Error;

    fn serve<M>(self, mk_routes: M) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        M: MakeRoutes,
    {
        let service = utils::MakeRoutesToHttp::new(mk_routes.into_make_service()).layer(self.layer);

        if let Some((cert_path, key_path)) = self.tls {
            Box::pin(async move {
                let rustls_conf =
                    axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
                        .await?;
                axum_server::bind_rustls(self.addr, rustls_conf)
                    .serve(service)
                    .await
            })
        } else {
            Box::pin(axum_server::bind(self.addr).serve(service))
        }
    }
}
