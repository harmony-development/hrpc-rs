use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use prost::Message as PbMsg;

use crate::{
    body::Body, common::extensions::Extensions, decode::*, encode::encode_protobuf_message,
};

/// Request parts.
pub struct Parts {
    /// Body of a request.
    pub body: Body,
    /// Extensions of a request.
    pub extensions: Extensions,
    /// Endpoint of a request.
    pub endpoint: Cow<'static, str>,
}

impl<T> From<Request<T>> for Parts {
    fn from(req: Request<T>) -> Self {
        Self {
            body: req.body,
            extensions: req.extensions,
            endpoint: req.endpoint,
        }
    }
}

impl<T> From<Parts> for Request<T> {
    fn from(parts: Parts) -> Self {
        Self {
            body: parts.body,
            extensions: parts.extensions,
            endpoint: parts.endpoint,
            message: PhantomData,
        }
    }
}

/// A hRPC request.
pub struct Request<T> {
    body: Body,
    endpoint: Cow<'static, str>,
    extensions: Extensions,
    message: PhantomData<T>,
}

impl<T> Debug for Request<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("body", &self.body)
            .field("endpoint", &self.endpoint)
            .field("extensions", &self.extensions)
            .finish()
    }
}

impl Request<()> {
    /// Create a request with an empty body.
    ///
    /// This is useful for hRPC socket requests, since they don't send any messages.
    pub fn empty() -> Request<()> {
        Self::new_with_body(Body::empty())
    }
}

impl<T> Request<T> {
    /// Creates a new request using the provided body.
    pub fn new_with_body(body: Body) -> Self {
        Self {
            body,
            extensions: Extensions::new(),
            endpoint: Cow::Borrowed(""),
            message: PhantomData,
        }
    }

    /// Get a mutable reference to the extensions of this response.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    /// Get an immutable reference to the extensions of this response.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Get a mutable reference to the endpoint of this response.
    #[inline]
    pub fn endpoint_mut(&mut self) -> &mut Cow<'static, str> {
        &mut self.endpoint
    }

    /// Get an immutable reference to the endpoint of this response.
    #[inline]
    pub fn endpoint(&self) -> &str {
        self.endpoint.as_ref()
    }

    #[allow(dead_code)]
    pub(crate) fn map<M>(self) -> Request<M> {
        Request {
            body: self.body,
            extensions: self.extensions,
            endpoint: self.endpoint,
            message: PhantomData,
        }
    }
}

impl<T: PbMsg> Request<T> {
    /// Create a new request with the specified message.
    ///
    /// This adds the [`HRPC_HEADER`] to the [`http::header::CONTENT_TYPE`]
    /// header for hRPC unary requests.
    pub fn new(msg: &T) -> Self {
        let encoded = encode_protobuf_message(msg).freeze();
        Self::new_with_body(Body::full(encoded))
    }
}

impl<T: PbMsg + Default> Request<T> {
    /// Extract the body from the request and decode it into the message.
    #[inline]
    pub async fn into_message(self) -> Result<T, DecodeBodyError> {
        decode_body(self.body).await
    }
}

/// Trait used for blanket impls on generated protobuf types.
pub trait IntoRequest<T> {
    /// Convert this to a hRPC request.
    fn into_request(self) -> Request<T>;
}

impl<T: PbMsg> IntoRequest<T> for T {
    fn into_request(self) -> Request<Self> {
        Request::new(&self)
    }
}

impl<T> IntoRequest<T> for Request<T> {
    fn into_request(self) -> Request<T> {
        self
    }
}

/// A request that has a message type of `()`. Used in places where the
/// request message type is not important.
pub type BoxRequest = Request<()>;
