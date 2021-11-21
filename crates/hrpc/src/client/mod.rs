use std::{
    fmt::{self, Debug, Formatter},
    future::Future,
};

use crate::{
    body::Body,
    decode,
    request::{self, BoxRequest},
    Response,
};

use self::transport::{TransportRequest, TransportResponse};

use super::Request;
use error::*;
use futures_util::TryFutureExt;
use socket::*;
use tower::{Layer, Service};

/// Error types.
pub mod error;
/// hRPC socket used for streaming RPCs.
pub mod socket;
/// hRPC client transports.
pub mod transport;

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        error::{ClientError, ClientResult},
        socket::Socket,
        transport::{TransportRequest, TransportResponse},
        Client,
    };
    pub use crate::{
        request::{IntoRequest, Request},
        response::Response,
    };
    pub use std::{borrow::Cow, convert::TryInto, fmt::Debug, future::Future};
    pub use tower::Service;
}

/// Generic client implementation with common methods.
pub struct Client<Inner> {
    transport: Inner,
}

impl<Inner: Debug> Debug for Client<Inner> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.transport)
            .finish()
    }
}

impl<Inner: Clone> Clone for Client<Inner> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
        }
    }
}

impl<Inner> Client<Inner> {
    /// Create a new client using the provided transport.
    pub fn new(transport: Inner) -> Client<Inner> {
        Client { transport }
    }
}

impl<Inner> Client<Inner>
where
    Inner: Service<TransportRequest, Response = TransportResponse> + 'static,
    Inner::Error: std::error::Error + 'static,
{
    /// Layer this client with a new [`Layer`].
    pub fn layer<S, L>(self, l: L) -> Client<S>
    where
        L: Layer<Inner, Service = S>,
        S: Service<TransportRequest, Response = TransportResponse>,
        S::Error: std::error::Error,
    {
        Client {
            transport: l.layer(self.transport),
        }
    }

    /// Executes a unary request and returns the decoded response.
    pub fn execute_request<Req: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        req: Request<Req>,
    ) -> impl Future<Output = ClientResult<Response<Resp>, Inner::Error>> + 'static {
        Service::call(&mut self.transport, TransportRequest::Unary(req.map()))
            .map_ok(|resp| resp.extract_unary().map::<Resp>())
            .map_err(ClientError::Transport)
    }

    /// Connect a socket with the server and return it.
    pub fn connect_socket<Req, Resp>(
        &mut self,
        req: Request<()>,
    ) -> impl Future<Output = ClientResult<Socket<Req, Resp>, Inner::Error>> + 'static
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        Service::call(&mut self.transport, TransportRequest::Socket(req))
            .map_ok(|resp| {
                let (tx, rx) = resp.extract_socket();

                Socket::new(rx, tx, socket::encode_message, socket::decode_message)
            })
            .map_err(ClientError::Transport)
    }

    /// Connect a socket with the server, send a message and return it.
    ///
    /// Used by the server streaming methods.
    pub fn connect_socket_req<Req, Resp>(
        &mut self,
        request: Request<Req>,
    ) -> impl Future<Output = ClientResult<Socket<Req, Resp>, Inner::Error>> + 'static
    where
        Req: prost::Message + Default + 'static,
        Resp: prost::Message + Default + 'static,
    {
        let request::Parts {
            body,
            extensions,
            endpoint,
            ..
        } = request.into();

        let request: BoxRequest = Request::from(request::Parts {
            body: Body::empty(),
            endpoint: endpoint.clone(),
            extensions,
        });

        let connect_fut = self.connect_socket(request);

        async move {
            let mut socket = connect_fut.await?;

            let message = decode::decode_body(body).await?;
            socket
                .send_message(message)
                .await
                .map_err(|err| ClientError::EndpointError {
                    hrpc_error: err,
                    endpoint,
                })?;

            Ok(socket)
        }
    }
}
