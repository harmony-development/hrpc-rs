use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

use prost::Message as PbMsg;

use crate::{body::Body, proto::Error as HrpcError, BoxError};

pub(crate) async fn decode_body<T: PbMsg + Default>(body: Body) -> Result<T, DecodeBodyError> {
    let buf = body
        .aggregate()
        .await
        .map_err(DecodeBodyError::InvalidBody)?;
    let decoded = T::decode(buf).map_err(DecodeBodyError::InvalidProtoMessage)?;
    Ok(decoded)
}

/// Errors that can occur while decoding the body of a [`crate::Request`].
#[derive(Debug)]
pub enum DecodeBodyError {
    /// The body contained an invalid protobuf message.
    InvalidProtoMessage(prost::DecodeError),
    /// An error occured while reading the body.
    InvalidBody(BoxError),
}

impl Display for DecodeBodyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtoMessage(err) => write!(
                f,
                "body was detected to be protobuf, but contains invalid protobuf message: {}",
                err
            ),
            Self::InvalidBody(err) => write!(f, "error occured while aggregating body: {}", err),
        }
    }
}

impl StdError for DecodeBodyError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidProtoMessage(err) => Some(err),
            Self::InvalidBody(err) => err.source(),
        }
    }
}

impl From<DecodeBodyError> for HrpcError {
    fn from(err: DecodeBodyError) -> Self {
        let hrpc_err = HrpcError::default().with_message(err.to_string());
        match err {
            DecodeBodyError::InvalidBody(_) => hrpc_err.with_identifier("hrpcrs.invalid-body"),
            DecodeBodyError::InvalidProtoMessage(_) => {
                hrpc_err.with_identifier("hrpcrs.invalid-proto-message")
            }
        }
    }
}
