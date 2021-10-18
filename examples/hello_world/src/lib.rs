/// `hello` package protobuf definitions and service.
pub mod hello {
    hrpc::include_proto!("hello");
}

/// A boxed error.
pub type BoxError = Box<dyn std::error::Error>;
