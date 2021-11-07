pub use crate::proto::Error as HrpcError;

/// Shorthand type for `Result<T, HrpcError>.
pub type ServerResult<T> = Result<T, HrpcError>;
