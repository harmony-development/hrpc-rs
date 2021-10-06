use super::{Request, HRPC_HEADER};
use crate::{encode_protobuf_message, hrpc_header_value, Response};
use error::ServerError;
use socket::Socket;

use async_trait::async_trait;
use axum::{
    body::{Body, HttpBody},
    extract::{FromRequest, RequestParts, WebSocketUpgrade},
    response::IntoResponse,
};
use bytes::Bytes;
use http::Method;
use std::future::Future;
use tower::service_fn;

pub mod error;
pub mod socket;

#[doc(inline)]
pub use axum::body::{box_body, BoxBody};
#[doc(inline)]
pub use http::StatusCode;
#[doc(inline)]
pub use tower::util::{BoxLayer, BoxService};

#[doc(hidden)]
pub mod prelude {
    pub use super::{
        error::*, handler_service, socket::*, ws_handler_service, HrpcLayer, HrpcService,
    };
    pub use crate::{IntoRequest, IntoResponse, Request, Response};
    pub use axum::{self, body::BoxBody, routing::BoxRoute, routing::IntoMakeService, Router};
    pub use axum_server::{self, Server};
    pub use bytes::{Bytes, BytesMut};
    pub use futures_util::{FutureExt, SinkExt, StreamExt};
    pub use http::{self, HeaderMap};
    pub use std::{
        collections::HashMap,
        time::{Duration, Instant},
    };
    pub use tokio::{self, sync::Mutex, try_join};
    pub use tower::{
        layer::util::{Identity, Stack},
        util::{BoxLayer, BoxService},
        Layer, Service, ServiceBuilder, ServiceExt,
    };
    pub use tower_http::{
        compression::CompressionLayer, decompression::DecompressionLayer, trace::TraceLayer,
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

pub type HrpcService = BoxService<http::Request<Body>, http::Response<BoxBody>, ServerError>;
pub type HrpcLayer =
    BoxLayer<HrpcService, http::Request<Body>, http::Response<BoxBody>, ServerError>;

pub fn handler_service<Req, Resp, HandlerFn, HandlerFut>(handler: HandlerFn) -> HrpcService
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message,
    HandlerFut: Future<Output = Result<Response<Resp>, ServerError>> + Send,
    HandlerFn: FnOnce(Request<Req>) -> HandlerFut + Clone + Send + 'static,
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
    BoxService::new(service)
}

pub fn ws_handler_service<Req, Resp, HandlerFn, HandlerFut, OnUpgradeFn>(
    handler: HandlerFn,
    on_upgrade: OnUpgradeFn,
) -> HrpcService
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + 'static,
    HandlerFut: Future<Output = Result<(), ServerError>> + Send,
    HandlerFn: FnOnce(Request<()>, Socket<Req, Resp>) -> HandlerFut + Clone + Send + 'static,
    OnUpgradeFn:
        FnOnce(http::Response<BoxBody>) -> http::Response<BoxBody> + Clone + Send + 'static,
{
    let service = service_fn(move |request: http::Request<Body>| {
        let handler = handler.clone();
        let on_upgrade = on_upgrade.clone();
        async move {
            let mut req_parts = RequestParts::new(request);
            let websocket_upgrade = WebSocketUpgrade::from_request(&mut req_parts).await?;
            let request = Request::from_request(&mut req_parts).await?;

            let response = websocket_upgrade
                .protocols(["hrpc-ws", "hrpc-ws-json"])
                .on_upgrade(|ws| async move {
                    let socket = Socket::new(ws);
                    let res = handler(request, socket.clone()).await;
                    if let Err(err) = res {
                        tracing::error!("error in websocket: {}", err);
                    }
                    socket.close().await;
                })
                .into_response()
                .map(box_body);

            let response = on_upgrade(response);

            Ok(response)
        }
    });
    BoxService::new(service)
}
