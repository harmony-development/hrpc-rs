use bytes::BytesMut;
use futures_util::{future::BoxFuture, SinkExt};
use prost::Message as PbMsg;

use crate::{common::socket::*, decode::DecodeBodyError, proto::Error as HrpcError};

pub use crate::common::socket::{ReadSocket, Socket, WriteSocket};

impl<Req, Resp> Socket<Req, Resp> {
    /// Send an error over the socket.
    #[inline]
    pub async fn send_error(&mut self, err: HrpcError) -> Result<(), HrpcError> {
        self.write.send_error(err).await
    }
}

impl<Req> WriteSocket<Req> {
    /// Send an error over the socket.
    pub async fn send_error(&mut self, err: HrpcError) -> Result<(), HrpcError> {
        let data = encode_hrpc_error(&mut self.buf, &err);
        self.ws_tx.send(SocketMessage::Binary(data)).await
    }
}

pub(super) struct SocketHandler {
    #[allow(dead_code)]
    pub(crate) inner:
        Box<dyn FnOnce(BoxedWsRx, BoxedWsTx) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
}

pub(super) fn encode_message<Msg: PbMsg>(buf: &mut BytesMut, msg: &Msg) -> Vec<u8> {
    crate::encode::encode_protobuf_message_to(buf, msg);
    // TODO: don't allocate here?
    let mut data = buf.to_vec();
    data.insert(0, 0);
    data
}

fn encode_hrpc_error(buf: &mut BytesMut, err: &HrpcError) -> Vec<u8> {
    crate::encode::encode_protobuf_message_to(buf, err);
    // TODO: don't allocate here?
    let mut data = buf.to_vec();
    data.insert(0, 1);
    data
}

pub(super) fn decode_message<Msg: PbMsg + Default>(raw: Vec<u8>) -> Result<Msg, HrpcError> {
    if raw.is_empty() {
        return Err(
            DecodeBodyError::InvalidProtoMessage(prost::DecodeError::new("empty protobuf message"))
                .into(),
        );
    }

    Msg::decode(raw.as_slice())
        .map_err(|err| HrpcError::from(DecodeBodyError::InvalidProtoMessage(err)))
}
