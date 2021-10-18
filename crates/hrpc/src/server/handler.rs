use bytes::Bytes;
use futures_util::{future::BoxFuture, Future};
use http::{header, Method, StatusCode};
use std::{convert::Infallible, future, marker::PhantomData};
use tower::{
    service_fn,
    util::{BoxLayer, BoxService},
    Layer, Service, ServiceBuilder,
};
use tower_http::map_response_body::MapResponseBodyLayer;

use super::{
    error::{CustomError, ServerError, ServerResult},
    gen_prelude::box_body,
    socket::Socket,
    utils::HeaderMapExt,
    ws::WebSocketUpgrade,
};
use crate::{
    bail, bail_result_as_response,
    body::{full_box_body, HyperBody},
    encode_protobuf_message, hrpc_header_value, BoxError, HttpRequest, HttpResponse,
    Request as HrpcRequest, Response as HrpcResponse, HRPC_HEADER,
};

/// Call future used by [`Handler`].
pub type CallFuture = BoxFuture<'static, Result<HttpResponse, Infallible>>;

/// A hRPC handler.
pub struct Handler {
    svc: BoxService<HttpRequest, HttpResponse, Infallible>,
}

impl Handler {
    /// Create a new handler from a [`Service`].
    pub fn new<S>(svc: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        Self {
            svc: BoxService::new(svc),
        }
    }

    /// Layer this handler.
    pub fn layer<L, S>(self, layer: L) -> Self
    where
        L: Layer<Self, Service = S>,
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        Self {
            svc: BoxService::new(layer.layer(self)),
        }
    }
}

impl Service<HttpRequest> for Handler {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = CallFuture;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        Service::call(&mut self.svc, req)
    }
}

/// Layer type that produces hRPC [`Handler`]s.
pub struct HrpcLayer {
    inner: BoxLayer<Handler, HttpRequest, HttpResponse, Infallible>,
}

impl HrpcLayer {
    /// Create a new [`HrpcLayer`].
    pub fn new<L, S, B>(layer: L) -> Self
    where
        L: Layer<Handler, Service = S> + Send + Sync + 'static,
        S: Service<HttpRequest, Response = http::Response<B>, Error = Infallible> + Send + 'static,
        S::Future: Send,
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError>,
    {
        Self {
            inner: BoxLayer::new(
                ServiceBuilder::new()
                    .layer(MapResponseBodyLayer::new(box_body))
                    .layer(layer)
                    .into_inner(),
            ),
        }
    }
}

impl<S> Layer<S> for HrpcLayer
where
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Service = Handler;

    fn layer(&self, inner: S) -> Self::Service {
        let handler = Handler::new(inner);
        Handler {
            svc: self.inner.layer(handler),
        }
    }
}

/// A handler that responses to any request with not found.
pub fn not_found() -> Handler {
    Handler::new(service_fn(|_| {
        future::ready(Ok((StatusCode::NOT_FOUND, "not found").as_error_response()))
    }))
}

#[doc(hidden)]
pub fn from_http_request<Msg: prost::Message + Default + 'static>(
    req: HttpRequest,
) -> ServerResult<HrpcRequest<Msg>> {
    let (parts, body) = req.into_parts();

    if parts.method != Method::POST {
        bail!((StatusCode::METHOD_NOT_ALLOWED, "method must be POST"));
    }

    if !parts.headers.header_eq(&header::CONTENT_TYPE, HRPC_HEADER) {
        bail!((
            StatusCode::BAD_REQUEST,
            "request content type not supported"
        ));
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
                bail_result_as_response!(WebSocketUpgrade::from_request(req), |err| {
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
