use self::{prelude::CustomError, ws::WebSocketUpgrade};

use super::{
    body::{full_box_body, HyperBody},
    encode_protobuf_message, hrpc_header_value, Request as HrpcRequest, Response as HrpcResponse,
    HRPC_HEADER,
};
use error::{ServerError, ServerResult};
use socket::Socket;

use bytes::Bytes;
use http::{header::HeaderName, HeaderMap, Method};
use std::{convert::Infallible, future::Future, marker::PhantomData, sync::Arc};
use tower::{
    layer::util::{Identity, Stack},
    service_fn, Layer, Service,
};

pub use super::{HrpcLayer, HrpcService, HttpRequest, HttpResponse};

pub mod error;
pub mod macros;
pub mod socket;
pub(crate) mod ws;

#[doc(inline)]
pub use http::StatusCode;

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        error::*, not_found_service, socket::*, unary_handler, ws_handler, HrpcLayer,
        HrpcMakeService, HrpcService, HttpRequest, HttpResponse, MakeHrpcService,
    };
    pub use crate::{body::box_body, Request as HrpcRequest, Response as HrpcResponse};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{FutureExt, SinkExt, StreamExt};
    pub use http::{self, HeaderMap};
    pub use hyper::{server::conn::AddrStream, service::make_service_fn, Server as HttpServer};
    pub use std::{
        collections::HashMap,
        convert::Infallible,
        time::{Duration, Instant},
    };
    pub use tokio::{self, sync::Mutex, try_join};
    pub use tower::{
        layer::{
            layer_fn,
            util::{Identity, Stack},
        },
        service_fn,
        util::{BoxLayer, BoxService},
        Layer, Service, ServiceBuilder, ServiceExt,
    };
    pub use tower_http::{
        classify::StatusInRangeAsFailures, map_request_body::MapRequestBodyLayer,
        map_response_body::MapResponseBodyLayer, trace::TraceLayer,
    };
    pub use tracing::{debug, error, info, info_span, trace, warn};
}

#[derive(Clone)]
pub struct HrpcMakeService {
    producers: Arc<Vec<BoxedMakeHrpcService>>,
    middleware: HrpcLayer,
}

impl HrpcMakeService {
    pub fn new(producers: Vec<BoxedMakeHrpcService>) -> Self {
        Self {
            producers: Arc::new(producers),
            middleware: HrpcLayer::new(Identity::new()),
        }
    }

    pub fn new_single(producer: BoxedMakeHrpcService) -> Self {
        Self::new(vec![producer])
    }

    pub fn layer(mut self, layer: HrpcLayer) -> Self {
        self.middleware = HrpcLayer::new(Stack::new(self.middleware, layer));
        self
    }
}

impl<T> Service<T> for HrpcMakeService {
    type Response = HrpcService;

    type Error = Infallible;

    type Future = std::future::Ready<Result<HrpcService, Infallible>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: T) -> Self::Future {
        let this = self.clone();
        let service = HrpcService::new(service_fn(move |request: HttpRequest| {
            let service = this
                .make_hrpc_service(&request)
                .unwrap_or_else(not_found_service);
            let mut service = this.middleware.layer(service);

            async move { Result::<_, Infallible>::Ok(service.call(request).await.unwrap()) }
        }));
        std::future::ready(Ok(service))
    }
}

impl MakeHrpcService for HrpcMakeService {
    fn make_hrpc_service(&self, request: &HttpRequest) -> Option<HrpcService> {
        let mut service = None;
        for producer in self.producers.iter() {
            if let Some(svc) = producer.make_hrpc_service(request) {
                service = Some(svc);
                break;
            }
        }
        service
    }
}

pub trait MakeHrpcService: Send + Sync + 'static {
    fn make_hrpc_service(&self, request: &HttpRequest) -> Option<HrpcService>;
}

pub type BoxedMakeHrpcService = Box<dyn MakeHrpcService>;

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
pub fn unary_handler<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> HrpcService
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
    HrpcService::new(service)
}

#[doc(hidden)]
pub fn ws_handler<Req, Resp, HandlerFn, HandlerFut, OnUpgradeFn>(
    handler: HandlerFn,
    on_upgrade: OnUpgradeFn,
) -> HrpcService
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
            let websocket_upgrade = match WebSocketUpgrade::from_request(req) {
                Ok(upgrade) => upgrade,
                Err(err) => {
                    tracing::error!("web socket upgrade error: {}", err);
                    return Ok(err.as_error_response());
                }
            };

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

    HrpcService::new(service)
}

#[doc(hidden)]
pub fn not_found_service() -> HrpcService {
    let service = service_fn(|_: HttpRequest| async {
        Ok((StatusCode::NOT_FOUND, "not found").as_error_response())
    });
    HrpcService::new(service)
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
