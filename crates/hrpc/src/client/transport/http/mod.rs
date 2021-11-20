use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

use http::Uri;

#[cfg(feature = "http_hyper_client")]
pub mod hyper;
#[cfg(feature = "http_hyper_client")]
pub use self::hyper::{Hyper, HyperError};

#[cfg(feature = "http_wasm_client")]
pub mod wasm;
#[cfg(feature = "http_wasm_client")]
pub use self::wasm::{Wasm, WasmError};

/// Check if a URI is a valid server URI or not.
fn check_uri(uri: Uri) -> Result<Uri, InvalidServerUrl> {
    matches!(uri.scheme_str(), Some("https" | "http"))
        .then(|| uri)
        .ok_or(InvalidServerUrl::InvalidScheme)
}

/// Map a scheme that can be `https` or `http` to `wss` or `ws` respectively.
fn map_scheme_to_ws(scheme: &str) -> Option<&'static str> {
    match scheme {
        "https" => Some("wss"),
        "http" => Some("ws"),
        _ => None,
    }
}

#[derive(Debug)]
/// Errors that can occur while parsing the URL given to `Client::new()`.
pub enum InvalidServerUrl {
    /// Occurs if URL scheme isn't `http` or `https`.
    InvalidScheme,
}

impl Display for InvalidServerUrl {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InvalidServerUrl::InvalidScheme => {
                write!(f, "invalid scheme, expected `http` or `https`")
            }
        }
    }
}

impl StdError for InvalidServerUrl {}
