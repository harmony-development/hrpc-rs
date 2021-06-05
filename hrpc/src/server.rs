use futures_util::stream::{SplitSink, SplitStream};
use prelude::*;
use std::{
    convert::Infallible,
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};
use warp::{ws::WebSocket, Rejection, Reply};

/// Useful filters.
pub mod filters {
    use super::*;

    pub use rate::rate_limit;

    pub mod rate {
        use super::*;

        use std::{net::SocketAddr, sync::Arc, time::Duration};

        use warp::Filter;

        /// A rate of requests per time period.
        #[derive(Debug, Copy, Clone)]
        pub struct Rate {
            num: u64,
            per: Duration,
        }

        impl Rate {
            /// Create a new rate.
            ///
            /// # Panics
            ///
            /// This function panics if `num` or `per` is 0.
            pub fn new(num: u64, per: Duration) -> Self {
                assert!(num > 0);
                assert!(per > Duration::from_millis(0));

                Rate { num, per }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct State {
            until: Instant,
            rem: u64,
        }

        impl State {
            pub fn new(rate: Rate) -> Self {
                Self {
                    until: Instant::now() + rate.per,
                    rem: rate.num,
                }
            }
        }

        /// Creates a filter that will return an `Err(error)` with `error`
        /// generated with the provided error function if rate limited.
        pub fn rate_limit<Err: CustomError + 'static>(
            rate: Rate,
            error: fn(Duration) -> Err,
        ) -> impl Filter<Extract = (), Error = warp::Rejection> + Sync + Send + Clone {
            let states = Arc::new(dashmap::DashMap::with_hasher(ahash::RandomState::new()));

            warp::addr::remote()
                .and_then(move |addr: Option<SocketAddr>| {
                    let now = Instant::now();

                    let mut state = states.entry(addr).or_insert_with(|| State::new(rate));
                    if now >= state.until {
                        state.until = now + rate.per;
                        state.rem = rate.num;
                    }

                    let res = if state.rem >= 1 {
                        state.rem -= 1;
                        Ok(())
                    } else {
                        Err(state.until - now)
                    };
                    drop(state);

                    let res =
                        res.map_err(|dur| warp::reject::custom(ServerError::Custom(error(dur))));

                    async move { res }
                })
                .untuple_one()
        }
    }
}

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        socket_common, unary_common, CustomError, ReadSocket, ServerError, Socket, WriteSocket,
    };
    pub use crate::{IntoRequest, Request};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{FutureExt, SinkExt, StreamExt};
    pub use http::HeaderMap;
    pub use std::{
        collections::HashMap,
        time::{Duration, Instant},
    };
    pub use tokio::{self, sync::Mutex, try_join};
    pub use tracing::{debug, error, info, info_span, trace, warn};
    pub use warp::{
        self,
        filters::BoxedFilter,
        reject::Reject,
        reply::Response,
        ws::{Message as WsMessage, Ws},
        Filter, Reply,
    };
}

#[doc(inline)]
pub use warp::http::StatusCode;

#[doc(hidden)]
pub mod unary_common {
    use super::{error, CustomError, ServerError};
    use bytes::BytesMut;
    use http::HeaderMap;
    use warp::Filter;

    #[doc(hidden)]
    pub fn base_filter<Req, Resp, Err, PreFunc>(
        pkg: &'static str,
        method: &'static str,
        pre: PreFunc,
    ) -> impl Filter<Extract = (Req, HeaderMap), Error = warp::Rejection> + Clone
    where
        Req: prost::Message + Default,
        Resp: prost::Message,
        PreFunc: Filter<Extract = (), Error = warp::Rejection> + Send + Clone,
        Err: CustomError + 'static,
    {
        warp::path(pkg)
            .and(warp::path(method))
            .and(warp::path::end())
            .and(pre)
            .and(warp::body::bytes())
            .and(warp::header::exact_ignore_case(
                "content-type",
                "application/hrpc",
            ))
            .map(Req::decode)
            .and_then(move |res: Result<Req, prost::DecodeError>| async move {
                res.map_err(|err| {
                    error!("received invalid protobuf message: {}", pkg);
                    warp::reject::custom(ServerError::<Err>::MessageDecode(err))
                })
            })
            .and(warp::header::headers_cloned())
    }

    #[doc(hidden)]
    pub fn encode<Resp, Err>(
        resp: Result<Resp, Err>,
    ) -> Result<warp::reply::Response, warp::Rejection>
    where
        Resp: prost::Message,
        Err: CustomError + 'static,
    {
        match resp {
            Ok(resp) => {
                let mut buf = BytesMut::with_capacity(resp.encoded_len());
                crate::encode_protobuf_message(&mut buf, resp);
                let mut resp = warp::reply::Response::new(buf.to_vec().into());
                resp.headers_mut()
                    .entry("content-type")
                    .or_insert("application/hrpc".parse().unwrap());
                Ok(resp)
            }
            Err(err) => {
                error!("{}", err);
                Err(warp::reject::custom(ServerError::Custom(err)))
            }
        }
    }
}

#[doc(hidden)]
pub mod socket_common {
    use crate::{HeaderMap, Request};

    use super::{CustomError, Socket, SocketError};
    use std::{future::Future, time::Duration};
    use tracing::error;
    use warp::{ws::Ws, Filter};

    #[doc(hidden)]
    pub fn base_filter<Err, PreFunc>(
        pkg: &'static str,
        method: &'static str,
        pre: PreFunc,
    ) -> impl Filter<Extract = (HeaderMap, Ws), Error = warp::Rejection> + Clone
    where
        PreFunc: Filter<Extract = (), Error = warp::Rejection> + Send + Clone,
        Err: CustomError + 'static,
    {
        warp::path(pkg)
            .and(warp::path(method))
            .and(warp::path::end())
            .and(pre)
            .and(warp::header::headers_cloned())
            .and(warp::ws())
    }

    #[doc(hidden)]
    pub async fn validator<Req, Resp, Err, Val, Fut, Func>(
        req: Request<Option<Req>>,
        sock: &mut Socket<Req, Resp>,
        validator_func: Func,
    ) -> Result<Val, SocketError>
    where
        Req: prost::Message + Default,
        Resp: prost::Message,
        Fut: Future<Output = Result<Val, Err>>,
        Func: FnOnce(Request<Option<Req>>) -> Fut,
        Err: CustomError + 'static,
    {
        let recv_fut = async move {
            let val;
            loop {
                if let Some(message) = sock.receive_message().await? {
                    let req = Request::from_parts((Some(message), req.into_parts().1));
                    match validator_func(req).await {
                        Ok(vall) => {
                            val = vall;
                            break;
                        }
                        Err(err) => {
                            error!("socked validation error: {}", err);
                            return Err(SocketError::ClosedNormally);
                        }
                    }
                }
            }
            Ok(val)
        };

        tokio::time::timeout(Duration::from_secs(10), recv_fut)
            .await
            .map_err(|_| {
                error!("socket validation request error: timeout");
                SocketError::ClosedNormally
            })?
    }
}

/// A cloneable web socket.
#[derive(Debug, Clone)]
pub struct SocketArc<Req: prost::Message + Default, Resp: prost::Message> {
    read: ReadSocketArc<Req>,
    write: WriteSocketArc<Resp>,
}

impl<Req: prost::Message + Default, Resp: prost::Message> SocketArc<Req, Resp> {
    /// Combine read and write sockets into one.
    pub fn combine(read: ReadSocketArc<Req>, write: WriteSocketArc<Resp>) -> Self {
        Self { read, write }
    }

    /// Split this socket into read and write counterparts.
    pub fn split(self) -> (ReadSocketArc<Req>, WriteSocketArc<Resp>) {
        (self.read, self.write)
    }

    /// Receive a message from the socket.
    ///
    /// Returns `Err(SocketError::ClosedNormally)` if the socket is closed normally.
    pub async fn receive_message(&self) -> Result<Option<Req>, SocketError> {
        self.read.receive_message().await
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, resp: Resp) -> Result<(), SocketError> {
        self.write.send_message(resp).await
    }
}

/// A web socket.
#[derive(Debug)]
pub struct Socket<Req: prost::Message + Default, Resp: prost::Message> {
    read: ReadSocket<Req>,
    write: WriteSocket<Resp>,
}

impl<Req: prost::Message + Default, Resp: prost::Message> Socket<Req, Resp> {
    pub fn new(ws: warp::ws::WebSocket) -> Self {
        let (tx, rx) = ws.split();
        Self {
            read: ReadSocket::new(rx),
            write: WriteSocket::new(tx),
        }
    }

    /// Combine read and write sockets into one.
    pub fn combine(read: ReadSocket<Req>, write: WriteSocket<Resp>) -> Self {
        Self { read, write }
    }

    /// Split this socket into read and write counterparts.
    pub fn split(self) -> (ReadSocket<Req>, WriteSocket<Resp>) {
        (self.read, self.write)
    }

    /// Make a cloneable and shared version of this socket.
    pub fn clonable(self) -> SocketArc<Req, Resp> {
        SocketArc {
            read: self.read.clonable(),
            write: self.write.clonable(),
        }
    }

    /// Receive a message from the socket.
    ///
    /// Returns `Err(SocketError::ClosedNormally)` if the socket is closed normally.
    pub async fn receive_message(&mut self) -> Result<Option<Req>, SocketError> {
        self.read.receive_message().await
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, resp: Resp) -> Result<(), SocketError> {
        self.write.send_message(resp).await
    }
}

/// A clonable read only socket.
#[derive(Debug, Clone)]
pub struct ReadSocketArc<Req: prost::Message + Default> {
    inner: Arc<Mutex<ReadSocket<Req>>>,
}

impl<Req: prost::Message + Default> ReadSocketArc<Req> {
    /// Receive a message from the socket.
    ///
    /// Returns `Err(SocketError::ClosedNormally)` if the socket is closed normally.
    pub async fn receive_message(&self) -> Result<Option<Req>, SocketError> {
        self.inner.lock().await.receive_message().await
    }
}

/// A read only socket.
#[derive(Debug)]
pub struct ReadSocket<Req: prost::Message + Default> {
    rx: SplitStream<WebSocket>,
    _req: PhantomData<Req>,
}

impl<Req: prost::Message + Default> ReadSocket<Req> {
    pub fn new(rx: SplitStream<WebSocket>) -> Self {
        Self {
            rx,
            _req: PhantomData,
        }
    }

    /// Create a clonable and shared version of ReadSocket.
    pub fn clonable(self) -> ReadSocketArc<Req> {
        ReadSocketArc {
            inner: Arc::new(Mutex::new(self)),
        }
    }

    /// Receive a message from the socket.
    ///
    /// Returns `Err(SocketError::ClosedNormally)` if the socket is closed normally.
    pub async fn receive_message(&mut self) -> Result<Option<Req>, SocketError> {
        let msg_maybe = tokio::time::timeout(Duration::from_nanos(1), self.rx.next())
            .await
            .map_or(Ok(None), Ok)?
            .transpose()
            .map_err(SocketError::Other)?;

        if let Some(msg) = msg_maybe {
            if msg.is_binary() {
                let msg_bin = Bytes::from(msg.into_bytes());
                return Req::decode(msg_bin)
                    .map(Some)
                    .map_err(SocketError::MessageDecode);
            } else if msg.is_close() {
                return Err(SocketError::ClosedNormally);
            }
        }
        Ok(None)
    }
}

/// A clonable write only socket.
#[derive(Debug)]
pub struct WriteSocketArc<Resp: prost::Message> {
    tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    buf: BytesMut,
    _resp: PhantomData<Resp>,
}

impl<Resp: prost::Message> WriteSocketArc<Resp> {
    /// Send a message over the socket.
    pub async fn send_message(&mut self, resp: Resp) -> Result<(), SocketError> {
        let resp = {
            crate::encode_protobuf_message(&mut self.buf, resp);
            self.buf.to_vec()
        };

        if let Err(e) = self.tx.lock().await.send(WsMessage::binary(resp)).await {
            return Err(if e.to_string().contains("closed normally") {
                SocketError::ClosedNormally
            } else {
                SocketError::Other(e)
            });
        } else {
            debug!("responded to client socket");
        }

        Ok(())
    }
}

impl<Resp: prost::Message> Clone for WriteSocketArc<Resp> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            buf: BytesMut::with_capacity(self.buf.capacity()),
            _resp: PhantomData,
        }
    }
}

/// A write only socket.
#[derive(Debug)]
pub struct WriteSocket<Resp: prost::Message> {
    tx: SplitSink<WebSocket, WsMessage>,
    buf: BytesMut,
    _resp: PhantomData<Resp>,
}

impl<Resp: prost::Message> WriteSocket<Resp> {
    pub fn new(tx: SplitSink<WebSocket, WsMessage>) -> Self {
        Self {
            tx,
            buf: BytesMut::new(),
            _resp: PhantomData,
        }
    }

    /// Create a clonable and shared version of WriteSocket.
    pub fn clonable(self) -> WriteSocketArc<Resp> {
        WriteSocketArc {
            tx: Arc::new(Mutex::new(self.tx)),
            buf: self.buf,
            _resp: self._resp,
        }
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, resp: Resp) -> Result<(), SocketError> {
        let resp = {
            crate::encode_protobuf_message(&mut self.buf, resp);
            self.buf.to_vec()
        };

        if let Err(e) = self.tx.send(WsMessage::binary(resp)).await {
            return Err(if e.to_string().contains("closed normally") {
                SocketError::ClosedNormally
            } else {
                SocketError::Other(e)
            });
        } else {
            debug!("responded to client socket");
        }

        Ok(())
    }
}

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: Debug + Display + Send + Sync {
    /// Status code that will be used in client response.
    fn code(&self) -> StatusCode;
    /// Message that will be used in client response.
    fn message(&self) -> Vec<u8>;
    /// Return a message decode error.
    fn decode_error() -> (StatusCode, Vec<u8>) {
        (
            StatusCode::BAD_REQUEST,
            json_err_bytes("invalid protobuf message"),
        )
    }
    /// Return a not fonud error.
    fn not_found_error() -> (StatusCode, Vec<u8>) {
        (StatusCode::NOT_FOUND, json_err_bytes("not found"))
    }
    /// Return an internal server error.
    fn internal_server_error() -> (StatusCode, Vec<u8>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json_err_bytes("internal server error"),
        )
    }
}

impl CustomError for std::convert::Infallible {
    fn code(&self) -> StatusCode {
        unreachable!()
    }

    fn message(&self) -> Vec<u8> {
        unreachable!()
    }
}

/// Return if the socket is closed normally, otherwise return the result.
#[macro_export]
macro_rules! return_closed {
    ($result:expr) => {{
        let res = $result;
        if matches!(res, Err($crate::server::SocketError::ClosedNormally)) {
            return;
        } else {
            res
        }
    }};
}

/// Return if the socket is closed normally, otherwise print the error if there is one and return.
#[macro_export]
macro_rules! return_print {
    ($result:expr) => {{
        let res = $crate::return_closed!($result);
        if let Err(err) = res {
            $crate::tracing::error!("error occured: {}", err);
            return;
        }
    }};
    ($result:expr, |$val:ident| $log:expr) => {{
        let res = $crate::return_closed!($result);
        match res {
            Err(err) => {
                $crate::tracing::error!("error occured: {}", err);
                return;
            }
            Ok($val) => $log,
        }
    }};
}

#[derive(Debug)]
pub enum SocketError {
    /// The socket is closed normally. This is NOT an error.
    ClosedNormally,
    /// Error occured while decoding protobuf data.
    MessageDecode(prost::DecodeError),
    /// Some error occured in socket.
    Other(warp::Error),
}

impl Display for SocketError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SocketError::ClosedNormally => write!(f, "socket closed normally"),
            SocketError::Other(err) => write!(f, "error occured in socket: {}", err),
            SocketError::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
        }
    }
}

impl StdError for SocketError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SocketError::ClosedNormally => None,
            SocketError::Other(err) => Some(err),
            SocketError::MessageDecode(err) => Some(err),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum ServerError<Err: CustomError> {
    MessageDecode(prost::DecodeError),
    Custom(Err),
}

impl<Err: CustomError> Display for ServerError<Err> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerError::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
            ServerError::Custom(err) => write!(f, "error occured: {}", err),
        }
    }
}

impl<Err: CustomError + StdError + 'static> StdError for ServerError<Err> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ServerError::MessageDecode(err) => Some(err),
            ServerError::Custom(err) => Some(err),
        }
    }
}

impl<Err: CustomError + 'static> warp::reject::Reject for ServerError<Err> {}

#[doc(hidden)]
pub async fn handle_rejection<Err: CustomError + 'static>(
    err: Rejection,
) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        Err::not_found_error()
    } else if let Some(e) = err.find::<ServerError<Err>>() {
        match e {
            ServerError::MessageDecode(_) => Err::decode_error(),
            ServerError::Custom(err) => (err.code(), err.message()),
        }
    } else {
        error!("unhandled rejection: {:?}", err);
        Err::internal_server_error()
    };

    let mut reply = warp::reply::Response::new(message.into());
    *reply.status_mut() = code;

    Ok(reply)
}

/// Creates a JSON error response from a message.
pub fn json_err_bytes(msg: &str) -> Vec<u8> {
    format!("{{ \"message\": \"{}\" }}", msg).into_bytes()
}

/// Serves multiple services' filters on the same address.
#[macro_export]
macro_rules! serve_multiple {
    {
        addr: $address:expr,
        err: $err:ty,
        filters: $first:expr, $( $filter:expr, )+
    } => {
        async move {
            use $crate::warp::Filter;

            $crate::warp::serve(
                $first $( .or($filter) )+
                    .with($crate::warp::filters::trace::request())
                    .recover($crate::server::handle_rejection::<$err>)
            )
            .run($address)
            .await
        }
    };
}

/// Serves multiple services' filters on the same address. Supports TLS.
#[macro_export]
macro_rules! serve_multiple_tls {
    {
        addr: $address:expr,
        err: $err:ty,
        key_file: $key_file:expr,
        cert_file: $cert_file:expr,
        filters: $first:expr, $( $filter:expr, )+
    } => {
        async move {
            use $crate::warp::Filter;

            $crate::warp::serve(
                $first $( .or($filter) )+
                    .with($crate::warp::filters::trace::request())
                    .recover($crate::server::handle_rejection::<$err>)
            )
            .tls()
            .key_path($key_file)
            .cert_path($cert_file)
            .run($address)
            .await
        }
    };
}
