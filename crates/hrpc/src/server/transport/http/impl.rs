use std::{
    borrow::Cow,
    convert::Infallible,
    net::SocketAddr,
    str::FromStr,
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, FutureExt, StreamExt};
use http::{header, HeaderMap, Method, StatusCode};
use hyper::server::conn::AddrStream;
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service,
};

use super::{
    box_body,
    ws::{WebSocketUpgrade, WebSocketUpgradeError},
    HttpRequest, HttpResponse,
};
use crate::{
    common::{
        extensions::Extensions,
        future::{self, Ready},
        transport::http::{
            content_header_value, version_header_name, version_header_value, ws_version,
            HeaderMapExt,
        },
    },
    proto::{Error as HrpcError, HrpcErrorIdentifier},
    request, response,
    server::{service::HrpcService, socket::SocketHandler, IntoMakeService, MakeRoutes},
    Request, Response, HRPC_CONTENT_MIMETYPE,
};

/// A service that wraps a [`HrpcService`], and takes HTTP requests and
/// produces HTTP responses.
pub struct HrpcServiceToHttp {
    inner: HrpcService,
    socket_addr: Option<SocketAddr>,
}

impl HrpcServiceToHttp {
    /// Create a new service by wrapping a [`HrpcService`].
    pub fn new(svc: HrpcService) -> Self {
        Self {
            inner: svc,
            socket_addr: None,
        }
    }

    /// Set a [`SocketAddr`] that will be passed to all requests made to this
    /// service. Using this method after another will overwrite the older value.
    pub fn with_socket_addr(mut self, addr: SocketAddr) -> Self {
        self.socket_addr = Some(addr);
        self
    }
}

impl Service<HttpRequest> for HrpcServiceToHttp {
    type Response = HttpResponse;

    type Error = Infallible;

    type Future = BoxFuture<'static, Result<HttpResponse, Infallible>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, mut req: HttpRequest) -> Self::Future {
        let (ws_upgrade, hrpc_req) = match WebSocketUpgrade::from_request(&mut req) {
            Ok(mut upgrade) => {
                upgrade = upgrade.protocols([ws_version()]);

                let (parts, body) = req.into_parts();

                let endpoint = Cow::Owned(parts.uri.path().to_string());

                let mut extensions = Extensions::new();
                extensions.insert(parts.extensions);
                extensions.insert(parts.headers);
                extensions.insert(parts.method);
                extensions.insert(parts.version);
                extensions.insert(parts.uri);

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
                    (None, from_unary_request(req))
                } else {
                    let message = err.to_string();
                    let mut resp = err_into_unary_response(
                        HrpcError::default()
                            .with_identifier("hrpc.http.bad-streaming-request")
                            .with_message(message),
                    );

                    *resp.status_mut() = match err {
                        WebSocketUpgradeError::MethodNotGet => StatusCode::METHOD_NOT_ALLOWED,
                        _ => StatusCode::BAD_REQUEST,
                    };

                    return Box::pin(futures_util::future::ready(Ok(resp)));
                }
            }
        };

        match hrpc_req {
            Ok(mut req) => {
                if let Some(socket_addr) = self.socket_addr {
                    req.extensions_mut().insert(socket_addr);
                }
                Box::pin(Service::call(&mut self.inner, req).map(|res| {
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

                        let parts: response::Parts = resp.into();

                        set_http_extensions(parts.extensions, &mut ws_resp);

                        return Ok(ws_resp);
                    }

                    Ok(into_unary_response(resp))
                }))
            }
            Err((status, err)) => {
                let mut resp = err_into_unary_response(err);
                *resp.status_mut() = status;
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
/// it produces to work with [`hyper`].
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

impl<M: MakeRoutes, L, S> Service<&AddrStream> for MakeRoutesToHttp<M, L>
where
    L: Layer<HrpcServiceToHttp, Service = S> + Clone + Send,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Response = L::Service;

    type Error = Infallible;

    type Future = Ready<Result<L::Service, Infallible>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<AddrStream>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: &AddrStream) -> Self::Future {
        let socket_addr = req.remote_addr();

        tracing::debug!("creating new service for: {}", socket_addr);

        let routes = Service::call(&mut self.inner, req)
            .into_inner()
            .expect("future must always contain value")
            .expect("this call is infallible");
        let http_service = HrpcServiceToHttp::new(routes.inner).with_socket_addr(socket_addr);
        future::ready(Ok(self.layer.layer(http_service)))
    }
}

/// Add HTTP specific extensions like [`HeaderMap`] and [`http::Extensions`] to the HTTP response.
pub(crate) fn set_http_extensions(mut exts: Extensions, resp: &mut HttpResponse) {
    if let Some(exts) = exts.remove::<http::Extensions>() {
        *resp.extensions_mut() = exts;
    }

    if let Some(headers) = exts.remove::<HeaderMap>() {
        resp.headers_mut().extend(headers);
    }

    let maybe_status = exts.remove::<StatusCode>().or_else(|| {
        exts.get::<HrpcError>().map(|err| {
            match HrpcErrorIdentifier::from_str(err.identifier.as_str()) {
                Ok(err_id) => err_id.into(),
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })
    });

    if let Some(status) = maybe_status {
        *resp.status_mut() = status;
    }
}

/// Convert this hRPC response into a unary HTTP response.
pub(crate) fn into_unary_response<T>(resp: Response<T>) -> HttpResponse {
    let parts = response::Parts::from(resp);

    let mut resp = http::Response::builder()
        .header(version_header_name(), version_header_value())
        .header(http::header::CONTENT_TYPE, content_header_value())
        .header(http::header::ACCEPT, content_header_value())
        .body(box_body(parts.body))
        .unwrap();

    set_http_extensions(parts.extensions, &mut resp);

    resp
}

/// Try to create a [`Request`] from a unary [`HttpRequest`].
pub(crate) fn from_unary_request<T>(
    req: HttpRequest,
) -> Result<Request<T>, (StatusCode, HrpcError)> {
    let (parts, body) = req.into_parts();

    if parts.method != Method::POST {
        return Err((
            StatusCode::METHOD_NOT_ALLOWED,
            ("hrpc.http.bad-unary-request", "method must be POST").into(),
        ));
    }

    if !parts
        .headers
        .header_eq(&header::CONTENT_TYPE, HRPC_CONTENT_MIMETYPE.as_bytes())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            (
                "hrpc.http.bad-unary-request",
                "request content type not supported",
            )
                .into(),
        ));
    }

    let endpoint = Cow::Owned(parts.uri.path().to_string());

    let mut extensions = Extensions::new();
    extensions.insert(parts.extensions);
    extensions.insert(parts.headers);
    extensions.insert(parts.method);
    extensions.insert(parts.version);
    extensions.insert(parts.uri);

    let req = Request::from(request::Parts {
        body: body.into(),
        extensions,
        endpoint,
    });

    Ok(req)
}

/// Convert this hRPC error into a HTTP response.
pub(crate) fn err_into_unary_response(err: HrpcError) -> HttpResponse {
    into_unary_response(Response::new(&err))
}