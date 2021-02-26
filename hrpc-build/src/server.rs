use super::{Method, Service};
use crate::{generate_doc_comment, generate_doc_comments, naive_snake_case};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate service for Server.
///
/// This takes some `Service` and will generate a `TokenStream` that contains
/// a public module containing the server service and handler trait.
pub fn generate<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let server_service = quote::format_ident!("{}Server", service.name());
    let server_trait = quote::format_ident!("{}", service.name());
    let server_mod = quote::format_ident!("{}_server", naive_snake_case(&service.name()));
    let generated_trait = generate_trait(service, proto_path, server_trait.clone());
    let service_doc = generate_doc_comments(service.comment());
    let (serve_filters, serve_combined_filters) = generate_filters(service, proto_path);

    quote! {
        /// Generated server implementations.
        pub mod #server_mod {
            use std::sync::Arc;
            use prost::Message;
            use hrpc::server::prelude::*;

            #generated_trait

            #service_doc
            #[derive(Debug, Clone)]
            pub struct #server_service<T: #server_trait> {
                inner: Arc<T>,
            }

            impl<T: #server_trait> #server_service<T> {
                /// Create a new service server.
                pub fn new(inner: T) -> Self {
                    Self { inner: Arc::new(inner) }
                }

                /// Start serving.
                pub async fn serve(self, address: impl Into<std::net::SocketAddr>) {
                    warp::serve(self.filters().recover(hrpc::server::handle_rejection::<T::Error>))
                        .run(address)
                        .await
                }

                /// Extract `warp` filters.
                ///
                /// This can be used to compose multiple services. See `serve_multiple` macro in `hrpc`.
                #[allow(clippy::redundant_clone)]
                pub fn filters(self) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
                    let server = self.inner;

                    #serve_filters
                    #serve_combined_filters.boxed()
                }
            }
        }
    }
}

fn generate_trait<T: Service>(service: &T, proto_path: &str, server_trait: Ident) -> TokenStream {
    let methods = generate_trait_methods(service, proto_path);
    let trait_doc = generate_doc_comment(&format!(
        "Generated trait containing hRPC methods that should be implemented for use with {}Server.",
        service.name()
    ));

    quote! {
        #trait_doc
        #[hrpc::async_trait]
        pub trait #server_trait : Send + Sync + 'static {
            const PING_PERIOD: u64;

            type Error: CustomError + Send + Sync + 'static;

            #methods
        }
    }
}

fn generate_trait_methods<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let streaming = (method.client_streaming(), method.server_streaming());
        match streaming {
            (true, true) | (false, false) => {}
            _ => continue,
        }

        let name = quote::format_ident!("{}", method.name());

        let (req_message, res_message) = method.request_response_name(proto_path);

        let method_doc = generate_doc_comments(method.comment());

        let method = quote! {
            #method_doc
            async fn #name(&self, request: #req_message) -> Result<#res_message, Self::Error>;
        };

        stream.extend(method);
    }

    stream
}

fn generate_filters<T: Service>(service: &T, proto_path: &str) -> (TokenStream, TokenStream) {
    let mut stream = TokenStream::new();
    let mut comb_stream = TokenStream::new();
    let mut comb_index = 0;

    for method in service.methods() {
        let name = quote::format_ident!("{}", method.name());

        let package_name = format!(
            "{}{}{}",
            service.package(),
            if service.package().is_empty() {
                ""
            } else {
                "."
            },
            service.identifier(),
        );
        let method_name = method.identifier();

        let (req_message, _) = method.request_response_name(proto_path);
        let streaming = (method.client_streaming(), method.server_streaming());

        let method = match streaming {
            (false, false) => quote! {
                let svr = server.clone();
                let #name = warp::path(#package_name)
                    .and(warp::path(#method_name))
                    .and(warp::body::bytes())
                    .and(warp::header::exact_ignore_case(
                        "content-type",
                        "application/hrpc",
                    ))
                    .and_then(move |bin| {
                        let svr = svr.clone();
                        async move {
                            let mut buf = BytesMut::new();
                            match <#req_message> :: decode(bin) {
                                Ok(req) => {
                                    match svr. #name (req) .await {
                                        Ok(bin) => {
                                            hrpc::encode_protobuf_message(&mut buf, bin);
                                            let mut resp = warp::reply::Response::new(buf.to_vec().into());
                                            resp
                                                .headers_mut()
                                                .entry("content-type")
                                                .or_insert("application/hrpc".parse().unwrap());
                                            Ok(resp)
                                        }
                                        Err(err) => {
                                            log::error!("{}/{}: {}", #package_name, #method_name, err);
                                            Err(warp::reject::custom(ServerError::Custom(err)))
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!("{}/{}: received invalid protobuf message: {}", #package_name, #method_name, err);
                                    Err(warp::reject::custom(ServerError::<T::Error>::MessageDecode(err)))
                                }
                            }
                        }
                    });
            },
            // TODO: Somehow make it so that most of the code here is shared
            (true, true) => quote! {
                let svr = server.clone();
                let #name = warp::path(#package_name)
                    .and(warp::path(#method_name))
                    .and(warp::ws())
                    .map(move |ws: warp::ws::Ws| {
                        let svr = svr.clone();
                        ws.on_upgrade(move |ws| async move {
                            use std::time::Instant;

                            let (mut tx, mut rx) = ws.split();
                            let mut buf = BytesMut::new();
                            const PING_DATA: &[u8] = &[45; 32];
                            let mut last_ping_time = Instant::now();

                            loop {
                                let msg_maybe = rx.next().await.map(Result::ok).flatten();
                                if let Some(msg) = msg_maybe {
                                    if msg.is_binary() {
                                        let msg_bin = Bytes::from(msg.into_bytes());
                                        let resp = match <#req_message> :: decode(msg_bin) {
                                            Ok(req) => {
                                                match svr. #name (req) .await {
                                                    Ok(bin) => {
                                                        hrpc::encode_protobuf_message(&mut buf, bin);
                                                        buf.to_vec()
                                                    }
                                                    Err(err) => {
                                                        log::error!("{}/{}: {}", #package_name, #method_name, err);
                                                        format!("{{ \"message\": \"{}\" }}", err).into_bytes()
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                log::error!("{}/{}: received invalid protobuf message: {}", #package_name, #method_name, err);
                                                "{ \"message\": \"invalid protobuf message\" }".as_bytes().to_vec()
                                            }
                                        };

                                        if let Err(e) = tx.send(WsMessage::binary(resp)).await {
                                            log::error!("{}/{}: error responding to client socket: {}", #package_name, #method_name, e);
                                        } else {
                                            log::debug!("{}/{}: responded to client socket", #package_name, #method_name);
                                        }
                                    } else if msg.is_pong() {
                                        let msg_bin = Bytes::from(msg.into_bytes());
                                        if PING_DATA != msg_bin {
                                            if let Err(e) = tx.send(WsMessage::close()).await {
                                                log::error!("{}/{}: error closing socket: {}", #package_name, #method_name, e);
                                            } else {
                                                log::debug!("{}/{}: closed client socket", #package_name, #method_name);
                                            }
                                            break;
                                        } else {
                                            log::debug!("{}/{}: received pong", #package_name, #method_name);
                                        }
                                    } else if msg.is_close() {
                                        break;
                                    }
                                }
                                if last_ping_time.elapsed().as_secs() > T::PING_PERIOD {
                                    if let Err(e) = tx.send(WsMessage::ping(PING_DATA)).await {
                                        log::error!("{}/{}: error pinging client socket: {}", #package_name, #method_name, e);
                                        log::error!("{}/{}: can't reach client, closing socket", #package_name, #method_name);
                                        break;
                                    } else {
                                        log::debug!("{}/{}: pinged client socket, last ping was {} ago", #package_name, #method_name, last_ping_time.elapsed().as_secs());
                                    }
                                    last_ping_time = Instant::now();
                                }
                            }
                        })
                    });
            },
            _ => continue,
        };

        if comb_index > 0 {
            comb_stream.extend(quote! {
                .or(#name)
            });
        } else {
            comb_stream.extend(quote! {
                #name
            });
        }
        comb_index += 1;

        stream.extend(method);
    }

    (stream, comb_stream)
}
