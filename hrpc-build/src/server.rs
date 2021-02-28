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

        let (req_message, res_message) = method.request_response_name(proto_path);

        let method_doc = generate_doc_comments(method.comment());

        let gen_validate_method = || {
            let name_validate = quote::format_ident!("{}_validate", name);
            let validate_doc = generate_doc_comment(&format!(
                "Validation for `{}` socket connection request.",
                name
            ));
            quote! {
                #validate_doc
                async fn #name_validate(&self, request: Request<()>) -> Result<(), Self::Error> {
                    let _ = request;
                    Ok(())
                }
            }
        };

        let method = match streaming {
            (false, false) => quote! {
                #method_doc
                async fn #name(&self, request: Request<#req_message>) -> Result<#res_message, Self::Error>;
            },
            (false, true) => {
                let validate = gen_validate_method();
                quote! {
                    #validate

                    #method_doc
                    async fn #name(&self) -> Result<Option<#res_message>, Self::Error>;
                }
            }
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
            (true, true) => {
                let validate = gen_validate_method();
                quote! {
                    #validate

                    #method_doc
                    async fn #name(&self, request: Option<#req_message>) -> Result<Option<#res_message>, Self::Error>;
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

        let concatted = format!("{}/{}", package_name, method_name);

        let method = match streaming {
            (false, false) => quote! {
                let svr = server.clone();
                let #name = unary_common::base_filter::<#req_message, T::Error>(#package_name, #method_name)
                    .and_then(move |msg, headers| {
                        let svr = svr.clone();
                        async move {
                            unary_common::encode(#package_name, #method_name, svr. #name (Request::from_parts((msg, headers))) .await)
                        }
                    })
                    .with(warp::filters::log::log(#concatted));
            },
            (false, true) => {
                let name_validate = quote::format_ident!("{}_validate", name);
                quote! {
                    let (svr, svr2) = (server.clone(), server.clone());
                    let #name = socket_common::base_filter(#package_name, #method_name)
                        .and_then(move |headers, ws: Ws| {
                            let svr = svr2.clone();
                            async move {
                                socket_common::process_validate::<T::Error>
                                    (#package_name, #method_name, svr. #name_validate (Request::from_parts(((), headers))) .await)
                                    .map(|_| ws)
                            }
                        })
                        .map(move |ws: Ws| {
                            let svr = svr.clone();

                            ws.on_upgrade(move |ws| async move {
                                let ((mut tx, mut rx), mut buf, mut lpt) = (ws.split(), BytesMut::new(), Instant::now());

                                while socket_common::process_request::<#req_message, T::Error>
                                    (#package_name, #method_name, &mut tx, &mut rx, T::SOCKET_PING_DATA).await.is_ok()
                                {
                                    if socket_common::check_ping(#package_name, #method_name, &mut tx, &mut lpt, T::SOCKET_PING_DATA, T::SOCKET_PING_PERIOD).await {
                                        break;
                                    }
                                    socket_common::respond::<#resp_message, T::Error>
                                        (#package_name, #method_name, svr. #name () .await, &mut tx, &mut buf).await;
                                }
                            })
                        })
                        .with(warp::filters::log::log(#concatted));
                }
            }
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) => {
                let name_validate = quote::format_ident!("{}_validate", name);
                quote! {
                    let (svr, svr2) = (server.clone(), server.clone());
                    let #name = socket_common::base_filter(#package_name, #method_name)
                        .and_then(move |headers, ws: Ws| {
                            let svr = svr2.clone();
                            async move {
                                socket_common::process_validate::<T::Error>
                                    (#package_name, #method_name, svr. #name_validate (Request::from_parts(((), headers))) .await)
                                    .map(|_| ws)
                            }
                        })
                        .map(move |ws: Ws| {
                            let svr = svr.clone();

                            ws.on_upgrade(move |ws| async move {
                                let ((mut tx, mut rx), mut buf, mut lpt) = (ws.split(), BytesMut::new(), Instant::now());

                                while let Ok(maybe_req) = socket_common::process_request::<#req_message, T::Error>
                                    (#package_name, #method_name, &mut tx, &mut rx, T::SOCKET_PING_DATA).await
                                {
                                    if socket_common::check_ping(#package_name, #method_name, &mut tx, &mut lpt, T::SOCKET_PING_DATA, T::SOCKET_PING_PERIOD).await {
                                        break;
                                    }
                                    socket_common::respond::<#resp_message, T::Error>
                                        (#package_name, #method_name, svr. #name (maybe_req) .await, &mut tx, &mut buf).await;
                                }
                            })
                        })
                        .with(warp::filters::log::log(#concatted));
                }
            }
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
