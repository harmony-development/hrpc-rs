use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use prost::Message as PbMsg;

use crate::{
    body::Body,
    common::extensions::Extensions,
    decode::{decode_body, DecodeBodyError},
    encode::encode_protobuf_message,
};

/// Response parts.
#[non_exhaustive]
#[derive(Debug)]
pub struct Parts {
    /// Body of a response.
    pub body: Body,
    /// Extensions of a response.
    pub extensions: Extensions,
}

impl<T> From<Response<T>> for Parts {
    fn from(resp: Response<T>) -> Self {
        resp.parts
    }
}

impl<T> From<Parts> for Response<T> {
    fn from(parts: Parts) -> Self {
        Self {
            parts,
            msg: PhantomData,
        }
    }
}

/// hRPC response type.
pub struct Response<T> {
    parts: Parts,
    msg: PhantomData<T>,
}

impl Response<()> {
    /// Creates a response with an empty body.
    pub fn empty() -> Self {
        Self::new_with_body(Body::empty())
    }
}

impl<T> Debug for Response<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("parts", &self.parts)
            .finish()
    }
}

impl<T> Response<T> {
    /// Creates a response with the provided body.
    pub fn new_with_body(body: Body) -> Self {
        Self {
            parts: Parts {
                body,
                extensions: Extensions::new(),
            },
            msg: PhantomData,
        }
    }

    /// Get a mutable reference to the extensions of this response.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }

    /// Get an immutable reference to the extensions of this response.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }

    #[allow(dead_code)]
    pub(crate) fn map<M>(self) -> Response<M> {
        Response {
            parts: self.parts,
            msg: PhantomData,
        }
    }
}

impl<T: PbMsg> Response<T> {
    /// Create a new hRPC response.
    pub fn new(msg: &T) -> Response<T> {
        Self::new_with_body(Body::full(encode_protobuf_message(msg)))
    }
}

impl<T: PbMsg + Default> Response<T> {
    /// Extract the body from the response and decode it into the message.
    #[inline]
    pub async fn into_message(self) -> Result<T, DecodeBodyError> {
        decode_body(self.parts.body).await
    }
}

/// Trait used for converting any type to a Response type.
pub trait IntoResponse<T> {
    /// Convert this to a hRPC response.
    fn into_response(self) -> Response<T>;
}

impl<T: PbMsg> IntoResponse<T> for T {
    fn into_response(self) -> Response<T> {
        Response::new(&self)
    }
}

impl<T> IntoResponse<T> for Response<T> {
    fn into_response(self) -> Response<T> {
        self
    }
}

/// A response that has a message type of `()`. Used in places where the
/// response message type is not important.
pub type BoxResponse = Response<()>;
