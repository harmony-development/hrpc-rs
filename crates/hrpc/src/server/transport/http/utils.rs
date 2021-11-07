use std::{borrow::Cow, convert::Infallible, str::FromStr};

use bytes::Bytes;
use futures_util::{future::BoxFuture, FutureExt, StreamExt};
use http::StatusCode;
use tower::{Layer, Service};

use super::ws::{WebSocketUpgrade, WebSocketUpgradeError};
use crate::{
    common::{
        extensions::Extensions,
        fut::{self, Ready},
        transport::http::{box_body, HttpRequest, HttpResponse},
    },
    proto::{Error as HrpcError, HrpcErrorIdentifier},
    request::{self, BoxRequest},
    server::{service::HrpcService, socket::SocketHandler, IntoMakeService, MakeRoutes},
    Request,
};

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
            Ok(upgrade) => {
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
                    let resp = ws_upgrade
                        .on_upgrade(|stream| {
                            let (ws_tx, ws_rx) = stream.split();
                            (socket_handler.inner)(Box::pin(ws_rx), Box::pin(ws_tx))
                        })
                        .into_response();

                    return Ok(resp);
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

                let mut http_resp: HttpResponse = resp.into();
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
/// it produces into [`HrpcServiceToHttp`].
pub struct MakeRoutesToHttp<S: MakeRoutes> {
    inner: IntoMakeService<S>,
}

impl<S: MakeRoutes> MakeRoutesToHttp<S> {
    /// Create a new service by wrapping an [`IntoMakeService`].
    pub fn new(inner: IntoMakeService<S>) -> Self {
        Self { inner }
    }
}

impl<S: MakeRoutes, T> Service<T> for MakeRoutesToHttp<S> {
    type Response = HrpcServiceToHttp;

    type Error = Infallible;

    type Future = Ready<Result<HrpcServiceToHttp, Infallible>>;

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
        fut::ready(Ok(HrpcServiceToHttp::new(routes.inner)))
    }
}

/// Layer that can be used to transform an [`IntoMakeService`] into a
/// [`MakeRoutesToHttp`].
pub struct MakeRoutesToHttpLayer;

impl<S: MakeRoutes> Layer<IntoMakeService<S>> for MakeRoutesToHttpLayer {
    type Service = MakeRoutesToHttp<S>;

    fn layer(&self, inner: IntoMakeService<S>) -> Self::Service {
        MakeRoutesToHttp::new(inner)
    }
}
