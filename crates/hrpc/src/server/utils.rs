use std::{net::ToSocketAddrs, time::Duration};

use crate::HttpRequest;

use super::{HrpcLayer, Server};

use http::{header::HeaderName, HeaderMap};
use tower::ServiceBuilder;
use tower_http::{classify::StatusInRangeAsFailures, trace::TraceLayer};

/// Helper methods for working with `HeaderMap`.
pub trait HeaderMapExt {
    /// Check if a header is equal to a bytes array. Ignores casing.
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool;
    /// Check if a header contains a string. Ignores casing for the header value.
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

/// Start serving a server on the specified address.
///
/// If given multiple addresses, it will try serving on each address and if
/// all of them returns an error, it will return the last error.
pub async fn serve<A, S>(server: S, address: A) -> Result<(), hyper::Error>
where
    A: ToSocketAddrs,
    S: Server,
{
    let mut addrs = address
        .to_socket_addrs()
        .expect("could not convert to socket address");

    let mut successful_addr = addrs.next().expect("no socket address provided");
    let mut builder = hyper::Server::try_bind(&successful_addr);
    for addr in addrs {
        builder = if builder.is_err() {
            successful_addr = addr;
            hyper::Server::try_bind(&successful_addr)
        } else {
            break;
        };
    }

    let server = builder?
        .http1_keepalive(true)
        .http2_keep_alive_interval(Some(Duration::from_secs(10)))
        .http2_keep_alive_timeout(Duration::from_secs(20))
        .serve(server.into_make_service());
    tracing::info!("serving at {}", successful_addr);
    server.await
}

/// A set of "recommended" layers, namely:
/// - [`TraceLayer`] for tracing requests
pub fn recommended_layers<F>(filter_headers: Option<F>) -> HrpcLayer
where
    F: Fn(&HeaderName) -> bool + Clone + Send + Sync + 'static,
{
    HrpcLayer::new(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new(StatusInRangeAsFailures::new(400..=599).into_make_classifier())
                    .make_span_with(move |request: &HttpRequest| {
                        let span = tracing::info_span!(
                            "request",
                            method = %request.method(),
                            endpoint = %request.uri().path(),
                            version = ?request.version(),
                        );
                        for (header_name, header_val) in request.headers() {
                            if let Some(f) = &filter_headers {
                                if f(header_name) {
                                    span.record(
                                        header_name.as_str(),
                                        &String::from_utf8_lossy(header_val.as_bytes()).as_ref(),
                                    );
                                }
                            } else {
                                span.record(
                                    header_name.as_str(),
                                    &String::from_utf8_lossy(header_val.as_bytes()).as_ref(),
                                );
                            }
                        }
                        span
                    }),
            )
            .into_inner(),
    )
}

pub(crate) mod downcast {
    /// Code originally from https://github.com/hyperium/http/blob/master/src/convert.rs,
    /// licensed under the following license:
    /*
    Copyright (c) 2017 http-rs authors

    Permission is hereby granted, free of charge, to any
    person obtaining a copy of this software and associated
    documentation files (the "Software"), to deal in the
    Software without restriction, including without
    limitation the rights to use, copy, modify, merge,
    publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software
    is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice
    shall be included in all copies or substantial portions
    of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
    */
    macro_rules! if_downcast_into {
        ($in_ty:ty, $out_ty:ty, $val:ident, $body:expr) => {{
            if std::any::TypeId::of::<$in_ty>() == std::any::TypeId::of::<$out_ty>() {
                // Store the value in an `Option` so we can `take`
                // it after casting to `&mut dyn Any`.
                let mut slot = Some($val);
                // Re-write the `$val` ident with the downcasted value.
                let $val = (&mut slot as &mut dyn std::any::Any)
                    .downcast_mut::<Option<$out_ty>>()
                    .unwrap()
                    .take()
                    .unwrap();
                // Run the $body in scope of the replaced val.
                $body
            }
        }};
    }

    pub(crate) use if_downcast_into;
}
