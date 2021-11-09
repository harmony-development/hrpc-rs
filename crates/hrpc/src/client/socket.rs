#[cfg(feature = "disable_socket_auto_ping")]
#[doc(inline)]
pub use crate::common::socket::{ReadSocket, Socket, WriteSocket};

#[cfg(not(feature = "disable_socket_auto_ping"))]
pub use crate::common::socket::Socket;
