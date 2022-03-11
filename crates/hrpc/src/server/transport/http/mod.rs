use bytes::Bytes;
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

use super::Transport;
use crate::{server::MakeRoutes, BoxError};

/// Underlying implementation of this transport.
pub mod r#impl;
/// Useful layers for HTTP.
pub mod layer;
mod ws;

#[doc(inline)]
pub use axum_server::HttpConfig;
/// A boxed HTTP body. This is used to unify response bodies.
pub type BoxBody = http_body::combinators::UnsyncBoxBody<Bytes, BoxError>;
/// A HTTP request.
pub type HttpRequest = http::Request<hyper::Body>;
/// A HTTP response.
pub type HttpResponse = http::Response<BoxBody>;

/// Convert a body with the correct attributes to a [`BoxBody`].
pub fn box_body<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    BoxBody::new(body.map_err(Into::into))
}

/// A transport based on [`hyper`] that supports TLS.
#[non_exhaustive]
pub struct Hyper<L> {
    addr: SocketAddr,
    layer: L,
    tls: Option<(PathBuf, PathBuf)>,
    config: HttpConfig,
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
            config: HttpConfig::new(),
        })
    }
}

impl<L> Hyper<L> {
    /// Layer this [`hyper`] server with a [`Layer`].
    pub fn layer<Layer>(self, layer: Layer) -> Hyper<Stack<Layer, L>> {
        Hyper {
            addr: self.addr,
            layer: Stack::new(layer, self.layer),
            tls: self.tls,
            config: self.config,
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

    /// Set new configuration for the [`hyper`] HTTP server.
    pub fn configure_hyper(mut self, config: HttpConfig) -> Self {
        self.config = config;
        self
    }
}

impl<L, S> Transport for Hyper<L>
where
    L: Layer<r#impl::HrpcServiceToHttp, Service = S> + Clone + Send + 'static,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Error = std::io::Error;

    fn serve<M>(self, mk_routes: M) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        M: MakeRoutes,
    {
        let service =
            r#impl::MakeRoutesToHttp::new(mk_routes.into_make_service()).layer(self.layer);

        if let Some((cert_path, key_path)) = self.tls {
            Box::pin(async move {
                let rustls_conf =
                    axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
                        .await?;
                axum_server::bind_rustls(self.addr, rustls_conf)
                    .http_config(self.config)
                    .serve(service)
                    .await
            })
        } else {
            Box::pin(
                axum_server::bind(self.addr)
                    .http_config(self.config)
                    .serve(service),
            )
        }
    }
}
