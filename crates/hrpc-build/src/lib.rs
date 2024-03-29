//! Code generation for `hrpc`.
#![deny(missing_docs)]

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream};
use quote::TokenStreamExt;

#[cfg(feature = "rustfmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustfmt")))]
use std::io::{self, Write};
#[cfg(feature = "rustfmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustfmt")))]
use std::process::{exit, Command};

/// Prost generator
mod prost;

pub use self::prost::{compile_protos, configure, Builder};

/// Service code generation for client
#[cfg(feature = "client")]
pub mod client;
/// Service code generation for server
#[cfg(feature = "server")]
pub mod server;

/// Service generation trait.
///
/// This trait can be implemented and consumed
/// by `client::generate` and `server::generate`
/// to allow any codegen module to generate service
/// abstractions.
pub trait Service {
    /// Comment type.
    type Comment: AsRef<str>;

    /// Method type.
    type Method: Method;

    /// Name of service.
    fn name(&self) -> &str;
    /// Package name of service.
    fn package(&self) -> &str;
    /// Identifier used to generate type name.
    fn identifier(&self) -> &str;
    /// Methods provided by service.
    fn methods(&self) -> &[Self::Method];
    /// Get comments about this item.
    fn comment(&self) -> &[Self::Comment];
}

/// Method generation trait.
///
/// Each service contains a set of generic
/// `Methods`'s that will be used by codegen
/// to generate abstraction implementations for
/// the provided methods.
pub trait Method {
    /// Comment type.
    type Comment: AsRef<str>;

    /// Name of method.
    fn name(&self) -> &str;
    /// Identifier used to generate type name.
    fn identifier(&self) -> &str;
    /// Method is streamed by client.
    fn client_streaming(&self) -> bool;
    /// Method is streamed by server.
    fn server_streaming(&self) -> bool;
    /// Get comments about this item.
    fn comment(&self) -> &[Self::Comment];
    /// Get options of this item.
    fn options(&self) -> Vec<(String, String)>;
    /// Type name of request and response.
    fn request_response_name(&self, proto_path: &str) -> (TokenStream, TokenStream);
}

/// Format files under the out_dir with rustfmt
#[cfg(feature = "rustfmt")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustfmt")))]
pub fn fmt(out_dir: &str) {
    let dir = std::fs::read_dir(out_dir).unwrap();

    for entry in dir {
        let file = entry.unwrap().file_name().into_string().unwrap();
        if !file.ends_with(".rs") {
            continue;
        }
        let result = Command::new("rustfmt")
            .arg("--emit")
            .arg("files")
            .arg("--edition")
            .arg("2018")
            .arg(format!("{}/{}", out_dir, file))
            .output();

        match result {
            Err(e) => {
                eprintln!("error running rustfmt: {:?}", e);
                exit(1)
            }
            Ok(output) => {
                if !output.status.success() {
                    io::stderr().write_all(&output.stderr).unwrap();
                    exit(output.status.code().unwrap_or(1))
                }
            }
        }
    }
}

// Generate a singular line of a doc comment
#[allow(dead_code)]
fn generate_doc_comment<S: AsRef<str>>(comment: S) -> TokenStream {
    let mut doc_stream = TokenStream::new();

    doc_stream.append(Ident::new("doc", Span::call_site()));
    doc_stream.append(Punct::new('=', Spacing::Alone));
    let comment = comment.as_ref();
    let comment_escaped = comment.replace("```", "\\`\\`\\`");
    doc_stream.append(Literal::string(&comment_escaped));

    let group = Group::new(Delimiter::Bracket, doc_stream);

    let mut stream = TokenStream::new();
    stream.append(Punct::new('#', Spacing::Alone));
    stream.append(group);
    stream
}

// Generate a larger doc comment composed of many lines of doc comments
#[allow(dead_code)]
fn generate_doc_comments<T: AsRef<str>>(comments: &[T]) -> TokenStream {
    let mut stream = TokenStream::new();

    for comment in comments {
        stream.extend(generate_doc_comment(comment));
    }

    stream
}

#[allow(dead_code)]
fn naive_snake_case(name: &str) -> String {
    let mut s = String::new();
    let mut it = name.chars().peekable();

    while let Some(x) = it.next() {
        s.push(x.to_ascii_lowercase());
        if let Some(y) = it.peek() {
            if y.is_uppercase() {
                s.push('_');
            }
        }
    }

    s
}

#[test]
fn test_snake_case() {
    for case in &[
        ("Service", "service"),
        ("ThatHasALongName", "that_has_a_long_name"),
        ("greeter", "greeter"),
        ("ABCServiceX", "a_b_c_service_x"),
    ] {
        assert_eq!(naive_snake_case(case.0), case.1)
    }
}
