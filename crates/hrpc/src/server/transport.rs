use std::{future, net::ToSocketAddrs, time::Duration};

use futures_util::future::BoxFuture;

use super::Service;

/// Trait for enabling generic transport implementations over a [`Service`].
pub trait Transport: Sized {
    /// The type of the error returned by a transport if it fails.
    type Error;

    /// Start serving a [`Service`].
    fn serve<S>(self, service: S) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        S: Service;
}

/// A transport based on [`hyper`].
#[non_exhaustive]
pub struct Hyper<Addr: ToSocketAddrs> {
    addr: Addr,
}

impl<Addr: ToSocketAddrs> Hyper<Addr> {
    /// Create a new `hyper` transport.
    pub fn new(addr: Addr) -> Self {
        Self { addr }
    }
}

impl<Addr: ToSocketAddrs> Transport for Hyper<Addr> {
    type Error = hyper::Error;

    fn serve<S>(self, service: S) -> BoxFuture<'static, Result<(), Self::Error>>
    where
        S: Service,
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
                let server = builder
                    .http1_keepalive(true)
                    .http2_keep_alive_interval(Some(Duration::from_secs(10)))
                    .http2_keep_alive_timeout(Duration::from_secs(20))
                    .serve(service.into_make_service());

                tracing::info!("serving at {}", successful_addr);

                Box::pin(server)
            }
            Err(err) => Box::pin(future::ready(Err(err))),
        }
    }
}
