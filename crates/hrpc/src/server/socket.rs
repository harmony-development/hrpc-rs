use futures_util::future::BoxFuture;

use crate::common::socket::{BoxedWsRx, BoxedWsTx};

#[cfg(feature = "disable_socket_auto_ping")]
#[doc(inline)]
pub use crate::common::socket::{ReadSocket, Socket, WriteSocket};

#[cfg(not(feature = "disable_socket_auto_ping"))]
pub use crate::common::socket::Socket;

pub(crate) struct SocketHandler {
    #[allow(dead_code)]
    pub(crate) inner:
        Box<dyn FnOnce(BoxedWsRx, BoxedWsTx) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
}
