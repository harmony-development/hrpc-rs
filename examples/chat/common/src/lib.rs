/// `chat` package protobuf definitions and service.
pub mod chat {
    hrpc::include_proto!("chat");
}

/// A boxed error.
pub type BoxError = Box<dyn std::error::Error>;
