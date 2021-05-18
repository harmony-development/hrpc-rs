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
                    warp::serve(
                        self.filters()
                            .with(warp::filters::trace::request())
                            .recover(hrpc::server::handle_rejection::<T::Error>)
                    )
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
            const SOCKET_PING_PERIOD: u64 = 15;
            const SOCKET_PING_DATA: [u8; 32] = [1; 32];

            type Error: CustomError + Send + Sync + 'static;

            #methods
        }
    }
}

fn generate_trait_methods<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let streaming = (method.client_streaming(), method.server_streaming());

        let name = quote::format_ident!("{}", method.name());
        let on_upgrade_response_name = quote::format_ident!("{}_on_upgrade", name);

        let (req_message, res_message) = method.request_response_name(proto_path);

        let method_doc = generate_doc_comments(method.comment());
        let on_upgrade_method = quote! {
            // Method that can be used to modify the response sent when the WebSocket is upgraded.
            fn #on_upgrade_response_name(&self, response: Response) -> Response {
                response
            }
        };

        let method = match streaming {
            (false, false) => quote! {
                #method_doc
                async fn #name(&self, request: Request<#req_message>) -> Result<#res_message, Self::Error>;
            },
            (false, true) => {
                quote! {
                    #on_upgrade_method
                    #method_doc
                    async fn #name(&self, validation_request: &Request<#req_message>)
                        -> Result<Option<#res_message>, Self::Error>;
                }
            }
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
            (true, true) => {
                quote! {
                    #on_upgrade_method
                    #method_doc
                    async fn #name(&self, validation_request: &Request<()>, request: Option<#req_message>)
                        -> Result<Option<#res_message>, Self::Error>;
                }
            }
        };

        stream.extend(method);
    }

    stream
}

fn generate_filters<T: Service>(service: &T, proto_path: &str) -> (TokenStream, TokenStream) {
    let mut stream = TokenStream::new();
    let mut comb_stream = TokenStream::new();

    for (index, method) in service.methods().iter().enumerate() {
        let name = quote::format_ident!("{}", method.name());
        let on_upgrade_response_name = quote::format_ident!("{}_on_upgrade", name);

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

        let (req_message, resp_message) = method.request_response_name(proto_path);
        let streaming = (method.client_streaming(), method.server_streaming());

        let socket_common_var = quote! {
            let ((mut tx, mut rx), mut buf, mut lpt) = (ws.split(), BytesMut::new(), Instant::now());
            let tx = &mut tx;
            let rx = &mut rx;

            let span = info_span!(#method_name);
            let _lock = span.enter();
        };
        let wrap_reply_with_protocol = |code| {
            quote! {
                let svr3 = svr.clone();
                let reply = #code .into_response();
                svr3. #on_upgrade_response_name (reply)
            }
        };
        let wrap_with_common_handling = |code| {
            quote! {
                if socket_common::check_ping(tx, &mut lpt, T::SOCKET_PING_DATA, T::SOCKET_PING_PERIOD).await
                {
                    break;
                }
                socket_common::respond::<#resp_message, T::Error>(#code, tx, &mut buf).await;
            }
        };

        let server_handling = wrap_with_common_handling(quote! { svr. #name (&req) .await });
        let server_upgrade = wrap_reply_with_protocol(quote! {
            ws.on_upgrade(move |ws| async move {
                #socket_common_var
                let req;
                loop {
                    let validate_start = Instant::now();
                    if let Ok(Some(request)) = socket_common::process_request::<#req_message, T::Error>(tx, rx, T::SOCKET_PING_DATA).await
                    {
                        req = Request::from_parts((request, headers));

                        if let Err(err) = socket_common::process_validate::<T::Error>(svr. #name (&req) .await.map(|_| ()))
                        {
                            error!("failed to validate socket request: {}", err);
                            socket_common::close_socket(tx).await;
                            return;
                        }
                        break;
                    } else if validate_start.elapsed().as_secs() >= T::SOCKET_PING_PERIOD {
                        error!("failed to validate socket request: timeout");
                        socket_common::close_socket(tx).await;
                        return;
                    }
                }

                while socket_common::process_request::<#req_message, T::Error>(tx, rx, T::SOCKET_PING_DATA).await.is_ok()
                {
                    #server_handling
                }
            })
        });
        let server_streaming = quote! {
            let svr = server.clone();
            let #name = socket_common::base_filter(#package_name, #method_name)
                .map(move |headers, ws: Ws| {
                    let svr = svr.clone();
                    #server_upgrade
                });
        };

        let both_handling =
            wrap_with_common_handling(quote! { svr. #name (&req, maybe_req) .await });
        let both_upgrade = wrap_reply_with_protocol(quote! {
            ws.on_upgrade(move |ws| async move {
                #socket_common_var
                while let Ok(maybe_req) = socket_common::process_request::<#req_message, T::Error>(tx, rx, T::SOCKET_PING_DATA).await
                {
                    #both_handling
                }
            })
        });
        let both_streaming = quote! {
            let (svr, svr2) = (server.clone(), server.clone());
            let #name = socket_common::base_filter(#package_name, #method_name)
                .and_then(move |headers, ws: Ws| {
                    let svr = svr2.clone();
                    let req = Request::from_parts(((), headers));
                    async move {
                        let span = info_span!(#method_name);
                        let _lock = span.enter();
                        socket_common::process_validate::<T::Error>(svr. #name (&req, None) .await.map(|_| ()))
                            .map(|_| (req, ws))
                            .map_err(warp::reject::custom)
                    }
                })
                .map(move |(req, ws): (Request<()>, Ws)| {
                    let svr = svr.clone();
                    #both_upgrade
                });
        };
        let unary = quote! {
            let svr = server.clone();
            let #name = unary_common::base_filter::<#req_message, T::Error>(#package_name, #method_name)
                .and_then(move |msg, headers| {
                    let svr = svr.clone();
                    async move {
                        let span = info_span!(#method_name);
                        let _lock = span.enter();
                        unary_common::encode(svr. #name (Request::from_parts((msg, headers))) .await)
                    }
                });
        };

        let method = match streaming {
            (false, false) => unary,
            (false, true) => server_streaming,
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) => both_streaming,
        };

        comb_stream.extend(if index > 0 {
            quote! {
                .or(#name)
            }
        } else {
            quote! {
                #name
            }
        });

        stream.extend(method);
    }

    (stream, comb_stream)
}
