use super::{Request, HRPC_HEADER};
use crate::{encode_protobuf_message, hrpc_header_value, Response};
use clone_box::{CloneBoxLayer, CloneBoxService};
use error::ServerError;
use socket::Socket;

use async_trait::async_trait;
use axum::{
    body::HttpBody,
    extract::{FromRequest, RequestParts, WebSocketUpgrade},
    response::IntoResponse,
    routing::BoxRoute,
};
use bytes::Bytes;
use http::Method;
use std::{convert::Infallible, future::Future, marker::PhantomData};
use tower::{service_fn, util::BoxLayer};

pub mod clone_box;
pub mod error;
pub mod macros;
pub mod socket;

#[doc(inline)]
pub use axum::body::{box_body, Body, BoxBody};
#[doc(inline)]
pub use http::StatusCode;

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        error::*, handler_service, socket::*, ws_handler_service, HrpcLayer, HrpcRoute,
        HrpcRouteLayer, HrpcService,
    };
    pub use crate::{Request, Response};
    pub use axum::{
        self,
        body::{box_body, BoxBody},
        response::IntoResponse,
        routing::BoxRoute,
        routing::IntoMakeService,
        Router,
    };
    pub use axum_server::{self, Server};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{FutureExt, SinkExt, StreamExt};
    pub use http::{self, HeaderMap};
    pub use hyper::{server::conn::AddrStream, service::make_service_fn, Body};
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
        compression::CompressionLayer, decompression::DecompressionLayer,
        map_response_body::MapResponseBodyLayer, trace::TraceLayer,
    };
    pub use tracing::{debug, error, info, info_span, trace, warn};
}

#[async_trait]
impl<Msg: prost::Message + Default + 'static> FromRequest for Request<Msg> {
    type Rejection = ServerError;

    async fn from_request(
        req: &mut axum::extract::RequestParts<Body>,
    ) -> Result<Self, Self::Rejection> {
        if req.method() != Method::POST {
            return Err(ServerError::MethodNotPost);
        }

        let header_map = req.headers().cloned().ok_or(ServerError::EmptyHeaders)?;

        if let Some(header) = header_map
            .get(http::header::CONTENT_TYPE)
            .map(|h| Bytes::copy_from_slice(h.as_bytes()))
        {
            if !header.eq_ignore_ascii_case(HRPC_HEADER) {
                return Err(ServerError::UnsupportedRequestType(header));
            }
        }

        let body = if std::any::TypeId::of::<()>() == std::any::TypeId::of::<Msg>() {
            Body::empty()
        } else {
            req.take_body().ok_or(ServerError::EmptyBody)?
        };

        Ok(Request {
            body,
            header_map,
            message: std::marker::PhantomData,
        })
    }
}

impl<Msg: prost::Message> IntoResponse for Response<Msg> {
    type Body = Body;

    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> http::Response<Self::Body> {
        let encoded = encode_protobuf_message(self.data).freeze();
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, hrpc_header_value())
            .header(http::header::ACCEPT, hrpc_header_value())
            .body(Body::from(encoded))
            .unwrap()
    }
}

pub type HrpcService = CloneBoxService<http::Request<Body>, http::Response<BoxBody>, ServerError>;
pub type HrpcLayer =
    CloneBoxLayer<HrpcService, http::Request<Body>, http::Response<BoxBody>, ServerError>;
pub type HrpcRoute = BoxRoute<Body, Infallible>;
pub type HrpcRouteLayer =
    BoxLayer<HrpcRoute, http::Request<Body>, http::Response<BoxBody>, Infallible>;

pub fn handler_service<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> HrpcService
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message,
    HandlerFut: Future<Output = Result<Response<Resp>, ServerError>> + Send,
    HandlerFn: FnOnce(Request<Req>) -> HandlerFut + Clone + Send + Sync + 'static,
{
    let service = service_fn(move |request: http::Request<Body>| {
        let handler = handler.clone();
        async move {
            let mut req_parts = RequestParts::new(request);
            let request = Request::<Req>::from_request(&mut req_parts).await?;

            let response = handler(request).await?;
            Ok(response.into_response().map(box_body))
        }
    });
    CloneBoxService::new(service)
}

pub fn ws_handler_service<Req, Resp, HandlerFn, HandlerFut, OnUpgradeFn>(
    handler: HandlerFn,
    on_upgrade: OnUpgradeFn,
) -> HrpcService
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
    HandlerFut: Future<Output = Result<(), ServerError>> + Send,
    HandlerFn: FnOnce(Request<()>, Socket<Req, Resp>) -> HandlerFut + Clone + Sync + Send + 'static,
    OnUpgradeFn:
        FnOnce(http::Response<BoxBody>) -> http::Response<BoxBody> + Clone + Sync + Send + 'static,
{
    let service = service_fn(move |request: http::Request<Body>| {
        let handler = handler.clone();
        let on_upgrade = on_upgrade.clone();
        async move {
            let mut req_parts = RequestParts::new(request);
            let websocket_upgrade = WebSocketUpgrade::from_request(&mut req_parts).await?;
            let request = Request {
                body: Body::empty(),
                header_map: req_parts.headers().cloned().unwrap_or_default(),
                message: PhantomData,
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
                .into_response()
                .map(box_body);

            let response = on_upgrade(response);

            Ok(response)
        }
    });
    CloneBoxService::new(service)
}
