use std::net::ToSocketAddrs;

use super::Server;

use http::{header::HeaderName, HeaderMap};

/// Helper methods for working with `HeaderMap`.
pub trait HeaderMapExt {
    /// Check if a header is equal to a bytes array. Ignores casing.
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool;
    /// Check if a header contains a string. Ignores casing for the header value.
    fn header_contains_str(&self, key: &HeaderName, value: &str) -> bool;
}

impl HeaderMapExt for HeaderMap {
    fn header_eq(&self, key: &HeaderName, value: &[u8]) -> bool {
        self.get(key).map_or(false, |header| {
            header.as_bytes().eq_ignore_ascii_case(value)
        })
    }

    fn header_contains_str(&self, key: &HeaderName, pat: &str) -> bool {
        self.get(key).map_or(false, |header| {
            header
                .to_str()
                .map_or(false, |value| value.to_ascii_lowercase().contains(pat))
        })
    }
}

/// Start serving a server on the specified address.
///
/// If given multiple addresses, it will try serving on each address and if
/// all of them returns an error, it will return the last error.
pub async fn serve<A, S>(server: S, address: A) -> Result<(), hyper::Error>
where
    A: ToSocketAddrs,
    S: Server,
{
    let mut addrs = address
        .to_socket_addrs()
        .expect("could not convert to socket address");

    let mut successful_addr = addrs.next().expect("no socket address provided");
    let mut builder = hyper::Server::try_bind(&successful_addr);
    for addr in addrs {
        builder = if builder.is_err() {
            successful_addr = addr;
            hyper::Server::try_bind(&successful_addr)
        } else {
            break;
        };
    }

    let server = builder?.serve(server.into_make_service());
    tracing::info!("serving at {}", successful_addr);
    server.await
}
