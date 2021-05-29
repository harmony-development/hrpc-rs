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
    let (serve_filters, serve_combined_filters, apis_push_filters) =
        generate_filters(service, proto_path);

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
                pub async fn serve<Err: CustomError + 'static, A: Into<std::net::SocketAddr>>(self, address: A) {
                    let filters = self.filters()
                        .with(warp::filters::trace::request())
                        .recover(hrpc::server::handle_rejection::<Err>);

                    warp::serve(filters).run(address).await
                }

                /// Convert this service to `warp` `Filter`s.
                ///
                /// This can be used to compose multiple services. See `serve_multiple` macro in `hrpc`.
                #[allow(clippy::redundant_clone)]
                pub fn filters(self) -> BoxedFilter<(impl Reply,)> {
                    let server = self.inner;

                    #serve_filters
                    #serve_combined_filters .boxed()
                }

                /// Extract `warp` filters, mapped to their URL API path.
                #[allow(clippy::redundant_clone)]
                pub fn filters_uncombined(self) -> HashMap<&'static str, BoxedFilter<(impl warp::Reply,)>> {
                    let server = self.inner;

                    #serve_filters

                    let mut apis = HashMap::new();
                    #apis_push_filters
                    apis
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
        let pre_name = quote::format_ident!("{}_pre", name);

        let (req_message, res_message) = method.request_response_name(proto_path);

        let method_doc = generate_doc_comments(method.comment());
        let on_upgrade_method = quote! {
            // Method that can be used to modify the response sent when the WebSocket is upgraded.
            fn #on_upgrade_response_name(&self, response: Response) -> Response {
                response
            }
        };
        let middleware_methods = quote! {
            // Filter to be run before all API operations but after API path is matched.
            fn #pre_name(&self) -> BoxedFilter<(Result<(), Self::Error>,)> {
                warp::any().map(|| Ok(())).boxed()
            }
        };
        let stream_handler = {
            quote! {
                #middleware_methods
                #on_upgrade_method
                #method_doc
                async fn #name(&self, validation_request: Request<()>, socket: Socket<#req_message, #res_message>);
            }
        };

        let method = match streaming {
            (false, false) => quote! {
                #middleware_methods
                #method_doc
                async fn #name(&self, request: Request<#req_message>) -> Result<#res_message, Self::Error>;
            },
            (false, true) => stream_handler,
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
            (true, true) => stream_handler,
        };

        stream.extend(method);
    }

    stream
}

fn generate_filters<T: Service>(
    service: &T,
    proto_path: &str,
) -> (TokenStream, TokenStream, TokenStream) {
    let mut stream = TokenStream::new();
    let mut comb_stream = TokenStream::new();
    let mut push_stream = TokenStream::new();

    for (index, method) in service.methods().iter().enumerate() {
        let name = quote::format_ident!("{}", method.name());
        let on_upgrade_response_name = quote::format_ident!("{}_on_upgrade", name);
        let pre_name = quote::format_ident!("{}_pre", name);

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
        let api_path = format!("{}/{}", package_name, method_name);

        let (req_message, resp_message) = method.request_response_name(proto_path);
        let streaming = (method.client_streaming(), method.server_streaming());

        let stream_handler = quote! {
            let svr = server.clone();
            socket_common::base_filter::<T::Error>(#package_name, #method_name, svr.#pre_name())
                .map(move |_, headers, ws: Ws| {
                    let svr = svr.clone();
                    let req = Request::from_parts(((), headers));
                    let svr3 = svr.clone();
                    let reply =
                        ws.on_upgrade(move |ws| async move {
                            let ((tx, rx), mut lpt) = (ws.split(), Instant::now());
                            let tx = Arc::new(Mutex::new(tx));

                            let socket = Socket::<#req_message, #resp_message>::new(tx.clone(), rx, T::SOCKET_PING_DATA);
                            let task = tokio::spawn(async move {
                                svr. #name (req, socket).await
                            });
                            let ping_task = tokio::spawn(async move {
                                while !socket_common::check_ping(&mut *tx.lock().await, &mut lpt, T::SOCKET_PING_DATA, T::SOCKET_PING_PERIOD).await { }
                            });

                            try_join!(task, ping_task).expect("failed to join handle in web socket handler");
                        }).into_response();
                    svr3. #on_upgrade_response_name (reply)
                })
        };

        let unary = quote! {
            let svr = server.clone();
            unary_common::base_filter::<#req_message, T::Error>(#package_name, #method_name, svr.#pre_name())
                .and_then(move |msg, headers| {
                    let svr = svr.clone();
                    async move {
                        unary_common::encode(svr. #name (Request::from_parts((msg, headers))) .await)
                    }
                })
        };

        let method = match streaming {
            (false, false) => unary,
            (false, true) => stream_handler,
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) => stream_handler,
        };

        let apply_middleware = quote! {
            let #name = {
                #method
            };
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
        push_stream.extend(quote! {
            apis.insert(#api_path, #name.boxed());
        });

        stream.extend(apply_middleware);
    }

    (stream, comb_stream, push_stream)
}
