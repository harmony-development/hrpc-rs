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
pub trait CustomError: Debug + Display {
    /// Status code that will be used in client response.
    fn code(&self) -> StatusCode;
    /// Message that will be used in client response.
    fn message(&self) -> &str;
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

impl<Err: CustomError + Send + Sync + 'static> warp::reject::Reject for ServerError<Err> {}

#[doc(hidden)]
pub async fn handle_rejection<Err: CustomError + Send + Sync + 'static>(
    err: Rejection,
) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "not found";
    } else if let Some(e) = err.find::<ServerError<Err>>() {
        match e {
            ServerError::MessageDecode(_) => {
                code = StatusCode::BAD_REQUEST;
                message = "invalid protobuf message";
            }
            ServerError::Custom(err) => {
                code = err.code();
                message = err.message();
            }
        }
    } else {
        log::error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "internal server error";
    }

    let json = warp::reply::json(&format!("{{ \"message\": \"{}\" }}", message));

    Ok(warp::reply::with_status(json, code))
}
