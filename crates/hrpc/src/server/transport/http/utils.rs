use std::{borrow::Cow, convert::Infallible, str::FromStr};

use bytes::Bytes;
use futures_util::{future::BoxFuture, FutureExt, StreamExt};
use http::StatusCode;
use tower::{
    layer::util::{Identity, Stack},
    util::BoxService,
    Layer, Service,
};

use super::ws::{WebSocketUpgrade, WebSocketUpgradeError};
use crate::{
    common::{
        extensions::Extensions,
        future::{self, Ready},
        transport::http::{box_body, HttpRequest, HttpResponse},
    },
    proto::{Error as HrpcError, HrpcErrorIdentifier},
    request::{self, BoxRequest},
    response,
    server::{service::HrpcService, socket::SocketHandler, IntoMakeService, MakeRoutes},
    Request, HRPC_WEBSOCKET_PROTOCOL,
};

/// A type alias for a boxed [`Service`] that takes [`HttpRequest`]s and
/// outputs [`HttpResponse`]s.
pub type HttpService = BoxService<HttpRequest, HttpResponse, Infallible>;

/// A service that wraps a [`HrpcService`], and takes HTTP requests and
/// produces HTTP responses.
pub struct HrpcServiceToHttp {
    inner: HrpcService,
}

impl HrpcServiceToHttp {
    /// Create a new service by wrapping a [`HrpcService`].
    pub fn new(svc: HrpcService) -> Self {
        Self { inner: svc }
    }
}

impl Service<HttpRequest> for HrpcServiceToHttp {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = BoxFuture<'static, Result<HttpResponse, Infallible>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, mut req: HttpRequest) -> Self::Future {
        let (ws_upgrade, hrpc_req) = match WebSocketUpgrade::from_request(&mut req) {
            Ok(mut upgrade) => {
                upgrade = upgrade.protocols([HRPC_WEBSOCKET_PROTOCOL]);

                let (parts, body) = req.into_parts();

                let endpoint = Cow::Owned(parts.uri.path().to_string());
                let mut extensions = Extensions::new();
                extensions.insert(parts);

                let req = Request::from(request::Parts {
                    body: body.into(),
                    extensions,
                    endpoint,
                });

                (Some(upgrade), Ok(req))
            }
            Err(err) => {
                // TODO: this is not good, find a way to properly get if a path is socket or unary
                if let WebSocketUpgradeError::MethodNotGet = err {
                    (None, BoxRequest::from_unary_request(req))
                } else {
                    return Box::pin(futures_util::future::ready(Ok(err.into())));
                }
            }
        };

        match hrpc_req {
            Ok(req) => Box::pin(Service::call(&mut self.inner, req).map(|res| {
                let mut resp = res.unwrap();

                if let (Some(socket_handler), Some(ws_upgrade)) =
                    (resp.extensions_mut().remove::<SocketHandler>(), ws_upgrade)
                {
                    let mut ws_resp = ws_upgrade
                        .on_upgrade(|stream| {
                            let (ws_tx, ws_rx) = stream.split();
                            (socket_handler.inner)(Box::pin(ws_rx), Box::pin(ws_tx))
                        })
                        .into_response();

                    let mut parts: response::Parts = resp.into();

                    if let Some(exts) = parts.extensions.remove::<http::Extensions>() {
                        *ws_resp.extensions_mut() = exts;
                    }

                    if let Some(headers) = parts.extensions.remove::<http::HeaderMap>() {
                        ws_resp.headers_mut().extend(headers);
                    }

                    ws_resp.extensions_mut().insert(parts.extensions);

                    return Ok(ws_resp);
                }

                let status = resp
                    .extensions_mut()
                    .remove::<StatusCode>()
                    .or_else(|| {
                        resp.extensions().get::<HrpcError>().map(|err| {
                            match HrpcErrorIdentifier::from_str(err.identifier.as_str()) {
                                Ok(err_id) => err_id.into(),
                                _ => StatusCode::INTERNAL_SERVER_ERROR,
                            }
                        })
                    })
                    .unwrap_or(StatusCode::OK);

                let mut http_resp = resp.into_unary_response();
                *http_resp.status_mut() = status;

                Ok(http_resp)
            })),
            Err((status, msg)) => {
                let resp = http::Response::builder()
                    .status(status)
                    .body(box_body(http_body::Full::new(Bytes::from_static(
                        msg.as_bytes(),
                    ))))
                    .unwrap();
                Box::pin(futures_util::future::ready(Ok(resp)))
            }
        }
    }
}

/// Layer that can be used to transform a [`HrpcService`] to [`HrpcServiceToHttp`].
pub struct HrpcServiceToHttpLayer;

impl Layer<HrpcService> for HrpcServiceToHttpLayer {
    type Service = HrpcServiceToHttp;

    fn layer(&self, inner: HrpcService) -> Self::Service {
        HrpcServiceToHttp::new(inner)
    }
}

/// Service that wraps an [`IntoMakeService`] and transforms the services
/// it produces into [`HttpService`].
pub struct MakeRoutesToHttp<S: MakeRoutes, L> {
    inner: IntoMakeService<S>,
    layer: L,
}

impl<S: MakeRoutes> MakeRoutesToHttp<S, Identity> {
    /// Create a new service by wrapping an [`IntoMakeService`].
    pub fn new(inner: IntoMakeService<S>) -> Self {
        Self {
            inner,
            layer: Identity::new(),
        }
    }
}

impl<M: MakeRoutes, L> MakeRoutesToHttp<M, L> {
    /// Layer the inner layer, that will be used to transform the produced services.
    pub fn layer<Layer>(self, layer: Layer) -> MakeRoutesToHttp<M, Stack<Layer, L>> {
        MakeRoutesToHttp {
            inner: self.inner,
            layer: Stack::new(layer, self.layer),
        }
    }
}

impl<M: MakeRoutes, T, L, S> Service<T> for MakeRoutesToHttp<M, L>
where
    L: Layer<HrpcServiceToHttp, Service = S> + Clone + Send,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Response = HttpService;

    type Error = Infallible;

    type Future = Ready<Result<HttpService, Infallible>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<T>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: T) -> Self::Future {
        let routes = Service::call(&mut self.inner, req)
            .into_inner()
            .unwrap()
            .unwrap();
        let http_service = HrpcServiceToHttp::new(routes.inner);
        future::ready(Ok(BoxService::new(self.layer.layer(http_service))))
    }
}
