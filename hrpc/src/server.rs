use std::{
    convert::Infallible,
    fmt::{self, Debug, Display, Formatter},
};
use warp::{Rejection, Reply};

#[doc(hidden)]
pub mod prelude {
    pub use super::{socket_common, unary_common, CustomError, ServerError};
    pub use crate::{IntoRequest, Request};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{SinkExt, StreamExt};
    pub use http::HeaderMap;
    pub use log;
    pub use std::time::Instant;
    pub use warp::{
        self,
        ws::{Message as WsMessage, Ws},
        Filter,
    };
}

#[doc(inline)]
pub use warp::http::StatusCode;

#[doc(hidden)]
pub mod unary_common {
    use crate::HeaderMap;

    use super::{CustomError, ServerError};
    use bytes::BytesMut;
    use warp::{filters::BoxedFilter, Filter};

    #[doc(hidden)]
    pub fn base_filter<Resp: prost::Message + Default, Err: CustomError + 'static>(
        pkg: &'static str,
        method: &'static str,
    ) -> BoxedFilter<(Resp, HeaderMap)> {
        warp::path(pkg)
            .and(warp::path(method))
            .and(warp::body::bytes())
            .and(warp::header::exact_ignore_case(
                "content-type",
                "application/hrpc",
            ))
            .and_then(move |bin| async move {
                Resp::decode(bin).map_err(|err| {
                    log::error!(
                        "{}/{}: received invalid protobuf message: {}",
                        pkg,
                        method,
                        err
                    );
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
        pkg: &'static str,
        method: &'static str,
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
                log::error!("{}/{}: {}", pkg, method, err);
                Err(warp::reject::custom(ServerError::Custom(err)))
            }
        }
    }
}

#[doc(hidden)]
pub mod socket_common {
    use std::time::Instant;

    use crate::HeaderMap;

    use super::{CustomError, ServerError};
    use bytes::{Bytes, BytesMut};
    use warp::{
        filters::BoxedFilter,
        ws::{Message as WsMessage, Ws},
        Filter, Future,
    };

    #[doc(hidden)]
    pub fn base_filter(pkg: &'static str, method: &'static str) -> BoxedFilter<(HeaderMap, Ws)> {
        warp::path(pkg)
            .and(warp::path(method))
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
        pkg: &'static str,
        method: &'static str,
        result: Result<(), Err>,
    ) -> Result<(), ServerError<Err>> {
        result.map_err(|err| {
            log::error!(
                "{}/{}: socket request validation failed: {}",
                pkg,
                method,
                err
            );
            ServerError::Custom(err)
        })
    }

    #[doc(hidden)]
    pub fn respond<'a, Resp: prost::Message + 'a, Err: CustomError + 'static>(
        pkg: &'static str,
        method: &'static str,
        resp: Result<Option<Resp>, Err>,
        tx: &'a mut futures_util::stream::SplitSink<warp::ws::WebSocket, WsMessage>,
        buf: &'a mut BytesMut,
    ) -> impl Future<Output = ()> + 'a {
        async move {
            use futures_util::SinkExt;

            let maybe_resp = match resp {
                Ok(Some(bin)) => {
                    crate::encode_protobuf_message(buf, bin);
                    Some(buf.to_vec())
                }
                Ok(None) => None,
                Err(err) => {
                    log::error!("{}/{}: {}", pkg, method, err);
                    Some(err.message())
                }
            };

            if let Some(resp) = maybe_resp {
                if let Err(e) = tx.send(WsMessage::binary(resp)).await {
                    log::error!(
                        "{}/{}: error responding to client socket: {}",
                        pkg,
                        method,
                        e
                    );
                } else {
                    log::debug!("{}/{}: responded to client socket", pkg, method);
                }
            }
        }
    }

    #[doc(hidden)]
    pub fn check_ping<'a>(
        pkg: &'static str,
        method: &'static str,
        tx: &'a mut futures_util::stream::SplitSink<warp::ws::WebSocket, WsMessage>,
        last_ping_time: &'a mut Instant,
        socket_ping_data: [u8; 32],
        socket_ping_period: u64,
    ) -> impl Future<Output = bool> + 'a {
        async move {
            use futures_util::SinkExt;

            let ping_lapse = last_ping_time.elapsed();
            if ping_lapse.as_secs() >= socket_ping_period {
                if let Err(e) = tx.send(WsMessage::ping(socket_ping_data)).await {
                    // TODO: replace this with error source downcasting?
                    if "Connection closed normally" == e.to_string() {
                        log::info!("{}/{}: socket closed normally", pkg, method);
                    } else {
                        log::error!("{}/{}: error pinging client socket: {}", pkg, method, e);
                        log::error!("{}/{}: can't reach client, closing socket", pkg, method);
                    }
                    return true;
                } else {
                    log::debug!(
                        "{}/{}: pinged client socket, last ping was {} ago",
                        pkg,
                        method,
                        last_ping_time.elapsed().as_secs()
                    );
                }
                *last_ping_time = Instant::now();
            }
            false
        }
    }

    #[doc(hidden)]
    pub async fn close_socket(
        pkg: &'static str,
        method: &'static str,
        tx: &mut futures_util::stream::SplitSink<warp::ws::WebSocket, WsMessage>,
    ) {
        use futures_util::SinkExt;

        if let Err(e) = tx.send(WsMessage::close()).await {
            log::error!("{}/{}: error closing socket: {}", pkg, method, e);
        } else {
            log::debug!("{}/{}: closed client socket", pkg, method);
        }
    }

    #[doc(hidden)]
    pub fn process_request<'a, Req: prost::Message + Default + 'a, Err: CustomError + 'static>(
        pkg: &'static str,
        method: &'static str,
        tx: &'a mut futures_util::stream::SplitSink<warp::ws::WebSocket, WsMessage>,
        rx: &'a mut futures_util::stream::SplitStream<warp::ws::WebSocket>,
        socket_ping_data: [u8; 32],
    ) -> impl Future<Output = Result<Option<Req>, ()>> + 'a {
        async move {
            use futures_util::{SinkExt, StreamExt};

            let msg_maybe = rx.next().await.map(Result::ok).flatten();

            if let Some(msg) = msg_maybe {
                if msg.is_binary() {
                    let msg_bin = Bytes::from(msg.into_bytes());
                    match Req::decode(msg_bin) {
                        Ok(req) => Ok(Some(req)),
                        Err(err) => {
                            log::error!(
                                "{}/{}: received invalid protobuf message: {}",
                                pkg,
                                method,
                                err
                            );
                            if let Err(e) = tx.send(WsMessage::binary(Err::decode_error().1)).await
                            {
                                log::error!(
                                    "{}/{}: error responding to client socket: {}",
                                    pkg,
                                    method,
                                    e
                                );
                            } else {
                                log::debug!("{}/{}: responded to client socket", pkg, method);
                            }
                            Ok(None)
                        }
                    }
                } else if msg.is_pong() {
                    let msg_bin = Bytes::from(msg.into_bytes());
                    if socket_ping_data != msg_bin.as_ref() {
                        close_socket(pkg, method, tx).await;
                        Err(())
                    } else {
                        log::debug!("{}/{}: received pong", pkg, method);
                        Ok(None)
                    }
                } else if msg.is_close() {
                    Err(())
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
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
        log::error!("unhandled rejection: {:?}", err);
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

            let filter = $first $( .or($filter) )+ ;

            $crate::warp::serve(filter.recover($crate::server::handle_rejection::<$err>))
                .run($address)
                .await
        }
    };
}
