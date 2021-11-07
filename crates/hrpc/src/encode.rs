use bytes::BytesMut;

/// Encodes a protobuf message into the given `BytesMut` buffer.
pub fn encode_protobuf_message_to<Msg: prost::Message>(buf: &mut BytesMut, msg: &Msg) {
    buf.reserve(msg.encoded_len().saturating_sub(buf.len()));
    buf.clear();
    // ignore the error since this can never fail
    let _ = msg.encode(buf);
}

/// Encodes a protobuf message into a new `BytesMut` buffer.
pub fn encode_protobuf_message<Msg: prost::Message>(msg: &Msg) -> BytesMut {
    let mut buf = BytesMut::new();
    encode_protobuf_message_to(&mut buf, msg);
    buf
}
