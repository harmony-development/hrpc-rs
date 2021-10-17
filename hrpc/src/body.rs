use super::BoxError;

use bytes::Bytes;
use http_body::Body;

/// A boxed [`Body`] trait object.
pub type BoxBody = http_body::combinators::BoxBody<Bytes, BoxError>;
/// A `hyper::Body`, mainly used for `Request` types.
pub type HyperBody = hyper::Body;

/// Convert a [`http_body::Body`] into a [`BoxBody`].
pub fn box_body<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    B::Error: Into<BoxError>,
{
    body.map_err(|b| b.into()).boxed()
}

/// Create an empty [`BoxBody`].
pub fn empty_box_body() -> BoxBody {
    box_body(http_body::Empty::new())
}

/// Create a "full" (single chunk) [`BoxBody`] containing the given data.
pub fn full_box_body(data: Bytes) -> BoxBody {
    box_body(http_body::Full::new(data))
}
