//! Utility proc macros for `hrpc`.
#![deny(missing_docs)]

use proc_macro::{Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree};

/// An attribute macro that turns an `async fn` into a hRPC handler function.
///
/// You should import either `hrpc::server::prelude` or `hrpc::make_handler` to be able to
/// use this macro properly.
#[proc_macro_attribute]
pub fn handler(_args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from_iter([
        TokenTree::Ident(Ident::new("make_handler", Span::mixed_site())),
        TokenTree::Punct(Punct::new('!', proc_macro::Spacing::Joint)),
        TokenTree::Group(Group::new(Delimiter::Brace, input)),
    ])
}
