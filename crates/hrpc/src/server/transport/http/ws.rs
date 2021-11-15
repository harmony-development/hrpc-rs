// The code in this file originally comes from https://github.com/tokio-rs/axum/blob/main/src/extract/ws.rs
// and is under the following license:
/*
Copyright (c) 2019 Tower Contributors

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

use bytes::Bytes;
use http::{
    header::{self, HeaderValue},
    Method, StatusCode,
};
use hyper::upgrade::{OnUpgrade, Upgraded};
use sha1::{Digest, Sha1};
use tokio_tungstenite::{
    tungstenite::protocol::{self, WebSocketConfig},
    WebSocketStream,
};

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    future::Future,
};

use crate::common::transport::http::{HeaderMapExt, WebSocket};

use super::{box_body, HttpRequest, HttpResponse};

/// Extractor for establishing WebSocket connections.
#[derive(Debug)]
pub(crate) struct WebSocketUpgrade {
    config: WebSocketConfig,
    protocols: Option<Box<[Cow<'static, str>]>>,
    extensions: Option<Box<[Cow<'static, str>]>>,
    sec_websocket_key: HeaderValue,
    on_upgrade: OnUpgrade,
    sec_websocket_protocol: Option<HeaderValue>,
}

impl WebSocketUpgrade {
    #[allow(dead_code)]
    /// Set the size of the internal message send queue.
    pub(crate) fn max_send_queue(mut self, max: usize) -> Self {
        self.config.max_send_queue = Some(max);
        self
    }

    #[allow(dead_code)]
    /// Set the maximum message size (defaults to 64 megabytes)
    pub(crate) fn max_message_size(mut self, max: usize) -> Self {
        self.config.max_message_size = Some(max);
        self
    }

    #[allow(dead_code)]
    /// Set the maximum frame size (defaults to 16 megabytes)
    pub(crate) fn max_frame_size(mut self, max: usize) -> Self {
        self.config.max_frame_size = Some(max);
        self
    }

    #[allow(dead_code)]
    /// Set the known protocols.
    pub(crate) fn protocols<I>(mut self, protocols: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'static, str>>,
    {
        self.protocols = Some(
            protocols
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        );
        self
    }

    #[allow(dead_code)]
    pub(crate) fn extensions<I>(mut self, extensions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'static, str>>,
    {
        self.extensions = Some(
            extensions
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        );
        self
    }

    /// Finalize upgrading the connection and call the provided callback with
    /// the stream.
    pub(crate) fn on_upgrade<F, Fut>(self, callback: F) -> WebSocketUpgradeResponse<F>
    where
        F: FnOnce(WebSocket<Upgraded>) -> Fut + Send + 'static,
        Fut: Future + Send + 'static,
    {
        WebSocketUpgradeResponse {
            extractor: self,
            callback,
        }
    }

    pub(crate) fn from_request(req: &mut HttpRequest) -> Result<Self, WebSocketUpgradeError> {
        if req.method() != Method::GET {
            return Err(WebSocketUpgradeError::MethodNotGet);
        }

        if !req
            .headers()
            .header_contains_str(&header::CONNECTION, "upgrade")
        {
            return Err(WebSocketUpgradeError::InvalidConnectionHeader);
        }

        if !req.headers().header_eq(&header::UPGRADE, b"websocket") {
            return Err(WebSocketUpgradeError::InvalidUpgradeHeader);
        }

        if !req
            .headers()
            .header_eq(&header::SEC_WEBSOCKET_VERSION, b"13")
        {
            return Err(WebSocketUpgradeError::InvalidWebsocketVersionHeader);
        }

        let sec_websocket_key =
            if let Some(key) = req.headers_mut().remove(header::SEC_WEBSOCKET_KEY) {
                key
            } else {
                return Err(WebSocketUpgradeError::WebsocketKeyHeaderMissing);
            };
        let on_upgrade = req.extensions_mut().remove::<OnUpgrade>().unwrap();
        let sec_websocket_protocol = req.headers_mut().remove(header::SEC_WEBSOCKET_PROTOCOL);

        Ok(Self {
            config: Default::default(),
            protocols: None,
            extensions: None,
            sec_websocket_key,
            on_upgrade,
            sec_websocket_protocol,
        })
    }
}

#[derive(Debug)]
pub(crate) enum WebSocketUpgradeError {
    MethodNotGet,
    InvalidConnectionHeader,
    InvalidUpgradeHeader,
    InvalidWebsocketVersionHeader,
    WebsocketKeyHeaderMissing,
}

impl Display for WebSocketUpgradeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MethodNotGet => f.write_str("request method is not get"),
            Self::InvalidConnectionHeader => f.write_str("request connection header is invalid"),
            Self::InvalidUpgradeHeader => f.write_str("request upgrade header is invalid"),
            Self::InvalidWebsocketVersionHeader => {
                f.write_str("request websocket version is invalid")
            }
            Self::WebsocketKeyHeaderMissing => {
                f.write_str("no websocket key header in request header")
            }
        }
    }
}

impl StdError for WebSocketUpgradeError {}

pub(crate) struct WebSocketUpgradeResponse<F> {
    extractor: WebSocketUpgrade,
    callback: F,
}

impl<F, Fut> WebSocketUpgradeResponse<F>
where
    F: FnOnce(WebSocket<Upgraded>) -> Fut + Send + 'static,
    Fut: Future + Send + 'static,
{
    pub(crate) fn into_response(self) -> HttpResponse {
        // check requested protocols
        let protocol = self
            .extractor
            .sec_websocket_protocol
            .as_ref()
            .and_then(|req_protocols| {
                let req_protocols = req_protocols.to_str().ok()?;
                let protocols = self.extractor.protocols.as_ref()?;
                req_protocols
                    .split(',')
                    .map(|req_p| req_p.trim())
                    .find(|req_p| protocols.iter().any(|p| p == req_p))
            });

        let protocol = match protocol {
            Some(protocol) => {
                if let Ok(protocol) = HeaderValue::from_str(protocol) {
                    Some(protocol)
                } else {
                    return http::Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(box_body(http_body::Full::new(Bytes::from_static(
                            b"`Sec-Websocket-Protocol` header is invalid",
                        ))))
                        .unwrap();
                }
            }
            None => None,
        };

        let callback = self.callback;
        let on_upgrade = self.extractor.on_upgrade;
        let config = self.extractor.config;

        tokio::spawn(async move {
            let upgraded = on_upgrade.await.expect("connection upgrade failed");
            let socket =
                WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, Some(config))
                    .await;
            callback(WebSocket::new(socket)).await;
        });

        let mut builder = http::Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(
                header::CONNECTION,
                HeaderValue::from_str("upgrade").unwrap(),
            )
            .header(header::UPGRADE, HeaderValue::from_str("websocket").unwrap())
            .header(
                header::SEC_WEBSOCKET_ACCEPT,
                sign(self.extractor.sec_websocket_key.as_bytes()),
            );

        if let Some(protocol) = protocol {
            builder = builder.header(header::SEC_WEBSOCKET_PROTOCOL, protocol);
        }

        let mut new_exts = String::new();

        if let Some(exts) = self.extractor.extensions {
            for ext in exts.iter() {
                if !new_exts.is_empty() {
                    new_exts.push_str("; ");
                }
                new_exts.push_str(ext.as_ref());
            }
        }

        if !new_exts.is_empty() {
            builder = builder.header(
                header::SEC_WEBSOCKET_EXTENSIONS,
                HeaderValue::from_str(&new_exts).expect("invalid websocket extensions"),
            );
        }

        builder.body(box_body(http_body::Empty::new())).unwrap()
    }
}

fn sign(key: &[u8]) -> HeaderValue {
    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    let b64 = Bytes::from(base64::encode(&sha1.finalize()));
    HeaderValue::from_maybe_shared(b64).expect("base64 is a valid value")
}
