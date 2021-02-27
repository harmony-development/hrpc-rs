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
    let client_mod = quote::format_ident!("{}_client", naive_snake_case(&service.name()));
    let methods = generate_methods(service, proto_path);

    let service_doc = generate_doc_comments(service.comment());

    quote! {
        /// Generated client implementations.
        #[allow(dead_code, unused_imports)]
        pub mod #client_mod {
            use prost::Message;
            use hrpc::{
                client::{
                    Client, Socket, Request, ClientResult, IntoRequest,
                    ReadSocket, WriteSocket,
                },
                reqwest::Client as ReqwestClient,
                url::Url,
            };

            #service_doc
            #[derive(Debug, Clone)]
            pub struct #service_ident {
                inner: Client,
            }

            impl #service_ident {
                pub fn new(inner: ReqwestClient, host_url: Url) -> ClientResult<Self> {
                    Ok(Self {
                        inner: Client::new(inner, host_url)?,
                    })
                }

                #methods
            }
        }
    }
}

fn generate_methods<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let make_method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary,
            (true, true) => generate_streaming,
            (false, true) => generate_server_streaming,
            (true, false) => generate_client_streaming,
        };

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

        stream.extend(generate_doc_comments(method.comment()));
        stream.extend(make_method(method, proto_path, path));
    }

    stream
}

fn generate_unary<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl IntoRequest<#request>,
        ) -> ClientResult<#response> {
            self.inner.execute_request(#path, request.into_request()).await
        }
    }
}

fn generate_streaming<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl IntoRequest<()>,
        ) -> ClientResult<Socket<#request, #response>> {
            self.inner.connect_socket(#path, request.into_request()).await
        }
    }
}

fn generate_client_streaming<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl IntoRequest<()>,
        ) -> ClientResult<WriteSocket<#request, #response>> {
            Ok(self.inner.connect_socket(#path, request.into_request()).await?.split().1)
        }
    }
}

fn generate_server_streaming<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl IntoRequest<()>,
        ) -> ClientResult<ReadSocket<#request, #response>> {
            Ok(self.inner.connect_socket(#path, request.into_request()).await?.split().0)
        }
    }
}
