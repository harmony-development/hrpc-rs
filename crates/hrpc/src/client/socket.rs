use bytes::BytesMut;
use prost::Message as PbMsg;

use crate::{common::socket::DecodeResult, decode::DecodeBodyError, proto::Error as HrpcError};

pub use crate::common::socket::{ReadSocket, Socket, SocketError, WriteSocket};

pub(super) fn encode_message<Msg: PbMsg>(buf: &mut BytesMut, msg: &Msg) -> Vec<u8> {
    crate::encode::encode_protobuf_message_to(buf, msg);
    // TODO: don't allocate here?
    buf.to_vec()
}

pub(super) fn decode_message<Msg: PbMsg + Default>(
    raw: Vec<u8>,
) -> Result<DecodeResult<Msg>, DecodeBodyError> {
    if raw.is_empty() {
        return Err(DecodeBodyError::InvalidProtoMessage(
            prost::DecodeError::new("empty protobuf message"),
        ));
    }

    let opcode = raw[0];

    if opcode == 0 {
        Msg::decode(&raw[1..])
            .map(DecodeResult::Msg)
            .map_err(DecodeBodyError::InvalidProtoMessage)
    } else if opcode == 1 {
        HrpcError::decode(&raw[1..])
            .map(DecodeResult::Error)
            .map_err(DecodeBodyError::InvalidProtoMessage)
    } else {
        Err(DecodeBodyError::InvalidBody(Box::new(
            HrpcError::from((
                "hrpcrs.http.invalid-socket-message-opcode",
                "invalid socket binary message opcode",
            ))
            .with_details(raw),
        )))
    }
}
