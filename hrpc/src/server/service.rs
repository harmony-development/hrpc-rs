use std::{convert::Infallible, sync::Arc};
use tower::{
    layer::util::{Identity, Stack},
    service_fn, Layer, Service,
};

use crate::{HrpcLayer, HrpcService, HttpRequest, HttpResponse};

use super::not_found_service;

pub trait MakeHrpcService: Send + Sync + 'static {
    fn make_hrpc_service(&self, request: &HttpRequest) -> Option<HrpcService>;
}

pub type BoxedMakeHrpcService = Arc<dyn MakeHrpcService>;

#[derive(Clone)]
pub struct HrpcMakeService {
    producer: BoxedMakeHrpcService,
    child: Option<BoxedMakeHrpcService>,
    middleware: HrpcLayer,
}

impl HrpcMakeService {
    pub fn new<Producer: MakeHrpcService>(producer: Producer) -> Self {
        Self {
            producer: Arc::new(producer),
            child: None,
            middleware: HrpcLayer::new(Identity::new()),
        }
    }

    pub fn producer<Producer: MakeHrpcService>(self, producer: Producer) -> Self {
        Self {
            producer: Arc::new(producer),
            child: Some(Arc::new(self)),
            middleware: HrpcLayer::new(Identity::new()),
        }
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
        self.producer.make_hrpc_service(request).or_else(|| {
            self.child
                .as_ref()
                .and_then(|producer| producer.make_hrpc_service(request))
        })
    }
}

pub struct CloneMakeService<S, F>
where
    F: Fn(&HttpRequest) -> bool + Send + Sync,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Clone + Send + Sync,
    S::Future: Send,
{
    service: S,
    f: F,
}

impl<S, F> CloneMakeService<S, F>
where
    F: Fn(&HttpRequest) -> bool + Send + Sync,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible> + Clone + Send + Sync,
    S::Future: Send,
{
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F> MakeHrpcService for CloneMakeService<S, F>
where
    F: Fn(&HttpRequest) -> bool + Send + Sync + 'static,
    S: Service<HttpRequest, Response = HttpResponse, Error = Infallible>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send,
{
    fn make_hrpc_service(&self, request: &HttpRequest) -> Option<HrpcService> {
        (self.f)(request).then(|| HrpcService::new(self.service.clone()))
    }
}
