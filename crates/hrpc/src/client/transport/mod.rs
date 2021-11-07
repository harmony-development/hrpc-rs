use std::error::Error as StdError;

use futures_util::future::BoxFuture;
use prost::Message as PbMsg;

use super::{error::ClientResult, socket::Socket};
use crate::{Request, Response};

/// Client HTTP transport.
#[cfg(feature = "http_client")]
pub mod http;

/// Trait for enabling generic client transport implementations.
pub trait Transport {
    /// The error type the transport can produce.
    type Error: StdError;

    /// Make a unary request to the server.
    fn call_unary<'a, Req, Resp>(
        &'a mut self,
        req: Request<Req>,
    ) -> BoxFuture<'a, ClientResult<Response<Resp>, Self::Error>>
    where
        Req: PbMsg + 'a,
        Resp: PbMsg + Default;

    /// Make a request to the server to create a [`Socket`].
    fn call_socket<Req, Resp>(
        &mut self,
        req: Request<()>,
    ) -> BoxFuture<'_, ClientResult<Socket<Req, Resp>, Self::Error>>
    where
        Req: PbMsg + 'static,
        Resp: PbMsg + Default + 'static;
}
