use bytes::Bytes;
use futures_util::{future::BoxFuture, Future, FutureExt};
use http::{header, Method, StatusCode};
use std::{convert::Infallible, future, marker::PhantomData, sync::Arc};
use tower::{
    layer::{layer_fn, util::Stack},
    service_fn,
    util::BoxService,
    Layer, Service,
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
    bail,
    body::{full_box_body, HyperBody},
    encode_protobuf_message, hrpc_header_value, BoxError, HttpRequest, HttpResponse,
    Request as HrpcRequest, Response as HrpcResponse, HRPC_HEADER,
};

/// Call future used by [`Handler`].
pub(crate) type CallFuture<'a> = BoxFuture<'a, Result<HttpResponse, Infallible>>;

/// A hRPC handler.
pub struct HrpcService {
    svc: BoxService<HttpRequest, HttpResponse, Infallible>,
}

impl HrpcService {
    /// Create a new handler from a [`tower::Service`].
    pub fn new<S>(svc: S) -> Self
    where
        S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        // If it's already a `Handler`, just use the service in that.
        super::utils::downcast::if_downcast_into!(S, HrpcService, svc, {
            return Self { svc: svc.svc };
        });

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
        HrpcService::new(layer.layer(self))
    }
}

impl Service<HttpRequest> for HrpcService {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = CallFuture<'static>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        Service::call(&mut self.svc, req)
    }
}

/// Layer type that produces hRPC [`Handler`]s.
#[derive(Clone)]
pub struct HrpcLayer {
    inner: Arc<dyn Layer<HrpcService, Service = HrpcService> + Sync + Send + 'static>,
}

impl HrpcLayer {
    /// Create a new [`HrpcLayer`] from a [`tower::Layer`].
    pub fn new<L, S, B>(layer: L) -> Self
    where
        L: Layer<HrpcService, Service = S> + Sync + Send + 'static,
        S: Service<HttpRequest, Response = http::Response<B>, Error = Infallible> + Send + 'static,
        S::Future: Send,
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError>,
    {
        // If it's already a HrpcLayer, no need to wrap it in stuff
        super::utils::downcast::if_downcast_into!(S, HrpcLayer, layer, {
            return Self { inner: layer.inner };
        });

        let inner_layer = layer_fn(move |svc: HrpcService| {
            let svc = layer.layer(svc);
            let svc = MapResponseBodyLayer::new(box_body).layer(svc);
            HrpcService::new(svc)
        });

        Self {
            inner: Arc::new(inner_layer),
        }
    }

    pub(crate) fn stack(inner: HrpcLayer, outer: HrpcLayer) -> Self {
        Self {
            inner: Arc::new(Stack::new(inner, outer)),
        }
    }
}

impl<S> Layer<S> for HrpcLayer
where
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Service = HrpcService;

    fn layer(&self, inner: S) -> Self::Service {
        self.inner.layer(HrpcService::new(inner))
    }
}

/// A handler that responses to any request with not found.
pub fn not_found() -> HrpcService {
    HrpcService::new(service_fn(|_| {
        future::ready(Ok((StatusCode::NOT_FOUND, "not found").as_error_response()))
    }))
}

#[doc(hidden)]
pub fn from_http_request<Msg: prost::Message + Default>(
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
pub fn into_http_response<Msg: prost::Message>(resp: HrpcResponse<Msg>) -> HttpResponse {
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
    Req: prost::Message + Default,
    Resp: prost::Message,
    HandlerFut: Future<Output = Result<HrpcResponse<Resp>, ServerError>> + Send,
    HandlerFn: FnOnce(HrpcRequest<Req>) -> HandlerFut + Clone + Send + 'static,
{
    let service = service_fn(move |req: HttpRequest| -> CallFuture<'_> {
        let request = match from_http_request(req) {
            Ok(request) => request,
            Err(err) => {
                tracing::error!("{}", err);
                return Box::pin(future::ready(Ok(err.as_error_response())));
            }
        };

        let fut = (handler.clone())(request);

        Box::pin(fut.map(|res| {
            let resp = match res {
                Ok(response) => into_http_response(response),
                Err(err) => {
                    tracing::error!("{}", err);
                    err.as_error_response()
                }
            };
            Ok(resp)
        }))
    });
    HrpcService::new(service)
}

#[doc(hidden)]
pub fn ws_handler<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> HrpcService
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
    HandlerFut: Future<Output = Result<(), ServerError>> + Send,
    HandlerFn: FnOnce(HrpcRequest<()>, Socket<Req, Resp>) -> HandlerFut + Clone + Send + 'static,
{
    let service = service_fn(move |req: HttpRequest| {
        let request = HrpcRequest {
            body: HyperBody::empty(),
            header_map: req.headers().clone(),
            message: PhantomData,
        };

        let websocket_upgrade = match WebSocketUpgrade::from_request(req) {
            Ok(upgrade) => upgrade,
            Err(err) => {
                tracing::error!("web socket upgrade error: {}", err);
                return future::ready(Ok(err.as_error_response()));
            }
        };

        let handler = handler.clone();
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

        future::ready(Ok(response))
    });

    HrpcService::new(service)
}
