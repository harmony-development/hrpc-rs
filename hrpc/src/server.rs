use std::{
    convert::Infallible,
    fmt::{self, Debug, Display, Formatter},
};
use warp::{Rejection, Reply};

#[doc(hidden)]
pub mod prelude {
    pub use super::{CustomError, ServerError};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{SinkExt, StreamExt};
    pub use log;
    pub use warp::{self, ws::Message as WsMessage, Filter};
}

#[doc(inline)]
pub use warp::http::StatusCode;

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: Debug + Display + Send + Sync {
    /// Status code that will be used in client response.
    fn code(&self) -> StatusCode;
    /// Message that will be used in client response.
    fn message(&self) -> Vec<u8>;
    /// Return a message decode error.
    fn decode_error() -> (StatusCode, Vec<u8>) {
        (
            StatusCode::BAD_REQUEST,
            json_err_bytes("invalid protobuf message"),
        )
    }
    /// Return a not fonud error.
    fn not_found_error() -> (StatusCode, Vec<u8>) {
        (StatusCode::NOT_FOUND, json_err_bytes("not found"))
    }
    /// Return an internal server error.
    fn internal_server_error() -> (StatusCode, Vec<u8>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_err_bytes("internal server error"),
        )
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum ServerError<Err: CustomError> {
    MessageDecode(prost::DecodeError),
    Custom(Err),
}

impl<Err: CustomError> Display for ServerError<Err> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerError::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
            ServerError::Custom(err) => write!(f, "error occured: {}", err),
        }
    }
}

impl<Err: CustomError + 'static> warp::reject::Reject for ServerError<Err> {}

#[doc(hidden)]
pub async fn handle_rejection<Err: CustomError + 'static>(
    err: Rejection,
) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        Err::not_found_error()
    } else if let Some(e) = err.find::<ServerError<Err>>() {
        match e {
            ServerError::MessageDecode(_) => Err::decode_error(),
            ServerError::Custom(err) => (err.code(), err.message()),
        }
    } else {
        log::error!("unhandled rejection: {:?}", err);
        Err::internal_server_error()
    };

    let mut reply = warp::reply::Response::new(message.into());
    *reply.status_mut() = code;

    Ok(reply)
}

/// Creates a JSON error response from a message.
pub fn json_err_bytes(msg: &str) -> Vec<u8> {
    format!("{{ \"message\": \"{}\" }}", msg).into_bytes()
}

/// Serves multiple services' filters on the same address.
#[macro_export]
macro_rules! serve_multiple {
    {
        addr: $address:expr,
        err: $err:ty,
        filters: $first:expr, $( $filter:expr, )+
    } => {
        async move {
            use $crate::warp::Filter;

            let filter = $first $( .or($filter) )+ ;

            $crate::warp::serve(filter.recover($crate::server::handle_rejection::<$err>))
                .run($address)
                .await
        }
    };
}
