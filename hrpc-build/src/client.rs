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
            use std::ops::{DerefMut, Deref};

            #service_doc
            #[derive(Debug, Clone)]
            pub struct #service_ident {
                inner: hrpc::Client,
            }

            impl #service_ident {
                pub fn new(inner: hrpc::reqwest::Client, server: hrpc::url::Url) -> hrpc::ClientResult<Self> {
                    Ok(Self {
                        inner: hrpc::Client::new(inner, server)?,
                    })
                }

                #methods
            }

            impl DerefMut for #service_ident {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.inner
                }
            }

            impl Deref for #service_ident {
                type Target = hrpc::Client;

                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }
        }
    }
}

fn generate_methods<T: Service>(service: &T, proto_path: &str) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let make_method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary,
            (true, true) => generate_socket,
            _ => continue,
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
            request: impl Into<#request>,
        ) -> hrpc::ClientResult<#response> {
            let req = self.inner.make_request(request.into(), #path)?;
            self.inner.execute_request(req).await
        }
    }
}

fn generate_socket<T: Method>(method: &T, proto_path: &str, path: String) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path);

    quote! {
        pub async fn #ident(
            &mut self,
        ) -> hrpc::ClientResult<hrpc::Socket<#request, #response>> {
            self.connect_socket(#path).await
        }
    }
}
