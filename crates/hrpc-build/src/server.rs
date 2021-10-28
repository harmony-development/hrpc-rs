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
    let (make_handlers, routes, handler_fields, handler_new, handler_clone, handler_set) =
        generate_handlers(service, proto_path);

    quote! {
        /// Generated server implementations.
        #[allow(unused_variables)]
        pub mod #server_mod {
            use hrpc::server::gen_prelude::*;

            #generated_trait

            #service_doc
            pub struct #server_service<T: #server_trait> {
                service: T,
                #handler_fields
            }

            impl<T: #server_trait> Clone for #server_service<T> {
                fn clone(&self) -> Self {
                    Self {
                        service: self.service.clone(),
                        #handler_clone
                    }
                }
            }

            impl<T: #server_trait> Service for #server_service<T> {
                fn make_routes(&self) -> Routes {
                    #routes
                }
            }

            impl<T: #server_trait> #server_service<T> {
                /// Create a new service server.
                pub fn new(service: T) -> Self {
                    Self {
                        service,
                        #handler_new
                    }
                }

                #make_handlers

                #handler_set
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
                async fn #name(&mut self, request: HrpcRequest<#req_message>) -> ServerResult<HrpcResponse<#res_message>> {
                    Err("not implemented".into())
                }
            },
            (true, true) | (false, true) => quote! {
                #middleware_methods
                #on_upgrade_method

                #method_doc
                async fn #name(&mut self, request: HrpcRequest<()>, socket: Socket<#req_message, #res_message>) -> ServerResult<()> {
                    Err("not implemented".into())
                }
            },
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", name),
        };

        stream.extend(method);
    }

    stream
}

fn generate_handlers<T: Service>(
    service: &T,
    proto_path: &str,
) -> (
    TokenStream,
    TokenStream,
    TokenStream,
    TokenStream,
    TokenStream,
    TokenStream,
) {
    let mut make_handlers = TokenStream::new();
    let mut routes = quote! {
        Routes::new()
    };
    let mut handler_fields = TokenStream::new();
    let mut handler_new = TokenStream::new();
    let mut handler_clone = TokenStream::new();
    let mut handler_set = TokenStream::new();

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

        let set_middleware_name = quote::format_ident!("set_{}_middleware", name);
        let middleware_field_name = quote::format_ident!("{}_middleware", name);
        handler_fields.extend(quote! {
            #middleware_field_name: Option<HrpcLayer>,
        });
        handler_set.extend(quote! {
            /// Set the middleware for this endpoint.
            pub fn #set_middleware_name<L, S, B>(mut self, layer: L) -> Self
            where
                L: Layer<Handler, Service = S> + Sync + Send + 'static,
                S: TowerService<HttpRequest, Response = _HttpResponse<B>, Error = Infallible> + Send + 'static,
                S::Future: Send,
                B: HttpBody<Data = Bytes> + Send + Sync + 'static,
                B::Error: Into<BoxError>,
            {
                self.#middleware_field_name = Some(HrpcLayer::new(layer));
                self
            }
        });

        let set_name = quote::format_ident!("set_{}_handler", name);
        let make_handler_body = match streaming {
            (false, false) => {
                handler_fields.extend(quote! {
                    #name: Option<Arc<dyn Fn(&mut T, HrpcRequest<#req_message>)
                        -> BoxFuture<'static, ServerResult<HrpcResponse<#res_message>>> + Send + Sync + 'static>>,
                });
                handler_set.extend(quote! {
                    /// Set the handler for this endpoint.
                    pub fn #set_name<Hndlr, HndlrFut>(mut self, handler: Hndlr) -> Self
                    where
                        Hndlr: Fn(&mut T, HrpcRequest<#req_message>) -> HndlrFut + Send + Sync + 'static,
                        HndlrFut: Future<Output = ServerResult<HrpcResponse<#res_message>>> + Send + 'static,
                    {
                        let handler = move |svr: &mut T, request: HrpcRequest<#req_message>|
                        -> BoxFuture<'static, ServerResult<HrpcResponse<#res_message>>> {
                            Box::pin(handler(svr, request))
                        };
                        self.#name = Some(Arc::new(handler));
                        self
                    }
                });
                quote! {
                    let mut svr = self.service.clone();

                    if let Some(handler) = self.#name.clone() {
                        let handler = move |request: HrpcRequest<#req_message>| handler(&mut svr, request);
                        unary_handler(handler)
                    } else {
                        let handler = move |request: HrpcRequest<#req_message>| async move { svr. #name (request).await };
                        unary_handler(handler)
                    }
                }
            }
            (true, false) => panic!(
                "{}.{}: Client streaming server unary method is invalid.",
                package_name, method_name
            ),
            (true, true) | (false, true) => {
                let on_upgrade_field_name = quote::format_ident!("{}_on_upgrade", name);
                let set_on_upgrade_name = quote::format_ident!("set_{}_on_upgrade", name);

                handler_new.extend(quote! { #on_upgrade_field_name: None, });
                handler_clone.extend(
                    quote! { #on_upgrade_field_name: self.#on_upgrade_field_name.clone(), },
                );
                handler_fields.extend(quote! {
                    #name: Option<Arc<dyn Fn(&mut T, HrpcRequest<()>, Socket<#req_message, #res_message>)
                        -> BoxFuture<'static, ServerResult<()>> + Send + Sync + 'static>>,
                    #on_upgrade_field_name: Option<fn(&mut T, HttpResponse) -> HttpResponse>,
                });
                handler_set.extend(quote! {
                    /// Set the handler for this endpoint.
                    pub fn #set_name<Hndlr, HndlrFut>(mut self, handler: Hndlr) -> Self
                    where
                        Hndlr: Fn(&mut T, HrpcRequest<()>, Socket<#req_message, #res_message>) -> HndlrFut + Send + Sync + 'static,
                        HndlrFut: Future<Output = ServerResult<()>> + Send + 'static,
                    {
                        let handler = move |svr: &mut T, request: HrpcRequest<()>, socket: Socket<#req_message, #res_message>|
                        -> BoxFuture<'static, ServerResult<()>> {
                            Box::pin(handler(svr, request, socket))
                        };
                        self.#name = Some(Arc::new(handler));
                        self
                    }

                    /// Set the on upgrade method for this endpoint.
                    pub fn #set_on_upgrade_name(mut self, on_upgrade: fn(&mut T, HttpResponse) -> HttpResponse) -> Self {
                        self.#on_upgrade_field_name = Some(on_upgrade);
                        self
                    }
                });
                quote! {
                    let on_upgrade = {
                        let mut svr = self.service.clone();
                        let on_upgrade = if let Some(on_upgrade) = self.#on_upgrade_field_name.clone() {
                            on_upgrade
                        } else {
                            T::#on_upgrade_response_name
                        };
                        move |response: HttpResponse| on_upgrade(&mut svr, response)
                    };

                    let mut svr = self.service.clone();
                    if let Some(handler) = self.#name.clone() {
                        let handler = move |request: HrpcRequest<()>, socket: Socket<#req_message, #res_message>| handler(&mut svr, request, socket);
                        ws_handler(handler, on_upgrade)
                    } else {
                        let handler = move |request: HrpcRequest<()>, socket: Socket<#req_message, #res_message>| async move { svr. #name (request, socket).await };
                        ws_handler(handler, on_upgrade)
                    }
                }
            }
        };

        // Apply middleware
        let make_handler_body = quote! {
            let #name = {
                #make_handler_body
            };
            match self.service. #pre_name (#endpoint) {
                Some(layer) => layer.layer(#name),
                None => Handler::new(#name),
            }
        };

        let make_handler_name = quote::format_ident!("{}_handler", name);
        let doc_comment = generate_doc_comment(format!(
            "Creates a new `Handler` for this endpoint (`{}`).",
            endpoint
        ));
        let make_handler_fn = quote! {
            #doc_comment
            pub fn #make_handler_name(&self) -> Handler {
                #make_handler_body
            }
        };

        routes.extend(quote! { .route(#endpoint, self. #make_handler_name ()) });
        make_handlers.extend(make_handler_fn);
        handler_new.extend(quote! { #name: None, #middleware_field_name: None, });
        handler_clone.extend(quote! { #name: self.#name.clone(), #middleware_field_name: self.#middleware_field_name.clone(), })
    }

    (
        make_handlers,
        routes,
        handler_fields,
        handler_new,
        handler_clone,
        handler_set,
    )
}
