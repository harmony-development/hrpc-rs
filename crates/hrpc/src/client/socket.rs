use bytes::BytesMut;
use prost::Message as PbMsg;

use crate::{decode::DecodeBodyError, proto::Error as HrpcError};

pub use crate::common::socket::{ReadSocket, Socket, WriteSocket};

pub(super) fn encode_message<Msg: PbMsg>(buf: &mut BytesMut, msg: &Msg) -> Vec<u8> {
    crate::encode::encode_protobuf_message_to(buf, msg);
    // TODO: don't allocate here?
    buf.to_vec()
}

pub(super) fn decode_message<Msg: PbMsg + Default>(raw: Vec<u8>) -> Result<Msg, HrpcError> {
    if raw.is_empty() {
        return Err(
            DecodeBodyError::InvalidProtoMessage(prost::DecodeError::new("empty protobuf message"))
                .into(),
        );
    }

    let opcode = raw[0];

    if opcode == 0 {
        Msg::decode(&raw[1..])
            .map_err(|err| HrpcError::from(DecodeBodyError::InvalidProtoMessage(err)))
    } else if opcode == 1 {
        Err(HrpcError::decode(&raw[1..])
            .unwrap_or_else(|err| HrpcError::from(DecodeBodyError::InvalidProtoMessage(err))))
    } else {
        Err(HrpcError::from((
            "hrpcrs.http.invalid-socket-message-opcode",
            "invalid socket binary message opcode",
        ))
        .with_details(raw))
    }
}
