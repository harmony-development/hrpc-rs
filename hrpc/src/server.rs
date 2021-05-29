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
use tokio::sync::Mutex;
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
        ) -> BoxedFilter<(Result<(), Err>,)> {
            let states = Arc::new(dashmap::DashMap::with_hasher(ahash::RandomState::new()));

            warp::any()
                .and(warp::addr::remote())
                .map(move |addr: Option<SocketAddr>| {
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

                    res.map_err(error)
                })
                .boxed()
        }
    }
}

#[doc(hidden)]
pub mod prelude {
    pub use super::{socket_common, unary_common, CustomError, ServerError, Socket};
    pub use crate::{IntoRequest, Request};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{SinkExt, StreamExt};
    pub use http::HeaderMap;
    pub use parking_lot::Mutex as PMutex;
    pub use std::{collections::HashMap, time::Instant};
    pub use tokio::{sync::Mutex, try_join};
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
    use crate::HeaderMap;

    use super::{error, CustomError, ServerError};
    use bytes::BytesMut;
    use warp::{filters::BoxedFilter, Filter};

    #[doc(hidden)]
    pub fn base_filter<Resp, Err>(
        pkg: &'static str,
        method: &'static str,
        pre: BoxedFilter<(Result<(), Err>,)>,
    ) -> BoxedFilter<(Resp, HeaderMap)>
    where
        Resp: prost::Message + Default,
        Err: CustomError + 'static,
    {
        warp::path(pkg)
            .and(warp::path(method))
            .and(warp::path::end())
            .and(pre)
            .and_then(|res: Result<(), Err>| async move {
                res.map_err(|err| warp::reject::custom(ServerError::<Err>::Custom(err)))
            })
            .and(warp::body::bytes())
            .and(warp::header::exact_ignore_case(
                "content-type",
                "application/hrpc",
            ))
            .and_then(move |_, bin| async move {
                Resp::decode(bin).map_err(|err| {
                    error!("received invalid protobuf message: {}", pkg);
                    warp::reject::custom(ServerError::<Err>::MessageDecode(err))
                })
            })
            .and(
                warp::header::headers_cloned().map(|headers: http::HeaderMap| {
                    headers
                        .into_iter()
                        .filter_map(|(k, v)| Some((k?, v)))
                        .collect()
                }),
            )
            .boxed()
    }

    #[doc(hidden)]
    pub fn encode<Resp: prost::Message, Err: CustomError + 'static>(
        resp: Result<Resp, Err>,
    ) -> Result<http::Response<warp::hyper::Body>, warp::Rejection> {
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
    use std::time::Instant;

    use crate::HeaderMap;

    use super::{debug, error, info, CustomError, ServerError};
    use futures_util::SinkExt;
    use warp::{
        filters::BoxedFilter,
        ws::{Message as WsMessage, Ws},
        Filter,
    };

    #[doc(hidden)]
    pub fn base_filter<Err: CustomError + 'static>(
        pkg: &'static str,
        method: &'static str,
        pre: BoxedFilter<(Result<(), Err>,)>,
    ) -> BoxedFilter<((), HeaderMap, Ws)> {
        warp::path(pkg)
            .and(warp::path(method))
            .and(warp::path::end())
            .and(pre)
            .and_then(|res: Result<(), Err>| async move {
                res.map_err(|err| warp::reject::custom(ServerError::<Err>::Custom(err)))
            })
            .and(
                warp::header::headers_cloned().map(|headers: http::HeaderMap| {
                    headers
                        .into_iter()
                        .filter_map(|(k, v)| Some((k?, v)))
                        .collect()
                }),
            )
            .and(warp::ws())
            .boxed()
    }

    #[doc(hidden)]
    pub fn process_validate<Err: CustomError + 'static>(
        result: Result<(), Err>,
    ) -> Result<(), ServerError<Err>> {
        result.map_err(|err| {
            error!("socket request validation failed: {}", err);
            ServerError::Custom(err)
        })
    }

    #[doc(hidden)]
    pub async fn check_ping<'a>(
        tx: &'a mut futures_util::stream::SplitSink<warp::ws::WebSocket, WsMessage>,
        last_ping_time: &'a mut Instant,
        socket_ping_data: [u8; 32],
        socket_ping_period: u64,
    ) -> bool {
        let ping_lapse = last_ping_time.elapsed();
        if ping_lapse.as_secs() >= socket_ping_period {
            if let Err(e) = tx.send(WsMessage::ping(socket_ping_data)).await {
                // TODO: replace this with error source downcasting?
                if "Connection closed normally" == e.to_string() {
                    info!("socket closed normally");
                } else {
                    error!("error pinging client socket: {}", e);
                    error!("can't reach client, closing socket");
                }
                return true;
            } else {
                debug!(
                    "pinged client socket, last ping was {} ago",
                    last_ping_time.elapsed().as_secs()
                );
            }
            *last_ping_time = Instant::now();
        }
        false
    }

    #[doc(hidden)]
    pub async fn close_socket(
        tx: &mut futures_util::stream::SplitSink<warp::ws::WebSocket, WsMessage>,
    ) {
        if let Err(e) = tx.send(WsMessage::close()).await {
            error!("error closing socket: {}", e);
        } else {
            info!("socket closed normally");
        }
    }
}

#[derive(Debug)]
pub struct Socket<Req: prost::Message + Default, Resp: prost::Message> {
    tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    rx: SplitStream<WebSocket>,
    buf: BytesMut,
    socket_ping_data: [u8; 32],
    _resp: PhantomData<Resp>,
    _req: PhantomData<Req>,
}

impl<Req: prost::Message + Default, Resp: prost::Message> Socket<Req, Resp> {
    #[doc(hidden)]
    pub fn new(
        tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
        rx: SplitStream<WebSocket>,
        socket_ping_data: [u8; 32],
    ) -> Self {
        Self {
            tx,
            rx,
            buf: BytesMut::new(),
            socket_ping_data,
            _req: PhantomData::default(),
            _resp: PhantomData::default(),
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
            } else if msg.is_pong() {
                let msg_bin = Bytes::from(msg.into_bytes());
                if self.socket_ping_data != msg_bin.as_ref() {
                    socket_common::close_socket(&mut *self.tx.lock().await).await;
                    return Err(SocketError::ClosedNormally);
                } else {
                    debug!("received pong");
                }
            } else if msg.is_close() {
                return Err(SocketError::ClosedNormally);
            }
        }
        Ok(None)
    }

    /// Send a message over the socket.
    pub async fn send_message(&mut self, resp: Resp) -> Result<(), SocketError> {
        let resp = {
            crate::encode_protobuf_message(&mut self.buf, resp);
            self.buf.to_vec()
        };

        if let Err(e) = self.tx.lock().await.send(WsMessage::binary(resp)).await {
            return Err(if "Connection closed normally" == e.to_string() {
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

/// Return if the socket is closed normally, otherwise print the error if there is one and return the result.
#[macro_export]
macro_rules! return_print {
    ($result:expr) => {{
        let res = $crate::return_closed!($result);
        if let Err(err) = &res {
            $crate::tracing::error!("error occured: {}", err);
        }
        res
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
