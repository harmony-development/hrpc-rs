use std::fmt::{self, Debug, Formatter};

use crate::{
    body::Body,
    decode,
    request::{self, BoxRequest},
    Response,
};

use self::transport::{TransportRequest, TransportResponse};

use super::Request;
use error::*;
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
    pub use std::{borrow::Cow, convert::TryInto, fmt::Debug};
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
    Inner: Service<TransportRequest, Response = TransportResponse>,
    Inner::Error: std::error::Error,
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
    pub async fn execute_request<Req: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        req: Request<Req>,
    ) -> ClientResult<Response<Resp>, Inner::Error> {
        Service::call(&mut self.transport, TransportRequest::Unary(req.map()))
            .await
            .map(|resp| resp.extract_unary().map::<Resp>())
            .map_err(ClientError::Transport)
    }

    /// Connect a socket with the server and return it.
    pub async fn connect_socket<Req, Resp>(
        &mut self,
        req: Request<()>,
    ) -> ClientResult<Socket<Req, Resp>, Inner::Error>
    where
        Req: prost::Message,
        Resp: prost::Message + Default,
    {
        let resp = Service::call(&mut self.transport, TransportRequest::Socket(req))
            .await
            .map_err(ClientError::Transport)?;

        let (tx, rx) = resp.extract_socket();

        Ok(Socket::new(
            rx,
            tx,
            socket::encode_message,
            socket::decode_message,
        ))
    }

    /// Connect a socket with the server, send a message and return it.
    ///
    /// Used by the server streaming methods.
    pub async fn connect_socket_req<Req, Resp>(
        &mut self,
        request: Request<Req>,
    ) -> ClientResult<Socket<Req, Resp>, Inner::Error>
    where
        Req: prost::Message + Default,
        Resp: prost::Message + Default,
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

        let mut socket = self.connect_socket(request).await?;

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
