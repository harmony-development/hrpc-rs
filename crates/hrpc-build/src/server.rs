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
    let (handlers, routes) = generate_handlers(service, proto_path);

    quote! {
        /// Generated server implementations.
        #[allow(unused_variables)]
        pub mod #server_mod {
            use hrpc::server::gen_prelude::*;

            #generated_trait

            #service_doc
            pub struct #server_service<T: #server_trait> {
                service: T,
            }

            impl<T: #server_trait> Clone for #server_service<T> {
                fn clone(&self) -> Self {
                    Self {
                        service: self.service.clone(),
                    }
                }
            }

            impl<T: #server_trait> MakeRoutes for #server_service<T> {
                fn make_routes(&self) -> Routes {
                    #handlers

                    #routes
                }
            }

            impl<T: #server_trait> #server_service<T> {
                /// Create a new service server.
                pub fn new(service: T) -> Self {
                    Self {
                        service,
                    }
                }

                /// Serves the service with `hyper`.
                pub async fn serve<Addr: ToSocketAddrs>(self, addr: Addr) -> Result<(), <Hyper<Addr> as Transport>::Error> {
                    let transport = Hyper::new(addr);
                    transport.serve(self).await
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
        pub trait #server_trait : Clone + Send + Sized + 'static {
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
            fn #on_upgrade_response_name(&mut self, response: HttpResponse) -> HttpResponse {
                response
            }
        };
        let middleware_methods = quote! {
            /// Optional middleware for this RPC.
            #[allow(unused_variables)]
            fn #pre_name(&self, endpoint: &'static str) -> Option<HrpcLayer> {
                None
            }
        };

        let method = match streaming {
            (false, false) => quote! {
                #middleware_methods

                #method_doc
                fn #name(&mut self, request: HrpcRequest<#req_message>) -> BoxFuture<'_, ServerResult<HrpcResponse<#res_message>>>;
            },
            (true, true) | (false, true) => quote! {
                #middleware_methods
                #on_upgrade_method

                #method_doc
                fn #name(&mut self, request: HrpcRequest<()>, socket: Socket<#req_message, #res_message>) -> BoxFuture<'_, ServerResult<()>>;
            },
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
        };

        stream.extend(method);
    }

    stream
}

fn generate_handlers<T: Service>(service: &T, proto_path: &str) -> (TokenStream, TokenStream) {
    let mut handlers = TokenStream::new();
    let mut routes = quote! {
        Routes::new()
    };

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

        let handler_body = match streaming {
            (false, false) => {
                quote! {
                    let mut svr = self.service.clone();

                    let handler = move |request: HrpcRequest<#req_message>| async move { svr. #name (request).await };
                    unary_handler(handler)
                }
            }
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) | (false, true) => {
                quote! {
                    let mut svr = self.service.clone();
                    let on_upgrade = move |response: HttpResponse| svr.#on_upgrade_response_name(response);

                    let mut svr = self.service.clone();
                    let handler = move |request: HrpcRequest<()>, socket: Socket<#req_message, #res_message>| async move { svr. #name (request, socket).await };
                    ws_handler(handler, on_upgrade)
                }
            }
        };

        // Apply middleware
        let handler = quote! {
            let #name = {
                #handler_body
            };
            let #name = match self.service. #pre_name (#endpoint) {
                Some(layer) => layer.layer(#name),
                None => HrpcService::new(#name),
            };
        };

        routes.extend(quote! { .route(#endpoint, #name) });
        handlers.extend(handler);
    }

    (handlers, routes)
}
