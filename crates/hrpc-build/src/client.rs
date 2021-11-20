use super::{Method, Service};
use crate::{generate_doc_comments, naive_snake_case};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate service for client.
///
/// This takes some `Service` and will generate a `TokenStream` that contains
/// a public module with the generated client.
pub fn generate<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let service_ident = quote::format_ident!("{}Client", service.name());
    let client_mod = quote::format_ident!("{}_client", naive_snake_case(service.name()));
    let methods = generate_methods(service, proto_path);

    let service_doc = generate_doc_comments(service.comment());

    let create_methods = quote! {
        impl<Inner> #service_ident<Inner> {
            /// Create a new client using the provided transport.
            pub fn new_transport(transport: Inner) -> Self {
                Self {
                    inner: Client::new(transport)
                }
            }

            /// Create a new client using the provided generic client.
            pub fn new_inner(client: Client<Inner>) -> Self {
                Self {
                    inner: client,
                }
            }
        }
    };

    #[allow(unused_variables)]
    let def_transport_impl = TokenStream::new();

    #[cfg(feature = "client_default_transport_hyper_http")]
    let def_transport_impl = quote! {
        use hrpc::{client::transport::http::Hyper, exports::http::Uri};

        impl #service_ident<Hyper> {
            /// Create a new client using HTTP transport.
            ///
            /// Panics if the passed URI is an invalid URI.
            pub fn new<U>(server: U) -> ClientResult<Self, <Hyper as Service<TransportRequest>>::Error>
            where
                U: TryInto<Uri>,
                U::Error: Debug,
            {
                let transport =
                    Hyper::new(server.try_into().expect("invalid URL"))
                        .map_err(ClientError::Transport)
                        .map_err(ClientError::Transport)?;
                Ok(Self {
                    inner: Client::new(transport),
                })
            }
        }
    };

    quote! {
        /// Generated client implementations.
        #[allow(dead_code, unused_imports)]
        pub mod #client_mod {
            use hrpc::client::prelude::*;

            #service_doc
            #[derive(Debug, Clone)]
            pub struct #service_ident<Inner> {
                inner: Client<Inner>,
            }

            impl<Inner> #service_ident<Inner>
            where
                Inner: Service<TransportRequest, Response = TransportResponse>,
                Inner::Error: std::error::Error,
            {
                #methods
            }

            #create_methods
            #def_transport_impl
        }
    }
}

fn generate_methods<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let path = format!(
            "/{}{}{}/{}",
            service.package(),
            if service.package().is_empty() {
                ""
            } else {
                "."
            },
            service.identifier(),
            method.identifier()
        );

        let make_method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary,
            (true, true) => generate_streaming,
            (false, true) => generate_server_streaming,
            (true, false) => panic!("{}: Client streaming server unary method is invalid.", path),
        };

        stream.extend(generate_doc_comments(method.comment()));
        stream.extend(make_method(method, proto_path, path));
    }

    stream
}

fn generate_unary<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident<Req>(&mut self, req: Req) -> ClientResult<Response<#response>, Inner::Error>
        where
            Req: IntoRequest<#request>,
        {
            let mut req = req.into_request();
            *req.endpoint_mut() = Cow::Borrowed(#path);
            self.inner.execute_request(req).await
        }
    }
}

fn generate_streaming<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident<Req>(&mut self, req: Req) -> ClientResult<Socket<#request, #response>, Inner::Error>
        where
            Req: IntoRequest<()>,
        {
            let mut req = req.into_request();
            *req.endpoint_mut() = Cow::Borrowed(#path);
            self.inner.connect_socket(req).await
        }
    }
}

fn generate_server_streaming<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident<Req>(&mut self, req: Req) -> ClientResult<Socket<#request, #response>, Inner::Error>
        where
            Req: IntoRequest<#request>,
        {
            let mut req = req.into_request();
            *req.endpoint_mut() = Cow::Borrowed(#path);
            self.inner.connect_socket_req(req).await
        }
    }
}
