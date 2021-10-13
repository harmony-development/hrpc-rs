use crate::return_err_as_resp;

use self::ws::WebSocketUpgrade;

use super::{
    body::{full_box_body, HyperBody},
    encode_protobuf_message, hrpc_header_value, Request as HrpcRequest, Response as HrpcResponse,
    HRPC_HEADER,
};
use error::{ServerError, ServerResult};
use socket::Socket;

use bytes::Bytes;
use http::{header::HeaderName, HeaderMap, Method};
use std::{convert::Infallible, future::Future, marker::PhantomData};
use tower::{service_fn, util::BoxLayer};

pub use super::{HttpRequest, HttpResponse};
pub use service::{Handler, MakeRouter, Router, RouterBuilder};

pub type HrpcLayer = BoxLayer<Handler, HttpRequest, HttpResponse, Infallible>;

pub mod error;
pub mod macros;
pub mod service;
pub mod socket;
pub(crate) mod ws;

#[doc(inline)]
pub use http::StatusCode;

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        error::*, serve, socket::*, unary_handler, ws_handler, Handler, HrpcLayer, HttpRequest,
        HttpResponse, MakeRouter, RouterBuilder,
    };
    pub use crate::{body::box_body, Request as HrpcRequest, Response as HrpcResponse};
    pub use tower::{layer::util::Identity, ServiceBuilder, ServiceExt};
}

/// Start serving.
pub async fn serve<A, S>(mk_service: S, address: A)
where
    A: Into<std::net::SocketAddr>,
    S: MakeRouter,
{
    let addr = address.into();

    let server = hyper::Server::bind(&addr).serve(mk_service.into_make_service());
    tracing::info!("serving at {}", addr);
    server.await.unwrap()
}

#[doc(hidden)]
pub fn from_http_request<Msg: prost::Message + Default + 'static>(
    req: HttpRequest,
) -> ServerResult<HrpcRequest<Msg>> {
    let (parts, body) = req.into_parts();

    if parts.method != Method::POST {
        return Err(ServerError::MethodNotPost);
    }

    if let Some(header) = parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .map(|h| Bytes::copy_from_slice(h.as_bytes()))
    {
        if !header.eq_ignore_ascii_case(HRPC_HEADER) {
            return Err(ServerError::UnsupportedRequestType(header));
        }
    }

    Ok(HrpcRequest {
        body,
        header_map: parts.headers,
        message: std::marker::PhantomData,
    })
}

#[doc(hidden)]
pub fn into_http_request<Msg: prost::Message>(resp: HrpcResponse<Msg>) -> HttpResponse {
    let encoded = encode_protobuf_message(resp.data).freeze();
    http::Response::builder()
        .header(http::header::CONTENT_TYPE, hrpc_header_value())
        .header(http::header::ACCEPT, hrpc_header_value())
        .body(full_box_body(encoded))
        .unwrap()
}

#[doc(hidden)]
pub fn unary_handler<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> Handler
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message,
    HandlerFut: Future<Output = Result<HrpcResponse<Resp>, ServerError>> + Send,
    HandlerFn: FnOnce(HrpcRequest<Req>) -> HandlerFut + Clone + Send + 'static,
{
    let service = service_fn(move |req: HttpRequest| {
        let handler = handler.clone();
        async move {
            let request = match from_http_request(req) {
                Ok(request) => request,
                Err(err) => {
                    tracing::error!("{}", err);
                    return Ok(err.into_response());
                }
            };
            let response = match handler(request).await {
                Ok(response) => response,
                Err(err) => {
                    tracing::error!("{}", err);
                    return Ok(err.into_response());
                }
            };
            Ok(into_http_request(response))
        }
    });
    Handler::new(service)
}

#[doc(hidden)]
pub fn ws_handler<Req, Resp, HandlerFn, HandlerFut, OnUpgradeFn>(
    handler: HandlerFn,
    on_upgrade: OnUpgradeFn,
) -> Handler
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
    HandlerFut: Future<Output = Result<(), ServerError>> + Send,
    HandlerFn: FnOnce(HrpcRequest<()>, Socket<Req, Resp>) -> HandlerFut + Clone + Send + 'static,
    OnUpgradeFn: FnOnce(HttpResponse) -> HttpResponse + Clone + Send + 'static,
{
    let service = service_fn(move |req: HttpRequest| {
        let handler = handler.clone();
        let on_upgrade = on_upgrade.clone();
        async move {
            let request = HrpcRequest {
                body: HyperBody::empty(),
                header_map: req.headers().clone(),
                message: PhantomData,
            };
            let websocket_upgrade =
                return_err_as_resp!(WebSocketUpgrade::from_request(req), |err| {
                    tracing::error!("web socket upgrade error: {}", err);
                });

            let response = websocket_upgrade
                .on_upgrade(|ws| async move {
                    let socket = Socket::new(ws);
                    let res = handler(request, socket.clone()).await;
                    if let Err(err) = res {
                        tracing::error!("{}", err);
                    }
                    socket.close().await;
                })
                .into_response();

            Ok(on_upgrade(response))
        }
    });

    Handler::new(service)
}

/// Helper methods for working with `HeaderMap`.
pub trait HeaderMapExt {
    /// Check if a header is equal to a bytes array.
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool;
    /// Check if a header contains a string.
    fn header_contains_str(&self, key: &HeaderName, value: &str) -> bool;
}

impl HeaderMapExt for HeaderMap {
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool {
        self.get(key).map_or(false, |header| {
            header.as_bytes().eq_ignore_ascii_case(value)
        })
    }

    fn header_contains_str(&self, key: &HeaderName, pat: &str) -> bool {
        self.get(key).map_or(false, |header| {
            header
                .to_str()
                .map_or(false, |value| value.to_ascii_lowercase().contains(pat))
        })
    }
}
