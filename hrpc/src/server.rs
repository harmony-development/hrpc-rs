use prelude::*;
use std::{
    convert::Infallible,
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
};
use tokio::sync::{mpsc, oneshot};
use warp::{reply::Response as WarpResponse, Rejection};

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
            /// # Notes
            /// This function WILL NOT panic if `num` or `per` is invalid,
            /// ie. either of them are `0`.
            pub const fn new(num: u64, per: Duration) -> Self {
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

                    futures_util::future::ready(res)
                })
                .untuple_one()
        }

        /// Creates a filter that will return an `Err(error)` with `error`
        /// generated with the provided error function if rate limited globally,
        /// irregardless of client address.
        pub fn rate_limit_global<Err: CustomError + 'static>(
            rate: Rate,
            error: fn(Duration) -> Err,
        ) -> impl Filter<Extract = (), Error = warp::Rejection> + Sync + Send + Clone {
            let state = Arc::new(parking_lot::Mutex::new(State::new(rate)));

            warp::any()
                .and_then(move || {
                    let now = Instant::now();

                    let mut state = state.lock();
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

                    futures_util::future::ready(res)
                })
                .untuple_one()
        }
    }
}

#[doc(hidden)]
pub mod prelude {
    pub use super::{socket_common, unary_common, CustomError, ServerError, Socket};
    pub use crate::{balanced_or_tree, debug_boxed, IntoRequest, Request};
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
    use crate::Request;
    use bytes::BytesMut;
    use futures_util::{future, FutureExt};
    use std::{future::Future, net::SocketAddr};
    use warp::Filter;

    #[doc(hidden)]
    pub fn base_filter<Req, Resp, Err, PreFunc, HandlerFut, Handler>(
        pkg: &'static str,
        method: &'static str,
        pre: PreFunc,
        handler: Handler,
    ) -> impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone
    where
        Req: prost::Message + Default,
        Resp: prost::Message,
        PreFunc: Filter<Extract = (), Error = warp::Rejection> + Send + Clone,
        Err: CustomError + 'static,
        HandlerFut: Future<Output = Result<Resp, Err>> + Send,
        Handler: FnOnce(Request<Req>) -> HandlerFut + Send + Clone,
    {
        warp::post()
            .and(warp::path(pkg))
            .and(warp::path(method))
            .and(pre)
            .and(warp::header::exact_ignore_case(
                "content-type",
                "application/hrpc",
            ))
            .and(warp::body::bytes())
            .map(Req::decode)
            .and_then(move |res: Result<Req, prost::DecodeError>| {
                future::ready(res.map_err(|err| {
                    error!("received invalid protobuf message: {}", pkg);
                    warp::reject::custom(ServerError::<Err>::MessageDecode(err))
                }))
            })
            .and(warp::header::headers_cloned())
            .and(warp::addr::remote())
            .and_then(move |msg, headers, addr: Option<SocketAddr>| {
                let handler = handler.clone();
                let request = Request::from_parts((msg, headers, addr));
                handler(request).map(encode)
            })
    }

    #[doc(hidden)]
    #[inline(always)]
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
                crate::encode_protobuf_message_to(&mut buf, resp);
                let mut resp = warp::reply::Response::new(buf.to_vec().into());
                resp.headers_mut()
                    .insert(http::header::CONTENT_TYPE, crate::hrpc_header_value());
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

    use super::{CustomError, ServerError, Socket, SocketError};
    use futures_util::TryFutureExt;
    use std::{future::Future, time::Duration};
    use tracing::error;
    use warp::{ws::Ws, Filter, Reply};

    #[doc(hidden)]
    pub fn base_filter<
        Req,
        Resp,
        Err,
        PreFunc,
        ValidationValue,
        Validation,
        ValidationFut,
        OnUpgrade,
        Handler,
        HandlerFut,
    >(
        pkg: &'static str,
        method: &'static str,
        pre: PreFunc,
        validation: Validation,
        on_upgrade: OnUpgrade,
        handler: Handler,
    ) -> impl Filter<Extract = (warp::reply::Response,), Error = warp::Rejection> + Clone
    where
        Req: prost::Message + Default + Clone + 'static,
        Resp: prost::Message + 'static,
        PreFunc: Filter<Extract = (), Error = warp::Rejection> + Send + Clone,
        Err: CustomError + 'static,
        ValidationValue: Send + 'static,
        ValidationFut: Future<Output = Result<ValidationValue, Err>> + Send,
        Validation: FnOnce(Request<Option<Req>>) -> ValidationFut + Send + Clone,
        OnUpgrade: Fn(warp::reply::Response) -> warp::reply::Response + Send + Clone,
        HandlerFut: Future<Output = ()> + Send + 'static,
        Handler: FnOnce(ValidationValue, Request<Option<Req>>, Socket<Req, Resp>) -> HandlerFut
            + Send
            + Clone
            + 'static,
    {
        warp::path(pkg)
            .and(warp::path(method))
            .and(pre)
            .and(warp::ws())
            .and(warp::header::headers_cloned())
            .and(warp::addr::remote())
            .and_then(move |ws: Ws, headers: HeaderMap, addr| {
                let req = Request::from_parts((None, headers, addr));
                let validation = (validation.clone())(req.clone());
                validation
                    .map_err(|err| warp::reject::custom(ServerError::Custom(err)))
                    .map_ok(move |val| (val, req, ws))
            })
            .untuple_one()
            .map(move |val, req: Request<Option<Req>>, ws: Ws| {
                let handler = handler.clone();
                let reply = ws
                    .on_upgrade(move |ws| {
                        let sock = Socket::<Req, Resp>::new(ws);
                        handler(val, req, sock)
                    })
                    .into_response();
                on_upgrade(reply)
            })
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
            let message = sock.receive_message().await?;

            let (_, headers, addr) = req.into_parts();
            let req = Request::from_parts((Some(message), headers, addr));

            match validator_func(req).await {
                Ok(val) => Ok(val),
                Err(err) => {
                    error!("socket validation error: {}", err);
                    Err(SocketError::Closed)
                }
            }
        };

        tokio::time::timeout(Duration::from_secs(10), recv_fut)
            .await
            .map_err(|_| {
                error!("socket validation request error: timeout");
                SocketError::Closed
            })?
    }
}

type SenderChanWithReq<Resp> = (Resp, oneshot::Sender<Result<(), SocketError>>);

/// A web socket.
#[derive(Debug, Clone)]
pub struct Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    rx: flume::Receiver<Result<Req, SocketError>>,
    tx: mpsc::Sender<SenderChanWithReq<Resp>>,
    close_chan: mpsc::Sender<()>,
}

impl<Req, Resp> Socket<Req, Resp>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
{
    pub fn new(mut ws: warp::ws::WebSocket) -> Self {
        let (recv_msg_tx, recv_msg_rx) = flume::bounded(64);
        let (send_msg_tx, mut send_msg_rx): (
            mpsc::Sender<SenderChanWithReq<Resp>>,
            mpsc::Receiver<SenderChanWithReq<Resp>>,
        ) = mpsc::channel(64);
        let (close_chan_tx, mut close_chan_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let mut buf = BytesMut::new();
            loop {
                tokio::select! {
                    Some(res_msg) = ws.next() => {
                        let resp = match res_msg {
                            Ok(msg) => {
                                if msg.is_binary() || msg.is_text() {
                                    let msg_bin = Bytes::from(msg.into_bytes());
                                    Req::decode(msg_bin).map_err(SocketError::MessageDecode)
                                } else if msg.is_close() {
                                    let _ = recv_msg_tx.send_async(Err(SocketError::Closed)).await;
                                    let _ = ws.close().await;
                                    return;
                                } else {
                                    continue;
                                }
                            }
                            Err(err) => {
                                let is_capped = err.to_string().contains("Space limit exceeded");
                                let res = Err(SocketError::from(err));
                                if !is_capped {
                                    let _ = recv_msg_tx.send_async(res).await;
                                    let _ = ws.close().await;
                                    return;
                                } else {
                                    res
                                }
                            }
                        };
                        if recv_msg_tx.send_async(resp).await.is_err() {
                            let _ = ws.close().await;
                            return;
                        }
                    }
                    Some((resp, chan)) = send_msg_rx.recv() => {
                        let resp = {
                            crate::encode_protobuf_message_to(&mut buf, resp);
                            buf.to_vec()
                        };

                        if let Err(e) = ws.send(WsMessage::binary(resp)).await {
                            debug!("socket send error: {}", e);
                            let msg = e.to_string();
                            let is_capped_or_queue_full = msg.contains("Space limit exceeded") || msg.contains("Send queue is full");
                            let _ = chan.send(Err(SocketError::from(e)));
                            if !is_capped_or_queue_full {
                                let _ = ws.close().await;
                                return;
                            }
                        } else {
                            debug!("responded to client socket");
                            if chan.send(Ok(())).is_err() {
                                let _ = ws.close().await;
                                return;
                            }
                        }
                    }
                    // If we get *anything*, it means that either the channel is closed
                    // or we got a close message
                    _ = close_chan_rx.recv() => {
                        if let Err(err) = ws.close().await {
                            let _ = recv_msg_tx.send_async(Err(SocketError::from(err))).await;
                        }
                        return;
                    }
                    else => tokio::task::yield_now().await,
                }
            }
        });

        Self {
            rx: recv_msg_rx,
            tx: send_msg_tx,
            close_chan: close_chan_tx,
        }
    }

    /// Receive a message from the socket.
    ///
    /// Returns `Err(SocketError::ClosedNormally)` if the socket is closed normally.
    /// This will block until getting a message if the socket is not closed.
    pub async fn receive_message(&self) -> Result<Req, SocketError> {
        if self.is_closed() {
            Err(SocketError::Closed)
        } else {
            self.rx
                .recv_async()
                .await
                .unwrap_or(Err(SocketError::Closed))
        }
    }

    /// Send a message over the socket.
    ///
    /// This will block if the inner send buffer is filled.
    pub async fn send_message(&self, resp: Resp) -> Result<(), SocketError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        if self.is_closed() || self.tx.send((resp, resp_tx)).await.is_err() {
            Err(SocketError::Closed)
        } else {
            resp_rx.await.unwrap_or(Err(SocketError::Closed))
        }
    }

    /// Return whether the socket is closed or not.
    pub fn is_closed(&self) -> bool {
        self.close_chan.is_closed()
    }

    /// Close the socket.
    pub async fn close(&self) {
        // We don't care about the error, it's closed either way
        let _ = self.close_chan.send(()).await;
    }
}

/// Trait that needs to be implemented to use an error type with a generated service server.
pub trait CustomError: Debug + Display + Send + Sync {
    /// Status code that will be used in client response.
    fn code(&self) -> StatusCode;
    /// Message that will be used in client response.
    fn message(&self) -> Vec<u8>;

    /// Status code and error body used to respond when a protobuf decode error occurs.
    const DECODE_ERROR: (StatusCode, &'static [u8]) = (
        StatusCode::BAD_REQUEST,
        r#"{ "message": "invalid protobuf message" }"#.as_bytes(),
    );
    /// Status code and error body used to respond when a not found error occurs.
    const NOT_FOUND_ERROR: (StatusCode, &'static [u8]) = (
        StatusCode::NOT_FOUND,
        r#"{ "message": "not found" }"#.as_bytes(),
    );
    /// Status code and error body used to respond when a method not allowed error occurs.
    const METHOD_NOT_ALLOWED: (StatusCode, &'static [u8]) = (
        StatusCode::METHOD_NOT_ALLOWED,
        r#"{ "message": "method not allowed" }"#.as_bytes(),
    );
    /// Status code and error body used to respond when an internal server error occurs.
    const INTERNAL_SERVER_ERROR: (StatusCode, &'static [u8]) = (
        StatusCode::INTERNAL_SERVER_ERROR,
        r#"{ "message": "internal server error" }"#.as_bytes(),
    );
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
        if matches!(res, Err($crate::server::SocketError::Closed)) {
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
    /// The socket is closed.
    Closed,
    /// Error occured while decoding protobuf data.
    MessageDecode(prost::DecodeError),
    /// Some error occured in socket.
    Other(warp::Error),
}

impl Display for SocketError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SocketError::Closed => write!(f, "socket is closed"),
            SocketError::Other(err) => write!(f, "error occured in socket: {}", err),
            SocketError::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
        }
    }
}

impl StdError for SocketError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SocketError::Closed => None,
            SocketError::Other(err) => Some(err),
            SocketError::MessageDecode(err) => Some(err),
        }
    }
}

impl From<warp::Error> for SocketError {
    fn from(err: warp::Error) -> Self {
        let msg = err.to_string();
        if msg.contains("Connection reset")
            || msg.contains("Connection closed")
            || msg.contains("closed connection")
        {
            SocketError::Closed
        } else {
            SocketError::Other(err)
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum ServerError<Err: CustomError> {
    MessageDecode(prost::DecodeError),
    Custom(Err),
    Warp(warp::Rejection),
}

impl<Err: CustomError> Display for ServerError<Err> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerError::MessageDecode(err) => write!(f, "invalid protobuf message: {}", err),
            ServerError::Custom(err) => write!(f, "error occured: {}", err),
            ServerError::Warp(err) => write!(f, "warp rejection: {:?}", err),
        }
    }
}

impl<Err: CustomError + StdError + 'static> StdError for ServerError<Err> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ServerError::MessageDecode(err) => Some(err),
            ServerError::Custom(err) => Some(err),
            ServerError::Warp(_) => None,
        }
    }
}

impl<Err: CustomError + 'static> warp::reject::Reject for ServerError<Err> {}

impl<Err: CustomError> From<Err> for ServerError<Err> {
    fn from(err: Err) -> Self {
        ServerError::Custom(err)
    }
}

impl<Err: CustomError> From<prost::DecodeError> for ServerError<Err> {
    fn from(err: prost::DecodeError) -> Self {
        ServerError::MessageDecode(err)
    }
}

impl<Err: CustomError> From<warp::Rejection> for ServerError<Err> {
    fn from(err: warp::Rejection) -> Self {
        ServerError::Warp(err)
    }
}

#[doc(hidden)]
pub async fn handle_rejection<Err: CustomError + 'static>(
    err: Rejection,
) -> Result<WarpResponse, Infallible> {
    fn make_response(data: (StatusCode, impl Into<warp::hyper::Body>)) -> WarpResponse {
        let mut reply = WarpResponse::new(data.1.into());
        *reply.status_mut() = data.0;
        reply
    }

    fn find_error<Err: CustomError + 'static>(rejection: &Rejection) -> WarpResponse {
        if let Some(e) = rejection.find::<ServerError<Err>>() {
            match e {
                ServerError::MessageDecode(_) => make_response(Err::DECODE_ERROR),
                ServerError::Custom(err) => make_response((err.code(), err.message())),
                ServerError::Warp(err) => find_error::<Err>(err),
            }
        } else if rejection.is_not_found() {
            make_response(Err::NOT_FOUND_ERROR)
        } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
            make_response(Err::METHOD_NOT_ALLOWED)
        } else {
            error!("unhandled rejection: {:?}", rejection);
            make_response(Err::INTERNAL_SERVER_ERROR)
        }
    }

    let reply = find_error::<Err>(&err);

    Ok(reply)
}

/// Creates a JSON error response from a message.
pub fn json_err_bytes(msg: &str) -> Vec<u8> {
    format!(r#"{{ "message": "{}" }}"#, msg).into_bytes()
}

/// Serves multiple services' filters on the same address.
#[macro_export]
macro_rules! serve_multiple {
    {
        addr: $address:expr,
        err: $err:ty,
        filters: $( $filter:expr, )+
    } => {
        async move {
            use $crate::warp::Filter;

            $crate::warp::serve(
                balanced_or_tree!( $( $filter:expr ),+ )
                    .with($crate::warp::filters::trace::request())
                    .with(warp::compression::gzip())
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
        filters: $( $filter:expr, )+
    } => {
        async move {
            use $crate::warp::Filter;

            $crate::warp::serve(
                balanced_or_tree!( $( $filter:expr ),+ )
                    .with($crate::warp::filters::trace::request())
                    .with(warp::compression::gzip())
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

// Thanks to https://github.com/seanmonstar/warp/issues/619#issuecomment-662716377
/// Takes a list of handler expressions and `or`s them together
/// in a balanced tree. That is, instead of `a.or(b).or(c).or(d)`,
/// it produces `(a.or(b)).or(c.or(d))`, thus nesting the types
/// less deeply, which provides improvements in compile time.
///
/// It also applies `::warp::Filter::boxed` to each handler expression
/// when in `debug_assertions` mode, improving compile time further.
#[macro_export]
macro_rules! balanced_or_tree {
    ($x:expr $(,)?) => { $crate::debug_boxed!($x) };
    ($($x:expr),+ $(,)?) => {
        balanced_or_tree!(@internal ; $($x),+; $($x),+)
    };
    (@internal $($left:expr),*; $head:expr, $($tail:expr),+; $a:expr $(,$b:expr)?) => {
        (balanced_or_tree!($($left,)* $head)).or(balanced_or_tree!($($tail),+))
    };
    (@internal $($left:expr),*; $head:expr, $($tail:expr),+; $a:expr, $b:expr, $($more:expr),+) => {
        balanced_or_tree!(@internal $($left,)* $head; $($tail),+; $($more),+)
    };
}

#[cfg(debug_assertions)]
#[allow(unused_macros)]
#[macro_export]
macro_rules! debug_boxed {
    ($x:expr) => {
        $crate::warp::Filter::boxed($x)
    };
}

#[cfg(not(debug_assertions))]
#[allow(unused_macros)]
#[macro_export]
macro_rules! debug_boxed {
    ($x:expr) => {
        $x
    };
}
