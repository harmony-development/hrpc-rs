use futures_util::future::BoxFuture;

use crate::common::socket::{BoxedWsRx, BoxedWsTx};

#[doc(inline)]
pub use crate::common::socket::{ReadSocket, Socket, WriteSocket};

pub(crate) struct SocketHandler {
    #[allow(dead_code)]
    pub(crate) inner:
        Box<dyn FnOnce(BoxedWsRx, BoxedWsTx) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
}
