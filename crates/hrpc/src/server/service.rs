use futures_util::{future::BoxFuture, Future, FutureExt};
use std::{convert::Infallible, sync::Arc};
use tower::{
    layer::{layer_fn, util::Stack},
    service_fn,
    util::BoxService,
    Layer, Service,
};

use super::{
    error::HrpcError,
    socket::{Socket, SocketHandler},
};
use crate::{request::BoxRequest, response::BoxResponse, Request, Response};

/// Call future used by [`HrpcService`].
pub(crate) type CallFuture<'a> = BoxFuture<'a, Result<BoxResponse, Infallible>>;

/// A hRPC handler.
pub struct HrpcService {
    svc: BoxService<BoxRequest, BoxResponse, Infallible>,
}

impl HrpcService {
    /// Create a new handler from a [`tower::Service`].
    pub fn new<S>(svc: S) -> Self
    where
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        // If it's already a `HrpcService`, just use the service in that.
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
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        HrpcService::new(layer.layer(self))
    }
}

impl Service<BoxRequest> for HrpcService {
    type Response = BoxResponse;

    type Error = Infallible;

    type Future = CallFuture<'static>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: BoxRequest) -> Self::Future {
        Service::call(&mut self.svc, req)
    }
}

/// Layer type that produces [`HrpcService`]s.
#[derive(Clone)]
pub struct HrpcLayer {
    inner: Arc<dyn Layer<HrpcService, Service = HrpcService> + Sync + Send + 'static>,
}

impl HrpcLayer {
    /// Create a new [`HrpcLayer`] from a [`tower::Layer`].
    pub fn new<L, S>(layer: L) -> Self
    where
        L: Layer<HrpcService, Service = S> + Sync + Send + 'static,
        S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
        S::Future: Send,
    {
        // If it's already a HrpcLayer, no need to wrap it in stuff
        super::utils::downcast::if_downcast_into!(S, HrpcLayer, layer, {
            return Self { inner: layer.inner };
        });

        let layer = layer_fn(move |svc| {
            let new_svc = layer.layer(svc);
            HrpcService::new(new_svc)
        });

        Self {
            inner: Arc::new(layer),
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
    S: Service<BoxRequest, Response = BoxResponse, Error = Infallible> + Send + 'static,
    S::Future: Send,
{
    type Service = HrpcService;

    fn layer(&self, inner: S) -> Self::Service {
        self.inner.layer(HrpcService::new(inner))
    }
}

/// A handler that responses to any request with not found.
pub fn not_found() -> HrpcService {
    HrpcService::new(tower::service_fn(|_| {
        futures_util::future::ready(Ok(HrpcError::new_not_found("not found").into()))
    }))
}

#[doc(hidden)]
pub fn unary_handler<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> HrpcService
where
    Req: prost::Message + Default,
    Resp: prost::Message,
    HandlerFut: Future<Output = Result<Response<Resp>, HrpcError>> + Send,
    HandlerFn: FnOnce(Request<Req>) -> HandlerFut + Clone + Send + 'static,
{
    let service = service_fn(move |req: BoxRequest| {
        (handler.clone())(req.map::<Req>())
            .map(|res| Ok(res.map_or_else(HrpcError::into, |resp| resp.map::<()>())))
    });
    HrpcService::new(service)
}

#[doc(hidden)]
pub fn ws_handler<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> HrpcService
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
    HandlerFut: Future<Output = Result<(), HrpcError>> + Send,
    HandlerFn: FnOnce(Request<()>, Socket<Req, Resp>) -> HandlerFut + Clone + Send + Sync + 'static,
{
    let service = service_fn(move |req: BoxRequest| {
        let handler = handler.clone();
        let socket_handler = SocketHandler {
            inner: Box::new(move |rx, tx| {
                Box::pin(async move {
                    let socket = Socket::new(rx, tx);
                    let res = handler(req, socket.clone()).await;
                    if let Err(err) = res {
                        tracing::error!("{}", err);
                    }
                    socket.close().await;
                })
            }),
        };

        let mut response = Response::empty();
        response.extensions_mut().insert(socket_handler);

        futures_util::future::ready(Ok(response))
    });

    HrpcService::new(service)
}
