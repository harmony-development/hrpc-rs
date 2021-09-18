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
    let (serve_filters, serve_combined_filters) = generate_filters(service, proto_path);

    let filters_method = quote! {
        /// Convert this service to `warp` `Filter`s.
        ///
        /// This can be used to compose multiple services. See `serve_multiple` macro in `hrpc`.
        #[allow(clippy::redundant_clone)]
        #[cfg(not(debug_assertions))]
        pub fn filters(self) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
            let server = self.inner;

            #serve_filters
            balanced_or_tree!(#serve_combined_filters)
        }

        #[allow(clippy::redundant_clone)]
        #[cfg(debug_assertions)]
        pub fn filters(self) -> BoxedFilter<(impl Reply,)> {
            let server = self.inner;

            #serve_filters
            balanced_or_tree!(#serve_combined_filters).boxed()
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
                pub async fn serve<Err: CustomError + 'static, A: Into<std::net::SocketAddr>>(self, address: A) {
                    let filters = self.filters()
                        .with(warp::filters::trace::request())
                        .with(warp::compression::gzip())
                        .recover(hrpc::server::handle_rejection::<Err>);

                    warp::serve(filters).run(address).await
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
        #[hrpc::async_trait]
        pub trait #server_trait : Send + Sync + 'static {
            type Error: CustomError + Send + Sync + 'static;

            /// Filter to be run before all API operations but after API path is matched,
            /// for all endpoints.
            #[allow(unused_variables)]
            fn middleware(&self, endpoint: &'static str) -> BoxedFilter<()> {
                warp::any().boxed()
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
        let validation_name = quote::format_ident!("{}_validation", name);
        let validation_value = quote::format_ident!("{}ValidationType", method.identifier());

        let (req_message, res_message) = method.request_response_name(proto_path);

        let method_doc = generate_doc_comments(method.comment());
        let on_upgrade_method = quote! {
            /// Method that can be used to modify the response sent when the WebSocket is upgraded.
            fn #on_upgrade_response_name(&self, response: Response) -> Response {
                response
            }
        };
        let middleware_methods = quote! {
            /// Filter to be run before all API operations but after API path is matched.
            #[allow(unused_variables)]
            fn #pre_name(&self, endpoint: &'static str) -> BoxedFilter<()> {
                warp::any().boxed()
            }
        };

        let method = match streaming {
            (false, false) => quote! {
                #middleware_methods
                #method_doc
                async fn #name(&self, request: Request<#req_message>) -> Result<#res_message, Self::Error>;
            },
            (false, true) => quote! {
                #middleware_methods
                #on_upgrade_method

                type #validation_value: Send + Sync;
                /// The message may be `None` or `Some`.
                async fn #validation_name(&self, request: Request<Option<#req_message>>) -> Result<Self::#validation_value, Self::Error>;

                #method_doc
                async fn #name(&self, validation_value: Self::#validation_value, socket: Socket<#req_message, #res_message>);
            },
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
            (true, true) => quote! {
                #middleware_methods
                #on_upgrade_method

                type #validation_value: Send + Sync;
                /// The message will always be `None`.
                async fn #validation_name(&self, request: Request<Option<#req_message>>) -> Result<Self::#validation_value, Self::Error>;

                #method_doc
                async fn #name(&self, validation_value: Self::#validation_value, socket: Socket<#req_message, #res_message>);
            },
        };

        stream.extend(method);
    }

    stream
}

fn generate_filters<T: Service>(service: &T, _proto_path: &str) -> (TokenStream, TokenStream) {
    let mut stream = TokenStream::new();
    let mut comb_stream = TokenStream::new();

    for method in service.methods().iter() {
        let name = quote::format_ident!("{}", method.name());
        let on_upgrade_response_name = quote::format_ident!("{}_on_upgrade", name);
        let pre_name = quote::format_ident!("{}_middleware", name);
        let validation_name = quote::format_ident!("{}_validation", name);

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

        let streaming = (method.client_streaming(), method.server_streaming());
        let endpoint = format!("/{}/{}", package_name, method_name);

        let wrap_stream_handler = |code| {
            quote! {
                let svr = server.clone();
                let svr2 = server.clone();
                let svr3 = server.clone();
                #[allow(unused_mut)]
                socket_common::base_filter(
                    #package_name, #method_name,
                    svr.middleware(#endpoint).and(svr.#pre_name(#endpoint)),
                    move |req| async move { svr2. #validation_name (req) .await },
                    move |reply| svr3. #on_upgrade_response_name (reply),
                    move |_val, _req, mut sock| async move { #code },
                )
            }
        };

        let unary = quote! {
            let svr = server.clone();
            unary_common::base_filter(
                #package_name, #method_name,
                svr.middleware(#endpoint).and(svr.#pre_name(#endpoint)).boxed(),
                move |request| async move { svr. #name (request).await },
            )
        };

        let method = match streaming {
            (false, false) => unary,
            (false, true) => wrap_stream_handler(quote! {
                hrpc::return_print!(
                    socket_common::validator(_req, &mut sock, |req| svr. #validation_name (req)).await,
                    |val| svr. #name (val, sock).await
                );
            }),
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) => wrap_stream_handler(quote! {
                svr. #name (_val, sock).await
            }),
        };

        let apply_middleware = quote! {
            let #name = {
                #method
            };
        };

        comb_stream.extend(quote! { #name, });

        stream.extend(apply_middleware);
    }

    (stream, comb_stream)
}
