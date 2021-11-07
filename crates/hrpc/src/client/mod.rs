use std::{
    fmt::{self, Debug, Formatter},
    sync::Arc,
};

use crate::{
    body::Body,
    decode,
    request::{self, BoxRequest},
    Response,
};

use self::transport::Transport;

use super::Request;
use error::*;
use socket::*;

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
        transport::Transport,
        Client,
    };
    pub use crate::{request::IntoRequest, Request, Response};
    pub use std::{borrow::Cow, convert::TryInto, fmt::Debug};
}

/// Generic client implementation with common methods.
pub struct Client<Inner: Transport> {
    transport: Inner,
    modify_request_preflight: Arc<dyn Fn(&mut BoxRequest) + Send + Sync>,
}

impl<Inner: Transport + Debug> Debug for Client<Inner> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.transport)
            .finish()
    }
}

impl<Inner: Transport + Clone> Clone for Client<Inner> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            modify_request_preflight: self.modify_request_preflight.clone(),
        }
    }
}

impl<Inner: Transport> Client<Inner> {
    /// Create a new client using the provided transport.
    pub fn new(transport: Inner) -> Self {
        Self {
            transport,
            modify_request_preflight: Arc::new(|_| ()),
        }
    }

    /// Set the function to modify request with before sending a request.
    pub fn modify_request_preflight_with(
        mut self,
        f: Arc<dyn Fn(&mut BoxRequest) + Send + Sync>,
    ) -> Self {
        self.modify_request_preflight = f;
        self
    }

    /// Executes a unary request and returns the decoded response.
    pub async fn execute_request<Req: prost::Message, Resp: prost::Message + Default>(
        &mut self,
        req: Request<Req>,
    ) -> ClientResult<Response<Resp>, Inner::Error> {
        self.transport.call_unary(req).await
    }

    /// Connect a socket with the server and return it.
    pub async fn connect_socket<Req, Resp>(
        &mut self,
        req: Request<()>,
    ) -> ClientResult<Socket<Req, Resp>, Inner::Error>
    where
        Req: prost::Message + 'static,
        Resp: prost::Message + Default + 'static,
    {
        self.transport.call_socket(req).await
    }

    /// Connect a socket with the server, send a message and return it.
    ///
    /// Used by the server streaming methods.
    pub async fn connect_socket_req<Req, Resp>(
        &mut self,
        request: Request<Req>,
    ) -> ClientResult<Socket<Req, Resp>, Inner::Error>
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

        let socket = self
            .connect_socket(Request::from(request::Parts {
                body: Body::empty(),
                endpoint: endpoint.clone(),
                extensions,
            }))
            .await?;

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
