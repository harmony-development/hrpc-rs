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
    let server_mod = quote::format_ident!("{}_server", naive_snake_case(service.name()));
    let generated_trait = generate_trait(service, proto_path, server_trait.clone());
    let service_doc = generate_doc_comments(service.comment());
    let (routes, router) = generate_routes(service, proto_path);

    let filters_method = quote! {
        /// Convert this service to `axum` `Router`.
        ///
        /// This can be used to compose multiple services. See `serve_multiple` macro in `hrpc`.
        pub fn into_make_service(self) -> IntoMakeService<BoxRoute> {
            let server = self.inner;

            #routes

            #router.into_make_service()
        }
    };

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
                    Self {
                        inner: Arc::new(inner),
                    }
                }

                /// Start serving.
                ///
                /// Note: this enables gzip compression and request tracing.
                pub async fn serve<A: Into<std::net::SocketAddr>>(self, address: A) {
                    let make_service = self.into_make_service();
                    let addr = address.into();

                    axum::Server::bind(&addr)
                        .serve(make_service)
                        .await
                        .unwrap()
                }

                #filters_method
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
        #[hrpc::exports::async_trait]
        pub trait #server_trait : Send + Sync + 'static {
            /// Filter to be run before all API operations but after API path is matched,
            /// for all endpoints.
            #[allow(unused_variables)]
            fn middleware(&self, endpoint: &'static str) -> HrpcLayer {
                BoxLayer::new(Identity::new())
            }

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
        let pre_name = quote::format_ident!("{}_middleware", name);

        let (req_message, res_message) = method.request_response_name(proto_path);

        let method_doc = generate_doc_comments(method.comment());
        let on_upgrade_method = quote! {
            /// Method that can be used to modify the response sent when the WebSocket is upgraded.
            fn #on_upgrade_response_name(&self, response: http::Response<BoxBody>) -> http::Response<BoxBody> {
                response
            }
        };
        let middleware_methods = quote! {
            /// Filter to be run before all API operations but after API path is matched.
            #[allow(unused_variables)]
            fn #pre_name(&self, endpoint: &'static str) -> HrpcLayer {
                BoxLayer::new(Identity::new())
            }
        };

        let method = match streaming {
            (false, false) => quote! {
                #middleware_methods

                #method_doc
                async fn #name(&self, request: Request<#req_message>) -> Result<Response<#res_message>, ServerError>;
            },
            (true, true) | (false, true) => quote! {
                #middleware_methods
                #on_upgrade_method

                #method_doc
                async fn #name(&self, request: Request<()>, socket: Socket<#req_message, #res_message>) -> Result<(), ServerError>;
            },
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
        };

        stream.extend(method);
    }

    stream
}

fn generate_routes<T: Service>(service: &T, proto_path: &str) -> (TokenStream, TokenStream) {
    let mut stream = TokenStream::new();
    let mut comb_stream = TokenStream::new();
    comb_stream.extend(quote! { Router::new() });

    for method in service.methods().iter() {
        let name = quote::format_ident!("{}", method.name());
        let on_upgrade_response_name = quote::format_ident!("{}_on_upgrade", name);
        let pre_name = quote::format_ident!("{}_middleware", name);

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
        let (req_message, res_message) = method.request_response_name(proto_path);

        let streaming = (method.client_streaming(), method.server_streaming());
        let endpoint = format!("/{}/{}", package_name, method_name);

        let method = match streaming {
            (false, false) => quote! {
                let svr = server.clone();

                let handler = move |request: Request<#req_message>| async { svr. #name (request) .await };
                handler_service(handler)
            },
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) | (false, true) => quote! {
                let svr = server.clone();

                let handler = {
                    let svr = svr.clone();
                    move |request: Request<()>, socket: Socket<#req_message, #res_message>| async { svr. #name (request, socket) .await }
                };
                let on_upgrade = move |response: http::Response<BoxBody>| svr. #on_upgrade_response_name (response);
                ws_handler_service(handler, on_upgrade)
            },
        };

        let apply_middleware = quote! {
            let #name = {
                #method
            };
            let #name = ServiceBuilder::new()
                .layer(server. #pre_name (#endpoint))
                .layer(server.middleware(#endpoint))
                .service(#name);
        };

        comb_stream.extend(quote! { .route(#endpoint, #name).boxed() });
        stream.extend(apply_middleware);
    }

    (stream, comb_stream)
}
